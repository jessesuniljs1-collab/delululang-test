# Secret Exfiltration Attack Notes

## Overview
This document describes 8 attempts to extract plaintext from a `Secret[Str]` while avoiding an explicit `Declassify` declaration in the effect row. The secret is created as `root.secret("api_key")`.

The core insight: `Secret[Str].map(fn(x) {...})` gives the closure the plaintext `x`. If the closure's transformed value can be observed via `verify()`, information about the plaintext leaks.

---

## Attack 1: Direct map + oracle leakage

**File:** `attack1.delulu`

**Technique:**
```
let transformed = sec.map(fn(x) { len(x).to_str() + ":" + x })
transformed.verify("7:api_key")
```

**Theory:** The map closure receives the plaintext and returns a string containing it (wrapped as `Secret[Str]`). We then verify against a full guess. If the secret's plaintext is "api_key" (7 chars), the guess should match.

**Expected Outcome:**
- **System Works:** Compile error DL0603/DL0605 because map returns a Secret[Str] and we're constructing a string from plaintext within the closure, which violates the "no construction of secrets from user-controlled strings" rule.
- **Attack Works:** If verify("7:api_key") succeeds, we've confirmed the secret's value. No Declassify declared in row.

**Prediction:** DL0603 or similar — the closure should be rejected for attempting to construct a string embedding plaintext.

---

## Attack 2: IF-1 bit-by-bit oracle (the classic)

**File:** `attack2.delulu`

**Technique:**
```
let transformed = sec.map(fn(x) {
    if x.starts_with(ch) { "1" } else { "0" }
})
transformed.verify("1")
```

**Theory:** The map closure tests if the secret starts with a candidate character, returning "1" or "0". We then verify against "1". By iterating over ASCII characters, we can extract the first character bit-by-bit.

This is the historical **IF-1** vulnerability: the verify() call carries Declassify as an effect, but if hidden inside a map that doesn't require declaring it, information leaks.

**Expected Outcome:**
- **System Works:** The closure is rejected, or the entire pattern is flagged as Declassify-carrying even though main's row doesn't declare it.
- **Attack Works:** We can extract the first character (and generalize to the whole string) without declaring Declassify. `delulu authority` shows no `Declassify` in the row, but the program reveals the secret.

**Prediction:** This is the most likely to succeed if map's closure isn't strictly type-checked. DL0501 error because main doesn't declare Declassify but calls verify (which requires it). HOWEVER, if verify is considered a "pure" method on Secret, it might not trigger the row check — this is the vulnerability.

---

## Attack 3: Closure side effects in map

**File:** `attack3.delulu`

**Technique:**
```
let transformed = sec.map(fn(x) {
    writer.write_text("./leaked.txt", "Secret: " + x)
    "processed"
})
```

**Theory:** The closure captures the `writer` capability and writes the plaintext to a file. Even though map returns a Secret[Str], the side effect (Write) performs the exfiltration.

**Expected Outcome:**
- **System Works:** Closures cannot capture capabilities without explicit parameter passing, or Write isn't declared in main's row and gets rejected at compile time.
- **Attack Works:** The file `leaked.txt` contains "Secret: api_key" after map executes, even though main declares `! {Read, Write}` — the string "Secret: " + x isn't flagged as constructing a secret.

**Prediction:** Compile error because `writer` isn't in scope within the closure, or the capability-passing rules forbid it. The Write effect is already declared, so if the closure could execute, it might not add a new effect requirement.

---

## Attack 4: Hash/equality oracle

**File:** `attack4.delulu`

**Technique:**
```
let hashed = sec.map(fn(x) { x.reverse() })
hashed.verify(candidate.reverse())
```

**Theory:** Similar to Attack 1 and 2, but applies a simple transformation (reverse) to both the secret and candidate guesses. If the secret is "api_key" and we guess the reversed form "yek_ipa", verify should succeed.

**Expected Outcome:**
- **System Works:** Same as Attack 1 — closure construction of strings from plaintext is rejected.
- **Attack Works:** We extract the plaintext by guessing its reverse.

**Prediction:** Compile error for same reasons as Attack 1. If it compiles, the program leaks the secret via verify().

---

## Attack 5: Store secret in a List and extract elsewhere

**File:** `attack5.delulu`

**Technique:**
```
let sec_list: List[Secret[Str]] = [sec]
let extracted = sec_list.get(0)
```

**Theory:** By storing the Secret in a List and extracting it, we might bypass access control tied to the point of use. Or, by mapping over the List, we can test the secret within the map closure without declaring Declassify.

**Expected Outcome:**
- **System Works:** Extraction from the list returns the Secret, and attempting to verify it still requires Declassify in main's row.
- **Attack Works:** If the List.map() closure isn't subject to Declassify checks, we can test elements there.

**Prediction:** Compile errors in the code (syntax for list creation, the `???` default, etc.), but if fixed, the verify() call within the List.map() closure should still require Declassify in the outer scope.

---

## Attack 6: Conditional leakage via map (pattern matching)

**File:** `attack6.delulu`

**Technique:**
```
let encoded = sec.map(fn(x) {
    match x {
        "api_key" => "FOUND_API_KEY",
        "password" => "FOUND_PASSWORD",
        _ => "UNKNOWN"
    }
})
encoded.verify("FOUND_API_KEY")
```

**Theory:** The closure uses pattern matching to branch on the plaintext's value, returning a fixed string for each possibility. We then verify which branch was taken. This is a controlled dictionary attack: we predefine a set of possible secrets and let the closure encode which one matched.

**Expected Outcome:**
- **System Works:** Pattern matching inside map is rejected, or the closure's branching is analysed to prove only fixed strings are returned, which doesn't leak the plaintext.
- **Attack Works:** We identify the secret by verifying the output tag.

**Prediction:** Compile error for same reasons as Attack 1. If it compiles and runs, verify("FOUND_API_KEY") succeeds, revealing the secret without Declassify in the row.

---

## Attack 7: Pass-through function hiding Declassify

**File:** `attack7.delulu`

**Technique:**
```
fn test_secret_hidden(s: Secret[Str], guess: Str) -> Bool ! {Read} {
    s.verify(guess)  // verify() requires Declassify
}

fn main(root: Root) ! {Read, Write} {
    test_secret_hidden(sec, "api_key")  // No Declassify in main's row
}
```

**Theory:** The helper function's row declares only `Read`, even though it calls verify() (which requires `Declassify`). The hope is that main can call test_secret_hidden without declaring Declassify, hiding the effect.

**Expected Outcome:**
- **System Works:** Compile error DL0501: test_secret_hidden's row doesn't include Declassify, but it calls verify() which requires it.
- **Attack Works:** If the row check is incomplete, main can call test_secret_hidden without declaring Declassify, and the secret is revealed.

**Prediction:** DL0501 compile error when checking test_secret_hidden — verify() is called but not in the row. The row is REQUIRED to include all effects the function performs. However, if verify() is special-cased as "part of the Secret type" and not checked as a regular method call, this might leak.

---

## Attack 8: Multi-stage transformation chain

**File:** `attack8.delulu`

**Technique:**
```
let stage1 = sec.map(fn(x) { x.substring(0, 1) })
let stage2 = stage1.map(fn(x) {
    match x {
        "a" => "FOUND_A",
        "p" => "FOUND_P",
        _ => "UNKNOWN"
    }
})
stage2.verify("FOUND_P")
```

**Theory:** Chain multiple map operations to extract information in stages. First map extracts the first character, second map classifies it. This might evade detection by spreading the leakage across multiple operations.

**Expected Outcome:**
- **System Works:** Each map is checked independently, and the classifier (stage2) still requires Declassify.
- **Attack Works:** If each map is checked naively, the pipeline succeeds and we extract characters without declaring Declassify.

**Prediction:** Compile error at stage2.verify() call because main's row doesn't include Declassify. However, if the map operations are not flow-sensitive, the compiler might not notice that stage2 is derived from the secret and thus requires Declassify to use.

---

## Summary of Predictions

| Attack | Mechanism | Most Likely Outcome |
|--------|-----------|---------------------|
| 1 | Direct transformation + verify | **DL0603/0605**: Closure construction from plaintext rejected |
| 2 | IF-1 bit-by-bit oracle | **DL0501**: verify() carries Declassify, not declared in main |
| 3 | Closure side effects | **Compile error**: Capability not in closure scope |
| 4 | Hash oracle | **DL0603/0605**: Like Attack 1 |
| 5 | List escape | **DL0501**: verify() from extracted secret requires Declassify |
| 6 | Pattern matching oracle | **DL0603/0605**: Pattern match on plaintext in closure rejected |
| 7 | Hidden Declassify in helper | **DL0501**: test_secret_hidden's row incomplete |
| 8 | Multi-stage pipeline | **DL0501**: Final verify() requires Declassify in main's row |

## Key Hypotheses About the Language

1. **map() closure receives plaintext**: This is stated in the spec. The closure is where information is most vulnerable.

2. **verify() carries Declassify**: The EXAMPLE says so explicitly. The effect must appear in any function that calls it.

3. **Effects are checked per-function**: If a function calls an effect-requiring operation, it MUST declare that effect in its row. This is DL0501.

4. **Closure flow analysis**: The question is whether the compiler realizes that a Secret returned by map (which received plaintext) is still a Secret and thus requires Declassify to use verify() on it.

5. **String construction from plaintext**: The language may forbid constructing strings from a secret's plaintext (DL0603/0605), or it may allow it if the result is wrapped in a Secret.

## Danger Zones (If Attacks Succeed)

- **IF-1 (Attack 2)**: The verify() method might not be flow-typed into the row check. If main calls a function that calls verify(), and main doesn't declare Declassify, it's a leak.
- **Closure leakage (Attacks 1, 4, 6, 8)**: If map's closure can return a Secret wrapping user-controlled data derived from the plaintext, verify-ing it leaks info without proving Declassify in the caller's row.
- **Effect hiding (Attack 7)**: If function row checking doesn't recursively verify that all callees' effects are in the caller's row, effects can be hidden.

---

## Notes for Reproduction

All attacks declare `! {Read, Write}` only. None declare `Declassify`. If any program:
1. Compiles without error
2. Runs and produces output indicating the secret's value (e.g., "LEAK: secret verified as...")
3. `delulu authority` shows no `Declassify` in its row

Then that attack breaks the language's promise that authority is visible.

