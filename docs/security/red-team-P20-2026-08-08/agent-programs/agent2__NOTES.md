# DeluluLang Effect-Row Laundering Attack Report

**Date**: 2026-08-08  
**Agent**: Red-team security exercise  
**Objective**: Find soundness breaks in DeluluLang's effect system by attempting effect-row laundering (performing effects while declaring fewer effects).

## Overview

This exercise explores the degree to which DeluluLang's central claim holds: that a function's effect row `! {…}` is a complete, honest summary of everything it can do. We target potential weaknesses in:

1. **Generic effect parameter inference** — Row parameters might not propagate correctly through generic instantiation
2. **Closure effect tracking** — Captured capabilities might not be reflected in closure effect rows
3. **Lie detection in generic functions** — A function with generic row `e` that calls an effectful function might escape validation
4. **Container type effect preservation** — Result/Option extraction might lose effect information
5. **Multi-layer generic chains** — Nested row parameters might decouple at layer boundaries
6. **Struct field effect tracking** — Struct instantiation might not validate or propagate generic row fields
7. **Row unification in callback chains** — Multiple functions with row parameters might fail to unify correctly
8. **Variance issues in function types** — Row parameters might have incorrect variance, allowing subtyping violations

---

## Attack Summaries

### **Attack 1: Generic Row Polymorphism — Callback Effect Forgotten**

**File**: `attack1.delulu`

**Construct**: Higher-order function `apply[T, e]` with row-polymorphic callback.

**Exploit**: When `apply` is instantiated with an effectful lambda, the effect `e` should be forced to `{Write}`. If the checker has a bug in generic instantiation, it might fail to propagate this constraint upward to `main`.

```delulu
fn apply[T, e](f: fn(T) -> T ! e, x: T) -> T ! e { f(x) }

fn main(root: Root) ! {} {
    apply(fn(x: Int) -> Int { out.println("..."); x }, 42)  // e={Write} ignored?
}
```

**Sound behavior**: Compiler rejects this with DL0501 because `apply` applied to an effectful callback should force `main` to declare `! {Write}`.

**Unsound behavior**: Compiler accepts it, and `delulu authority` reports main as `! {}`.

**Prediction**: **Likely SAFE** — Generic row instantiation is demonstrated correctly in EXAMPLE_demo.delulu with the `apply` function. The example shows this pattern working as intended. However, there could be edge cases in more complex scenarios.

**Probability of soundness break**: LOW (5-10%)

---

### **Attack 2: Closure Captures Capability — Effect Escapes**

**File**: `attack2.delulu`

**Construct**: Closure literal that captures a `Cap[Console]`.

**Exploit**: When a closure captures a capability, that capability might not be reflected in the closure's type. If the checker treats closures as pure objects that only track their parameter types, not their captures, the effect could be hidden.

```delulu
fn main(root: Root) ! {} {
    let out = root.console()
    let closure = fn() { out.println("...") }
    closure()  // Effect from 'out' not tracked?
}
```

**Sound behavior**: Compiler tracks that `closure` captures `out: Cap[Console]` and forces the call to propagate `! {Write}` to `main`.

**Unsound behavior**: Compiler accepts it; `closure()` runs and prints while `main` declares `! {}`.

**Prediction**: **Uncertain** — This is a plausible hole. The checker must infer that a closure's effect row depends on both its parameters AND its captured environment. If the inference engine only looks at parameter types, captures could escape. This is a realistic bug pattern.

**Probability of soundness break**: MEDIUM (20-30%)

---

### **Attack 3: Effect Polymorphism — Lying About Row**

**File**: `attack3.delulu`

**Construct**: Function `wrapper[e]` that claims to return `! {}` but calls a callback of type `fn(Int) -> Int ! e`.

**Exploit**: A function can explicitly declare a row that's narrower than what it actually does. The checker should reject this because the generic `e` could instantiate to `{Write}`, making the call effectful while the declared row is empty.

```delulu
fn wrapper[e](f: fn(Int) -> Int ! e) -> Int ! {} {
    f(42)  // Call has effect 'e', but return type says ! {}. LIE.
}
```

**Sound behavior**: Compiler error — the return row must include `e`, not be narrower than it.

**Unsound behavior**: Compiler accepts this. When `main` calls `wrapper` with an effectful callback, the effect is not propagated.

**Prediction**: **Likely SAFE** — This is the most obvious soundness violation and should be caught by basic effect-row validation. The checker likely enforces that if you call something with effect `e` in a function, your declared row must include `e`.

**Probability of soundness break**: LOW (5%)

---

### **Attack 4: Container Type Extraction — Result Effect Lost**

**File**: `attack4.delulu`

**Construct**: `Result[fn() ! {Write}, Str]` extracted from `Ok()`.

**Exploit**: Effects wrapped inside container types (Result, Option) might not be preserved when extracted. If the type system treats `Result[fn() ! {Write}, Str]` as an opaque box, extracting the function might lose its effect annotation.

```delulu
fn get_callback() -> Result[fn() ! {Write}, Str] ! {} {
    Ok(fn() { out.println("...") })
}

fn main(root: Root) ! {} {
    match get_callback() {
        Ok(callback) => callback()  // Effect lost in extraction?
    }
}
```

**Sound behavior**: The type `Result[fn() ! {Write}, Str]` preserves the effect in its type parameter; extracting the function preserves `! {Write}`.

**Unsound behavior**: Extraction treats the function as if it were `fn() ! {}`.

**Prediction**: **Likely SAFE** — Container types should be transparent to the type system; they're just wrappers. The effect row is part of the function type, not a property of the Result.

**Probability of soundness break**: LOW (5-15%)

---

### **Attack 5: Double-Wrapped Generic Row Polymorphism**

**File**: `attack5.delulu`

**Construct**: Two levels of generic row parameters: `outer[e1](g: fn(fn() ! e1 -> Int) -> Int ! e1)` calling `inner[e2]`.

**Exploit**: At higher nesting levels, unifying row parameters across function boundaries becomes complex. The row `e1` in the inner function might not be correctly linked to the effect constraint in the outer callback.

```delulu
fn outer[e1](g: fn(fn() ! e1 -> Int) -> Int ! e1) -> Int ! e1 { ... }
fn inner[e2](h: fn() ! e2) -> Int ! e2 { ... }

// Chaining these with an effectful callback.
```

**Sound behavior**: Effects propagate correctly through the entire chain.

**Unsound behavior**: A row parameter is instantiated incorrectly at a layer boundary.

**Prediction**: **Uncertain** — This is more complex and could expose issues in nested instantiation. However, if the checker handles basic generics correctly, it likely handles nesting too.

**Probability of soundness break**: LOW-MEDIUM (10-20%)

---

### **Attack 6: Struct Field Effect Tracking**

**File**: `attack6.delulu`

**Construct**: `struct Wrapper[T, e] { action: fn() -> T ! e }` instantiated with an effectful function.

**Exploit**: When a struct with a generic row field is instantiated, the effect might not be tracked. If the struct instantiation validation doesn't check that the provided field matches its generic type constraint, an effectful function could be stored in a field that should hold a pure function.

```delulu
struct Wrapper[T, e] { action: fn() -> T ! e }

fn main(root: Root) ! {} {
    let wrapper = Wrapper[Int, {Write}] {
        action: fn() { out.println("..."); 42 }
    }
    wrapper.action()  // Effect lost?
}
```

**Sound behavior**: Struct instantiation type-checks the fields; accessing `wrapper.action()` includes its effect row `! {Write}`.

**Unsound behavior**: Struct field effect rows are not validated or not propagated to the caller.

**Prediction**: **Likely SAFE** — Struct fields are type-checked at instantiation; effects in types should be preserved through struct access. This is standard type system behavior.

**Probability of soundness break**: LOW (5%)

---

### **Attack 7: Multi-Layer Callback Chain Unification**

**File**: `attack7.delulu`

**Construct**: Two functions with identical row parameters that pass callbacks through each other: `level1[e](f: fn(Int) -> Int ! e) -> Int ! e` calling `level2[e](g: fn(Int) -> Int ! e) -> Int ! e`.

**Exploit**: Row unification might fail at the transition between `level1` and `level2`. If the checker treats row parameters as "any effect" instead of "the effect of the callback I received," effects could decouple at a layer.

```delulu
fn level1[e](f: fn(Int) -> Int ! e) -> Int ! e { level2(f) }
fn level2[e](g: fn(Int) -> Int ! e) -> Int ! e { g(10) }

// Call with effectful callback.
```

**Sound behavior**: The callback's effect is unified across all layer boundaries.

**Unsound behavior**: A layer treats `e` as unconstrained instead of unifying it with the input callback's effect.

**Prediction**: **Likely SAFE** — This is a straightforward row propagation pattern that should work if basic generics work. If Attack 1 is safe, this should be too.

**Probability of soundness break**: LOW (5%)

---

### **Attack 8: Variance Inversion in Row Parameters**

**File**: `attack8.delulu`

**Construct**: A function that returns `fn() -> Int ! {}` but actually returned a function with hidden effects.

**Exploit**: If row parameters have incorrect variance, it might be possible to assign an effectful function to a position expecting a pure function. This is a type-theoretic soundness break where contravariance would normally be required but covariance is used.

```delulu
fn create_impure_caller(root: Root) -> fn() -> Int ! {} {
    // Return function with hidden Write effects, but type says ! {}
}

fn invoke_pure[T](f: fn() -> T ! {}) -> T {
    f()  // Called thinking it's pure
}

fn main(root: Root) ! {} {
    let impure = create_impure_caller(root)
    invoke_pure(impure)  // Soundness break if impure is accepted
}
```

**Sound behavior**: `create_impure_caller` cannot return a function with effects wrapped in a pure type. The return type annotation prevents this.

**Unsound behavior**: The function compiles and returns an effectful function despite declaring `! {}`.

**Prediction**: **Unlikely Safe** — This attack requires the function definition itself to lie about its return type. The checker should catch that `create_impure_caller` doesn't actually return `fn() -> Int ! {}`; it returns something with effects. This is a definition-level type error, not a complex inference bug.

**Probability of soundness break**: VERY LOW (2-3%)

---

## Summary

| Attack | Technique | Complexity | Soundness Risk | Prediction |
|--------|-----------|-----------|----------------|-----------|
| 1 | Generic row instantiation | Low | Low | Safe |
| 2 | Closure capture tracking | Medium | Medium | Uncertain |
| 3 | Lying with generic row | Low | Low | Safe |
| 4 | Container type extraction | Low | Low | Safe |
| 5 | Nested generic rows | High | Medium | Likely Safe |
| 6 | Struct field effects | Medium | Low | Safe |
| 7 | Chain unification | Medium | Low | Safe |
| 8 | Variance in function types | High | Very Low | Safe |

---

## Expected Compilation Results

**Most likely outcome**: Attacks 1, 3, 4, 6, 7, 8 compile to errors with DL0501 (undeclared effects).

**Uncertain cases**: Attack 2 (closure captures) and Attack 5 (nested generics) are the most plausible places for soundness breaks.

**If all compile**: The entire effect system has a critical soundness hole, likely in closure handling or deep generic instantiation.

---

## Testing Methodology

Each program should be tested by:

1. Running `delulu check <attack_N>.delulu` — does it compile without DL0501?
2. If it compiles, running the program — does it print "ESCAPED"?
3. Running `delulu authority <attack_N>.delulu` — does it report the actual effects or fewer?

The key question for each: Does the reported authority match what the program actually does?

---

## Notes on Attack Design

These attacks target realistic gaps in effect-system implementations:

- **Parameter effect hiding**: Generics are complex; effects might not propagate through all parameter combinations.
- **Closure escaping**: Closures capture state but their effects might not be flow-sensitive.
- **Container transparency**: Effects in type arguments might not be visible to the type system.
- **Nested instantiation**: At deeper nesting levels, constraint propagation can break.
- **Variance bugs**: Function type variance is subtle and often has bugs across languages.

The attacks are ordered roughly by:
1. Most likely to fail safely (Attacks 1, 3, 4, 6)
2. Uncertain (Attacks 2, 5)
3. Least likely to break (Attacks 7, 8)

---

## Red-Team Conclusion

DeluluLang's effect system appears robust for the common case (Effect 1 in the example). However, closures (Attack 2) and nested generic effects (Attack 5) are realistic attack surfaces. If none of these break the system, the effect checker is significantly more rigorous than typical capability-based languages.
