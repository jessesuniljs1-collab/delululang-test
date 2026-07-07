# Language Pack — `hi-IN` (हिन्दी / Hindi) — STARTER

**Status: starter/overview.** Terminology decided here; Opus 4.8 (or any human/AI) completes the full
catalog from `en-US.md` §3 + `LOCALIZATION_PLUGIN_GUIDE.md`. All terms glossed in English. Prose only.

```toml
[meta]
locale = "hi-IN"
name = "हिन्दी"
fallback = "en-US"
version = "1.0.0"
coverage = { cli = 100, diagnostics = 100, explain = 15 }
direction = "ltr"
```

## 1. Script & rendering notes (Devanagari)
Devanagari uses **combining marks** (मात्रा / matras) and conjunct consonants: one visual cluster can
be several Unicode code points. The CLI/LSP panel layout must measure **grapheme-cluster display
width**, not code-point or byte count (Unicode UAX #29 segmentation) — this is the key loader
requirement for this pack. LTR. English/Latin technical tokens are common in Indian developer prose
and stay verbatim.

## 2. Concept vocabulary (term — English gloss)
| en-US | hi-IN | gloss |
|---|---|---|
| authority | अधिकार (adhikār) | what code may do to the system |
| effect | प्रभाव (prabhāv) | observable world interaction |
| effect row | प्रभाव पंक्ति | the effect set in a type `! {Read}` |
| capability | क्षमता (Cap) | unforgeable single-authority value |
| Root | मूल क्षमता | the one capability `main` receives |
| grant | अनुदान (anudān) | authority handed to code at runtime |
| attenuation ⊑ | न्यूनीकरण | narrowing authority; less, never more |
| secret | गोपनीय (Secret) | opaque value; crosses only via `expose` |
| capability scope | क्षमता क्षेत्र | concrete bound (paths/hosts/envelope) |
| plugin | प्लगइन | runtime code bounded by its grant |
| actor | ऐक्टर | isolated concurrent state, message-only |
| reference capability | संदर्भ क्षमता | who may alias/mutate/send |
| broker | ब्रोकर | out-of-process authority holder |
| manifest | मैनिफ़ेस्ट | `delulu.toml`, the authority ceiling |
| diagnostic | निदान (nidān) | compiler message with a stable code |
| repair | सुधार (sudhār) | machine-applicable fix |

## 3. Representative catalog strings (Opus completes the rest)
```toml
[DL0501]
message = "फ़ंक्शन `{fn}` `{effect}` करता है, परन्तु उसकी प्रभाव पंक्ति इसकी अनुमति नहीं देती"
label   = "यह कॉल `{effect}` करता है"
help    = "`{effect}` को प्रभाव पंक्ति में जोड़ें (इससे अधिकार बढ़ता है, CI अस्वीकार कर सकता है) या कॉल हटाएँ"

[DL0602]
message = "`Secret` मान यहाँ प्रवाहित नहीं हो सकता — `Secret[{inner}]`, `{inner}` नहीं है"
help    = "गोपनीय मान केवल `expose` से बाहर आता है, जो `Declassify` प्रभाव वहन करता है"

[DL0802]
message = "अनुरोधित अधिकार धारक के अनुदान से अधिक है"
help    = "अनुरोध को न्यून करें; सुधार में सटीक प्रतिच्छेदन दिया गया है"

[cli.grant-prompt.title]
message = "यह प्रोग्राम माँगता है: {summary} — क्या आप अनुमति देते हैं?"

[cli.authority.header]
message = "`{program}` का अधिकार — यह प्रोग्राम आपके सिस्टम के साथ क्या कर सकता है:"

[cli.locale-picker.prompt]
message = "अपने कंपाइलर का लहजा चुनें (केवल मानव-पाठ बदलता है — कोड और JSON कभी नहीं):"
```

## 4. Register
Formal Hindi (आप-form), Sanskrit-derived technical vocabulary where established, Latin tokens
verbatim. Honesty clauses bind. Welcome note untranslated (§6.3).
