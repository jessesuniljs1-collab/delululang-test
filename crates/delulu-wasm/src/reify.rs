//! `str` and `==` of a list, an option, a result, a sum or a record on the WASM engine, answered by the interpreter's own
//! `Value::display` and `Value::eq` — one rule for both engines, not a copy of it. The guest passes a value's pointer and
//! its type's descriptor (`codegen::type_desc`); the host rebuilds the value as the interpreter's `Value` from the guest's
//! memory, a cell met twice rebuilt once — so a value that holds itself holds itself here too, and `display` and `eq`
//! meet it as they meet the interpreter's.
//!
//! The guest's memory is read as untrusted: a `.dwx` can be written by hand. Every read is bounds-checked, a tag outside
//! its type's constructors is refused, and so is a variant cell met again inside itself — which no compiled program can
//! build (a variant is never written after it is made), and which would otherwise recurse without end.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use delulu_runtime::Value;

/// A type as its descriptor writes it.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum TyDesc {
    Int,
    Bool,
    Float,
    Str,
    Unit,
    Option(Box<TyDesc>),
    Result(Box<TyDesc>, Box<TyDesc>),
    List(Box<TyDesc>),
    /// A `Map[K, V]` (RW 3.4): its header as a list's, its pairs 16 bytes each — the key's slot, then the value's.
    Map(Box<TyDesc>, Box<TyDesc>),
    Record(u32),
    Sum(u32),
}

/// A descriptor: the type, and each record and sum it reaches.
#[derive(Debug, Default, PartialEq)]
pub(crate) struct Desc {
    pub root: Option<TyDesc>,
    /// Record `n`: its name and its fields, in declaration order.
    pub records: HashMap<u32, (String, Vec<(String, TyDesc)>)>,
    /// Sum `n`: its constructors, in declaration order (the tag is the index), each with its payload's types.
    pub sums: HashMap<u32, Vec<(String, Vec<TyDesc>)>>,
}

/// Parse a descriptor — `T` then `|R<n>=<Name>{<field>:<T>,…}` and `|E<n>={<Ctor>(<T>…),…}` for each record and sum. A
/// type is `i` `b` `f` `s` `u`, `o<T>`, `r<T><T>`, `l<T>`, `m<K><V>`, `R<n>.` or `E<n>.`. `None` for anything else.
pub(crate) fn parse(text: &str) -> Option<Desc> {
    let mut p = Parser { s: text.as_bytes(), at: 0 };
    let mut d = Desc { root: Some(p.ty()?), ..Desc::default() };
    while p.eat(b'|') {
        match p.next()? {
            b'R' => {
                let n = p.num()?;
                p.expect(b'=')?;
                let name = p.ident()?;
                p.expect(b'{')?;
                let mut fields = Vec::new();
                while !p.eat(b'}') {
                    if !fields.is_empty() {
                        p.expect(b',')?;
                    }
                    let f = p.ident()?;
                    p.expect(b':')?;
                    fields.push((f, p.ty()?));
                }
                d.records.insert(n, (name, fields));
            }
            b'E' => {
                let n = p.num()?;
                p.expect(b'=')?;
                p.expect(b'{')?;
                let mut ctors = Vec::new();
                while !p.eat(b'}') {
                    if !ctors.is_empty() {
                        p.expect(b',')?;
                    }
                    let c = p.ident()?;
                    p.expect(b'(')?;
                    let mut payload = Vec::new();
                    while !p.eat(b')') {
                        payload.push(p.ty()?);
                    }
                    ctors.push((c, payload));
                }
                d.sums.insert(n, ctors);
            }
            _ => return None,
        }
    }
    (p.at == p.s.len()).then_some(d)
}

struct Parser<'s> {
    s: &'s [u8],
    at: usize,
}

impl Parser<'_> {
    fn next(&mut self) -> Option<u8> {
        let c = *self.s.get(self.at)?;
        self.at += 1;
        Some(c)
    }
    fn eat(&mut self, c: u8) -> bool {
        let hit = self.s.get(self.at) == Some(&c);
        self.at += usize::from(hit);
        hit
    }
    fn expect(&mut self, c: u8) -> Option<()> {
        self.eat(c).then_some(())
    }
    fn num(&mut self) -> Option<u32> {
        let start = self.at;
        while self.s.get(self.at).is_some_and(u8::is_ascii_digit) {
            self.at += 1;
        }
        std::str::from_utf8(&self.s[start..self.at]).ok()?.parse().ok()
    }
    fn ident(&mut self) -> Option<String> {
        let start = self.at;
        while self.s.get(self.at).is_some_and(|c| c.is_ascii_alphanumeric() || *c == b'_') {
            self.at += 1;
        }
        (self.at > start).then(|| String::from_utf8_lossy(&self.s[start..self.at]).into_owned())
    }
    fn ty(&mut self) -> Option<TyDesc> {
        Some(match self.next()? {
            b'i' => TyDesc::Int,
            b'b' => TyDesc::Bool,
            b'f' => TyDesc::Float,
            b's' => TyDesc::Str,
            b'u' => TyDesc::Unit,
            b'o' => TyDesc::Option(Box::new(self.ty()?)),
            b'r' => TyDesc::Result(Box::new(self.ty()?), Box::new(self.ty()?)),
            b'l' => TyDesc::List(Box::new(self.ty()?)),
            b'm' => {
                let k = self.ty()?;
                TyDesc::Map(Box::new(k), Box::new(self.ty()?))
            }
            b'R' => {
                let n = self.num()?;
                self.expect(b'.')?;
                TyDesc::Record(n)
            }
            b'E' => {
                let n = self.num()?;
                self.expect(b'.')?;
                TyDesc::Sum(n)
            }
            _ => return None,
        })
    }
}

/// Rebuilds values from one guest memory: records and lists by their cell's address, so a cell met twice is one `Rc`.
pub(crate) struct Reifier<'m> {
    mem: &'m [u8],
    desc: &'m Desc,
    records: HashMap<u32, Value>,
    lists: HashMap<u32, Value>,
    maps: HashMap<u32, Value>,
    /// The variant cells being rebuilt, innermost last — one met again inside itself is refused.
    inside: HashSet<u32>,
}

impl<'m> Reifier<'m> {
    pub(crate) fn new(mem: &'m [u8], desc: &'m Desc) -> Self {
        Reifier { mem, desc, records: HashMap::new(), lists: HashMap::new(), maps: HashMap::new(), inside: HashSet::new() }
    }

    /// The value of the descriptor's type whose cell is at `ptr`.
    pub(crate) fn root(&mut self, ptr: i32) -> Result<Value, String> {
        let t = self.desc.root.clone().ok_or("a descriptor with no type")?;
        self.cell(ptr as u32, &t)
    }

    fn bytes<const N: usize>(&self, at: u32) -> Result<[u8; N], String> {
        let a = at as usize;
        a.checked_add(N)
            .and_then(|end| self.mem.get(a..end))
            .and_then(|b| b.try_into().ok())
            .ok_or_else(|| format!("a value at {at:#x} lies outside the guest's memory"))
    }
    fn u32_at(&self, at: u32) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.bytes::<4>(at)?))
    }
    fn offset(at: u32, by: u64) -> Result<u32, String> {
        u32::try_from(u64::from(at) + by).map_err(|_| format!("a value past {at:#x} lies outside the guest's memory"))
    }

    /// The value held in the 8-byte slot at `at`, as `store_ty` wrote it: an `Int` or `Float` in all eight bytes, a `Bool`
    /// or a pointer in the first four, a `Unit` in none.
    fn slot(&mut self, at: u32, t: &TyDesc) -> Result<Value, String> {
        Ok(match t {
            TyDesc::Int => Value::Int(i64::from_le_bytes(self.bytes::<8>(at)?)),
            TyDesc::Float => Value::Float(f64::from_le_bytes(self.bytes::<8>(at)?)),
            TyDesc::Bool => Value::Bool(self.u32_at(at)? != 0),
            TyDesc::Unit => Value::Unit,
            _ => {
                let ptr = self.u32_at(at)?;
                self.cell(ptr, t)?
            }
        })
    }

    /// The value whose cell (or string) is at `ptr`.
    fn cell(&mut self, ptr: u32, t: &TyDesc) -> Result<Value, String> {
        let desc = self.desc;
        match t {
            TyDesc::Str => {
                let len = self.u32_at(ptr)?;
                let start = Self::offset(ptr, 4)? as usize;
                let bytes = start
                    .checked_add(len as usize)
                    .and_then(|end| self.mem.get(start..end))
                    .ok_or_else(|| format!("a string at {ptr:#x} lies outside the guest's memory"))?;
                Ok(Value::str(String::from_utf8_lossy(bytes).into_owned()))
            }
            TyDesc::Record(n) => {
                if let Some(v) = self.records.get(&ptr) {
                    return Ok(v.clone());
                }
                let (name, fields) = desc.records.get(n).ok_or_else(|| format!("record {n} is not described"))?;
                let cell = Rc::new(RefCell::new(Vec::with_capacity(fields.len())));
                let v = Value::Record { name: Rc::from(name.as_str()), fields: cell.clone() };
                self.records.insert(ptr, v.clone());
                for (k, (f, ft)) in fields.iter().enumerate() {
                    let fv = self.slot(Self::offset(ptr, 8 * k as u64)?, ft)?;
                    cell.borrow_mut().push((f.clone(), fv));
                }
                Ok(v)
            }
            TyDesc::List(elem) => {
                if let Some(v) = self.lists.get(&ptr) {
                    return Ok(v.clone());
                }
                let len = self.u32_at(ptr)?;
                let data = self.u32_at(Self::offset(ptr, 8)?)?;
                // The whole span first, so a header claiming more elements than the memory holds allocates nothing.
                let end = u64::from(data) + 8 * u64::from(len);
                if end > self.mem.len() as u64 {
                    return Err(format!("a list at {ptr:#x} reaches past the guest's memory"));
                }
                let cell = Rc::new(RefCell::new(Vec::with_capacity(len as usize)));
                let v = Value::List(cell.clone());
                self.lists.insert(ptr, v.clone());
                for i in 0..len {
                    let ev = self.slot(Self::offset(data, 8 * u64::from(i))?, elem)?;
                    cell.borrow_mut().push(ev);
                }
                Ok(v)
            }
            // A map's pairs, each key made the interpreter's `MapKey` — so `display` walks them in its own order and `eq`
            // compares them as it compares its own, whatever order a hand-written module left them in.
            TyDesc::Map(kt, vt) => {
                if let Some(v) = self.maps.get(&ptr) {
                    return Ok(v.clone());
                }
                let len = self.u32_at(ptr)?;
                let data = self.u32_at(Self::offset(ptr, 8)?)?;
                let end = u64::from(data) + 16 * u64::from(len);
                if end > self.mem.len() as u64 {
                    return Err(format!("a map at {ptr:#x} reaches past the guest's memory"));
                }
                let cell = Rc::new(RefCell::new(std::collections::BTreeMap::new()));
                let v = Value::Map(cell.clone());
                self.maps.insert(ptr, v.clone());
                for i in 0..len {
                    let pair = Self::offset(data, 16 * u64::from(i))?;
                    let key = self.slot(pair, kt)?;
                    let key = delulu_runtime::value::MapKey::of(&key)
                        .ok_or_else(|| format!("a map at {ptr:#x} holds a key that cannot be one"))?;
                    let value = self.slot(Self::offset(pair, 8)?, vt)?;
                    cell.borrow_mut().insert(key, value);
                }
                Ok(v)
            }
            TyDesc::Option(p) => self.variant(ptr, &[("None", &[]), ("Some", std::slice::from_ref(&**p))]),
            TyDesc::Result(o, e) => self.variant(ptr, &[("Ok", std::slice::from_ref(&**o)), ("Err", std::slice::from_ref(&**e))]),
            TyDesc::Sum(n) => {
                let ctors = desc.sums.get(n).ok_or_else(|| format!("sum {n} is not described"))?;
                let named: Vec<(&str, &[TyDesc])> = ctors.iter().map(|(c, p)| (c.as_str(), p.as_slice())).collect();
                self.variant(ptr, &named)
            }
            TyDesc::Int | TyDesc::Bool | TyDesc::Float | TyDesc::Unit => Err("a scalar where a cell was expected".into()),
        }
    }

    /// A variant cell — `[tag:u32]` then each payload in an 8-byte slot from offset 4 — of a type with `ctors`.
    fn variant(&mut self, ptr: u32, ctors: &[(&str, &[TyDesc])]) -> Result<Value, String> {
        let tag = self.u32_at(ptr)?;
        let (name, payload) = ctors.get(tag as usize).ok_or_else(|| format!("a variant at {ptr:#x} has no constructor {tag}"))?;
        if !self.inside.insert(ptr) {
            return Err(format!("a variant at {ptr:#x} holds itself"));
        }
        let mut fields = Vec::with_capacity(payload.len());
        for (i, pt) in payload.iter().enumerate() {
            let at = Self::offset(ptr, 4 + 8 * i as u64);
            match at.and_then(|at| self.slot(at, pt)) {
                Ok(v) => fields.push(v),
                Err(e) => {
                    self.inside.remove(&ptr);
                    return Err(e);
                }
            }
        }
        self.inside.remove(&ptr);
        Ok(Value::variant(name, fields))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_descriptor_parses_and_a_malformed_one_does_not() {
        let d = parse("lR0.|R0=Node{v:i,next:oR0.,tag:E1.}|E1={Red(),Box(fs)}").expect("well formed");
        assert_eq!(d.root, Some(TyDesc::List(Box::new(TyDesc::Record(0)))));
        assert_eq!(d.records[&0].0, "Node");
        assert_eq!(d.records[&0].1[1], ("next".into(), TyDesc::Option(Box::new(TyDesc::Record(0)))));
        assert_eq!(d.sums[&1], vec![("Red".into(), vec![]), ("Box".into(), vec![TyDesc::Float, TyDesc::Str])]);
        for bad in ["", "x", "o", "R0", "i|", "i|R0=N{a:i", "i|Q0=", "ii", "l|R0=N{}"] {
            assert_eq!(parse(bad), None, "{bad:?}");
        }
    }

    /// Memory a hand-written module could hold: a list claiming more elements than the memory has, a string past its end,
    /// a tag with no constructor, and a variant cell that holds itself — each refused, none a crash or a vast allocation.
    #[test]
    fn a_malformed_guest_value_is_refused() {
        let mut mem = vec![0u8; 64];
        // A list header at 0: len 0x7fff_ffff, data 16.
        mem[0..4].copy_from_slice(&0x7fff_ffffu32.to_le_bytes());
        mem[8..12].copy_from_slice(&16u32.to_le_bytes());
        let list = parse("li").unwrap();
        assert!(Reifier::new(&mem, &list).root(0).unwrap_err().contains("past the guest's memory"));
        // A string at 32 claiming 1,000 bytes.
        mem[32..36].copy_from_slice(&1000u32.to_le_bytes());
        let s = parse("os").unwrap();
        mem[40..44].copy_from_slice(&1u32.to_le_bytes()); // `Some` at 40, its payload pointer at 44 -> 32
        mem[44..48].copy_from_slice(&32u32.to_le_bytes());
        assert!(Reifier::new(&mem, &s).root(40).unwrap_err().contains("outside the guest's memory"));
        // A tag 7 of an `Option`.
        mem[40..44].copy_from_slice(&7u32.to_le_bytes());
        assert!(Reifier::new(&mem, &s).root(40).unwrap_err().contains("no constructor 7"));
        // `Link(E0)` whose payload points at itself.
        let chain = parse("E0.|E0={Nil(),Link(E0.)}").unwrap();
        mem[48..52].copy_from_slice(&1u32.to_le_bytes());
        mem[52..56].copy_from_slice(&48u32.to_le_bytes());
        assert!(Reifier::new(&mem, &chain).root(48).unwrap_err().contains("holds itself"));
    }
}
