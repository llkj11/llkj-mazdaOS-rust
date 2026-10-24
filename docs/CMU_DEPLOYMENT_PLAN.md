# Path from host UI to the owner's 2018 Mazda6

Status: engineering plan based on static inspection of the owner's NA 70.00.100A
package. No car connection, installed-firmware detection, hardware launch, firmware
flash, supervisor change, stock-interlock override or installer is authorized by
this plan. The current work is host simulation and source conversion.

## What the supplied package actually establishes

The package's filesystem contains:

- ARM ELF32 little-endian EABI5 libraries. `lib/libc-2.11.1.so` names the interpreter
  `/lib/ld-linux.so.3`. The float calling convention still needs explicit ARM
  attribute verification; `gnueabihf` must not be assumed.
- `lib/modules/3.0.35/` and glibc 2.11.1. These describe this package, not proof of
  the owner's installed firmware or current car hardware.
- An existing Vivante/Wayland/IVI display stack, vendor EGL/GLES libraries, filtered
  touchscreen/mouse/keyboard input and an initially-hidden surface policy.
- `jci/mmui/mmui_config.xml`: ordinary frontend domains use priority 20, while the
  backup/parking domain has priority 70. The replacement must retain stock priority
  arbitration and immediately yield for camera/parking and other higher priorities.
- `jci/sm/sm.conf`: Opera launches `/jci/gui/index.html`, with `reset_board="yes"`.
  Killing/replacing that process without understanding supervision can reset the
  unit. The project must not ship a simple process-kill/startup-script replacement.

Selected source hashes are in `platform-evidence.json`. Inspection read these
files as data; none of the ARM programs or startup scripts was executed.

## Compatibility problem to solve first

Current [official Rust target documentation](https://doc.rust-lang.org/rustc/platform-support.html)
sets the ordinary Arm GNU Linux baseline at Linux 3.2 and glibc 2.17. The package
is older on both counts. [Rust's Arm ABI documentation](https://doc.rust-lang.org/rustc/platform-support/arm-linux.html)
also distinguishes the soft/hard float calling conventions. A successfully built
ARM binary is insufficient proof that it will start, render, or survive on this CMU.

Evaluate these paths before choosing one:

1. A carefully constrained custom Rust target/sysroot for the actual OEM ABI,
   validating every required libc symbol and syscall against the firmware. This
   requires toolchain work and bench tests; it is not a supported drop-in target.
2. A portable Rust render/state core with a small isolated platform bridge built
   against the OEM graphics/input ABI. If separate processes are used, define a
   bounded local protocol for frames and typed infotainment observations. Do not
   casually mix musl and glibc/vendor libraries in one process. A static musl build
   still does not resolve the unsupported kernel baseline by itself.
3. A reduced `no_std`/`alloc` runtime plus audited platform shims if the standard
   library cannot be made compatible. The present font/cache/window path uses
   `std`; this would require explicit engineering, not just a target flag.

Do not replace the car's libc, kernel, compositor or whole firmware to make the
prototype fit. Preserve the stock service stack and its recovery behavior.

## Milestones and acceptance evidence

| Gate | Work | Evidence required before proceeding |
| --- | --- | --- |
| 1. Faithful host UI | Port every Settings screen/control and subsequent Mazda contexts from extracted declarations, artwork and layout | Context coverage ledger, source references, interaction tests, rendered comparisons, owner trial; font is the only design change |
| 2. Vehicle identity | Confirm exact CMU part/build, installed region/version, equipment and rollback/recovery arrangements | Owner-provided read-only system/version evidence and exact matching hardware/firmware records; package filename alone is insufficient |
| 3. ABI and runtime | Establish CPU/float ABI, loader, libc symbols, kernel calls, EGL/GLES/Wayland/IVI versions and limits | Repeatable cross-build, static linkage/symbol audit, host emulation where appropriate, then independent matching bench CMU proof |
| 4. Native surface | Run the native renderer on a spare isolated bench CMU using the existing compositor | Correct 800×480 output, bounded CPU/RAM, no long startup delays, surface registration/visibility and clean exit verified |
| 5. Read-only services/input | Map real Commander/touch and typed infotainment observations | Recorded/replayed input, schema validation, loss/staleness tests and read-only protocol captures; no raw vehicle bus writes |
| 6. Priority/fallback | Retain OEM arbitration, camera priority, calls/warnings, startup/shutdown and watchdog behavior | Repeated bench fault-injection/restart/camera-priority tests; stock UI recoverable after every failure |
| 7. Restricted operations | Separately understand audio/phone and each authorized infotainment setting | Explicit capability boundaries and service-specific tests. Vehicle/safety settings continue to use the OEM path and stock interlocks |
| 8. Owner installation proposal | Prepare an exact-version, reversible userland deployment with a documented removal/recovery route | Concrete files, compatibility report, backup/rollback proof and owner approval before any car-side action |

The first real launch should be temporary, nonpersistent and bench-only. Keep the
stock HMI present and recoverable. Persistence comes only after the earlier gates
are proven and a separate installation proposal is approved. The final installed
form is intended to be a native userland frontend, not a replacement ECU firmware.

## Factory equipment evidence for the host fixture

The owner's VIN was checked through NHTSA's official decoder and identifies a
2018 Mazda6 Touring, 2.5L/187 hp. The VIN and personal screenshot are not retained
in this repository. Mazda's [2018 model information](https://news.mazdausa.com/2018-03-15-2018-mazda6-premium-performance-design-heighten-appeal)
and [specification deck](https://news.mazdausa.com/download/2018-Mazda6-Spec-Deck_Version-3.pdf)
establish the US Touring's standard six-speaker audio. Bose starts at Grand Touring;
the factory Active Driving Display starts at Grand Touring Reserve. Accordingly,
the default host fixture uses standard audio and no HUD. Explicit alternative
fixtures remain available. This factory-equipment inference does not verify
aftermarket changes, the installed CMU/firmware identity, current service payloads
or any runtime ABI; gate 2 is still incomplete.

## Boundaries that remain in force

No powertrain, steering, braking, restraints, CAN/LIN or VIP MCU write capability is
part of the UI. Host Safety/Vehicle Settings controls manipulate synthetic fixture
state only. Their existence is not permission to implement safety-critical vehicle
writes later. Actual car settings should remain delegated to the stock interface
until an operation's scope and OEM policy are individually understood.

USB preparation, the inherited CX-5 update-scanner mechanism and car firmware
flashing are not deployment shortcuts for this project. Any future hardware step
requires its own current-state evidence and explicit authorization.
