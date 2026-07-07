# Language Pack — `ko-KR` (한국어 / Korean) — STARTER

**Status: starter/overview.** Terminology decided here; Opus 4.8 (or any human/AI) completes the full
catalog from `en-US.md` §3 + `LOCALIZATION_PLUGIN_GUIDE.md`. All terms glossed in English. Prose only.

```toml
[meta]
locale = "ko-KR"
name = "한국어"
fallback = "en-US"
version = "1.0.0"
coverage = { cli = 100, diagnostics = 100, explain = 20 }
direction = "ltr"
```

## 1. Script & rendering notes
Hangul syllable blocks are **double-width** (display-width layout, as zh/ja). Latin technical tokens
mix freely in Korean developer prose; established loanwords are normal.

## 2. Concept vocabulary (term — English gloss)
| en-US | ko-KR | gloss |
|---|---|---|
| authority | 권한 (gwonhan) | what code may do to the system |
| effect | 효과 (hyogwa) | observable world interaction |
| effect row | 효과 행 | the effect set in a type `! {Read}` |
| capability | 케이퍼빌리티 (Cap) | unforgeable single-authority value (loanword) |
| Root | 루트 능력 | the one capability `main` receives |
| grant | 부여 (buyeo) | authority handed to code at runtime |
| attenuation ⊑ | 감쇠 (gamsoe) | narrowing authority; less, never more |
| secret | 시크릿 (Secret) | opaque value; crosses only via `expose` |
| capability scope | 권한 범위 | concrete bound (paths/hosts/envelope) |
| plugin | 플러그인 | runtime code bounded by its grant |
| actor | 액터 | isolated concurrent state, message-only |
| reference capability | 참조 케이퍼빌리티 | who may alias/mutate/send |
| broker | 브로커 | out-of-process authority holder |
| manifest | 매니페스트 | `delulu.toml`, the authority ceiling |
| diagnostic | 진단 (jindan) | compiler message with a stable code |
| repair | 수정 (sujeong) | machine-applicable fix |

## 3. Representative catalog strings (Opus completes the rest)
```toml
[DL0501]
message = "함수 `{fn}`이(가) `{effect}`을(를) 수행하지만 효과 행에서 허용되지 않았습니다"
label   = "이 호출은 `{effect}`을(를) 수행합니다"
help    = "효과 행에 `{effect}`을(를) 추가하거나(권한이 확대되므로 CI가 거부할 수 있음) 호출을 제거하세요"

[DL0602]
message = "`Secret` 값은 여기로 전달될 수 없습니다 — `Secret[{inner}]`은(는) `{inner}`이(가) 아닙니다"
help    = "비밀 값은 `expose`를 통해서만 꺼낼 수 있으며, 이때 `Declassify` 효과가 발생합니다"

[DL0802]
message = "요청한 권한이 보유자의 부여 범위를 초과합니다"
help    = "요청을 감쇠하세요. 수정 항목으로 정확한 교집합을 제시합니다"

[cli.grant-prompt.title]
message = "이 프로그램의 요청: {summary} — 허용하시겠습니까?"

[cli.authority.header]
message = "`{program}`의 권한 — 이 프로그램이 시스템에 할 수 있는 일:"

[cli.locale-picker.prompt]
message = "컴파일러의 말투를 선택하세요 (사람이 읽는 텍스트만 변경 — 코드와 JSON은 불변):"
```

## 4. Register
Formal polite (합쇼체/해요체 mixed as in standard dev tools; -습니다 for diagnostics). Korean
particle alternation (이/가, 을/를, 은/는) depends on the preceding word's final sound — with runtime
placeholders use the `이(가)` dual-form convention shown above. Keep Latin tokens verbatim. Honesty
clauses bind. Welcome note untranslated (§6.3).
