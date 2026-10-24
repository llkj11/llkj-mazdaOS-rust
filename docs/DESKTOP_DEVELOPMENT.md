# Host-only native UI development

## Prerequisites and run

Install stable Rust from [the official Rust distribution](https://rust-lang.org/tools/install/).
On macOS, install Xcode or Apple's Command Line Tools for the native window code.
No Node.js, browser, webview, Docker, CMU, firmware image or vehicle cable is needed.

```sh
cargo fetch --locked
cargo run --locked -p mazda-desktop
```

On Debian/Ubuntu, minifb also needs development libraries:

```sh
sudo apt-get install libx11-dev libxcursor-dev libxrandr-dev libxi-dev libwayland-dev libxkbcommon-dev
```

The desktop uses minifb's native window and an 800×480 software framebuffer.
On macOS minifb links Metal/MetalKit. It does not embed Opera, Chromium or another
browser. No new external dependencies were added by this increment.

## Controls and behavior

| Input | Behavior |
| --- | --- |
| Arrows | Rotate the Commander selection; wrap at either end |
| Enter / Space | Open a page's controls, then activate the selected control |
| Backspace | Leave controls and return to page navigation; cancel a pending page selection |
| H / M | Music page |
| N | Drive status page; there is no navigation engine yet |
| F | Display preferences |
| 1 / 2 / 3 / 4 | Synthetic Park / 80 km/h Drive / unavailable / stationary Reverse |
| Mouse click | Open a page or activate a media/preference card |
| Escape / window close | Quit |

On Music, Enter focuses the center Play/Pause control. Rotate to Previous/Next.
Actions update the mock library's title/artist/album and playback state. There is
no audio output, track download or stock media service call.

Settings edit display units and day/night colors for this session. Editing requires
both a valid speed of exactly zero and Gear::Park. Unknown/invalid speed, any
movement, Reverse, Neutral or other gear locks settings. Source refresh revokes
existing settings focus on data loss. Pointer and Commander inputs use the same
policy. This host policy is not a tested vehicle safety system and never overrides
Mazda's stock interlocks. Preferences do not write to disk or the vehicle.

US display units default to mph and Fahrenheit. Internally observations remain
km/h and Celsius. Missing data stays missing; it is never converted to stationary.

## Verify and package

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo build --locked -p mazda-desktop
cargo run --locked -p mazda-desktop -- --smoke-test
```

The smoke test opens a real native window, submits 90 frames and exits. It needs a
desktop session; use ordinary unit/integration tests on a headless runner. Tests
cover mock playback, track wrap, input routing, units, missing readings, live
settings revocation and framebuffer appearance changes. They do not prove real
vehicle service integration or stock-camera/startup/shutdown behavior.

On a Mac, create a local app with:

```sh
sh tools/package-macos.sh
```

The development bundle is `target/macos/Mazda OS Simulator.app`, ad-hoc signed
locally. It contains only the simulator executable, not CMU tooling or firmware.
Double-click it in Finder to run. It is not notarized or a public distribution.
Build artifacts remain ignored by Git. No Docker rebuild/restart is applicable.

## Architecture and next hardware boundary

`mazda-core` remains the typed read-only vehicle capability. `mazda-mock` owns all
synthetic mutations. `mazda-ui` returns media intents; only the desktop app routes
them to that mock. No live adapter exists. Display settings affect rendering only.
Phone is still an explicit placeholder, and Drive is an observational status page.

Before any future hardware proposal, establish the Mazda6's actual CMU identity,
installed firmware, graphics ABI and Commander interface from suitable evidence.
The upstream CX-5 USB mechanism must not be retargeted based on a matching package
name. ARM cross-compilation, Wayland/EGL/GLES rendering, read-only stock services,
camera priority and clean fallback are still unimplemented/unverified. This task
does not prepare media, flash firmware or modify a car.
