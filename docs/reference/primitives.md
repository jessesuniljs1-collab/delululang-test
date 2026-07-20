<!-- GENERATED FILE — DO NOT EDIT BY HAND.
     Written by `delulu-conform --reference` from `delulu_check::prim_table::PRIM_TABLE` (fenced against `check::method_sig`).
     `delulu-conform --check-reference` fails the build if this file drifts from the source. -->

# The primitive table

Every primitive operation, by receiver. **Arity is normative**: the checker refuses a call carrying more arguments than the row states (`DL0403`), and the column is proven in both directions against that gate.

Capability operations are the only source of primitive effects (T-CapOp), which is why this table is the checker's single source of truth rather than prose.

> **Coverage (invariant 42):** 59 of 59 anchors in this chapter have both an accepting and a rejecting conformance witness (100.0%). Items marked otherwise are **not stable** until witnessed — see `STAGE9_BUILD_ORDER.md` D10.


## `root`

| Method | Arity | Anchor | Coverage |
|---|---|---|---|
| `console` | 0 | `ref.prim.root.console` | covered |
| `fs_read` | 1 | `ref.prim.root.fs_read` | covered |
| `fs_write` | 1 | `ref.prim.root.fs_write` | covered |
| `http` | 1 | `ref.prim.root.http` | covered |
| `clock` | 0 | `ref.prim.root.clock` | covered |
| `rand` | 0 | `ref.prim.root.rand` | covered |
| `declassify` | 0 | `ref.prim.root.declassify` | covered |
| `foreign_load` | 0 | `ref.prim.root.foreign_load` | covered |
| `plugin_host` | 0 | `ref.prim.root.plugin_host` | covered |
| `secret` | 1 | `ref.prim.root.secret` | covered |
| `foreign` | 1 | `ref.prim.root.foreign` | covered |
| `python` | 1 | `ref.prim.root.python` | covered |

## `console`

| Method | Arity | Anchor | Coverage |
|---|---|---|---|
| `println` | 1 | `ref.prim.console.println` | covered |
| `print` | 1 | `ref.prim.console.print` | covered |
| `readline` | 0 | `ref.prim.console.readline` | covered |

## `fs_read`

| Method | Arity | Anchor | Coverage |
|---|---|---|---|
| `read_text` | 1 | `ref.prim.fs_read.read_text` | covered |
| `list_dir` | 1 | `ref.prim.fs_read.list_dir` | covered |
| `narrow` | 1 | `ref.prim.fs_read.narrow` | covered |

## `fs_write`

| Method | Arity | Anchor | Coverage |
|---|---|---|---|
| `write_text` | 2 | `ref.prim.fs_write.write_text` | covered |
| `append_text` | 2 | `ref.prim.fs_write.append_text` | covered |

## `http`

| Method | Arity | Anchor | Coverage |
|---|---|---|---|
| `get` | 1 | `ref.prim.http.get` | covered |

## `clock`

| Method | Arity | Anchor | Coverage |
|---|---|---|---|
| `now_ms` | 0 | `ref.prim.clock.now_ms` | covered |

## `root`

| Method | Arity | Anchor | Coverage |
|---|---|---|---|
| `actuator` | 1 | `ref.prim.root.actuator` | covered |
| `sensor` | 1 | `ref.prim.root.sensor` | covered |

## `actuator`

| Method | Arity | Anchor | Coverage |
|---|---|---|---|
| `command` | 1 | `ref.prim.actuator.command` | covered |

## `sensor`

| Method | Arity | Anchor | Coverage |
|---|---|---|---|
| `read` | 0 | `ref.prim.sensor.read` | covered |

## `root`

| Method | Arity | Anchor | Coverage |
|---|---|---|---|
| `compute` | 1 | `ref.prim.root.compute` | covered |

## `compute`

| Method | Arity | Anchor | Coverage |
|---|---|---|---|
| `dispatch` | 2 | `ref.prim.compute.dispatch` | covered |

## `rand`

| Method | Arity | Anchor | Coverage |
|---|---|---|---|
| `int` | 2 | `ref.prim.rand.int` | covered |
| `float` | 0 | `ref.prim.rand.float` | covered |

## `plugin`

| Method | Arity | Anchor | Coverage |
|---|---|---|---|
| `get` | 1 | `ref.prim.plugin.get` | covered |
| `unload` | 0 | `ref.prim.plugin.unload` | covered |

## `python`

| Method | Arity | Anchor | Coverage |
|---|---|---|---|
| `import` | 1 | `ref.prim.python.import` | covered |
| `of_int` | 1 | `ref.prim.python.of_int` | covered |
| `of_float` | 1 | `ref.prim.python.of_float` | covered |
| `of_str` | 1 | `ref.prim.python.of_str` | covered |
| `of_bool` | 1 | `ref.prim.python.of_bool` | covered |
| `list` | 1 | `ref.prim.python.list` | covered |
| `to_int` | 1 | `ref.prim.python.to_int` | covered |
| `to_float` | 1 | `ref.prim.python.to_float` | covered |
| `to_str` | 1 | `ref.prim.python.to_str` | covered |
| `to_bool` | 1 | `ref.prim.python.to_bool` | covered |

## `pyobj`

| Method | Arity | Anchor | Coverage |
|---|---|---|---|
| `attr` | 1 | `ref.prim.pyobj.attr` | covered |
| `call` | 1 | `ref.prim.pyobj.call` | covered |
| `call_method` | 2 | `ref.prim.pyobj.call_method` | covered |
| `index` | 1 | `ref.prim.pyobj.index` | covered |

## `secret`

| Method | Arity | Anchor | Coverage |
|---|---|---|---|
| `map` | 1 | `ref.prim.secret.map` | covered |
| `verify` | 1 | `ref.prim.secret.verify` | covered |
| `expose` | 1 | `ref.prim.secret.expose` | covered |

## `str`

| Method | Arity | Anchor | Coverage |
|---|---|---|---|
| `len` | 0 | `ref.prim.str.len` | covered |
| `trim` | 0 | `ref.prim.str.trim` | covered |
| `contains` | 1 | `ref.prim.str.contains` | covered |
| `starts_with` | 1 | `ref.prim.str.starts_with` | covered |
| `split` | 1 | `ref.prim.str.split` | covered |
| `slice` | 2 | `ref.prim.str.slice` | covered |

## `list`

| Method | Arity | Anchor | Coverage |
|---|---|---|---|
| `len` | 0 | `ref.prim.list.len` | covered |
| `get` | 1 | `ref.prim.list.get` | covered |
| `push` | 1 | `ref.prim.list.push` | covered |
| `map` | 1 | `ref.prim.list.map` | covered |
