#!/bin/bash
# usage: race.sh cond mode base path swapdir [extra flags]
# mints a capability on `base`, waits for console input, swaps `swapdir` for a junction to forbidden, then lets the program use it.
DL=/c/Users/jesse/AppData/Local/Temp/claude/D--nelan-DeluluLang/eb34182b-835b-44af-b5ec-07064fcae52c/scratchpad/authority/bin/dl
cond=$1; mode=$2; base=$3; p=$4; swap=$5; shift 5
rm -f go.flag o_race.txt
touch -d '1 second ago' stamp_r
sleep 1
( printf '%s\n%s\n%s\n' "$mode" "$base" "$p"; while [ ! -e go.flag ]; do sleep 0.3; done; echo go ) | $DL run $cond progs/race.delulu "$@" > o_race.txt 2>&1 &
pid=$!
for i in $(seq 1 100); do grep -q 'minted' o_race.txt 2>/dev/null && break; sleep 0.3; done
echo "minted-seen: $(grep -c minted o_race.txt)"
# swap: move the real dir aside, put a junction to forbidden in its place
mv "$swap" "${swap}.real" && cmd //c "mklink /J $(echo $swap | tr / '\\') forbidden" > /dev/null
ls -la "$swap" | head -3
touch go.flag
wait $pid
echo "program-exit=$?"
# restore
cmd //c "rmdir $(echo $swap | tr / '\\')" > /dev/null; mv "${swap}.real" "$swap"
grep -E '^(OK|ERR):|^error|minted' o_race.txt | cut -c1-300
grep -q 'B-FORBIDDEN-CANARY' o_race.txt && echo "!!!FORBIDDEN-CANARY-LEAK!!!"
find . -newer stamp_r -type f ! -name 'o_*' ! -name 'stamp_r' ! -name 'go.flag' ! -path './progs*' ! -path './state*' | tr '\n' ' '; echo
ls forbidden
