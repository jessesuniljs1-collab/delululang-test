# Language Pack — `fr-FR` (Français / French) — STARTER

**Status: starter/overview.** Terminology decided; Opus 4.8 (or any human/AI) completes the catalog
from `en-US.md` §3 + `LOCALIZATION_PLUGIN_GUIDE.md`. Glosses per pack rule #4. Prose only.

```toml
[meta]
locale = "fr-FR"
name = "Français (France)"
fallback = "en-US"
version = "1.0.0"
coverage = { cli = 100, diagnostics = 100, explain = 15 }
direction = "ltr"
```

## 1. Script notes
Latin, LTR; French quotation marks « » with narrow no-break spaces are the correct typography for
quoted identifiers in prose; plain backticks around code tokens as in en-US.

## 2. Concept vocabulary (term — English gloss)
autorité (authority) · effet (effect) · ligne d'effets (effect row) · capacité / Cap (capability) ·
capacité racine (Root) · octroi (grant) · atténuation ⊑ (attenuation) · secret / Secret (secret) ·
portée de capacité (capability scope) · greffon / plugin (plugin) · acteur (actor) · capacité de
référence (reference capability) · courtier (broker) · manifeste (manifest) · diagnostic (diagnostic)
· réparation (repair).

## 3. Representative catalog strings (Opus completes the rest)
```toml
[DL0501]
message = "la fonction « {fn} » effectue « {effect} », mais sa ligne d'effets ne le permet pas"
label   = "cet appel effectue « {effect} »"
help    = "ajoutez « {effect} » à la ligne d'effets (cela élargit l'autorité ; la CI peut refuser) ou supprimez l'appel"

[DL0602]
message = "une valeur « Secret » ne peut pas circuler ici — « Secret[{inner}] » n'est pas « {inner} »"
help    = "un secret ne sort que par « expose », qui porte l'effet « Declassify »"

[cli.grant-prompt.title]
message = "ce programme demande : {summary} — autoriser ?"

[cli.authority.header]
message = "Autorité de « {program} » — ce que ce programme peut faire à votre système :"

[cli.locale-picker.prompt]
message = "choisissez la voix de votre compilateur (texte humain uniquement — codes et JSON inchangés) :"
```

## 4. Register
Vouvoiement (formal *vous*), technical French; Latin tokens verbatim. Honesty clauses bind. Welcome
note untranslated (§6.3).
