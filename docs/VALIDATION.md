# First Mazda6 host increment — validation

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
shell, BusyBox and privileged fixtures were not run. Linux/Windows runtime tests
and remote GitHub CI have not yet run for this branch. No car was connected or
modified, and no firmware payload was executed.

Local tooling initially suffered intermittent executable termination in the
Documents file-provider workspace. A helper outside the repository uses copies of
the verified official tools in a temporary directory and keeps rustup proxy
commands out of Cargo's plugin search. A missing local LLVM library link was also
repaired. Packaging clears metadata only from its generated app bundle before
signing, resolving Finder metadata signature failures. These are local build
environment issues, not changes to the vehicle or system-wide security settings.
