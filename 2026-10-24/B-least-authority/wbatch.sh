#!/bin/bash
# usage: wbatch.sh cond casesfile [extra flags]   cases: mode|base|path|body per line
# after each run, lists files created/changed (newer than a stamp) anywhere under cwd other than scratch.
DL=/c/Users/jesse/AppData/Local/Temp/claude/D--nelan-DeluluLang/eb34182b-835b-44af-b5ec-07064fcae52c/scratchpad/authority/bin/dl
prog=${PROG:-progs/wr.delulu}
cond=$1; cases=$2; shift 2
n=0
while IFS='|' read -r mode base p body; do
  n=$((n+1))
  printf '%s\n%s\n%s\n%s\n' "$mode" "$base" "$p" "$body" > in_w.txt
  touch -d '1 second ago' stamp_w
  sleep 1
  $DL run $cond $prog "$@" < in_w.txt > o_w.txt 2>&1
  ec=$?
  res=$(grep -E '^(OK|ERR):|^error|^warning' o_w.txt | head -1 | cut -c1-${WCUT:-200})
  changed=$(find . -newer stamp_w \( -type f -o -type d \) ! -name 'in_*' ! -name 'o_*' ! -name 'stamp_w' ! -path './progs*' ! -path './state*' 2>/dev/null | tr '\n' ' ')
  printf '#%02d exit=%s mode=%s base=[%s] path=[%s] body=[%s]\n     -> %s\n     changed: %s\n' $n $ec "$mode" "$base" "$p" "$body" "$res" "$changed"
done < "$cases"
