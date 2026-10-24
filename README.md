# mazda-os

## This fork: 2018 Mazda6 desktop development

This fork continues the existing Rust native simulator for a **2018 Mazda6**.
It runs entirely on the host, using synthetic observations and a software-rendered
800×480 UI. That resolution is inherited from upstream; the car's exact display,
CMU part number, installed firmware and runtime ABI have not been verified.
No vehicle adapter, firmware installation, USB preparation or interlock bypass is
part of this development path. Upstream history and the MIT license are retained.

Track ongoing work in [conversion coverage](docs/CONVERSION_PROGRESS.md) and the
[evidence-based plan for the car](docs/CMU_DEPLOYMENT_PLAN.md).

The active conversion now starts from the owner's **actual NA 70.00.100A UI**.
It restores Mazda's five circular home icons, background, red focus effect,
category order, initial Communication selection, and bounded Commander rotation.
The typography uses Inter. Read [the stock UI port guide](docs/STOCK_UI_PORT.md)
for extraction, build/run instructions, exact source evidence and incomplete areas.
The prior custom dashboard is superseded and preserved on a separate local branch.

The [security review](docs/SECURITY_REVIEW.md) covers the upstream audit and later
font/import changes. Proprietary owner firmware/UI assets stay local and are not
part of the public source. The app requires an offline artwork import before run.
Home/category navigation, root menus, Display, Sound, Clock and System controls are
implemented in host state. Display mode, brightness/contrast, screen-off/clock,
reset, tone/balance, ALC/guidance and equipment-specific checkboxes are usable in
simulation. Clock adds manual time, 12h/24h, GPS Sync, the original time-zone list
and daylight-saving preference. These remain in memory; no GPS, time-zone
calculation or Mac clock setting is connected. System adds Tool Tips/VR Examples,
conditional unit preferences, About/disclosures, unavailable version information
and restore of implemented host preferences. The original language selector now changes existing screen text using NA English,
Spanish and Canadian French dictionaries. The [Music Database Update screens](docs/MUSIC_UPDATE_PORT.md)
now support source-guided search, selection, confirmation, progress, errors and retry
with synthetic service data; no USB scanning or updater is connected. Full global
language/voice support remains pending. Devices now has the NA root plus
[source-guided Bluetooth simulation](docs/BLUETOOTH_PORT.md), including paired-device
profiles, SSP confirmation, connection/unpairing flows and the four-digit pairing-code keypad.
No Bluetooth radio, phone, contacts or OS settings are accessed. The default Touring fixture uses standard audio and no HUD; explicit
`--audio-profile bose`, `--audio-profile bose-j12` and `--hud` fixtures retain the
other source layouts for testing. Remaining Settings contexts, radio, phone, maps, cameras and actual
vehicle services still require conversion or integration.

`cargo test --locked` selects host crates by default. `apps/cmu-inspect` is preserved
upstream research for a different car and excluded from default builds and CI.
The upstream overview below describes its original CX-5 research context.

A modern replacement frontend/runtime for first-generation Mazda Connect (Gen 6.5), targeting the stock CMU. The long-term goal is to own the infotainment UX while preserving Mazda's integrated display, Commander input, audio, camera, and other stock services where practical.

The project is desktop-first and deliberately **not** a vehicle-control project.

## Approach

- Build and exercise the UI off-car at the factory 800×480 resolution.
- Keep application behavior independent of the graphics backend; desktop uses a software framebuffer, while the CMU target is Wayland/EGL/OpenGL ES.
- Treat vehicle state as typed, read-only application input through `MazdaReadOnly`.
- Reuse stock CMU services only after their interfaces are understood.
- Keep the first hardware integration report-only and explicitly bound to the owner's exact CMU build, retaining the stock HMI as the fallback.

```text
                  UI / application
                        │
             ┌──────────┴──────────┐
             │                     │
         Renderer             MazdaReadOnly
             │                     │
       desktop / EGL         mock / CMU adapter
```

Application code does not receive raw CAN/LIN access, D-Bus connections, VIP flashing, low-level device access, arbitrary shell execution, or arbitrary filesystem writes. Powertrain, braking, steering, restraints, ADAS, and other safety-critical systems are out of scope.

The first CMU integration is observational and bench-only. Any future mutating infotainment operation must be individually understood and isolated from vehicle-control buses.

## Repository layout

```text
apps/
  cmu-inspect/    firmware-gated, report-only CMU metadata collector
  desktop/       800×480 desktop simulator
crates/
  mazda-core/    domain types and read-only capability boundary
  mazda-mock/    deterministic development backend
  mazda-ui/      UI model, renderer trait, software framebuffer
docs/
  BENCH_SETUP.md
  ROADMAP.md
```

## Run the simulator

```bash
cargo run -p mazda-desktop
```

Controls:

- Arrow keys: rotate Commander knob
- Enter / Space: select
- Backspace: back
- H: Home dashboard
- M: Music → Now Playing
- N: Navigation → Drive
- F: Favorites service (not connected in the host preview)
- Escape: quit

In this fork, the five home choices retain the firmware's original order and
rotation stops at the ends. Enter opens a category. Backspace returns Home with
the prior selection. Display and Sound settings manipulate host preview values while valid
Park observations are present; Clock controls follow the same conservative host
gate. Hold Clock +/- to repeat; AM/PM changes separately. Other service actions remain disconnected. See the
stock UI port guide for the current conversion boundary. Keys **1 / 2 / 3 / 4** change
synthetic vehicle observations only.

Verification:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```

## Hardware development

The first inspection utility is hard-coded for the owner's 2019.5 CX-5 GT on screen version
`70.00.100 NA N`. It prepares a firmware-gated, report-only USB payload; it is not a general Mazda
tool. A passive Mac-to-CMU cable is not supported because both ports are USB hosts. Read
[`docs/CMU_USB_INSPECTION.md`](docs/CMU_USB_INSPECTION.md) before preparing or inserting media.

The firmware check happens only after the stock update scanner has begun privileged execution, so
it cannot contain the entry path. Prepared media is for a spare, physically isolated bench CMU
only; do not insert it into an installed vehicle.

No persistence, networking, remote shell, VIP access, CAN access, or LIN access is implemented.
Bench work remains preferred for later mutating features; see [`docs/BENCH_SETUP.md`](docs/BENCH_SETUP.md).

## Disclaimer

This project is not affiliated with or endorsed by Mazda Motor Corporation. Mazda and Mazda Connect are trademarks of their respective owners.
