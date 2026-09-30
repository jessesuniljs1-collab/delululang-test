#!/usr/bin/env python3
"""Validate a `delulu audit export --format ocsf` file against the published OCSF schema (PS-E-06, D-V2-78).

The schema is READ at run time, never copied into this repository: from a local checkout of
github.com/ocsf/ocsf-schema (`--schema DIR`), or file by file from its tag on raw.githubusercontent.com
(the default), cached in a scratch directory.

For every event it checks what the schema says, not what the exporter meant:
  * the class: its file's own `uid` and category give the event's `class_uid`, and `category_uid` agrees;
  * `type_uid` = `class_uid` * 100 + `activity_id`; `metadata.version` is the schema's version;
  * every attribute the class (with everything it extends and includes) marks `required` is present;
  * no attribute the class does not define (bar `unmapped`), recursively into every object;
  * each object's own `required` attributes and its `at_least_one` / `just_one` constraints;
  * every value's type against the dictionary (string, integer, boolean, array or object), and every
    enumerated `_id` against its enum.

Exit 0 when every event validates, 1 when one does not (each problem printed), 2 on a bad invocation.
Usage: scripts/ocsf-validate.py EXPORT.jsonl [--schema DIR] [--version 1.8.0] [--cache DIR]
"""

import json
import os
import sys
import tempfile
import urllib.request

# The class files the exporter can emit, by class_uid. A table, because the raw host cannot list a
# directory; it cannot lie, because each file's own uid and category must produce the class_uid.
CLASS_FILES = {
    0: "events/base_event.json",
    1001: "events/system/file_activity.json",
    1007: "events/system/process_activity.json",
    2004: "events/findings/detection_finding.json",
    3005: "events/iam/user_access.json",
    4002: "events/network/http_activity.json",
}
# A class file's `extends` names a category-level file in its own directory, or the base event.
CATEGORY_FILES = {
    "system": "events/system/system.json",
    "finding": "events/findings/finding.json",
    "iam": "events/iam/iam.json",
    "network": "events/network/network.json",
    "base_event": "events/base_event.json",
}
PRIMITIVES = {
    "string_t": str, "integer_t": int, "long_t": int, "timestamp_t": int, "boolean_t": bool,
    "float_t": (int, float), "json_t": object,
}


class Schema:
    def __init__(self, local, version, cache):
        self.local, self.version, self.cache = local, version, cache
        self.files = {}

    def get(self, rel):
        if rel in self.files:
            return self.files[rel]
        if self.local:
            path = os.path.join(self.local, rel)
        else:
            path = os.path.join(self.cache, rel)
            if not os.path.exists(path):
                os.makedirs(os.path.dirname(path), exist_ok=True)
                url = f"https://raw.githubusercontent.com/ocsf/ocsf-schema/v{self.version}/{rel}"
                with urllib.request.urlopen(url, timeout=60) as r:
                    data = r.read()
                with open(path, "wb") as f:
                    f.write(data)
        with open(path, encoding="utf-8") as f:
            self.files[rel] = json.load(f)
        return self.files[rel]

    def dictionary(self):
        return self.get("dictionary.json")["attributes"]

    def base_type(self, t):
        """A dictionary type (`hostname_t`, `port_t`, …) down to its primitive (`string_t`, …)."""
        types = self.get("dictionary.json").get("types", {}).get("attributes", {})
        seen = set()
        while t not in PRIMITIVES and t in types and t not in seen:
            seen.add(t)
            t = types[t].get("type", t)
        return t

    def merged(self, chain):
        """Attributes of a file and everything it extends or includes; a child's entry refines its parent's.

        A profile's attributes are known but optional: a profile binds only an event whose
        `metadata.profiles` names it, and an export names none."""
        attrs = {}
        for rel in chain:
            doc = self.get(rel)
            for inc in doc.get("attributes", {}).get("$include", []) or []:
                profile = inc.startswith("profiles/")
                for k, v in self.get(inc).get("attributes", {}).items():
                    if isinstance(v, dict):
                        entry = attrs.setdefault(k, {})
                        entry.update({**v, "requirement": "optional"} if profile else v)
            for k, v in doc.get("attributes", {}).items():
                if isinstance(v, dict):
                    attrs.setdefault(k, {}).update(v)
            for prof in doc.get("profiles", []) or []:
                for k, v in self.get(f"profiles/{prof}.json").get("attributes", {}).items():
                    if isinstance(v, dict) and k not in attrs:
                        attrs[k] = {**v, "requirement": "optional"}
        return attrs

    def class_chain(self, rel):
        chain, doc = [rel], self.get(rel)
        while doc.get("extends"):
            parent = CATEGORY_FILES[doc["extends"]]
            chain.insert(0, parent)
            doc = self.get(parent)
        return chain

    def object_chain(self, name):
        chain, rel = [], f"objects/{name}.json"
        while True:
            chain.insert(0, rel)
            doc = self.get(rel)
            if not doc.get("extends"):
                return chain, doc
            rel = f"objects/{doc['extends']}.json"


def check_value(schema, where, name, spec, value, problems):
    dic = schema.dictionary().get(name, {})
    t = spec.get("type") or dic.get("type")
    is_array = spec.get("is_array", dic.get("is_array", False))
    # The dictionary's enum (0 Unknown, 99 Other for every `_id`) and the class's or object's own, as the
    # OCSF compiler merges them. `class_uid`, `category_uid` and `type_uid` are compiled per class, so
    # they are checked by the rules in `check_event`, not by an enum here.
    enum = None
    if name not in ("class_uid", "category_uid", "type_uid") and (spec.get("enum") or dic.get("enum")):
        enum = {**(dic.get("enum") or {}), **(spec.get("enum") or {})}
    if is_array:
        if not isinstance(value, list):
            problems.append(f"{where}.{name}: an array is required, found {type(value).__name__}")
            return
        items = value
    else:
        items = [value]
    for item in items:
        if t is None:
            problems.append(f"{where}.{name}: the dictionary has no type for it")
            continue
        base = schema.base_type(t)
        if base in PRIMITIVES:
            py = PRIMITIVES[base]
            if isinstance(item, bool) and base != "boolean_t":
                problems.append(f"{where}.{name}: {base} expected, found a boolean")
            elif not isinstance(item, py):
                problems.append(f"{where}.{name}: {base} expected, found {type(item).__name__}")
            elif enum is not None and base in ("integer_t", "long_t") and str(item) not in enum:
                problems.append(f"{where}.{name}: {item} is not in its enum {sorted(enum)}")
        else:
            check_object(schema, f"{where}.{name}", t, item, problems)


def check_attrs(schema, where, attrs, obj, problems, constraints=None):
    for k, spec in attrs.items():
        if spec.get("requirement") == "required" and k not in obj:
            problems.append(f"{where}: the required `{k}` is missing")
    for k, v in obj.items():
        if k == "unmapped" and where == "event":
            continue
        if k not in attrs:
            problems.append(f"{where}: `{k}` is not an attribute of this class or object")
            continue
        check_value(schema, where, k, attrs[k], v, problems)
    for kind, names in (constraints or {}).items():
        present = [n for n in names if n in obj]
        if kind == "at_least_one" and not present:
            problems.append(f"{where}: at least one of {names} is required")
        if kind == "just_one" and len(present) != 1:
            problems.append(f"{where}: exactly one of {names} is required, found {present}")


def check_object(schema, where, name, obj, problems):
    if not isinstance(obj, dict):
        problems.append(f"{where}: an object ({name}) is required")
        return
    chain, doc = schema.object_chain(name)
    constraints = {}
    for rel in chain:
        constraints.update(schema.get(rel).get("constraints", {}) or {})
    check_attrs(schema, where, schema.merged(chain), obj, problems, constraints)


def check_event(schema, ev):
    problems = []
    cuid = ev.get("class_uid")
    rel = CLASS_FILES.get(cuid)
    if rel is None:
        return [f"class_uid {cuid} is not a class this validator knows"]
    chain = schema.class_chain(rel)
    doc = schema.get(rel)
    categories = schema.get("categories.json")["attributes"]
    # The category is named by the nearest file of the chain that names one (a class's category-level parent).
    cat_name = next((schema.get(r).get("category") for r in reversed(chain) if schema.get(r).get("category")), None)
    cat_uid = categories[cat_name]["uid"] if cat_name in categories else 0
    want = cat_uid * 1000 + doc.get("uid", 0) if cat_name in categories else 0
    if cuid != want:
        problems.append(f"class_uid {cuid}, but `{rel}` defines {want}")
    if ev.get("category_uid") != cat_uid:
        problems.append(f"category_uid {ev.get('category_uid')}, but the class's category is {cat_uid}")
    if ev.get("type_uid") != cuid * 100 + (ev.get("activity_id") or 0):
        problems.append(f"type_uid {ev.get('type_uid')} is not class_uid*100 + activity_id")
    if ev.get("metadata", {}).get("version") != schema.version:
        problems.append(f"metadata.version is {ev.get('metadata', {}).get('version')}, the schema is {schema.version}")
    attrs = schema.merged(chain)
    constraints = {}
    for r in chain:
        constraints.update(schema.get(r).get("constraints", {}) or {})
    check_attrs(schema, "event", attrs, ev, problems, constraints)
    return problems


def main(argv):
    args = argv[1:]
    export, local, version, cache = None, None, "1.8.0", None
    while args:
        a = args.pop(0)
        if a in ("--schema", "--version", "--cache") and args:
            v = args.pop(0)
            local, version, cache = (v, version, cache) if a == "--schema" else (local, v, cache) if a == "--version" else (local, version, v)
        elif not a.startswith("-") and export is None:
            export = a
        else:
            print(f"error: unknown argument `{a}`\n{__doc__}", file=sys.stderr)
            return 2
    if export is None:
        print(__doc__, file=sys.stderr)
        return 2
    schema = Schema(local, version, cache or tempfile.mkdtemp(prefix="ocsf-schema-"))
    if schema.get("version.json").get("version") != version:
        print(f"error: the schema says it is {schema.get('version.json').get('version')}, not {version}", file=sys.stderr)
        return 2
    bad = 0
    events = 0
    with open(export, encoding="utf-8") as f:
        for n, line in enumerate(f, 1):
            if not line.strip():
                continue
            events += 1
            try:
                ev = json.loads(line)
            except json.JSONDecodeError as e:
                print(f"line {n}: not JSON: {e}")
                bad += 1
                continue
            for p in check_event(schema, ev):
                print(f"line {n} (class {ev.get('class_uid')}): {p}")
                bad += 1
    if events == 0:
        print("error: the export holds no events", file=sys.stderr)
        return 1
    print(f"{'ok' if bad == 0 else 'FAILED'}: {events} event(s) against OCSF {version}; {bad} problem(s)")
    return 0 if bad == 0 else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv))
