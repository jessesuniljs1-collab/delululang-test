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
use std::collections::{BTreeMap, HashMap, HashSet};
use std::rc::Rc;

use delulu_runtime::value::MapKey;
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
    let mut p = Parser { s: text.as_bytes(), at: 0, depth: 0 };
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

/// How deep a descriptor's types may nest. The descriptor is read from the guest's memory, so its nesting is the guest's
/// to choose: a hand-written one a million levels deep recursed past the host's stack (DATA-DEPTH-1). No type the
/// compiler describes nests past the parser's own bound on a written type (128 levels).
const MAX_DESC_DEPTH: u32 = 256;

struct Parser<'s> {
    s: &'s [u8],
    at: usize,
    /// The types being parsed, around the next one.
    depth: u32,
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
        if self.depth >= MAX_DESC_DEPTH {
            return None;
        }
        self.depth += 1;
        let t = self.ty_here();
        self.depth -= 1;
        t
    }
    fn ty_here(&mut self) -> Option<TyDesc> {
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
        let desc: &'m Desc = self.desc;
        let t = desc.root.as_ref().ok_or("a descriptor with no type")?;
        let rebuilt = self.build(Step::Cell(ptr as u32, t));
        if rebuilt.is_err() {
            self.inside.clear();
        }
        rebuilt
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

    /// Rebuild the value `first` reads. DATA-DEPTH-1: rebuilt by recursion, a value nested past the host's native stack —
    /// a linked list of 100,000 records built in a loop — overflowed it and killed the process. The reads and the cells
    /// still to finish wait on a worklist instead, and each value read waits on `done` until the cell holding it is
    /// finished, so depth costs heap, bounded by the guest's memory. A cell's parts are read in order, as before.
    fn build(&mut self, first: Step<'m>) -> Result<Value, String> {
        let mut steps = vec![first];
        let mut done: Vec<Value> = Vec::new();
        while let Some(step) = steps.pop() {
            match step {
                Step::Slot(base, by, t) => {
                    let at = Self::offset(base, by)?;
                    match self.scalar(at, t)? {
                        Some(v) => done.push(v),
                        None => steps.push(Step::Cell(self.u32_at(at)?, t)),
                    }
                }
                Step::Run { base, stride, next, end, t } => {
                    if next < end {
                        steps.push(Step::Run { base, stride, next: next + 1, end, t });
                        steps.push(Step::Slot(base, stride * u64::from(next), t));
                    }
                }
                Step::Cell(ptr, t) => self.open(ptr, t, &mut steps, &mut done)?,
                Step::List(cell, n, v) => {
                    let parts = take_last(&mut done, n)?;
                    cell.borrow_mut().extend(parts);
                    done.push(v);
                }
                Step::Record(cell, names, v) => {
                    let parts = take_last(&mut done, names.len())?;
                    cell.borrow_mut().extend(names.into_iter().zip(parts));
                    done.push(v);
                }
                Step::Map(cell, keys, v) => {
                    let parts = take_last(&mut done, keys.len())?;
                    cell.borrow_mut().extend(keys.into_iter().zip(parts));
                    done.push(v);
                }
                Step::Variant(ptr, name, n) => {
                    let fields = take_last(&mut done, n)?;
                    self.inside.remove(&ptr);
                    done.push(Value::variant(name, fields));
                }
            }
        }
        match (done.pop(), done.is_empty()) {
            (Some(v), true) => Ok(v),
            _ => Err("a value rebuilt from no parts, or from too many".into()),
        }
    }

    /// The value held in the 8-byte slot at `at`, as `store_ty` wrote it, if its type holds it there: an `Int` or `Float`
    /// in all eight bytes, a `Bool` in the first four, a `Unit` in none. `None` for a type whose slot holds a pointer.
    fn scalar(&self, at: u32, t: &TyDesc) -> Result<Option<Value>, String> {
        Ok(Some(match t {
            TyDesc::Int => Value::Int(i64::from_le_bytes(self.bytes::<8>(at)?)),
            TyDesc::Float => Value::Float(f64::from_le_bytes(self.bytes::<8>(at)?)),
            TyDesc::Bool => Value::Bool(self.u32_at(at)? != 0),
            TyDesc::Unit => Value::Unit,
            _ => return Ok(None),
        }))
    }

    /// The string whose cell is at `ptr`: `[len:u32]` then its bytes.
    fn string(&self, ptr: u32) -> Result<Value, String> {
        let len = self.u32_at(ptr)?;
        let start = Self::offset(ptr, 4)? as usize;
        let bytes = start
            .checked_add(len as usize)
            .and_then(|end| self.mem.get(start..end))
            .ok_or_else(|| format!("a string at {ptr:#x} lies outside the guest's memory"))?;
        Ok(Value::str(String::from_utf8_lossy(bytes).into_owned()))
    }

    /// Open the cell at `ptr`, of type `t`: a string, or a cell met before, is done at once; any other cell is made
    /// (and remembered, so a cell met again inside itself is this one), and the step that finishes it goes on `steps`
    /// under the reads of its parts, so they run first and in order.
    fn open(&mut self, ptr: u32, t: &'m TyDesc, steps: &mut Vec<Step<'m>>, done: &mut Vec<Value>) -> Result<(), String> {
        let desc = self.desc;
        match t {
            TyDesc::Str => done.push(self.string(ptr)?),
            TyDesc::Record(n) => {
                if let Some(v) = self.records.get(&ptr) {
                    done.push(v.clone());
                    return Ok(());
                }
                let (name, fields) = desc.records.get(n).ok_or_else(|| format!("record {n} is not described"))?;
                let cell = Rc::new(RefCell::new(Vec::with_capacity(fields.len())));
                let v = Value::Record { name: Rc::from(name.as_str()), fields: cell.clone() };
                self.records.insert(ptr, v.clone());
                steps.push(Step::Record(cell, fields.iter().map(|(f, _)| f.clone()).collect(), v));
                for (k, (_, ft)) in fields.iter().enumerate().rev() {
                    steps.push(Step::Slot(ptr, 8 * k as u64, ft));
                }
            }
            TyDesc::List(elem) => {
                if let Some(v) = self.lists.get(&ptr) {
                    done.push(v.clone());
                    return Ok(());
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
                steps.push(Step::List(cell, len as usize, v));
                steps.push(Step::Run { base: data, stride: 8, next: 0, end: len, t: elem });
            }
            // A map's pairs, each key made the interpreter's `MapKey` — so `display` walks them in its own order and `eq`
            // compares them as it compares its own, whatever order a hand-written module left them in.
            TyDesc::Map(kt, vt) => {
                if let Some(v) = self.maps.get(&ptr) {
                    done.push(v.clone());
                    return Ok(());
                }
                let len = self.u32_at(ptr)?;
                let data = self.u32_at(Self::offset(ptr, 8)?)?;
                let end = u64::from(data) + 16 * u64::from(len);
                if end > self.mem.len() as u64 {
                    return Err(format!("a map at {ptr:#x} reaches past the guest's memory"));
                }
                let mut keys = Vec::with_capacity(len as usize);
                for i in 0..len {
                    let at = Self::offset(data, 16 * u64::from(i))?;
                    let key = match &**kt {
                        TyDesc::Str => Some(self.string(self.u32_at(at)?)?),
                        other => self.scalar(at, other)?,
                    };
                    let key = key
                        .as_ref()
                        .and_then(delulu_runtime::value::MapKey::of)
                        .ok_or_else(|| format!("a map at {ptr:#x} holds a key that cannot be one"))?;
                    keys.push(key);
                }
                let cell = Rc::new(RefCell::new(BTreeMap::new()));
                let v = Value::Map(cell.clone());
                self.maps.insert(ptr, v.clone());
                steps.push(Step::Map(cell, keys, v));
                let values = Self::offset(data, 8)?;
                steps.push(Step::Run { base: values, stride: 16, next: 0, end: len, t: vt });
            }
            TyDesc::Option(p) => {
                self.variant(ptr, &[("None", &[]), ("Some", std::slice::from_ref(&**p))], steps)?;
            }
            TyDesc::Result(o, e) => {
                self.variant(ptr, &[("Ok", std::slice::from_ref(&**o)), ("Err", std::slice::from_ref(&**e))], steps)?;
            }
            TyDesc::Sum(n) => {
                let ctors = desc.sums.get(n).ok_or_else(|| format!("sum {n} is not described"))?;
                let named: Vec<(&'m str, &'m [TyDesc])> = ctors.iter().map(|(c, p)| (c.as_str(), p.as_slice())).collect();
                self.variant(ptr, &named, steps)?;
            }
            TyDesc::Int | TyDesc::Bool | TyDesc::Float | TyDesc::Unit => return Err("a scalar where a cell was expected".into()),
        }
        Ok(())
    }

    /// Open a variant cell — `[tag:u32]` then each payload in an 8-byte slot from offset 4 — of a type with `ctors`: the
    /// step that makes it, under the reads of its payload.
    fn variant(&mut self, ptr: u32, ctors: &[(&'m str, &'m [TyDesc])], steps: &mut Vec<Step<'m>>) -> Result<(), String> {
        let tag = self.u32_at(ptr)?;
        let (name, payload) = ctors.get(tag as usize).ok_or_else(|| format!("a variant at {ptr:#x} has no constructor {tag}"))?;
        if !self.inside.insert(ptr) {
            return Err(format!("a variant at {ptr:#x} holds itself"));
        }
        steps.push(Step::Variant(ptr, name, payload.len()));
        for (i, pt) in payload.iter().enumerate().rev() {
            steps.push(Step::Slot(ptr, 4 + 8 * i as u64, pt));
        }
        Ok(())
    }
}

/// A step of `Reifier::build`: read the slot at a base and an offset, read a run of slots one by one, open the cell a
/// pointer names, or finish a cell from the values its parts left on `done`.
enum Step<'m> {
    Slot(u32, u64, &'m TyDesc),
    Run { base: u32, stride: u64, next: u32, end: u32, t: &'m TyDesc },
    Cell(u32, &'m TyDesc),
    List(Rc<RefCell<Vec<Value>>>, usize, Value),
    Record(Rc<RefCell<Vec<(String, Value)>>>, Vec<String>, Value),
    Map(Rc<RefCell<BTreeMap<MapKey, Value>>>, Vec<MapKey>, Value),
    Variant(u32, &'m str, usize),
}

/// The last `n` values of `done`, in order.
fn take_last(done: &mut Vec<Value>, n: usize) -> Result<Vec<Value>, String> {
    let at = done.len().checked_sub(n).ok_or("a cell finished before its parts were read")?;
    Ok(done.split_off(at))
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
