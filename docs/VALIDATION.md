# Mazda6 host simulator — validation

## First increment (before home/typography)

Validated on an arm64 Mac with Rust 1.99.1, 2026-10-24.

| Check | Result and scope |
| --- | --- |
| Upstream baseline | Built unchanged at `7ef7553`; 18 host tests passed; native window captured |
| Format | `cargo fmt --all -- --check` passed |
| Static checks | `cargo clippy --all-targets --locked -- -D warnings` passed for default host members |
| Tests | `cargo test --locked` passed: core 3, mock 4, UI 13, desktop 6 = 26 |
| Release/package | Local arm64 app built; ad-hoc signature verified; deliverable executable matches signed build copy |
| Final native smoke | Exact delivered executable rendered 90 frames at 800×480 and exited with status 0 |
| Live Commander input | Pause changed the control to Play; Next changed metadata to Tycho / A Walk / Dive |
| Live preferences | US to metric and night to day changes displayed correctly |
| Live revocation | Changing the mock to driving removed settings focus; Enter could not edit |
| Missing observations | Unavailable mock displayed missing speed/temperature and Unknown gear |
| Pointer policy | Automated routing/policy tests passed; successful native pointer activation was not established by UI automation |
| Preserved files | Original MIT license and Cargo.lock byte-for-byte unchanged |

Host scenarios and actions are synthetic. The tests cover data validity, track wrap,
input routing, missing measurements, preference changes, revocation after data loss
and changes to rendered pixels. There is no claim of audio playback, maps, phone,
camera behavior, CMU rendering, firmware compatibility or vehicle safety validation.
The original pixel font remains; scalable typography and accessibility semantics
are future UI work.

The inherited CMU utility is excluded from default builds/tests/CI. Its Linux
shell, BusyBox and privileged fixtures were not run. Linux/Windows runtime tests were not run locally. The subsequently pushed first
increment passed macOS and Ubuntu GitHub CI (run 40111264964). No car was connected or
modified, and no firmware payload was executed.

Local tooling initially suffered intermittent executable termination in the
Documents file-provider workspace. A helper outside the repository uses copies of
the verified official tools in a temporary directory and keeps rustup proxy
commands out of Cargo's plugin search. A missing local LLVM library link was also
repaired. Packaging clears metadata only from its generated app bundle before
signing, resolving Finder metadata signature failures. These are local build
environment issues, not changes to the vehicle or system-wide security settings.


## Firmware-based UI and typography increment, 2026-10-24

This supersedes the custom-dashboard experiment, preserved locally as `044d3e8`.
The design basis is the owner's exact NA 70.00.100A reinstall archive, rehashed
before extraction. The original archive hashes were rechecked unchanged after
work. 4,499 GUI files and 58 fonts were extracted locally as data; 29 PNG images
were converted offline for the native port. No firmware updater, script, native
program or JavaScript was run. See `STOCK_UI_PORT.md` and the source hash ledger.

| Check | Result and scope |
| --- | --- |
| Format and static checks | Rust formatting and strict host Clippy passed |
| Rust tests | 25 passed: core 3, mock 4, UI 14, desktop 4 |
| Importer tests | 6 synthetic tests passed, including all five PNG filter modes, CRC/end-chunk/truncation checks, color/alpha, expansion bounds and wrong package rejection |
| Home behavior | Original five domain order, default Communication selection, end clamping, saved Home focus and circular hit-area formula tested |
| Root lists | Source-derived menu order, final-row scrolling and disconnected-action immutability tested |
| Data boundary | Missing/moving/non-Park data cannot authorize preferences; all service/settings mutations currently disabled |
| Artwork rendering | Raw format bounds, clipping, alpha compositing and scaling tested; original artwork used unchanged except specified Compass CSS opacity |
| Visual inspection | Native home states, original-font/Inter A/B, root menu, Settings first pass and Compass fallback renders inspected |
| Native app | Ad-hoc signature and delivered signed executable equality verified; 90-frame native smoke exited 0 |
| Live keyboard | Home selection opened Compass; Back retained the home choice; Entertainment opened the restored source list |
| Live pointer | Routing/geometry tests pass; successful native pointer activation has not been established by UI automation |
| Git boundary | Owner firmware, extracted Mazda/JCI source/artwork/fonts and local app/captures stay out of Git |

The original-font comparison is a Rust render of the same source-guided geometry
using the extracted Tipperary font, not an original Opera runtime screenshot. A
localhost browser reference server was blocked by automatic approval review before
starting; direct file URLs were blocked by browser policy. No workaround was used.
Fractional CSS positions are rounded to framebuffer pixels. Focus/activation
animations, full Settings controls, deep screens, platform accessibility and all
real audio/phone/maps/GPS/vehicle services remain unimplemented or unverified.
The Compass screen uses Mazda's no-navigation fixture, not a finding that the
owner's car lacks navigation. Clock value is fixed at 12:00 for repeatability.

This increment was tested locally on the arm64 Mac. Remote CI has not run for it;
only the earlier pushed host increment has passed GitHub macOS/Ubuntu CI. No Docker
container rebuild or restart applies. No vehicle was connected or modified.

## Source-guided Display controls, 2026-10-24

This adds mode selection, bounded brightness/contrast adjustment, off/clock states,
consumed-input wake and No/Yes reset to the native Display screen. It preserves
the original test image, tab/row geometry and button/slider artwork. English
`Mode` is rendered as the firmware dictionary's `System` label. Twelve additional
PNG data assets bring the local render set to 41. The preview tone adjustment is
an approximation, not an OEM calibration or a system-backlight change.

- 31 host Rust tests passed; six importer and five inert-catalog parser tests passed.
- Formatting and strict all-targets Clippy passed. Display state tests cover
  range limits, pointer endpoints, reset defaults/cancel/confirm, wake and loss of
  Park authorization. The desktop regression also clears a stale lock notice when
  valid Park observations return.
- Live Mac Commander input adjusted brightness to +2, entered contrast editing,
  revoked editing under the driving fixture and opened the original reset dialog
  with No selected. Native pointer activation is still unverified; tests establish
  routing and drag behavior in the model, not delivery of real mouse events.
- The rebuilt signed 0.5.0 package was independently extracted, its signature
  verified, and its exact delivered binary completed 90 native frames with exit 0.
- The earlier stock-artwork commit `fda77dc` passed macOS/Ubuntu CI in run
  40127145756; catalog/platform-plan commit `d8d016a` passed in run 40127725660.
  Later commit checks must be verified separately.

The full 192-context Settings conversion, OEM services, exact firmware animation
timing and car runtime remain incomplete. No Docker project applies.
