# DeluluLang

> **Authority and effects are part of the type of every program — total, verifiable, and
> enforced across all code, all dependencies, and all runtime-loaded plugins.**

DeluluLang (`.delulu`) is a programming language where every function, module, and plugin
carries its **authority and effects in its type**, so the compiler can answer — mechanically —
the question no mainstream toolchain can: *"what can this program actually do to my system?"*

```delulu
fn fib(n: Int) -> Int {                       // provably pure: no row means !{}
  if n < 2 { n } else { fib(n - 1) + fib(n - 2) }
}

fn greet(out: Cap[Console], name: Str) ! {Write} {   // the row is what it does;
  out.println("hello, " + name)                      // the capability is its permission
}
```

Primary users: AI agents, LLMs, and future AI systems — the population for whom unrestricted
authority is most dangerous. Humans use it to learn and to **review AI-written code**. No
discrimination between holders: the only asymmetry is the grant relation — what you grant
cannot exceed what you hold, and what you granted cannot break you.

## Status

Stage 1 ("Skeleton") — under construction. The design is complete and committed:

| Document | What it is |
|---|---|
| [`docs/design/CONSTITUTION.md`](docs/design/CONSTITUTION.md) | The v1.0 language constitution — identity, semantics, honesty clauses |
| [`docs/design/SOUNDNESS_AUDIT.md`](docs/design/SOUNDNESS_AUDIT.md) | The soundness audit — rules R-1…R-7 that keep authority in the type |
| [`docs/design/STAGE1_SPECIFICATION.md`](docs/design/STAGE1_SPECIFICATION.md) … `STAGE10_…` | Buildable stage-by-stage specifications, Skeleton → Industrial |

## Honesty

This project never claims "faster than C," "lowest tokens," or "unbreakable." Guarantees are
stated relative to a named threat model; strength comes from defense in depth (type-system proof
→ WASM/WASI floor → microVM containment → human-held broker keys), and every trust assumption
(compiler, hardware, hypervisor, side channels) is named. See the constitution, §5.14 and §9.

## Building

```
rustup default stable
cargo build
cargo test
```

*Be delulu: write code as if no program can ever exceed its authority — then make the compiler
make it true.* 🐦‍🔥
