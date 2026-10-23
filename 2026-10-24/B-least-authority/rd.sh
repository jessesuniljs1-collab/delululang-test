#!/bin/bash
# usage: rd.sh cond base path [engine-flag...]
DL=/c/Users/jesse/AppData/Local/Temp/claude/D--nelan-DeluluLang/eb34182b-835b-44af-b5ec-07064fcae52c/scratchpad/authority/bin/dl
cond=$1; base=$2; p=$3; shift 3
printf '%s\n%s\n' "$base" "$p" > in_rd.txt
$DL run $cond progs/rd.delulu "$@" < in_rd.txt > o_rd.txt 2>&1
ec=$?
grep -v '^guard:\|^lease:\|^custody:' o_rd.txt | grep -v '^ *$' | grep -v '^ *|\|^ *-->\|explain:' | head -${LINES_MAX:-6}
echo "exit=$ec"
