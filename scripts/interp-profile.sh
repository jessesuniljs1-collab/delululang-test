#!/usr/bin/env bash
#
# Profile and time the interpreter on RW 3.24's benchmarks, in one command (on-demand routine run 16, loop engineering).
# The laptop handed RW 3.24 back on 2026-11-01 asking that the next attempt start from a profile; run 16 built the
# harness below by hand, and its lesson was that a cut is judged by callgrind's instruction count — deterministic, where
# the VM's wall time moved ±10% between identical runs — beside an interleaved A/B timing, never by one or the other.
#
# usage: scripts/interp-profile.sh [--base REF] [--rounds N] [--profile NAME] [PROGRAM.delulu …]
#   Builds, in a scratch directory outside the checkout (INTERP_PROFILE_DIR, default ${TMPDIR:-/tmp}/interp-profile), a
#   release harness that runs THIS checkout's interpreter (delulu-runtime, no WASM engine) on each program's `main`,
#   with the console granted, and prints for each: `Ir=` (instructions under callgrind, when valgrind is installed),
#   `ms=` (the minimum of 5 runs) and `out=` (a hash of what it printed — a cut changes no output).
#   Without PROGRAMs it writes and runs the five below: Study C's three, the laptop's micro set, and a sort of 20,000.
#   --base REF     also builds the harness against REF's sources (a detached worktree, removed after) and times the two
#                  interleaved, N rounds (default 3), printing `base=` and `new=` per round — and both `Ir=`.
#   --profile NAME write callgrind's per-line annotation of benchmark NAME to the scratch directory and print its top
#                  functions: where the next cut is.
# It writes nothing in the checkout.
set -uo pipefail
repo=$(git rev-parse --show-toplevel) || exit 2
work=${INTERP_PROFILE_DIR:-${TMPDIR:-/tmp}/interp-profile}
base='' rounds=3 profile='' progs=()
while [ $# -gt 0 ]; do
    case $1 in
        --base) base=${2:?--base needs a ref}; shift 2 ;;
        --rounds) rounds=${2:?--rounds needs a number}; shift 2 ;;
        --profile) profile=${2:?--profile needs a benchmark name}; shift 2 ;;
        -*) echo "interp-profile.sh: unknown option $1" >&2; exit 2 ;;
        *) progs+=("$(realpath "$1")"); shift ;;
    esac
done
mkdir -p "$work/progs" || exit 2

# The harness: one program, `main` run REPS times on fresh interpreters, the minimum printed to stderr.
harness() { # DIR CRATES
    mkdir -p "$1/src"
    cat > "$1/Cargo.toml" <<EOF
[package]
name = "interp-profile"
version = "0.1.0"
edition = "2021"

[dependencies]
delulu-runtime = { path = "$2/delulu-runtime", default-features = false }
delulu-check = { path = "$2/delulu-check" }

[profile.release]
debug = "line-tables-only"

[workspace]
EOF
    cat > "$1/src/main.rs" <<'EOF'
use std::rc::Rc;
use delulu_check::check_source;
use delulu_runtime::{set_capture, take_capture, Grants, Interp, Value};
fn main() {
    let path = std::env::args().nth(1).expect("a program");
    let reps: usize = std::env::args().nth(2).map(|s| s.parse().expect("a count")).unwrap_or(1);
    let src = std::fs::read_to_string(&path).expect("the program reads");
    let checked = check_source(0, &src);
    assert!(!checked.has_errors(), "{:?}", checked.diagnostics);
    let (mut best, mut out) = (u128::MAX, String::new());
    for _ in 0..reps {
        let interp = Interp::new(&checked.module);
        let root = Value::Root(Rc::new(Grants { console: true, ..Default::default() }.build_root()));
        set_capture(true);
        let t = std::time::Instant::now();
        let r = interp.run_main(root);
        best = best.min(t.elapsed().as_micros());
        out = take_capture().unwrap_or_default();
        r.unwrap_or_else(|e| panic!("the program faulted: {e:?}"));
    }
    eprintln!("{:.2}", best as f64 / 1000.0);
    print!("{out}");
}
EOF
    cp "$repo/Cargo.lock" "$1/Cargo.lock"
    (cd "$1" && CARGO_TARGET_DIR="$1/target" cargo build -q --release --offline) || return 1
}

if [ ${#progs[@]} -eq 0 ]; then
    cat > "$work/progs/loop_sum_1m.delulu" <<'EOF'
module bench

fn main(root: Root) ! {Write} {
  var i = 0
  var acc = 0
  while i < 1000000 {
    acc = acc + i
    i = i + 1
  }
  let out = root.console()
  out.println(str(acc))
}
EOF
    cat > "$work/progs/fib_recursive_24.delulu" <<'EOF'
module bench

fn fib(n: Int) -> Int {
  if n < 2 { n } else { fib(n - 1) + fib(n - 2) }
}

fn main(root: Root) ! {Write} {
  let out = root.console()
  out.println(str(fib(24)))
}
EOF
    cat > "$work/progs/nested_calls_macro.delulu" <<'EOF'
module bench

fn a(x: Int) -> Int { b(x) + 1 }
fn b(x: Int) -> Int { c(x) + 1 }
fn c(x: Int) -> Int { x + 1 }

fn main(root: Root) ! {Write} {
  var i = 0
  var total = 0
  while i < 200000 {
    total = total + a(i)
    i = i + 1
  }
  let out = root.console()
  out.println(str(total))
}
EOF
    cat > "$work/progs/micro.delulu" <<'EOF'
module bench

fn id(x: Int) -> Int { x }

fn main(root: Root) ! {Write} {
  let f = fn(x: Int) -> Int { x }
  let xs = [1, 2, 3, 4]
  var i = 0
  var acc = 0
  while i < 300000 {
    acc = acc + id(i)
    acc = acc + f(i)
    acc = acc + xs[i % 4]
    if i >= 0 {
      let a = i
      if a >= 0 {
        let b = a
        acc = acc + b
      }
    }
    i = i + 1
  }
  let out = root.console()
  out.println(str(acc))
}
EOF
    cat > "$work/progs/sort_20k.delulu" <<'EOF'
module algo


fn at0(xs: List[Int], i: Int) -> Int {
    match xs.get(i) {
        Some(v) => v
        None => 0
    }
}

fn lcg(x: Int) -> Int {
    (x * 1103515245 + 12345) % 2147483648
}

fn gen(n: Int, seed: Int, modulo: Int) -> List[Int] {
    let out = []
    var x = seed
    var i = 0
    while i < n {
        x = lcg(x)
        out.push(x / 65536 % modulo)
        i = i + 1
    }
    out
}

fn merge(a: List[Int], b: List[Int]) -> List[Int] {
    let out = []
    let na = a.len()
    let nb = b.len()
    var i = 0
    var j = 0
    while i < na && j < nb {
        let x = at0(a, i)
        let y = at0(b, j)
        if x <= y {
            out.push(x)
            i = i + 1
        } else {
            out.push(y)
            j = j + 1
        }
    }
    while i < na {
        out.push(at0(a, i))
        i = i + 1
    }
    while j < nb {
        out.push(at0(b, j))
        j = j + 1
    }
    out
}

fn msort(xs: List[Int]) -> List[Int] {
    let n = xs.len()
    if n <= 1 {
        xs
    } else {
        let mid = n / 2
        merge(msort(xs.slice(0, mid)), msort(xs.slice(mid, n)))
    }
}

fn qsort(xs: List[Int]) -> List[Int] {
    let n = xs.len()
    if n <= 1 {
        xs
    } else {
        let p = at0(xs, n / 2)
        let lo = xs.filter(fn(x: Int) -> Bool { x < p })
        let eq = xs.filter(fn(x: Int) -> Bool { x == p })
        let hi = xs.filter(fn(x: Int) -> Bool { x > p })
        qsort(lo).concat(eq).concat(qsort(hi))
    }
}

fn digest(xs: List[Int]) -> Int {
    var h = 7
    for x in xs {
        h = (h * 31 + x) % 1000000007
    }
    h
}

fn is_sorted(xs: List[Int]) -> Bool {
    var ok = true
    var i = 1
    while i < xs.len() {
        if at0(xs, i - 1) > at0(xs, i) {
            ok = false
        }
        i = i + 1
    }
    ok
}

fn main(root: Root) ! {Write} {
    let out = root.console()
    let data = gen(20000, 42, 1000000)
    let ms = msort(data)
    let qs = qsort(data)
    out.println("msort " + str(is_sorted(ms)) + " " + str(digest(ms)) + " qsort " + str(qs == ms) + " " + str(digest(qs)))
}
EOF
    progs=("$work"/progs/loop_sum_1m.delulu "$work"/progs/fib_recursive_24.delulu "$work"/progs/nested_calls_macro.delulu \
           "$work"/progs/micro.delulu "$work"/progs/sort_20k.delulu)
fi

harness "$work/new" "$repo/crates" || { echo "interp-profile.sh: the harness did not build" >&2; exit 1; }
if [ -n "$base" ]; then
    rm -rf "$work/base-src"
    git -C "$repo" worktree add -q --detach "$work/base-src" "$base" || exit 1
    harness "$work/base" "$work/base-src/crates"; built=$?
    git -C "$repo" worktree remove --force "$work/base-src"
    [ $built -eq 0 ] || { echo "interp-profile.sh: the harness did not build at $base" >&2; exit 1; }
fi

ir() { # BIN PROG
    command -v valgrind >/dev/null || { echo none; return; }
    valgrind --tool=callgrind --callgrind-out-file=/dev/null "$1" "$2" 1 2>&1 >/dev/null | awk '/refs:/ {print $NF}'
}
for p in "${progs[@]}"; do
    name=$(basename "$p" .delulu)
    reps=5; case $name in sort*) reps=2 ;; esac
    out=$("$work/new/target/release/interp-profile" "$p" 1 2>/dev/null | md5sum | cut -c1-8)
    if [ -n "$base" ]; then
        bout=$("$work/base/target/release/interp-profile" "$p" 1 2>/dev/null | md5sum | cut -c1-8)
        [ "$out" = "$bout" ] || echo "$name: OUTPUT DIFFERS from $base ($bout, now $out)"
        echo "$name Ir: base=$(ir "$work/base/target/release/interp-profile" "$p") new=$(ir "$work/new/target/release/interp-profile" "$p") out=$out"
        for r in $(seq "$rounds"); do
            b=$("$work/base/target/release/interp-profile" "$p" $reps 2>&1 >/dev/null)
            n=$("$work/new/target/release/interp-profile" "$p" $reps 2>&1 >/dev/null)
            echo "$name round $r ms: base=$b new=$n"
        done
    else
        echo "$name Ir=$(ir "$work/new/target/release/interp-profile" "$p") ms=$("$work/new/target/release/interp-profile" "$p" $reps 2>&1 >/dev/null) out=$out"
    fi
done
if [ -n "$profile" ]; then
    p=$(printf '%s\n' "${progs[@]}" | grep "/$profile.delulu$") || { echo "interp-profile.sh: no benchmark $profile" >&2; exit 2; }
    valgrind --tool=callgrind --callgrind-out-file="$work/$profile.cg" "$work/new/target/release/interp-profile" "$p" 1 >/dev/null 2>&1
    callgrind_annotate --auto=yes "$work/$profile.cg" > "$work/$profile.annotated" 2>/dev/null
    callgrind_annotate "$work/$profile.cg" 2>/dev/null | sed -n '/file:function/,$p' | sed -n '3,22p' | sed "s#$repo/crates/##; s#\[/.*##"
    echo "per-line annotation: $work/$profile.annotated"
fi
