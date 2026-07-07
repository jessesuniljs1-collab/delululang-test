# Language Pack — `zh-CN` (简体中文 / Simplified Chinese) — STARTER

**Status: starter/overview.** Terminology is *decided* here; the implementing model (Opus 4.8, or any
human/AI) completes the full catalog from `en-US.md` §3's key list + `LOCALIZATION_PLUGIN_GUIDE.md`.
Every term is glossed in English (pack rule #4). Prose only — codes/JSON/spans never change.

```toml
[meta]
locale = "zh-CN"
name = "简体中文"
fallback = "en-US"
version = "1.0.0"
coverage = { cli = 100, diagnostics = 100, explain = 20 }
direction = "ltr"
```

## 1. Script & rendering notes
Han characters are **double-width** — the CLI/LSP panel layout must measure *display width*, not
char count (already required by the loader, §5 of the guide). No combining marks, no RTL. Terminal
font must include CJK glyphs; degrade to en-US if unavailable is acceptable (fallback chain).

## 2. Concept vocabulary (term — English gloss)
| en-US | zh-CN | gloss |
|---|---|---|
| authority | 权限 (quánxiàn) | what code may do to the system |
| effect | 作用 | observable world interaction |
| effect row | 作用行 | the set of effects in a type `! {Read}` |
| capability | 能力 (Cap) | unforgeable single-authority value |
| Root | 根能力 | the one capability `main` receives |
| grant | 授予 | authority handed to code at runtime |
| attenuation ⊑ | 衰减 | narrowing authority; give less never more |
| secret | 秘密 (Secret) | opaque value; crosses only via `expose` |
| capability scope | 能力范围 | the concrete bound (paths/hosts/envelope) |
| plugin | 插件 | runtime code still bounded by its grant |
| actor | 参与者 / actor | isolated concurrent state, message-only |
| reference capability | 引用能力 | who may alias/mutate/send a value |
| broker | 代理 (broker) | out-of-process authority holder |
| manifest | 清单 | `delulu.toml`, the authority ceiling |
| diagnostic | 诊断 | compiler message with a stable code |
| repair | 修复 | machine-applicable fix on a diagnostic |

**Keywords are NOT translated here** (that is a syntax morph — `SYNTAX_MORPH_SPEC.md`). If a `zh-CN`
*human keyword morph* is desired, this table's terms are the suggested aliases (e.g. `fn`→`函数`),
but that ships separately from this prose catalog.

## 3. Representative catalog strings (Opus completes the rest)
```toml
[DL0501]
message = "函数 `{fn}` 执行了 `{effect}`，但其作用行未允许"
label   = "此调用执行 `{effect}`"
help    = "将 `{effect}` 加入作用行（这会扩大权限，CI 可能否决），或移除该调用"

[DL0602]
message = "`Secret` 值不能流向此处 —— `Secret[{inner}]` 不是 `{inner}`"
help    = "秘密只能通过 `expose` 越界，且 `expose` 携带 `Declassify` 作用"

[DL0802]
message = "请求的权限超出持有者的授予范围"
help    = "请衰减该请求；修复项给出精确的交集"

[cli.grant-prompt.title]
message = "此程序请求：{summary} —— 是否允许？"

[cli.authority.header]
message = "`{program}` 的权限 —— 此程序能对你的系统做什么："

[cli.locale-picker.prompt]
message = "选择编译器的语气（只改人类可读文本 —— 代码与 JSON 永不改变）："
```

## 4. Register
Neutral-formal technical Mandarin; keep Latin technical tokens (`DL0501`, `Cap`, `expose`, effect
names) verbatim. Honesty clauses bind (no superlatives). Welcome note untranslated (§6.3).
