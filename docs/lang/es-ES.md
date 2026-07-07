# Language Pack — `es-ES` (Español / Spanish) — STARTER

**Status: starter/overview.** Terminology decided; Opus 4.8 (or any human/AI) completes the catalog
from `en-US.md` §3 + `LOCALIZATION_PLUGIN_GUIDE.md`. Glosses per pack rule #4. Prose only.
*(Neutral technical Spanish — usable across es-ES/es-419; a distinct es-419 variant may fork later.)*

```toml
[meta]
locale = "es-ES"
name = "Español"
fallback = "en-US"
version = "1.0.0"
coverage = { cli = 100, diagnostics = 100, explain = 15 }
direction = "ltr"
```

## 1. Script notes
Latin, LTR; opening ¿/¡ in questions/exclamations where natural.

## 2. Concept vocabulary (term — English gloss)
autoridad (authority) · efecto (effect) · fila de efectos (effect row) · capacidad / Cap (capability)
· capacidad raíz (Root) · concesión (grant) · atenuación ⊑ (attenuation) · secreto / Secret (secret)
· alcance de la capacidad (capability scope) · plugin (plugin) · actor (actor) · capacidad de
referencia (reference capability) · intermediario / broker (broker) · manifiesto (manifest) ·
diagnóstico (diagnostic) · reparación (repair).

## 3. Representative catalog strings (Opus completes the rest)
```toml
[DL0501]
message = "la función `{fn}` realiza `{effect}`, pero su fila de efectos no lo permite"
label   = "esta llamada realiza `{effect}`"
help    = "añade `{effect}` a la fila de efectos (esto amplía la autoridad; la CI puede vetarlo) o elimina la llamada"

[DL0602]
message = "un valor `Secret` no puede fluir aquí — `Secret[{inner}]` no es `{inner}`"
help    = "un secreto solo sale mediante `expose`, que lleva el efecto `Declassify`"

[cli.grant-prompt.title]
message = "este programa solicita: {summary} — ¿permitir?"

[cli.authority.header]
message = "Autoridad de `{program}` — lo que este programa puede hacer con tu sistema:"

[cli.locale-picker.prompt]
message = "elige la voz de tu compilador (solo cambia el texto humano — los códigos y el JSON nunca cambian):"
```

## 4. Register
Informal *tú* (the norm in Spanish developer tooling), neutral technical vocabulary; Latin tokens
verbatim. Honesty clauses bind. Welcome note untranslated (§6.3).
