#!/bin/bash
# usage: gen.sh cond casesfile   cases: kind|base|path|body   (kind = read|write|append)
# For each case: generate a fixed program, run it on the interpreter and on the wasm engine, compare.
DL=/c/Users/jesse/AppData/Local/Temp/claude/D--nelan-DeluluLang/eb34182b-835b-44af-b5ec-07064fcae52c/scratchpad/authority/bin/dl
cond=$1; cases=$2; shift 2
n=0
while IFS='|' read -r kind base p body; do
  n=$((n+1))
  # escape backslashes for the DeluluLang string literal
  eb=${base//\\/\\\\}; ep=${p//\\/\\\\}; ebody=${body//\\/\\\\}
  case $kind in
    read)  call="let c = root.fs_read(\"$eb\")
    match c.read_text(\"$ep\") {
        Ok(t) => out.println(\"OK:\" + t)
        Err(e) => out.println(\"ERR:\" + str(e))
    }" ;;
    write) call="let c = root.fs_write(\"$eb\")
    match c.write_text(\"$ep\", \"$ebody\") {
        Ok(u) => out.println(\"OK:written\")
        Err(e) => out.println(\"ERR:\" + str(e))
    }" ;;
    append) call="let c = root.fs_write(\"$eb\")
    match c.append_text(\"$ep\", \"$ebody\") {
        Ok(u) => out.println(\"OK:appended\")
        Err(e) => out.println(\"ERR:\" + str(e))
    }" ;;
  esac
  cat > progs/gen_case.delulu <<EOF2
module gen_case

fn main(root: Root) ! {Read, Write} {
    let out = root.console()
    $call
}
EOF2
  r=()
  for eng in interp wasm; do
    if [ $eng = wasm ]; then fl="--engine wasm"; else fl=""; fi
    $DL run $cond progs/gen_case.delulu $fl > o_g_$eng.txt 2>&1
    ec=$?
    res=$(grep -E '^(OK|ERR):|^error|^warning' o_g_$eng.txt | head -1 | sed 's/WASM engine: WASM trap: //' | cut -c1-${GCUT:-170})
    canary=""; grep -q 'B-FORBIDDEN-CANARY' o_g_$eng.txt && canary="!!!LEAK!!!"
    r+=("$eng exit=$ec $canary $res")
  done
  # compare the classification (first 60 chars after the code), flag disagreement
  a=$(echo "${r[0]}" | sed 's/^interp //'); b=$(echo "${r[1]}" | sed 's/^wasm //')
  flag=""; [ "$(echo "$a" | cut -c1-60)" != "$(echo "$b" | cut -c1-60)" ] && flag=" <<< ENGINES DIFFER?"
  printf '#%02d %s base=[%s] path=[%s]%s\n   %s\n   %s\n' $n "$kind" "$base" "$p" "$flag" "${r[0]}" "${r[1]}"
done < "$cases"
