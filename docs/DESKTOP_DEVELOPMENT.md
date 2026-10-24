# Host-only native simulator development

The active UI is now based on the owner's extracted NA 70.00.100A firmware.
Start with [the stock UI extraction, build/run and scope guide](STOCK_UI_PORT.md).
The custom dashboard from the previous experiment is superseded.

Install stable Rust from [the official Rust distribution](https://rust-lang.org/tools/install/).
On macOS, install Xcode or Apple's Command Line Tools. Python 3.9+ and system
`unzip` are used for offline import. On Debian/Ubuntu the window backend requires:

```sh
sudo apt-get install libx11-dev libxcursor-dev libxrandr-dev libxi-dev libwayland-dev libxkbcommon-dev
```

```sh
cargo fetch --locked
python3 tools/import-firmware-ui.py /path/to/cmu150_NA_70.00.100A_reinstall.up target/owner-firmware
cargo run --locked -p mazda-desktop
```

The desktop remains an 800×480 native minifb window and Rust software framebuffer.
It does not embed Opera, Chromium, a webview or execute the firmware's JavaScript.
Bundled Inter and Swash provide proportional antialiased text; original Mazda
artwork is loaded from a local offline conversion. No runtime downloads occur.

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
python3 tools/test-import-firmware-ui.py
cargo run --locked -p mazda-desktop -- --smoke-test
sh tools/package-macos.sh --stock
```

The smoke test needs a desktop session, submits 90 native frames and exits. CI
runs synthetic unit/parser tests and builds without firmware files. The app itself
requires the owner's local artwork. There are no Docker containers to restart.

`mazda-core` retains typed read-only vehicle observations; `mazda-mock` owns all
synthetic mutations. No live vehicle adapter, firmware flashing, USB preparation,
CAN/LIN writes, or interlock bypass is part of this simulator. The inherited
`apps/cmu-inspect` research utility remains excluded from default builds and CI.
