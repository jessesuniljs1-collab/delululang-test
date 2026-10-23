#!/bin/bash
# usage: nbatch.sh cond casesfile [extra flags]   cases: mode|scope|url|body|header per line  (prog = sec.delulu)
# Per case: records connections the listener saw (new lines in conn.log) and the program's result.
DL=/c/Users/jesse/AppData/Local/Temp/claude/D--nelan-DeluluLang/eb34182b-835b-44af-b5ec-07064fcae52c/scratchpad/authority/bin/dl
prog=${PROG:-progs/sec.delulu}
cond=$1; cases=$2; shift 2
n=0
while IFS='|' read -r mode scope url body hname; do
  n=$((n+1))
  before=$(wc -l < conn.log)
  printf '%s\n%s\n%s\n%s\n%s\n' "$mode" "$scope" "$url" "$body" "$hname" > in_n.txt
  $DL run $cond $prog "$@" < in_n.txt > o_n.txt 2>&1
  ec=$?
  sleep 0.4
  after=$(wc -l < conn.log)
  conns=$((after-before))
  newc=""
  if [ $conns -gt 0 ]; then newc=$(tail -n $conns conn.log | sed -E 's/hex=(.{0,24}).*/hex=\1../' | tr '\n' ';' | cut -c1-200); fi
  leak=""; grep -q 'tok-SECRET' o_n.txt && leak=" !!!SECRET-IN-OUTPUT!!!"
  grep -q 'tok-SECRET' conn.log && leak="$leak !!!SECRET-IN-CONNLOG(plain)!!!"
  res=$(grep -E '^(OK|ERR):|^error|^note|^warning' o_n.txt | head -2 | cut -c1-${NCUT:-220} | tr '\n' ' ')
  printf '#%02d exit=%s CONNS=%d %s url=[%s] mode=%s%s\n     -> %s\n' $n $ec $conns "$newc" "$url" "$mode" "$leak" "$res"
done < "$cases"
