# Language Pack — `ar-SA` (العربية / Arabic) — STARTER (RTL)

**Status: starter/overview.** Terminology decided here; Opus 4.8 (or any human/AI) completes the full
catalog from `en-US.md` §3 + `LOCALIZATION_PLUGIN_GUIDE.md`. All terms glossed in English. Prose only.
**This is the reference RTL pack** — its direction/bidi notes apply to any future RTL locale (Hebrew,
Farsi, Urdu).

```toml
[meta]
locale = "ar-SA"
name = "العربية"
fallback = "en-US"
version = "1.0.0"
coverage = { cli = 100, diagnostics = 100, explain = 12 }
direction = "rtl"          # the ONLY thing that turns on RTL rendering
```

## 1. Script & direction notes (RTL — the load-bearing part of this pack)
- `direction = "rtl"` tells the **human-text renderer** to align panels right and mirror box-drawing
  (Stage-8 loader responsibility, guide §5). It affects **only** the human-text path.
- **Source code, machine output, spans, and JSON stay LTR and unchanged** — a `.delulu` file is the
  same bytes regardless of the reader's locale; a byte-offset span means the same thing. RTL never
  touches the program.
- **Bidi mixing:** Arabic prose embedding Latin technical tokens (`DL0501`, `Cap`, `expose`, effect
  names, `{placeholders}`) is bidirectional. The renderer should rely on the Unicode Bidi Algorithm
  (UAX #9); authors **should not** insert manual RLM/LRM marks in catalog strings unless a specific
  token renders ambiguously — keep strings clean and let the terminal's bidi handle it. Test rendering
  in a real RTL-capable terminal.
- Arabic is **connected script with contextual letterforms**; treat display width by grapheme cluster.
  No double-width (unlike CJK).

## 2. Concept vocabulary (term — English gloss)
| en-US | ar-SA | gloss |
|---|---|---|
| authority | صلاحية (ṣalāḥiyya) | what code may do to the system |
| effect | تأثير (taʾthīr) | observable world interaction |
| effect row | صف التأثيرات | the effect set in a type `! {Read}` |
| capability | قدرة (Cap) | unforgeable single-authority value |
| Root | القدرة الجذرية | the one capability `main` receives |
| grant | منح (manḥ) | authority handed to code at runtime |
| attenuation ⊑ | تضييق (taḍyīq) | narrowing authority; less, never more |
| secret | سر (Secret) | opaque value; crosses only via `expose` |
| capability scope | نطاق القدرة | concrete bound (paths/hosts/envelope) |
| plugin | إضافة (iḍāfa) | runtime code bounded by its grant |
| actor | فاعل (fāʿil) / actor | isolated concurrent state, message-only |
| reference capability | قدرة مرجعية | who may alias/mutate/send |
| broker | وسيط (wasīṭ) | out-of-process authority holder |
| manifest | البيان (al-bayān) | `delulu.toml`, the authority ceiling |
| diagnostic | تشخيص (tashkhīṣ) | compiler message with a stable code |
| repair | إصلاح (iṣlāḥ) | machine-applicable fix |

## 3. Representative catalog strings (Opus completes the rest)
```toml
[DL0501]
message = "الدالة `{fn}` تنفّذ `{effect}` لكن صف التأثيرات لا يسمح بذلك"
label   = "هذا الاستدعاء ينفّذ `{effect}`"
help    = "أضف `{effect}` إلى صف التأثيرات (هذا يوسّع الصلاحية، وقد يرفضه CI) أو احذف الاستدعاء"

[DL0602]
message = "لا يمكن تمرير قيمة `Secret` هنا — `Secret[{inner}]` ليست `{inner}`"
help    = "السر يخرج فقط عبر `expose`، وهو يحمل تأثير `Declassify`"

[DL0802]
message = "الصلاحية المطلوبة تتجاوز ما يملكه المانح"
help    = "ضيّق الطلب؛ يقدّم الإصلاح التقاطع الدقيق"

[cli.grant-prompt.title]
message = "يطلب هذا البرنامج: {summary} — هل تسمح؟"

[cli.authority.header]
message = "صلاحية `{program}` — ما الذي يمكن لهذا البرنامج فعله بنظامك:"

[cli.locale-picker.prompt]
message = "اختر لهجة المُصرِّف (تغيّر النص البشري فقط — الرموز و JSON لا تتغيّر أبدًا):"
```

## 4. Register
Modern Standard Arabic, formal. Latin technical tokens and `{placeholders}` verbatim. Honesty clauses
bind. Welcome note untranslated (§6.3) — it renders LTR-as-authored even in an RTL panel (the note is
not a catalog string).
