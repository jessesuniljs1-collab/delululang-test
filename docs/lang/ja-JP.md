# Language Pack — `ja-JP` (日本語 / Japanese) — STARTER

**Status: starter/overview.** Terminology decided here; Opus 4.8 (or any human/AI) completes the full
catalog from `en-US.md` §3 + `LOCALIZATION_PLUGIN_GUIDE.md`. All terms glossed in English. Prose only.

```toml
[meta]
locale = "ja-JP"
name = "日本語"
fallback = "en-US"
version = "1.0.0"
coverage = { cli = 100, diagnostics = 100, explain = 20 }
direction = "ltr"
```

## 1. Script & rendering notes
Kanji/kana are **double-width** (display-width layout, as zh-CN). Mixed scripts (kanji + katakana
loanwords + Latin technical tokens) are normal in Japanese technical prose — do not force-translate
established loanwords.

## 2. Concept vocabulary (term — English gloss)
| en-US | ja-JP | gloss |
|---|---|---|
| authority | 権限 (kengen) | what code may do to the system |
| effect | 作用 (sayō) | observable world interaction |
| effect row | 作用行 | the effect set in a type `! {Read}` |
| capability | ケイパビリティ (Cap) | unforgeable single-authority value (established loanword) |
| Root | ルート能力 | the one capability `main` receives |
| grant | 付与 (fuyo) | authority handed to code at runtime |
| attenuation ⊑ | 減衰 (gensui) | narrowing authority; less, never more |
| secret | シークレット (Secret) | opaque value; crosses only via `expose` |
| capability scope | 権限範囲 | concrete bound (paths/hosts/envelope) |
| plugin | プラグイン | runtime code bounded by its grant |
| actor | アクター | isolated concurrent state, message-only |
| reference capability | 参照ケイパビリティ | who may alias/mutate/send |
| broker | ブローカー | out-of-process authority holder |
| manifest | マニフェスト | `delulu.toml`, the authority ceiling |
| diagnostic | 診断 (shindan) | compiler message with a stable code |
| repair | 修復 (shūfuku) | machine-applicable fix |

## 3. Representative catalog strings (Opus completes the rest)
```toml
[DL0501]
message = "関数 `{fn}` は `{effect}` を実行していますが、作用行で許可されていません"
label   = "この呼び出しは `{effect}` を実行します"
help    = "作用行に `{effect}` を追加する（権限が拡大するため CI が拒否する可能性があります）か、呼び出しを削除してください"

[DL0602]
message = "`Secret` の値をここに渡すことはできません — `Secret[{inner}]` は `{inner}` ではありません"
help    = "秘密は `expose` を通じてのみ取り出せます（`Declassify` 作用を伴います）"

[DL0802]
message = "要求された権限は保持者の付与範囲を超えています"
help    = "要求を減衰させてください。修復として正確な共通部分を提示します"

[cli.grant-prompt.title]
message = "このプログラムの要求：{summary} — 許可しますか？"

[cli.authority.header]
message = "`{program}` の権限 — このプログラムがシステムに対してできること："

[cli.locale-picker.prompt]
message = "コンパイラの口調を選んでください（人間向けテキストのみ変更 — コードと JSON は不変）："
```

## 4. Register
Polite-form (です/ます調) for CLI/diagnostics — standard for Japanese developer tools. Keep Latin
technical tokens verbatim. Honesty clauses bind. Welcome note untranslated (§6.3).
