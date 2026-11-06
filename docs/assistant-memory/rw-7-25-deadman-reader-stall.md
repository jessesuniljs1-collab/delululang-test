---
name: rw-7-25-deadman-reader-stall
description: "RW 7.25 (flaky a_long_chain_does_not_kill_a_healthy_monitor) re-read 2026-11-05 from another AI's analysis Jesse pasted — cause NOT established; the cloud's 'monitor still running' and '1.9 s queue' readings are wrong; a beat stamped/unstamped but not handed over loses to the expiry check (RW 4.87's named uncovered case)"
metadata:
  node_type: memory
  type: project
  originSessionId: eb34182b-835b-44af-b5ec-07064fcae52c
  modified: 2026-11-06T03:02:34.772Z
---

Jesse pasted another AI's read-only analysis of RW 7.25 ("Head Chef Go Through This", 2026-11-05 evening). Head chef
verified against source and the Windows job log (`127204705719`, push run `42015437785` on `5fb865c`).

**Confirmed:**
- **Race:** `brokerd.rs` ~1851–1855: a reader stamps `clock.now()` in the tuple, then `whole.send`; the loop (~1905)
  judges `as_of` = earliest handed-over arrival, else `clock.now()`. A stamped-not-handed-over beat loses. WIDER than
  stated: the stamp comes only after accept + thread spawn + full frame read; unaccepted beats are unstamped. The hearing
  clock (DEADMAN-DEAF-1) covers only whole-process stops (the hearing thread ran: no deaf line while armed). RW 4.87
  itself lists *"one reader thread starved while the rest of the daemon runs (D-V2-127's microseconds)"* as uncovered.
- **The cloud's RW 7.25 reading is wrong on two points:** (1) "monitor still running (exit None)" — its stderr shows the
  watch had ENDED; it hadn't exited because after DL1403 `beat()` → `read_chain()` reads the WHOLE chain (~1.3 s at 101k
  records). (2) The 1,879 ms `ListPage` is the TEST's own `grants list` at the 2 s check (`node_states`,
  `monitor_cli.rs:911`) — after the firing.
- **Job log facts:** failed at "four periods into the watch" (the first check, 2 s); dead-man fired after only **5
  beats** (≈ first 60–100 ms of the watch at `--poll 25`, 2 beats/loop), then ≥500 ms invisible.
- **Extra evidence:** after a fatal end the monitor sends one RunsUnder, then exits — so ≤1 cheap request was ahead of
  that ListPage, yet it waited 1,879 ms between stamp and `taken` (2 ms handling): the broker's request path stalled ~2 s
  on that runner. Not proof (reader vs handler-loop stall unknown; after firing the hearing thread stops listening, so no
  deaf line could appear).
- Monitor-side pause is still possible: the beater beats only while `waiting` (in a broker request), `monitor_cmd.rs` ~376.

**Recommended (not started; Jesse not yet answered):** instrument send/accept/stamp/hand-over/handling/expiry + monitor
progress as job artifacts; a deterministic witness pausing a reader between stamp and hand-over across a deadline (red
first); fix = don't fire while a reader is mid-read or holds a stamped beat (the `reading` counter exists; bounded by
SERVE_READ_TIMEOUT 5 s). Never lengthen the timeout or accept a green re-run. RW 7.25's text needs correcting.
Related: [[delulu-timing-tests]], [[a-latency-may-be-a-deadline]], [[cloud-reviews-2026-11-05]].

**2026-11-05 late — progress (usage limit hit mid-step-2).** Jesse: "Correct the records first, then build the
deterministic witness. Do not implement the proposed reader-count fix as written" (his pasted follow-up correctly
refuted 3 of my claims: the whole-chain read precedes "ended"; 5 beats is a count not a 60–100 ms timeline; the ListPage
delay is stamp-to-handler time, not queue occupancy or ordered vs firing — and the reader-count fix lets overlapping
incomplete connections hold off expiry forever). Order agreed: 1 records → 2 witness + per-beat timing → 3 coordinate
complete AUTHENTICATED beats with expiry (pre-receipt delay a separate problem) → 4 counter-tests (incomplete conns,
invalid keys, unrelated traffic can't keep a silent monitor alive).
- Step 1 DONE: branch `laptop/rw-7-25-records` commit `7dc9162`, **PR #130 open, CI not yet read** (RW 7.25 row,
  AI_ONBOARDING §10, hidden-complexity audit §11.1 dated correction + top note, V2_LOG + CLOUD_SYNC_LOG entries — the
  sync-log entry also records the sync and the Windows re-runs). Survey 1629/19511 0 errors 6 warnings; doctor 29; doc
  gates 64/0. Land on green by fast-forward.
- Step 2 NOT STARTED in code. Design: unit test in `brokerd.rs` tests using `start_daemon` (in-process `serve_inner`);
  a `#[cfg(test)]` seam in the reader (`brokerd.rs` ~1851) after building `(conn, req, Instant::now(), clock.now())` and
  before `whole.send`, keyed by state dir, that pauses a Beat; arm period ~2000 ms, send a beat at once (stamped well
  inside), hold the pause past the deadline (poll the chain for the dead-man revocation), release; assert the beat is
  answered Ok (today: DL1403 → red). Land it `#[ignore]`d with RW 4.98 as reason until step 3, or with step 3. Timing:
  per-node ring of last beats (heard/taken/handled after arming) logged by `log_fired`, plus waiting-queue length and
  the accept thread's `reading` count (share the Arc); monitor side: beat send times/RTT and longest loop turn on a
  dead-man end. Beat path: `beat_deadman_arrived(node,key,arrived,now)` in `delulu-broker/src/deadman.rs` ~183
  (refreshes deadline from `now`; fires if `arrived >= deadline`). New row RW 4.98 for the race once witnessed.

**2026-11-05 night — step 2 DONE (not yet landed).** Branch `laptop/rw-7-25-witness` commit `f3a2b2a`, stacked on #130
(`7dc9162`). RW 4.98 (new row): the witness `brokerd.rs` `a_beat_stamped_in_time_but_handed_over_late_keeps_its_dead_man`
via `#[cfg(test)] mod handover_seam` (holds one Beat of one state dir between stamp and `whole.send`) — red 10/10 at the
dead-man assertion with `--ignored`, landed `#[ignore = "RW 4.98: …"]`; control `the_hand_over_seam_holds_one_beat_of_one_daemon`.
Timing: `broker.log` `BeatTimes::judged` on every firing (last 4 beats stamped/taken/answered since serving, waiting
count, `reading` count) + `reader_delays` (≥100 ms read or hand-over); monitor `progress_line` (last 6 beats, longest
turn) on an unclean end with a dead-man. End-to-end in `monitor_cli a_watch_held_outside_the_daemon_lets_its_dead_man_fire`.
Mutants M1076–M1080 red. HANDOFF §11.2 lesson added. Next: land #130 → PR for the witness branch → step 3 (authenticated
beat coordination + counter-tests; un-ignore the witness) → record the Windows re-runs in their rows.

**2026-11-05 late night — step 3+4 DONE (pushed, not landed).** #130 LANDED (`7dc9162`, run 42063019141 14/14). #131 =
witness branch (`f3a2b2a` + `126fc4e` correcting the witness: beat at ~1.5 s, check 2.4 s, + twin green — see
[[a-witness-needs-a-twin]]); its CI 42067392201 running. Fix branch `laptop/rw-4-98-fix` `ab10583` (D-V2-209): readers note
a whole Beat (node, key, hearing-clock time) under `HeardBeats`' lock (bounded 1,024); the loop fixes `as_of` + takes notes
under that lock, applies via new `Broker::deadman_heard` (key must match, heard < deadline, deadline = max(deadline,
heard+period), not a beat). Witness green, un-ignored. Counter-tests: `incomplete_connections_do_not_hold_a_silent_dead_man_off`,
`beats_with_a_wrong_key_do_not_hold_a_silent_dead_man_off` (+ existing busy-daemon). Mutants M1081 (the withdrawn
reader-count fix → incomplete-connections red) … M1085 red. RW 4.98 closed; RW 7.25 cause still unknown. Counters:
D-V2-209, M1085. NEXT: land #131 on green, then open + land the fix PR on green.

**2026-11-05 latest — #132 LANDED `ab10583` (push run 42075485590 green). An independent review (pasted by Jesse) found 4
issues in #131/#132, all confirmed + fixed on `laptop/rw-4-98-review` `31b1692`, PR #133 (CI running):** (1) BeatTimes
unbounded (any node a beat named) → armed-only, prune, cap 256; witness via `beat_times_seam` (my first witness read only
the end state → M1087 survived → seam now records every node ever kept); (2) readers noted beats BEFORE the DL1406 version
gate → wrong-version beats with the right key kept a dead-man alive (92 held it 2 s) → note only WIRE_VERSION beats;
(3) `answered` timed before write_frame → `handled` + `written`/not-written; (4) monitor noted beats only after answer and
printed progress after joining the beater (15 s request vs 5 s collector) → noted when sent ("no answer yet"), printed before
join. Mutants M1086–M1090 red; NOT witnessed: write-timed-after-write, progress-before-join. Lesson in HANDOFF 11.2: a
witness of a bound checks every state; apply every handler gate before acting early. Broker Miri on ab10583 not yet run.

**#133 LANDED `31b1692`** (PR run 42083936108 14/14). Master push run 42087864563 read green (14 + 2 skipped). Open: broker Miri (nightly) on the dead-man code; RW 7.25 itself.

**2026-11-06 — "finish this completely and verify":** both unwitnessed orders now witnessed on `laptop/rw-4-98-orders`
`125d36f`, PR #134 (answer_seam + `a_beats_answer_is_timed_when_it_is_written`, M1091; `end_the_watch` +
`a_watch_says_its_progress_before_it_waits_for_a_stuck_beater`, M1092). Dispatched on master 31b1692: miri-slow
42093191962; witness.yml RW 7.25 test ×3 Windows (42093261520, 42093278446, 42093295263) ×3 macOS (42093310306,
42093323509, 42093337696) — 5 read green and confirmed to have RUN the test (log: "... ok", 1 passed, measured line).
Then: a final records PR with the results; RW 7.25 stays open unless a run goes red.

**2026-11-06 — DONE except RW 7.25 itself.** #134 LANDED `125d36f` (PR 42094208050, push 42097929801 green). miri-slow
42093191962 SUCCESS (broker 1 h 25 min, 222 passed incl. deadman_heard; check 1 h 36; syntax 16 min). RW 7.25 witness runs
6/6 green, each confirmed to have run the test. Final records PR #135 (`fddb1f6`) open, CI running. RW 4.98 fully closed
(M1076–M1092). RW 7.25 open by nature: read both timing lines at its next red.

**#135 LANDED `fddb1f6`** (PR 42108805025, push 42112445483, each 14 + 2 skipped, read green). This thing is finished; only RW 7.25's own cause stays open.

**2026-11-06 — Jesse: "Why isn't it solved yet".** Honest answer: neither red reproduced; my six runs were the ONE test
ALONE (filter), while both reds came in push runs running the WHOLE `monitor_cli` target (23 tests side by side, each
starting daemons + monitors, threads = CPUs). So reproduce as the push run does, many times: `witness.yml` `repeat` +
`stress` inputs (branch `laptop/rw-7-25-repeat` `e485984`). Dispatched 01:54 UTC: Win r20/s0 42146850780, Win r12/s3
42146857227, mac r20/s0 42146863757, mac r12/s2 42146870039; laptop loop ×15 (Defender real-time ON, 16 CPUs). Source
reading for the next red: the watch loop (`monitor_cmd.rs` ~515) beats twice a turn; between the two beats is
`follower.read_new()` (local file I/O, seeks — cheap), NOT covered by the beater (`waiting` false); the sleep after a turn
is excluded from "longest turn" but shows as a gap in the beats' sent times. macOS red was revoked at the record right
after the test's delegation (i.e. right after an append); Windows red after 5 beats ≈ 3rd turn. Hypothesis to MEASURE,
not claim: a stall on the 30 MB day file after an append (Windows scanner; read the runner's Defender status the repeat
run now prints) or the monitor descheduled under the sibling tests' load.

**2026-11-06 ~03:00 UTC — REPRODUCED on macOS** (`42148295822`, run 9 of 150, whole target, 2 busy loops, 5-CPU
runner; log copy in scratchpad `rw725_red_42148295822.log`). Old (pre-d3da12b) timing: monitor's last kept beats +1960,
+2004, +2004, +2092 (0 ms answers), next +3944 refused; daemon stamped/taken/handled/written each in the same ms (offset
3734 ms) and fired on time. ⇒ **daemon heard everything; the MONITOR went silent 1,852 ms**, beater silent too (so not in a
request, or the whole process stalled). Began ~2.1 s into the watch = the test's delegation append (10-29 macOS red too).
Windows runner: Defender real-time OFF (measured). Rounds: r1 64/64; laptop 15/15 + 30/30 (16 hogs); r2 mac s0 #2 green.
d3da12b = timing on one wall clock (`Moment`), turn steps, last turn apart, longest sleep (M1093–M1098 red). Round 3 on
d3da12b: mac `42153802328` `42153809227` (s2) `42153816861` (s4); win `42153823661` (s3) `42153830411` (s6).
**Web lead (Jesse: "Search web about this thing"):** macOS launchd jobs with no ProcessType / Background are CPU- and
I/O-throttled (IOPOL_THROTTLE tier); GitLab runner jobs via LaunchAgent ran ~6× slower until `ProcessType Interactive`
(gitlab-runner#29089). witness.yml now prints `debug.lowpri_throttle_enabled`, the step's `getiopolicy_np` and any
plist ProcessType on macOS — measure before claiming. Jesse 2026-11-06: development per the reviews is MORE important —
record, land, move to RW 4.95.
