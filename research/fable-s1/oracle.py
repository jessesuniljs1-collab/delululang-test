#!/usr/bin/env python3
"""T3 complete-program task — specification and fixed external oracle.

WRITTEN BEFORE any DeluluLang implementation of this task was run (FABLE audit, session 1, 2026-11-14).

TASK. Read `input/doc.json` (UTF-8). It should be a JSON object with a key `items`: an array of objects,
each with `name` (string), `qty` (integer) and `tags` (array of strings). Other keys anywhere are ignored.
A repeated key takes its LAST value. Write ONE line to stdout and the same line (plus a newline) to
`output/out.json`:

    {"count":N,"total_qty":T,"names":[UPPERCASED names in order],"tag_counts":{tag:count,... sorted by tag}}

compact separators, non-ASCII left raw, strings JSON-escaped.

FAILURES. Print one line and write NO output file:
    {"error":"input"}   the file is missing or is not valid UTF-8
    {"error":"parse"}   the text is not valid JSON (RFC 8259), or an integer does not fit in signed 64 bits
    {"error":"schema"}  valid JSON of the wrong shape (missing key, wrong type, a non-integer qty)

Usage:
    python -I oracle.py make <dir>      write every fixture to <dir>/<name>/input/doc.json and <dir>/<name>/expected.txt
    python -I oracle.py one <doc.json>  print the expected line for one document
"""
import json
import sys
from pathlib import Path

I64_MIN, I64_MAX = -(2**63), 2**63 - 1


class Overflow(Exception):
    pass


def _int(text):
    n = int(text)
    if not I64_MIN <= n <= I64_MAX:
        raise Overflow(text)
    return n


def expected(raw: bytes) -> str:
    try:
        text = raw.decode("utf-8")
    except UnicodeDecodeError:
        return '{"error":"input"}'
    try:
        doc = json.loads(text, parse_int=_int, parse_constant=lambda c: (_ for _ in ()).throw(ValueError(c)))
    except (ValueError, Overflow, RecursionError):
        return '{"error":"parse"}'
    try:
        if not isinstance(doc, dict):
            raise TypeError
        items = doc["items"]
        if not isinstance(items, list):
            raise TypeError
        total, names, counts = 0, [], {}
        for it in items:
            if not isinstance(it, dict):
                raise TypeError
            name, qty, tags = it["name"], it["qty"], it["tags"]
            if not isinstance(name, str) or isinstance(qty, bool) or not isinstance(qty, int) or not isinstance(tags, list):
                raise TypeError
            for t in tags:
                if not isinstance(t, str):
                    raise TypeError
                counts[t] = counts.get(t, 0) + 1
            total += qty
            names.append(name.upper())
    except (KeyError, TypeError):
        return '{"error":"schema"}'
    out = {"count": len(items), "total_qty": total, "names": names, "tag_counts": dict(sorted(counts.items()))}
    return json.dumps(out, separators=(",", ":"), ensure_ascii=False)


def fixtures():
    big = {"items": [{"name": f"item{i}", "qty": i % 7 - 3, "tags": [f"t{i % 5}", "all"]} for i in range(2000)]}
    deep = "[" * 40 + "]" * 40
    return {
        "f01_basic": b'{"items":[{"name":"bolt","qty":3,"tags":["metal","small"]},{"name":"nut","qty":5,"tags":["metal"]}]}',
        "f02_escapes": b'{"items":[{"name":"say \\"hi\\"","qty":1,"tags":["a\\\\b","line\\nbreak","tab\\there","sl\\/ash"]}]}',
        "f03a_raw_utf8": '{"items":[{"name":"café","qty":2,"tags":["日本","é"]}]}'.encode("utf-8"),
        "f03b_u_escape": b'{"items":[{"name":"caf\\u00e9","qty":2,"tags":["\\u65e5\\u672c"]}]}',
        "f04_trailing_comma": b'{"items":[{"name":"a","qty":1,"tags":[]},]}',
        "f05_missing_key": b'{"items":[{"name":"a","tags":[]}]}',
        "f06_wrong_type": b'{"items":[{"name":"a","qty":"1","tags":[]}]}',
        "f07_deep_ignored": ('{"ignored":' + deep + ',"items":[{"name":"a","qty":1,"tags":["x"]}],"more":{"k":[1.5e3,-0.25,true,false,null]}}').encode(),
        "f08_overflow": b'{"items":[{"name":"a","qty":9223372036854775808,"tags":[]}]}',
        "f09_empty": b'{"items":[]}',
        "f10_duplicate_key": b'{"items":[{"name":"a","qty":1,"qty":2,"tags":[]}]}',
        "f11_invalid_utf8": b'{"items":[{"name":"\xff\x80","qty":1,"tags":[]}]}',
        "f12_whitespace_negative": b' \n\t{ "items" : [ { "name" : "a" , "qty" : -3 , "tags" : [ "x" , "x" ] } , { "tags":[], "qty": 0, "name": "b" } ] } \r\n',
        "f13_float_qty": b'{"items":[{"name":"a","qty":1.0,"tags":[]}]}',
        "f14_not_object": b'[1,2,3]',
        "f15_garbage_after": b'{"items":[]} x',
        "f16_leading_zero": b'{"items":[{"name":"a","qty":01,"tags":[]}]}',
        "f17_big_2000": json.dumps(big, separators=(",", ":")).encode(),
    }


def main():
    if sys.argv[1] == "make":
        root = Path(sys.argv[2])
        for name, raw in fixtures().items():
            d = root / name
            (d / "input").mkdir(parents=True, exist_ok=True)
            (d / "output").mkdir(parents=True, exist_ok=True)
            (d / "input" / "doc.json").write_bytes(raw)
            (d / "expected.txt").write_text(expected(raw) + "\n", encoding="utf-8", newline="\n")
        print(f"wrote {len(fixtures())} fixtures to {root}")
    elif sys.argv[1] == "one":
        sys.stdout.reconfigure(encoding="utf-8", newline="\n")
        print(expected(Path(sys.argv[2]).read_bytes()))


if __name__ == "__main__":
    main()
