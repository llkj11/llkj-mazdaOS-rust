# mazda-os

## This fork: 2018 Mazda6 desktop development

This fork continues the existing Rust native simulator for a **2018 Mazda6**.
It runs entirely on the host, using synthetic observations and a software-rendered
800×480 UI. That resolution is inherited from upstream; the car's exact display,
CMU part number, installed firmware and runtime ABI have not been verified.
No vehicle adapter, firmware installation, USB preparation or interlock bypass is
part of this development path. Upstream history and the MIT license are retained.

Start with [the security review](docs/SECURITY_REVIEW.md), then
[build/run instructions](docs/DESKTOP_DEVELOPMENT.md). The first increment adds
Commander and pointer media controls, session-local US/metric units and day/night
appearance, and four mock vehicle scenarios. Audio playback, Bluetooth, phone,
maps, cameras and stock service integration remain future work.

`cargo run --locked -p mazda-desktop` launches the simulator. `cargo test --locked`
now selects only the host crates by default. `apps/cmu-inspect` remains preserved
upstream research for a different car; it is excluded from default builds and CI.
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
- H / M: Home or Music → Now Playing
- N: Navigation → Drive
- F: Favorites → Settings
- Escape: quit

In this fork, Enter opens the selected page's controls. Rotate with arrows, then
Enter to activate. Backspace returns to page navigation. Clicking cards/buttons
uses the same actions and settings policy. Keys **1 / 2 / 3 / 4** select parked,
driving, unavailable and reverse mock observations. Settings are editable only
with valid zero speed and Park; losing that evidence revokes editing immediately.

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
