# Language Pack — `de-DE` (Deutsch / German) — STARTER

**Status: starter/overview.** Terminology decided; Opus 4.8 (or any human/AI) completes the catalog
from `en-US.md` §3 + `LOCALIZATION_PLUGIN_GUIDE.md`. Glosses per pack rule #4. Prose only.

```toml
[meta]
locale = "de-DE"
name = "Deutsch"
fallback = "en-US"
version = "1.0.0"
coverage = { cli = 100, diagnostics = 100, explain = 15 }
direction = "ltr"
```

## 1. Script notes
Latin, LTR. German compounds can be long — prefer the established loanword where the compound would
be unwieldy; keep panel-width limits in mind.

## 2. Concept vocabulary (term — English gloss)
Befugnis (authority) · Effekt (effect) · Effektzeile (effect row) · Capability / Cap (capability —
established loanword) · Wurzel-Capability (Root) · Gewährung (grant) · Abschwächung ⊑ (attenuation) ·
Geheimnis / Secret (secret) · Geltungsbereich (capability scope) · Plugin (plugin) · Aktor (actor) ·
Referenz-Capability (reference capability) · Broker (broker) · Manifest (manifest) · Diagnose
(diagnostic) · Reparatur (repair).

## 3. Representative catalog strings (Opus completes the rest)
```toml
[DL0501]
message = "Funktion `{fn}` führt `{effect}` aus, aber ihre Effektzeile erlaubt das nicht"
label   = "dieser Aufruf führt `{effect}` aus"
help    = "füge `{effect}` der Effektzeile hinzu (das erweitert die Befugnis; die CI kann ablehnen) oder entferne den Aufruf"

[DL0602]
message = "ein `Secret`-Wert darf hier nicht fließen — `Secret[{inner}]` ist nicht `{inner}`"
help    = "ein Geheimnis verlässt seine Hülle nur über `expose`, das den Effekt `Declassify` trägt"

[cli.grant-prompt.title]
message = "dieses Programm verlangt: {summary} — erlauben?"

[cli.authority.header]
message = "Befugnis von `{program}` — was dieses Programm mit deinem System tun kann:"

[cli.locale-picker.prompt]
message = "wähle die Stimme deines Compilers (nur menschlicher Text — Codes und JSON bleiben unverändert):"
```

## 4. Register
Informal *du* (the norm in German developer tooling), precise technical German; Latin tokens
verbatim. Honesty clauses bind. Welcome note untranslated (§6.3).
