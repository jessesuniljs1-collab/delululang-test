<!-- GENERATED FILE — DO NOT EDIT BY HAND.
     Written by `delulu-conform --reference` from `delulu_syntax::token` (the `TokenKind` enum and its keyword table).
     `delulu-conform --check-reference` fails the build if this file drifts from the source. -->

# Tokens

The complete lexical surface. Every row is a `TokenKind` variant; the
index is fenced against the enum by a drift guard in `token.rs`, so a token cannot
be added, renamed or removed without this chapter changing with it.

| Token | Lexeme | Description |
|---|---|---|
| `Ident` | *(literal)* | identifier `name` |
| `Int` | *(literal)* | integer `0` |
| `Float` | *(literal)* | float `0` |
| `Str` | *(literal)* | string literal |
| `KwFn` | `fn` | `fn` |
| `KwLet` | `let` | `let` |
| `KwVar` | `var` | `var` |
| `KwIf` | `if` | `if` |
| `KwElse` | `else` | `else` |
| `KwWhile` | `while` | `while` |
| `KwReturn` | `return` | `return` |
| `KwMatch` | `match` | `match` |
| `KwModule` | `module` | `module` |
| `KwImport` | `import` | `import` |
| `KwPub` | `pub` | `pub` |
| `KwType` | `type` | `type` |
| `KwEffect` | `effect` | `effect` |
| `KwTrue` | `true` | `true` |
| `KwFalse` | `false` | `false` |
| `KwActor` | `actor` | `actor` |
| `KwSpawn` | `spawn` | `spawn` |
| `KwConsume` | `consume` | `consume` |
| `KwRecover` | `recover` | `recover` |
| `LParen` | `(` | `(` |
| `RParen` | `)` | `)` |
| `LBrace` | `{` | `{` |
| `RBrace` | `}` | `}` |
| `LBracket` | `[` | `[` |
| `RBracket` | `]` | `]` |
| `Comma` | `,` | `,` |
| `Dot` | `.` | `.` |
| `Colon` | `:` | `:` |
| `Arrow` | `->` | `->` |
| `FatArrow` | `=>` | `=>` |
| `Bang` | `!` | `!` |
| `Question` | `?` | `?` |
| `Pipe` | `|` | `\|` |
| `Underscore` | `_` | `_` |
| `At` | `@` | `@` |
| `Eq` | `=` | `=` |
| `EqEq` | `==` | `==` |
| `NotEq` | `!=` | `!=` |
| `Lt` | `<` | `<` |
| `Le` | `<=` | `<=` |
| `Gt` | `>` | `>` |
| `Ge` | `>=` | `>=` |
| `Plus` | `+` | `+` |
| `Minus` | `-` | `-` |
| `Star` | `*` | `*` |
| `Slash` | `/` | `/` |
| `Percent` | `%` | `%` |
| `AndAnd` | `&&` | `&&` |
| `OrOr` | `||` | `\|\|` |
| `Term` | *(literal)* | end of statement |
| `Eof` | *(literal)* | end of file |

## Reserved words

Lexed as identifiers and refused at declaration sites (`DL0106`), so a member name like `root.secret(…)` stays legal while `secret` cannot be *declared*:

`async`, `await`, `iso`, `val`, `ref`, `box`, `tag`, `trn`, `plugin`, `secret`, `cap`, `for`, `in`, `break`, `continue`, `trait`, `impl`, `where`, `pure`
