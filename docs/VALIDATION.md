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

## Sound and factory Touring fixture, 2026-10-24

The source-guided Sound increment adds the original standard/Bose/Bose J12 rows,
bounded pivot/progress sliders, tone/ALC tick marks, labels and enabled/disabled
checkbox art. The default host fixture is standard audio/no HUD, based on the
factory US 2018 Touring equipment evidence in the deployment plan. This is not
on-car equipment detection. Six additional PNGs were checked against the existing
source inventory and decoded as data; 47 render assets are now required.

- 41 Rust tests pass: core 3, desktop 6, mock 4, UI 28. Six importer and five
  catalog parser tests pass. Formatting and strict all-targets Clippy pass.
- New tests cover every slider range/end, centered pivots, profile row order and
  feature exclusion, guidance availability, checkboxes, pointer drag release/tab
  cancellation, edit revocation and unchanged media observations. Shared tab tests
  establish no-HUD navigation, directional second/third positioning and first
  off-screen arrow jumps. Center-cropped mirrored pixel rows are checked too.
- Fifteen deterministic Sound renders exercise all three profiles, negative bass,
  the last checkbox, unavailable guidance and the conservative driving fixture.
  These are source-guided native renders, not an independently run Opera UI.
- The live Mac preview opened Sound, adjusted Bass left of center, scrolled to the
  final Beep row and checked it. Switching to the driving fixture preserved the
  checkbox and blocked Enter, using the original disabled artwork/label colors.
  Native pointer activation remains unverified despite model coverage.
- The signed 0.6.0 package was independently extracted, signature checked and its
  delivered executable compared byte-for-byte with the signed build. Three native
  smoke runs (standard, Bose, Bose J12/HUD/guidance unavailable) each rendered 90
  frames and exited 0. Final source changes require a fresh package check.

No real tone, volume, speaker, prompt or OEM setting changed. The Sound host gate
is conservative Park-only; exact OEM permission policy, service payload handling
(including guidance-volume status adjustment), SBN/prompt timing, animations and
touch-release timing remain incomplete. Full Settings conversion remains active.

## Clock root, manual adjustment and time-zone list, 2026-10-24

Three source-defined contexts (ClockTab, SetClock and SelectTimeZone) now have
native host behavior. Eighteen PNGs bring the fixed local asset set to 65; all
101 time-zone labels are read from the owner's original dictionary/declaration
into ignored text. Their source identities are in the evidence ledger.

- 52 Rust tests pass: core 3, desktop 8, mock 4, UI 37. Eight importer and five
  catalog tests pass. Formatting and strict all-targets Clippy pass. No dependency
  or lockfile changes were needed.
- Tests cover 12h/24h wrap, separate AM/PM, minutes without hour carry,
  GPS/navigation restrictions, Park revocation, time-zone bounds/radio selection,
  child Back routing, pointer geometry, disabled-row/button skipping, metadata
  bounds and 150px text rendering. Hold tests establish short release, 1500ms
  threshold/250ms repeat, cancellation and bounded catch-up after a stall.
- Ten deterministic Clock renders cover root, both formats, manual adjustment,
  first/last time-zone selection, DST, GPS/navigation and driving fixtures.
  They are source-guided Rust renders, not an independent Opera runtime capture.
- In the live Mac app a short hours+ changed 12:00 PM to 1:00 PM, a 2400ms hold
  advanced to 4:00 PM, and minutes+ changed to 4:01 PM. Separate AM/PM and 24h
  selection worked; rotation stopped before disabled AM/PM in 24h. Selecting
  another time-zone radio stayed in the list. GPS Sync disabled manual time;
  the driving fixture grayed controls and blocked Enter. Park restored editing.
  Native pointer activation remains unverified; model pointer tests pass.
- The signed 0.7.0 app was independently extracted, its ad-hoc signature checked
  and its executable compared byte-for-byte with the signed build. Two native
  smoke checks (default and GPS/navigation-available fixtures) each rendered 90
  frames and exited 0 from that delivered executable.

The displayed time is a deterministic, manually adjustable host value. No ticking
clock, GPS source, UTC/time-zone/DST calculation, OEM service write, UMP tooltip
timing, animations or original service-settle behavior is claimed. The original
labels may contain historical time-zone information. Full Settings and vehicle
runtime remain incomplete. No Docker service applies and no car was connected.

## System controls, information and host restore, 2026-10-24

Five syssettings contexts (SystemTab, About, AgreementsDisclaimers,
FactoryResetConfirm and FactoryResetProgress) and sysupdate's SoftwareInfo
unavailable state now have native host routes. Eleven more PNGs bring the fixed
asset set to 76. The 2791-byte local disclosure file is decoded from literal text;
its proprietary content and screenshots remain outside Git.

- 60 Rust tests pass: core 3, desktop 8, mock 4, UI 45. Ten importer and five
  catalog tests pass. Formatting and strict all-targets Clippy pass. No crates or
  lockfile changes were needed.
- Tests cover independent conditional unit rows, source order/labels, preferences,
  nested About/Version Back routes, 28px scroll bounds, default No/explicit Yes,
  progress completion/cancellation, service/Park revocation, preserved equipment,
  unchanged source observations, pointer routes and invalid inputs. Text tests
  cover bounded legal metadata, paragraph breaks, long Unicode words and signed
  viewport clipping. Shared chrome tests prove full-screen reset hides status and
  Back cancels. Existing tab-arrow regression still passes.
- Fourteen deterministic System renders cover root/help state, both unit controls,
  About, disclosure start/end, unavailable versions, No/Yes/progress/completion,
  missing services and driving. Original legal text is wrapped in Inter within
  source bounds; browser normal-line-height uses a documented 34px approximation.
  This remains static-source/native proof, not an independently run Opera render.
- The live 0.8.0 Mac preview toggled Tool Tips and VR Examples, opened disclosures,
  scrolled to their final paragraph, returned through About and opened the version
  dialog. Restore opened on No with no status bar, showed progress after explicit
  Yes, returned to System and cleared both help preferences. A driving observation
  cancelled pending confirmation and blocked edits; Park restored navigation.
  Native mouse activation remains unverified despite model coverage.
- The signed app was independently extracted outside Documents sync; its ad-hoc
  signature verified and its executable matched the signed build byte-for-byte.
  Two native smoke checks (default and units/New-CMU/services-unavailable fixtures)
  each rendered 90 frames and exited 0 from that delivered executable.

Tool Tip/VR help overlays, full language/localization, database-update contexts,
OEM reset/service timing, meter animation, exact scrollbar behavior, drag/flick
and left-button focus transitions remain incomplete. Mock restore completes after
1500ms and affects only implemented host preferences. The full Settings and wider
conversion objective remains active; car ABI/services/runtime are still unproven.
No Docker service applies and no vehicle was connected or modified.

## Language selection and owner dictionary text, 2026-10-24

Four further contexts (ChangeLanguage, LanguageConf, LanguageChangeProgress,
LanguageChangeError) now have native host routes. The three-entry NA language
fixture is drawn from cached declarations and original dictionary mappings; it
is not a captured LanguagesList payload from the car. The global source list,
UK/Korean fallback contexts and wider script/font coverage remain incomplete.

- 69 Rust tests pass: core 3, desktop 9, mock 4, UI 53. Twelve importer and five
  catalog tests pass. Formatting and strict all-targets Clippy pass. No crates or
  lockfile changes were required.
- Tests cover complete-file decoding, every truncated fixture prefix, duplicate
  locales/keys, invalid counts/UTF-8, app-scoped aliases, exact string IDs, failed
  load preservation, current/candidate separation, default No, success after mock
  completion, error/retry, Park revocation, Back/global exit, pointer routes and
  host restore to English. Long localized words cannot underflow centering, and
  a longer paragraph stays inside the existing three-line Compass area.
- Forty deterministic locale/flow renders cover English/Spanish/French Home,
  Applications, Communication, Entertainment, Compass, Display, Sound, Clock,
  System and language list; confirmation, progress, failure/retry, selected radios
  and French About/disclosure/version layouts are also included. These are native
  source-guided renders, not an independent Opera comparison. The disclosure body
  remains the original English text; its translated title can occupy two lines.
- Live Mac 0.9.0 checks exercised No cancellation, English → Spanish → French,
  translated Home/System/tab text, original microphone/radio artwork, the two-line
  French disclosure heading and cancellation while driving. Park restored editing.
  After the final oversized-word guard, the exact delivered package was reopened,
  switched English → French → English and left in Park. Error/retry has model and
  render evidence, not a live native failure-path trial. Native mouse activation
  remains unverified.
- The final signed 0.9.0 package was independently extracted, its ad-hoc signature
  verified and its executable compared with the signed build. Three exact-binary
  native smoke checks (English, Spanish, French/failure fixture) each rendered 90
  frames and exited 0.

One original voice PNG brings the fixed image set to 77. The 302082-byte dictionary
asset stays local; all selected source identities match the original extraction
inventory. No firmware JavaScript, native program or voice engine was run. No
Docker service applies, and no OS language setting or vehicle was changed. Full
Settings, other Mazda domains, OEM timing/services and car runtime remain active
work, as recorded in the conversion ledger and deployment plan.
