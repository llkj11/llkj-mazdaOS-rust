# Music Database Update: source UI and simulated services

This increment ports the music update branch of `sysupdateApp.js` into Rust.
It is accessible at Settings → System → Music Database Update. It retains
Dialog3/List2 geometry, source labels, initial focus and status/full-screen
behavior while rendering text in Inter. Centered dialog text has a 15px ink
clearance within each side of the source 646px box so wider Inter runs do not
collide with the shared Back arc. This is a host simulation, not an updater.

## Static source basis

The owner's pinned NA 70.00.100A extraction supplies:

- `jci/gui/apps/sysupdate/js/sysupdateApp.js`: context declarations, ready callbacks,
  package filtering, version substitution, progress and button enablement.
- `jci/gui/common/controls/Dialog3/js/Dialog3Ctrl.js` and its CSS: zero-based
  `initialFocus`, styles 02/03/13/17/23, 64px full-screen content offset, button
  positions, text wrapping and the determinate meter's centered background crop.
- `jci/gui/common/controls/List2/js/List2Ctrl.js` and CSS: style02 title and normal
  style06 rows. Current has no activation; package rows have no caret.
- Original `sysupdate` and `common` English/Spanish/Canadian French dictionaries,
  already converted into the ignored owner-local language catalog.
- Four additional original images: IndeterminateMeter_2, progress background,
  progress highlight and dialog title separator. The fixed image set is now 81.

The public hash ledger contains 140 selected source identities. All were rehashed
and compared with the original extraction inventory. No firmware code executed.
The firmware sends UI events to MMUI; that external state machine and real
service responses have not been reproduced or validated on hardware.

## Context coverage in this increment

| Firmware context | Host behavior |
| --- | --- |
| SearchMusicDBUpdates | Cancel/Search; Search initially focused |
| SearchingForMusicUpdates | Original meter first frame and Cancel |
| AvailableMusicPackages | Current row and synthetic SIM-1 package row |
| ConfirmMusicUpdateInstall | Current version or `-----`, selected package, Back/Install |
| MusicRebootReflash | Preparing to Update; no firmware restart |
| InstallingMusicDBUpdates | Full-screen source progress meter and messages |
| RetryingMusicDBUpdates | Same full-screen meter for a synthetic retry |
| UpdateMusicDBSuccess | Source success/restart messages, OK returns to originating row |
| MusicUpdateSearchFailed | Source error and Cancel/Retry |
| NoNewMusicPackages | Default no-media result, OK |
| MusicDBPkgValidationFailed | Certificate failure fixture, OK |
| MusicDBPkgRetryValidationFailed | Original titled style17 and error text |
| FunctionNotAvailable | Full-screen unsuccessful/cataloging message |
| UpdateMusicDBNoSuccess | Cancel/Retry; retry initially focused |
| NeedFailSafeImageUpdate | Source dealer/fail-safe message; no fail-safe operation |

Source screenshots and independent Opera rendering are not available as a visual
oracle. Source declarations, CSS, images and dictionaries support these mappings;
the Rust renders and native Mac checks establish the host implementation only.

## Run and exercise

After the offline import, the ordinary run defaults to no update media:

```sh
cargo run --locked -p mazda-desktop
```

To exercise the success path without any real update package:

```sh
cargo run --locked -p mazda-desktop -- --music-update available
```

Other finite fixtures: `no-media`, `search-fails`, `invalid`, `install-fails`,
`retry-invalid`, `unavailable`, `needs-failsafe`. Add `--language es_MX` or
`--language fr_CN` for owner-local translations. Arbitrary filenames/paths are
rejected. `install-fails` fails the first attempt and succeeds after explicit
Retry; `retry-invalid` fails retry validation. Fixtures remain outside product UI.

Search takes 1200ms, preparation 800ms and installation/retry 4000ms of host
simulation time. These are test timings, not measurements of Mazda's services.
Each advance can enter only one context; long stalls cannot skip confirmation
or all intermediate screens. Before any success, the installed version is
unavailable; success changes only the in-memory mock version to `SIM-1`.

Arrow keys and Enter/Space use Commander navigation. Back retains the originating
System row. Current is non-activating. Synthetic Driving/unavailable speed or lost
service readiness disables Search/Install/Retry and revokes pending simulated
work. Global domain shortcuts cancel the host flow. This revocation rule is a host
policy, not an assertion that an actual in-progress vehicle update is cancellable.

## Limits and continued work

- No USB enumeration, media scan, archive parsing at runtime, payload validation,
  updater launch, network call, system restart, vehicle write or persistent version
  storage exists. The original restart/vehicle wording has no connected action.
- The source spinner is an animated PNG. The bounded importer currently decodes
  only its default PNG image; frame animation remains to be ported. Progress fill
  changes with time and uses the original centered artwork rather than scaling it.
- One synthetic package and unavailable/current mock versions are covered. Real
  package-list transport, arbitrary error payloads, auto-update notifications,
  OEM global-event transition rules and frame-perfect animation remain unverified.
- Software/fail-safe update branches, global languages, and the remaining Settings
  contexts are still required. A 15-context host port does not complete the
  192-context Settings catalog or the full conversion.
- Native keyboard routing has been exercised; pointer routes have model checks,
  but real Mac pointer activation remains unverified.

See [the conversion ledger](CONVERSION_PROGRESS.md) and
[the car deployment gates](CMU_DEPLOYMENT_PLAN.md). A stock-service integration
must separately understand update supervision and failure recovery. This module
must not be repurposed into a firmware installation tool.
