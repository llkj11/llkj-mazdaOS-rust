# Owner-firmware UI port: NA 70.00.100A

The design reference is the owner's actual `cmu150_NA_70.00.100A_reinstall.up`,
not the earlier custom dashboard. Its SHA-256 is
`5938019df7e333447c7bf60aa8f6a7e7f9cd50d6eeb7b6e33605800b85bd4ad0`.

## Extraction and source evidence

The package is a password-protected ZIP. Its rootfs entry is a gzip wrapper around
a gzip tar archive. An offline importer reads that archive, retains regular files
only under `jci/gui/` and `jci/fonts/`, and rejects traversal, links, devices and
oversized files. It does not run update instructions, shell scripts, firmware
executables, or extracted JavaScript. The publicly documented archive password is
referenced in [Pwning Mazda](https://kyprizel.net/work/mazda-cmu-fun.pdf); use it only
for offline reading of the owner's package. Firmware signature verification is
not implemented; the importer pins the exact reviewed package SHA-256.

The local extraction contains **4,499 GUI files and 58 fonts**. The complete local
inventory records path, byte count and SHA-256 for every extracted regular file.
A small decoder converts **76 original PNGs** into bounded RGBA images for Rust.
An inert declaration reader also extracts 101 original English time-zone labels
to an owner-local, bounded text file; no firmware JavaScript is evaluated.
The original disclosure text is decoded from quoted string literals joined with
`+` into owner-local `legal.txt`; calls and other expressions are rejected.
The three Compass mask images apply the firmware CSS's 0.29 opacity to represent
its invalid-position state. Other artwork is neither redrawn nor recolored.
[Selected source identities](firmware-ui-evidence.json) record the inputs used.

| Extracted source below `jci/gui/` | Ported behavior or layout |
| --- | --- |
| `apps/system/controls/MainMenu/js/MainMenuCtrl.js` | Five coin order; initial index 2 (Communication); rotation clamps at 0/4; circular hit areas; selection restoration |
| `apps/system/controls/MainMenu/css/MainMenuCtrl.css` | 800px home layout, coin rectangles, focus transforms, glow, ellipse, centered category label |
| `common/transitions/css/TransitionsGen.css` | 64px status area, 800×416 main area |
| `common/css/common.css` | Original 800×480 background |
| `common/controls/StatusBar/css/StatusBarCtrl.css` | Status artwork, category title, Home icon and right-aligned clock |
| `apps/system/js/systemApp.js` and its English dictionary | Original five destination events and visible root-menu item order |
| `common/controls/List2/css/List2Ctrl.css` | Original list background, red focus art, separators, 64px rows and text column |
| `apps/syssettings/js/syssettingsApp.js`, `common/controls/Tabs/*` | Original conditional seven/eight settings tabs, four visible at once, directional tab position/history and offscreen arrow jumps |
| `apps/audiosettings/js/audiosettingsApp.js`, English dictionary, List2/Slider controls | Standard/Bose/Bose J12 row order, control ranges/defaults, tick/label/pivot behavior, checkboxes and Guidance Volume enable state |
| `apps/syssettings/js/syssettingsApp.js`, English dictionary, ClockSettings2/Ump3/Button controls | Clock root, manual time, time-zone radio list, GPS/navigation enablement, original digit/button geometry and held-input cadence |
| `apps/syssettings/js/legalDisclosure.js`, ScrollDetail control/template, sysupdate SoftwareInfo, Dialog3/LeftBtn | System root, conditional unit rows, About/disclosures, unavailable version dialog, restore confirmation/progress, shared Back and full-screen chrome |
| `apps/emnavi/controls/Compass/*`, `apps/emnavi/js/emnaviApp.js`, English dictionary | Original Compass fallback, background/rail, unavailable position fields, no-navigation explanation |

Home uses the original normal/focused coin images, original background and
reflections, original red focus glow, original order and final focus positions.
CSS fractional positions are rounded to whole framebuffer pixels (less than one
pixel). Home starts on Communication and does **not** wrap. Home/Back retains the
last selected domain. Music opens Entertainment; Navigation opens the original
Navigation domain's Compass fallback. Favorites is not repurposed as Settings.

The typography change is bundled Inter, shaped/rasterized in Rust with Swash;
embedded font hinting is disabled.
The home label remains in its original centered label area; clock alignment and
all home artwork positions are retained. No sidebar, dashboard tiles, invented
cover artwork, or alternate home categories are present.

## Build, import and run

Requirements: stable Rust, Python 3.9+, system `unzip`, and the native window
prerequisites listed in [desktop development](DESKTOP_DEVELOPMENT.md).

```sh
# Prompts for the publicly documented firmware archive password.
python3 tools/import-firmware-ui.py /path/to/cmu150_NA_70.00.100A_reinstall.up target/owner-firmware
cargo run --locked -p mazda-desktop
# Or point at an existing local conversion:
cargo run --locked -p mazda-desktop -- --assets /path/to/render-assets
# Explicit equipment/service fixtures; no real amplifier or HUD is detected:
cargo run --locked -p mazda-desktop -- --audio-profile bose
cargo run --locked -p mazda-desktop -- --audio-profile bose-j12 --hud --guidance-disabled
# Clock enablement fixture; does not provide a GPS/time service:
cargo run --locked -p mazda-desktop -- --navigation-available --clock-gps-sync
# System unit-row/service fixtures; do not detect actual car equipment:
cargo run --locked -p mazda-desktop -- --temperature-units --distance-units
cargo run --locked -p mazda-desktop -- --distance-units --new-cmu --system-services-unavailable
```

The destination must be empty, and the firmware must match the pinned hash. Files
remain local under ignored `target/owner-firmware/`. The default executable refuses
to start if the original artwork is missing, rather than substituting a new UI.
`MZD_FIRMWARE_PASSWORD` may supply the archive password non-interactively.
Older imports need reimporting into an empty directory: the current runtime
requires all 76 images, `clock-zones.txt` and `legal.txt`. Missing inputs fail explicitly.

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
python3 tools/test-import-firmware-ui.py
cargo run --locked -p mazda-desktop -- --smoke-test
sh tools/package-macos.sh --stock
```

The Mac app is `target/macos/Mazda Connect Typography Preview.app`. It includes only
the native simulator, converted artwork and notices, not the firmware archive,
installer instructions, original JavaScript, or native vehicle binaries. This is
a local development package for its owner, not a public redistributable release.
No Docker components or container rebuilds are involved.

## Controls

- Arrows: rotate through the five home choices; stop at either end.
- Enter/Space: open a category; enter Display/Sound/Clock rows; cycle a mode, toggle a checkbox or enter/leave slider adjustment.
- Clock: select Adjust Time for the six-button toolbar. Rotate between Back,
  hours -/+, minutes -/+ and AM/PM. A short +/- press changes on release;
  holding repeats every 250ms after the source's 1500ms hold threshold (first
  repeat at 1750ms). AM/PM is unavailable in 24h. Backspace returns to Clock.
- System: Enter toggles Tool Tips/VR Examples or a unit preference. Conditional
  unit rows can be enabled independently by the flags above. About opens the
  disclosure/version routes; rotate to scroll disclosures in 28px steps. Version
  uses the original unavailable strings until a service supplies values. Restore
  starts on No; selecting Yes runs a local mock reset of implemented preferences.
- Backspace: leave slider adjustment or cancel reset; leave rows for tabs, then return Home.
- H: Home. M: Entertainment. N: Navigation/Compass fallback.
- F: reports that the Favorites service is disconnected; it does not open Settings.
- 1–4: synthetic Park, Drive, unavailable and Reverse observations.
- Pointer: circular home hit areas and root-list/tab selection use the same routes.
- Escape/window close: quit.

## Actual scope and remaining work

This is the first source-guided native port, not the complete Mazda HMI. Home's
final states and five routes are ported. Applications, Entertainment and
Communication root lists use the firmware's visible default entries; actual
vehicle configuration can show different entries. List services are disconnected
and report that fact in the window title. There is no radio, phone, Bluetooth,
media playback or vehicle adapter. Existing mock playback code is preserved in
`mazda-mock`, but this UI does not pretend those stock services have been ported.

Settings Display now has its source image header, three-row viewport with a clipped
trailing row, exact English labels, Auto/Day/Night choices, -5..5 brightness/contrast
sliders, display-off/clock and reset confirmation. These manipulate session-local
host state only. The preview image responds to slider changes using an approximate
software tone curve; it does not set the Mac backlight or reproduce OEM calibration.
Pointer sliders support dragging; Enter enters/leaves knob adjustment. Reset starts
on No; Yes restores Display defaults only. Any input wakes the display without
activating the covered control. Remaining Settings tabs and deeper contexts still need
conversion. Invalid/moving/non-Park observations revoke ongoing editing/confirmation
and cannot authorize preference changes.

Sound now uses its original 64px rows below the tab header at y=133, with five full
rows and the clipped next row. Standard audio has Bass/Treble (-6..6), Fade/Balance
(-8..8), ALC (0..7), Guidance Volume (0..63) and Beep. Bose substitutes Centerpoint
and AudioPilot for ALC; the J12 variant omits Centerpoint. NA English strings retain
the registered marks. Bass/Treble/Fade/Balance fills extend from the midpoint;
ALC/Guidance fill from the start. Source tick marks, center marks, end labels and
enabled/disabled checkbox artwork are imported without executing firmware code.
The Sound increment added six PNGs; the current Clock increment adds eighteen
more, bringing that local render set to 65 images. System adds eleven more PNGs
for dialogs, disclosure and shared Back, bringing the current set to 76.

All Sound values are host-session state; there is no audible output, amplifier,
navigation prompt or OEM-setting write. The firmware's Guidance Volume message
mapping subtracts one for positive status values; actual service mapping and the
associated SBN/prompt are not ported. The host applies its conservative Park gate
to Sound edits too; this is not proof of identical OEM permission policy. Missing
guidance-service availability independently disables that row and revokes editing.
The default equipment fixture is standard audio/no HUD, appropriate to the factory
2018 US Touring configuration; runtime equipment detection remains future work.
Changing test profiles cancels interaction and keeps values only in memory.
The Compass is explicitly the firmware's **no-navigation host fixture**; its text
is not a claim about whether the owner's actual car has navigation. It displays
unavailable coordinates and a dimmed direction mask. Maps/GPS are not connected.
Clock starts at 12:00 PM for repeatability and can now be adjusted in memory.
Its status-bar and off-screen clock labels share that value. The original ClockTab
has five NA rows: Adjust Time, GPS Sync, Time Format, Time Zone Select and Daylight
Savings Time. GPS Sync disables manual adjustment; with the explicit navigation
available fixture it also disables time-zone/DST changes. This fixture does not
change the Compass fallback or establish the owner's installed navigation state.
The manual screen uses original background, colon and six-button UMP artwork with
150px Inter digits. Font sizes are bounded at 160px, with hinting still disabled
and existing 64-run/512-glyph cache limits retained. Source hour changes preserve
the AM/PM half-day in 12h, and minute changes wrap without hour carry. Held input
is cancelled when the target, focus, pointer location or host editing gate changes;
backlogs are bounded. Disabled rows/buttons are skipped during rotation.

Select Time Zone retains all 101 firmware labels and their original order locally.
Selecting a radio stays in the list; Back restores its root row. Labels reflect
this historical firmware, not verified current time-zone rules. GPS, time-zone and
DST calculations, a ticking time service, OEM messages and the original service
settle delay are not implemented. Time-zone/DST selections do not recalculate the
displayed time. No OS clock is changed. UMP tooltips/dwell and animations remain.
Home focus/activation
animations, deep context transitions, exact touch-release timing and platform
accessibility semantics remain incomplete. This work does not certify vehicle
startup, cameras, interlocks, firmware compatibility, or runtime integration.

## System controls and information

System preserves the NA row order from `_populateListCtrl`. The initial fixture
has Tool Tips, VR Examples, Language, Music Database Update, Restore All Factory
Settings and About; temperature/distance availability starts absent, as in the
declaration's helper defaults. Explicit flags add either row in its original place.
Temperature and distance preferences remain independent; their initial °C/km
values follow the source row declarations, not the owner's current settings.
Simulated ignition/CAN readiness defaults present and can be withdrawn; restore is
then disabled, and the source's New-CMU fixture also disables Distance. All edits
additionally use the conservative host Park gate, not a certified OEM policy.

About's disclosure page reads the full original legal text locally, wraps Inter
within the source 640px body, preserves paragraph breaks and clips scrolling text
to the 740×416 panel. The native line height uses a 34px approximation of the
browser's normal 28px body text. ScrollDetail background/header/shadow/scrollbar
artwork is original. Exact browser metrics, scrollbar compositing, drag/flick,
left-button focus transitions and meter animation are not fully reproduced.
Version Information uses Dialog3 style23 and its unavailable service strings;
the imported firmware filename is not presented as the car's running OS version.
Restore uses the original full-screen No/Yes and progress artwork, hides the
status bar, defaults to No, and after a synthetic 1500ms resets Display, Sound,
Clock and System preferences only. Equipment, source observations, files and OS
settings remain unchanged. Loss of Park/services cancels before completion.
Language selection, complete localization, music-update contexts, real Tool Tip/
VR help overlays and OEM service/reset/progress timing remain pending.

## Render comparisons

```sh
cargo run --locked -p mazda-desktop --example render-previews -- target/inter-previews
cargo run --locked -p mazda-desktop --example render-previews -- target/tipperary-previews target/owner-firmware/render-assets target/owner-firmware/reference/jci/fonts/Tipperary.ttf
```

Both save native framebuffer PPMs. The second uses the extracted original font in
the **same Rust layout**; it is a typography A/B, not a screenshot of the original
Opera runtime. Geometry is checked against the extracted CSS/JS and original
artwork. An independent original-browser render was not obtained: automatic
approval review rejected a localhost reference server, and browser policy rejected
direct file URLs. No server or browser-policy workaround was used.

Mazda/JCI assets and fonts remain proprietary owner-supplied material, outside the
project's MIT license. They, generated captures and local app binaries are not
committed or uploaded. Inter's SIL OFL license is included separately. The earlier
custom dashboard is preserved only on the local `feature/home-typography` branch.


## Full Settings work in progress

[Coverage and remaining work](CONVERSION_PROGRESS.md) tracks 192 declared contexts
across the six relevant firmware apps. `tools/catalog-settings.py` parses literal
declarations without evaluating JavaScript; dynamic expressions remain explicit
inert data. Generate the owner-local catalog with:

```sh
python3 tools/catalog-settings.py target/owner-firmware/reference target/owner-firmware/settings-catalog.json
python3 tools/test-catalog-settings.py
```

A catalogued context is not an implemented screen. The current Display/Sound/Clock work
does not claim completion of all Settings or the full conversion. The
[deployment plan](CMU_DEPLOYMENT_PLAN.md) explains the older ARM/kernel/libc ABI,
stock compositor and priority/fallback constraints that must be resolved before
any vehicle proposal.
