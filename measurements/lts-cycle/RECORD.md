# The LTS-cycle drill — recording

**Stage 10 phase 10k, spec §4, acceptance criterion 5.** This is the honest record of what the
advisory-feed mechanism does end to end, and — just as importantly — where the *mechanism* drill
this directory runs stops and the *calendar-time* cycle criterion 5 names begins.

## What criterion 5 asks for, and what can and cannot be done in a sitting

> **Criterion 5:** one full LTS cycle exercised (a backported security fix released on the LTS
> train with advisory + DL1903 feed entry); the drill timeline published.

A "full LTS cycle" is, by construction, a thing that takes calendar time: a release train ships a
minor, months pass, a vulnerability is found in the field, a fix is *backported* to the still-
supported LTS minor, an advisory is published, and downstream builds begin flagging the vulnerable
version. No single build session can compress 12-week trains and 24-month backport windows into
itself, and this record does not pretend to. This is the same honesty posture Stage 10 takes
elsewhere for criteria that need something the session cannot supply (invariant-45-style: the
mechanism is built and proven; the part that needs the world is recorded as pending, never faked).

**What IS built, proven, and reproduced here** is the whole *machinery* that cycle runs on:

- an advisory can be **filed on the registry**, scoped to the package it names, and refused
  otherwise (`crates/delulu-registry/src/tests.rs`);
- the feed **exports** to the exact local-file shape a build reads;
- `delulu build` **warns (DL1903)** on a resolved dependency at an advised version, and still
  succeeds — an advisory is information, not a wall;
- `delulu build --deny-advisories` **fails** on that same version — the CI gate;
- the **backported fix** (a patched version) makes the gate pass again — the cycle closes;
- and the **skip branch** holds: `--deny-advisories` with no feed refuses rather than reporting a
  clean scan against evidence it does not have.

**What is NOT done here, and is recorded as pending calendar time:** a real 12-week release train;
a real 24-month backport window aged in production; a real CVE registered with a CNA (spec §4 names
"CNA registration or partner CNA" as process, and DeluluLang has registered none — there is no CVE
number to cite, and none is invented). Criterion 5 remains **PENDING-ADOPTION** on the timed cycle,
exactly as criterion 6 (external adoption) carries D3's marker: the mechanism is witnessed, the
lived cycle awaits the calendar.

## The drill timeline (what `run-demo.sh` actually did)

Real run, 2026-07-20, Windows 11, release build. All steps reproduce via
`bash measurements/lts-cycle/run-demo.sh` (which builds `delulu` and `delulu-registry` itself, then
runs against a scratch on-disk registry and a scratch package — no network, no server process, no
hardware).

| Step | Event (modelled) | Command | Outcome |
|---|---|---|---|
| 1 | `netlib` released at **1.2.0** on its train | scaffold pkg | — |
| 2 | vulnerability found; **advisory filed** on the registry | `delulu-registry advisory file --package netlib --id DLSA-2026-0007 --affected 1.2.0 --patched 1.2.1` | `filed advisory DLSA-2026-0007 for netlib` |
| 3 | feed **synced** to the local file a build reads | `delulu-registry advisory export --out delulu.advisories.json` | 1 record written |
| 4 | a build on the vulnerable version | `delulu build netlib/` | **DL1903 warning**, exit 0 |
| 5 | the **CI gate** on the vulnerable version | `delulu build netlib/ --deny-advisories` | **DL1903 error**, exit 1 |
| 6 | **backport** ships as **1.2.1** | bump version | — |
| 7 | the CI gate on the patched version | `delulu build netlib/ --deny-advisories` | clean, exit 0 |
| 8 | skip-branch: the gate with no evidence | remove feed, `--deny-advisories` | **refused** ("no advisory feed"), exit 1 |

`DLSA-2026-0007` is a **drill identifier, not a real advisory or CVE.** It exists only inside this
scratch run; no advisory has been published to any registry, and no CVE has been registered.

## The registry is the source of truth; the build reads it offline

Step 2 files the advisory into the registry's own on-disk feed (`advisories/<package>`, JSONL, one
append-only file per package — the same shape and discipline as the index). Step 3 exports it to a
local `delulu.advisories.json`. Step 4 onward reads *only that local file*. This split is
deliberate and is the same property the lockfile gives dependency resolution: **the registry being
down never breaks a build, and never silences an advisory a build already holds.** A build does not
phone the registry at build time; it consults the feed it last synced. The trade that buys — a
build can be N syncs behind the registry — is stated, not hidden: a feed is only as current as its
last export, and `--deny-advisories` gates against *that* snapshot, which is why an absent or stale
feed under the gate refuses rather than silently passing.

## The one rule worth restating

Passes 5 and 8 both **fail the build**, and for two different reasons that must not be collapsed:

- **Pass 5** fails because a *known* advisory matched a resolved version — the gate did its job.
- **Pass 8** fails because the gate was asked to enforce a feed it *could not find*. A CI gate that
  opens because it could not locate its evidence is the gate opening on damage. "No feed, so
  nothing to deny" is exactly the fail-open this detector refuses — the same rule DL1905 draws for
  a missing hardware sign-off record. Note the asymmetry, which is correct: **without**
  `--deny-advisories`, an absent feed is silence (there is genuinely nothing known to warn about);
  **with** it, an absent feed is a refusal (you asked for a gate, and a gate with nothing to check
  guarantees nothing).

## What this is not

This drill demonstrates the advisory/detector machinery and its gate. It is not evidence about the
security of any real dependency, not a CVE feed, and not a claim that DeluluLang is a CNA or has
registered any advisory with one. It says nothing about response times to real vulnerabilities,
which are a matter of the (calendar-time) process criterion 5's timed half will measure when a real
cycle has run.

## Reproducing

```
bash measurements/lts-cycle/run-demo.sh
```

Prints `6 passed, 0 failed` and exits 0 on success. It builds the binaries it uses rather than
trusting a pre-existing one — the discipline `measurements/robotics-demo/run-demo.sh` adopted after
an earlier draft measured a stale build and printed a confident page of zeros.
