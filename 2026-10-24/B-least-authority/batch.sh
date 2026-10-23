#!/bin/bash
# usage: batch.sh cond casesfile [extra flags]   cases: base<TAB>path per line
DL=/c/Users/jesse/AppData/Local/Temp/claude/D--nelan-DeluluLang/eb34182b-835b-44af-b5ec-07064fcae52c/scratchpad/authority/bin/dl
prog=${PROG:-progs/rd.delulu}
cond=$1; cases=$2; shift 2
n=0
while IFS='|' read -r base p; do
  n=$((n+1))
  printf '%s\n%s\n' "$base" "$p" > in_b.txt
  $DL run $cond $prog "$@" < in_b.txt > o_b.txt 2>&1
  ec=$?
  canary=""; grep -q 'B-FORBIDDEN-CANARY' o_b.txt && canary=" !!!FORBIDDEN-CANARY-LEAK!!!"
  grep -q 'B-ALLOWED-CANARY' o_b.txt && canary="$canary [allowed-canary]"
  res=$(grep -E '^(OK|ERR):|^error|^warning' o_b.txt | head -1 | cut -c1-230)
  printf '#%02d exit=%s base=[%s] path=[%s]%s\n     -> %s\n' $n $ec "$base" "$p" "$canary" "$res"
done < "$cases"
