# Language Pack — `pt-BR` (Português do Brasil / Brazilian Portuguese) — STARTER

**Status: starter/overview.** Terminology decided; Opus 4.8 (or any human/AI) completes the catalog
from `en-US.md` §3 + `LOCALIZATION_PLUGIN_GUIDE.md`. Glosses per pack rule #4. Prose only.

```toml
[meta]
locale = "pt-BR"
name = "Português (Brasil)"
fallback = "en-US"
version = "1.0.0"
coverage = { cli = 100, diagnostics = 100, explain = 15 }
direction = "ltr"
```

## 1. Script notes
Latin, LTR. Brazilian (not European) conventions — the larger developer community; a pt-PT variant
may fork later.

## 2. Concept vocabulary (term — English gloss)
autoridade (authority) · efeito (effect) · linha de efeitos (effect row) · capacidade / Cap
(capability) · capacidade raiz (Root) · concessão (grant) · atenuação ⊑ (attenuation) · segredo /
Secret (secret) · escopo da capacidade (capability scope) · plugin (plugin) · ator (actor) ·
capacidade de referência (reference capability) · intermediador / broker (broker) · manifesto
(manifest) · diagnóstico (diagnostic) · reparo (repair).

## 3. Representative catalog strings (Opus completes the rest)
```toml
[DL0501]
message = "a função `{fn}` realiza `{effect}`, mas sua linha de efeitos não permite"
label   = "esta chamada realiza `{effect}`"
help    = "adicione `{effect}` à linha de efeitos (isso amplia a autoridade; o CI pode vetar) ou remova a chamada"

[DL0602]
message = "um valor `Secret` não pode fluir aqui — `Secret[{inner}]` não é `{inner}`"
help    = "um segredo só sai por `expose`, que carrega o efeito `Declassify`"

[cli.grant-prompt.title]
message = "este programa solicita: {summary} — permitir?"

[cli.authority.header]
message = "Autoridade de `{program}` — o que este programa pode fazer com o seu sistema:"

[cli.locale-picker.prompt]
message = "escolha a voz do seu compilador (muda apenas o texto humano — códigos e JSON nunca mudam):"
```

## 4. Register
Informal *você* (norm in Brazilian dev tooling); Latin tokens verbatim. Honesty clauses bind. Welcome
note untranslated (§6.3).
