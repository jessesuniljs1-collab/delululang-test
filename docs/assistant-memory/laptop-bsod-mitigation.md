---
name: laptop-bsod-mitigation
description: BSOD cause is the NVIDIA display driver (nvlddmkm) — 5 crashes now (11,12,25,26,27 Jul 2026), all on the SAME never-updated driver build; hardware is clean; discrete-only was NOT actually in effect as of 27 Jul; full build speed still authorized
metadata: 
  node_type: memory
  type: project
  originSessionId: bcacf460-7d6e-4c46-a621-53fcfa099e56
  modified: 2026-07-27T17:20:59.354Z
---

Jesse's HP OMEN laptop (hybrid AMD iGPU + NVIDIA RTX 4050 dGPU, Win 11 build 26200) BSODs are the **NVIDIA display driver, `nvlddmkm`**. Read fault buckets with `Get-WinEvent -FilterHashtable @{LogName='Application'; Id=1001}`; read the bugcheck codes from the **System** log, same Id, provider `WER-SystemErrorReporting`.

| date | bugcheck | Windows' own fault bucket |
|---|---|---|
| 2026-07-11 08:32 | 0x133 DPC_WATCHDOG | (pair with below) |
| 2026-07-12 10:00 | **0x9F** P1=3 | " |
| 2026-07-25 19:16 | **0x9F** P1=3 | `0x9F_3_DXG_POWER_IRP_TIMEOUT_nvlddmkm!rmapiLockAcquire` |
| 2026-07-26 22:43 | 0x133 P1=1 P2=0x1e00 | `0x133_ISR_nvlddmkm!gpumgrGetSubDeviceCountFromGpu` |
| **2026-07-27 21:50** | **0x9F** P1=3 | **bucket EMPTY (`Fault bucket , type 0`)** — identified by convergence, NOT by Windows naming it |

**Crash 5 rests on weaker evidence than 3 and 4, and must be reported that way.** Identified by: identical bugcheck code *and* subtype to 12/25 Jul; `nvlddmkm` appearing 5× in the dump's module strings (vs `amdkmdag` 1×, `dxgkrnl` 1×); no sleep/resume events in the window, so runtime-D3 idle power-down — the same Optimus path Windows named twice; and a byte-identical driver. No kernel debugger is installed (no `cdb.exe` in the SDK paths); installing one is how to make this as strong as the earlier two.

Both fault classes are one story: the NVIDIA **resource-manager lock** stalling — during a dynamic dGPU power transition (0x9F) or inside an **ISR** (0x133).

**Hardware is clean, and this is what Jesse most needs (he has only this laptop):** ZERO `Microsoft-Windows-WHEA-Logger` events across all five crashes. Failing CPU/RAM/PCIe raises **0x124 WHEA_UNCORRECTABLE_ERROR**; no dump is 0x124. SSD (KIOXIA KBG50ZNV1T02) Healthy. Nothing indicates dying silicon.

**A FALSE PATTERN, recorded so nobody rediscovers it:** Kernel-Power **Id 105 "Power source change"** lands within ~20 s of 4 of the 5 crashes, which looks like AC/battery transitions triggering the fault. It is **boot enumeration** — each of those 105s is within 2 min of a Kernel-Boot Id 20. Always filter Id 105 against boot times before concluding anything from it.

**STATE VERIFIED ON THE MACHINE 2026-07-27 22:23 — two of Jesse's beliefs did not match it:**
- **Discrete-only was NOT in effect at 22:23.** `Win32_VideoController` showed **AMD Radeon driving the display at 1920×1080×60** and the RTX 4050 with **no display mode, `Availability=8` (Off Line)** — parked, which only happens under Optimus. Last boot 21:55, *before* he changed the setting; a BIOS GPU-switching change needs a reboot. **Decisive check: the adapter with a non-null `CurrentHorizontalResolution` is the one actually rendering.**
- **CONFIRMED APPLIED after the 22:47 reboot:** roles fully reversed — **RTX 4050 now drives the panel at 1920×1080 @ 165 Hz, `Availability=3` (full power)**; AMD Radeon has no mode and is `Availability=8`. The iGPU stays *enumerated* (HP parks it rather than removing it), which is normal for this mode — judge by which adapter has a resolution, never by presence. The dGPU running at full power while driving the display is what removes the runtime-D3 idle power-down the 0x9F sits in. Side effect worth knowing: the panel went **60 Hz → 165 Hz**, so the iGPU path had been capping refresh rate. **New baseline: zero crashes in true discrete mode; clock starts 2026-07-27 22:47.**
- **The NVIDIA driver has NEVER changed: 32.0.16.1062 (Studio 610.62), dated 2026-06-11, across all five crashes.** The 07-26 "reinstall" was the same build. It is the one variable never varied. Do not read "reinstalled" as "updated".
- `PowerMizer`/`PerfLevelSrc` values are absent → "Power management mode = Prefer maximum performance" was **never applied** either.
- AMD iGPU driver **is** updated: 32.0.31021.5001 (2026-06-28), up from 2024-07. Fast Startup off. BIOS F.32. Win 11 25H2/26200.

**Levers, narrowest first, all Jesse's call:** (1) actually apply discrete-only in BIOS and **reboot**, then verify with the resolution check above — removes the iGPU/dGPU handoff *and* runtime D3, the mechanism both fault classes sit in, at a battery/thermal cost; (2) NVIDIA Control Panel → Manage 3D Settings → **Power management mode = Prefer maximum performance** (stops the dGPU parking; reversible, setting-only); (3) **update the NVIDIA driver off 32.0.16.1062** — the untouched variable across five crashes, and the highest-value single change.

**Why:** Jesse has one laptop and the crashes threaten the project; he asked for the reason, explicitly NOT for reduced performance.

**How to apply:** **Do NOT throttle builds** — full parallelism stays authorized (2026-07-12 order, reaffirmed 2026-07-26 and 2026-07-27). The fault is in the GPU driver; cargo saturates CPU and never touches the GPU. Sustained load is a plausible *aggravator* for the 0x133 ISR/DPC overrun only; the 0x9F lock stall during a GPU power transition has no CPU-load mechanism at all. Keep the commit-after-every-green-phase habit permanently: work has survived five BSODs without losing a byte. Dump copies at `D:\Minidump copy\`. See [[delululang-project]].
