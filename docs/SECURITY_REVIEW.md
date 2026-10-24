# Initial static review — 2026-10-24

Reviewed upstream `mkaoudis/mazda-os` at
`7ef75534584e288dc4625562c6f87112a4f18482`, before running its code.

## Conclusion and execution boundary

No evidence of cryptomining, credential harvesting, hidden outbound communication,
obfuscation, persistence, or malicious host installation was found in the reviewed
repository. This is a bounded manual static review with automated searches, not a
malware-free certification. The host simulator can proceed with locked dependencies.
The separate hardware utility is **not approved for vehicle use by this review**.

## What was inspected

- All 25 current tracked files: five crate manifests, workspace manifest, full
  lockfile, Rust sources, POSIX collector, Python FAT parser, license, workflow,
  configuration and documentation. MIT copyright and permission notice retained.
- History metadata and changed paths across 48 main-line reachable commits and 61
  commits reachable from all fetched branches; content searches across all 122
  unique reachable blobs. No binary blobs, submodules, symlinks, executable-mode
  files, Git LFS pointers, Cargo configuration, local build scripts, hooks, Docker
  files, or packaged releases were present in the fetched source trees. Historical
  versions were screened by searches, not individually line-audited in full.
- All 95 registry packages in `Cargo.lock`, including foreign-platform entries:
  downloaded archives as data, verified every SHA-256 against the lockfile,
  inventoried files and rejected traversal/link archive entries before reading.
  All use crates.io; no Git dependencies or custom registries in the lockfile.
- Direct dependencies are `minifb 0.28.0`, `font8x8 0.3.1`, and `plist 1.10.0`
  (the latter belongs to the hardware utility). Focused review of desktop native
  build code and sensitive process/network patterns, including `minifb`'s macOS
  Objective-C sources, `cc`, compiler discovery, and compiler-version probes.
  `minifb` compiles local windowing code and links Metal/MetalKit; it does not
  download a native runtime during this build. The core, mock and UI contain no
  process launch, network transport or vehicle-device access.
- OSV batch lookup of all 95 exact package versions: one result,
  [RUSTSEC-2024-0384](https://rustsec.org/advisories/RUSTSEC-2024-0384.html),
  marks `instant 0.1.13` unmaintained. It is a minifb WebAssembly dependency and
  absent from the active macOS dependency tree. This is a maintenance warning,
  not evidence of malware. No other advisory was returned at review time.

## Material findings

1. **Privileged USB launcher, intentionally present upstream.**
   `apps/cmu-inspect` constructs an update filename containing shell command
   substitution. It uses a documented CMU update-scanner injection to run its
   collector as root before checking firmware. The collector reads fixed system
   observations and writes reports/receipts to USB. The host preparer queries
   `diskutil`, clears generated-file attributes with `xattr`, and creates/removes
   payload files. It is not a passive read and is specific to a 2019.5 CX-5,
   not the user's 2018 Mazda6. Preserve it as upstream research; do not run
   `prepare-usb`, prepare media, connect to a CMU, or run its shell payload here.
2. **Upstream CI downloads and executes BusyBox 1.19.0.** The download has a fixed
   SHA-256, but that alone does not establish binary safety. The fixture uses
   `sudo chroot` and recursive ownership repair. That binary was not downloaded,
   executed, reverse-engineered, or cleared by this review. Upstream action refs
   were also mutable. This fork replaces that workflow with host-only checks and
   fixed action commits. The workflow still trusts GitHub runners, official
   Ubuntu development packages and the pinned action implementations; those
   actions were not exhaustively audited. No vehicle fixture runs in fork CI.
3. **Firmware assumptions differ.** Upstream documentation targets its author's
   CX-5. Matching a firmware filename does not establish Mazda6 compatibility,
   installed firmware, CMU part number, graphics ABI, or input/service interfaces.

## Limits

All crate checksums establish consistency with the supplied lockfile, not publisher
trust. Registry sources and dependency build paths were screened, not exhaustively
proved correct; target-specific proc macros, all native binaries in foreign-platform
crates, compiler internals, OS frameworks and external action implementations were
not reverse-engineered. A text-pattern scan cannot rule out subtle malicious logic.
No commercial antivirus, dynamic malware sandbox, or network packet capture was
used. Build/test execution is restricted to reviewed host code; it does not certify
hardware behavior. No firmware package is committed or redistributed with the fork.

The official Rust toolchain was installed in this task's local `work/toolchain`
directory without shell-profile changes. The installer SHA-256 matched the checksum
served over HTTPS by `static.rust-lang.org`:
`f7c7e11eb2e07f006e532a8aaac77b32f4527d305ac386c0c633ca6133d35ca9`.
Repository code was first built offline after this static gate. There are no Docker
containers for this project to rebuild or restart.


## Home and typography increment, 2026-10-24

This section supplements the original upstream review above. `Cargo.lock` changes
only for the bitmap-font removal and eight registry packages listed below. No
workflow or vehicle utility behavior is added. Inter's binary font and OFL notice
were acquired as data, verified against official Google Fonts Git blob identities
and SHA-256 values, and kept unmodified; provenance is in the font asset directory.

Before compiling, all eight archives were hash-verified against the updated lock,
checked for archive path/link hazards, inventoried for build scripts, and their
manifests and sources screened for process, network, credential, filesystem and
unsafe install behavior. None adds a build script. The bytemuck derive proc macro
uses the already locked proc-macro2/quote/syn libraries. Sensitive-pattern matches
were documentation examples or a test-only rebaseline environment variable. No
malware, mining, credential access or runtime network behavior was found in this
focused review. This is not an exhaustive semantic audit or guarantee of safety.

| Added package | Version | Archive SHA-256 |
| --- | --- | --- |
| bytemuck | 1.25.2 | `95832e849adfb21180ccb6826a99da14e5d266ae5c2e668e1602cf234f153797` |
| bytemuck_derive | 1.12.1 | `6a1f896587b6f2c069c73d2f0913e2d590c3990285cd2f0b6aa02b786b4c679c` |
| font-types | 0.12.7 | `835ae9d781464a79026cde387f9cbc650c9a4909d7cf7e658b127e3c35d2f35d` |
| read-fonts | 0.41.0 | `046a7d674daf459825b32f5062056d6882db0d2f5a479fbd76ccfc870ac18709` |
| skrifa | 0.44.0 | `819ab7d62b1d3e72d9d9dea5650bac30424f9111364bb94928dbf5ecad1baa68` |
| swash | 0.2.10 | `6c2499c2d826531388872b2268718aed907a39bd785ab0dcfe57fab26283f92e` |
| yazi | 0.2.1 | `e01738255b5a16e78bbb83e7fbba0a1e7dd506905cfc53f4622d89015a03fbb5` |
| zeno | 0.3.3 | `6df3dc4292935e51816d896edcd52aa30bc297907c26167fec31e2b0c6a32524` |

An OSV query for these eight exact versions returned no advisories on 2026-10-24.
That lookup can change and does not replace source review. An initially considered
fontdue option was rejected before compilation because its parser carried an
unmaintained notice, RUSTSEC-2026-0192; fontdue/ttf-parser are absent from the final
lockfile. The earlier `instant` WebAssembly maintenance warning remains as scoped
above. No firmware data, local binary, or user file is part of this source change.

Runtime text is drawn exclusively from the bundled font; the new UI has no file,
network, credential or vehicle-write capability. Unknown or invalid vehicle data
continues to lock preferences. The package includes both the project MIT notice
and Inter's SIL OFL 1.1 notice. No Docker configuration or containers are involved.


## Owner-firmware UI extraction and port, 2026-10-24

The explicit follow-up authorizes local extraction as the visual/navigation
reference. The importer pins the previously inventoried whole-file SHA-256 and
uses the system `unzip` program to read one rootfs entry. The archive password is
publicly documented, not a recovered personal credential. Python gzip/tar readers
handle both compression wrappers as data. The importer writes only bounded regular
files under `jci/gui/` and `jci/fonts/`; it refuses absolute/traversal paths and does
not create archive links/devices or preserve executable modes. None of the archive
update instructions, scripts, native programs or JavaScript is executed. The final
font renderer disables embedded hinting instructions and rasterizes outlines. A
preliminary original-font comparison used the library's embedded hinting before
that option was disabled; it did not run firmware native programs or scripts. The PNG converter supports only bounded RGB/RGBA8, validates chunk CRCs,
requires the end chunk, and bounds decompression to declared dimensions. Runtime
Rust accepts a fixed-name bounded raw RGBA format, not arbitrary image plugins.

The extracted source establishes the original layout, labels and routes. It is
not an audit of all proprietary firmware logic. No claim is made about Mazda's
package signature, all native binaries in the firmware, or real vehicle safety.
No dynamic firmware execution, firmware installation, device connection, network
service, radio/phone/GPS integration or vehicle writes occurs. Original Downloads
archives were rehashed and remain unchanged. Firmware-derived source/artwork/fonts
and local captures/app packages are excluded from Git; only source identities and
our importer/native code are committed. The local app's owner-artwork notice makes
clear that Mazda/JCI resources are not covered by this project's MIT license.

No new crates were added beyond the reviewed typography set above. The native app
reads local converted assets, and the comparison example can read an explicitly
supplied original font. This adds local data reads, not a runtime download or
firmware execution path. Six synthetic importer tests check PNG filters, color and
alpha, CRC/truncation, format/expansion bounds, and rejection of an unreviewed
package hash. Host tests check native asset bounds and compositing, source-guided
navigation, and that disconnected actions leave mock observations unchanged.

A localhost reference HTTP server was rejected by automatic approval review before
starting. A direct-file browser reference was also rejected by browser URL policy.
Neither was bypassed. Verification uses static source/artwork inspection, native
renders and the actual reviewed Rust app; no independent Opera/browser render is
claimed. There are no Docker containers to rebuild/restart.


## Settings catalog, Display controls and platform plan

The static Settings declaration reader treats function calls, member references and
callbacks as inert expressions. It never evaluates firmware JavaScript. Synthetic
parser tests cover nested objects, comments/strings/Unicode, malformed declarations,
member references, callback expressions and reference cycles. The resulting owner
catalog remains under ignored `target/`; only our parser and metadata are public.

Display control changes modify an in-memory host session and preview pixels only.
There is no transport for changing OEM or Mac system settings. Source observations
remain read-only. Edit/drag/reset state is revoked when valid zero speed and Park
are lost. Screen-off is a local render state; user activity restores it. The reset
button resets Display values only and defaults to No. The additional twelve artwork
files are bounded PNG data decoded with the already reviewed converter, not new
scripts/native libraries. No dependencies or Docker services were added.

The platform plan reads a small whitelist of original configuration files and ARM
libraries as data. ELF metadata, paths and hashes inform the plan; ARM libraries and
startup scripts were not executed. Their bytes remain local. Kernel 3.0.35/glibc
2.11.1 and the OEM compositor/supervisor imply unresolved deployment work, not a
claim that the host binary is car-compatible. No install/flash payload is created.

## Sound controls and equipment fixtures

The Sound port reads the original audiosettings declaration/callback text and
List2/Slider CSS/JavaScript as data. Six additional PNGs (checks and slider ticks)
were verified against the existing extraction's source hashes and decoded with
the reviewed bounded converter. Firmware JavaScript/native programs were not run.
The asset allowlist grows from 41 to 47. No crates, build dependencies, network
client, shell invocation, credential reads or Docker component were added.

Tone, balance, ALC/guidance and Bose checkbox changes are in-memory fixture values;
they do not emit OEM messages or change real audio. Model and public-route tests
cover limits, all three equipment profiles, guidance-service disablement, drag
cancellation, Park revocation and unchanged media observations. A centered/mirrored
crop helper handles original pivot artwork while preserving bounded row stride.
CLI profile values are a fixed enum and invalid arguments fail before window run.
VIN decoding used the official NHTSA endpoint only for the requested trim lookup;
the VIN, mileage and screenshot are excluded from the source/docs/artwork upload.

## Clock contexts and bounded owner text

The Clock port statically reads syssettings, ClockSettings2, Ump3 and Button
declarations/CSS plus the English dictionary. Eighteen additional PNGs were
checked against the extraction inventory and decoded as data, bringing the fixed
allowlist to 65. The existing inert literal reader is factored into
`tools/firmware_literals.py` for both importer and catalog; it does not evaluate
calls, callbacks or JavaScript. Only literal dictionary strings in the declared
time-zone order are emitted to ignored `clock-zones.txt`. Expression values,
control characters and oversized labels are rejected. The runtime requires a
version header, valid UTF-8, 1..128 nonempty rows, at most 256 Unicode scalars per
row and a 132000-byte file limit. Owner text is not embedded in public source.

The 150px clock digits require raising the renderer font-size cap from 96 to
160px; outline hinting remains disabled and cache/run-length bounds are unchanged.
Clock values, format, GPS Sync, time-zone index and DST are session-local state.
There are no OS time-setting calls, GPS/network transport or OEM/device writes.
Held +/- input uses bounded repeats and is cancelled on target/focus/policy loss
or pointer departure. Source-defined disabled rows are skipped. Synthetic tests
cover those boundaries; native keyboard behavior was exercised separately.

No crates, lockfile changes, build dependencies, runtime shell commands,
credential reads or Docker components were added. Firmware-native programs,
scripts and JavaScript were not executed. This supplements the bounded review;
it is not a review of the complete firmware or a vehicle compatibility claim.

## System preferences, disclosures and simulated restore

Eleven additional PNGs (Dialog3, ScrollDetail, meter and LeftBtn) are checked
against the extraction inventory, decoded offline and added to the fixed 76-name
allowlist. The original disclosure's object value is accepted only as quoted
strings joined by `+`, with comments/escapes parsed as inert text. Calls, property
access and other expressions fail; JavaScript is not evaluated. The resulting
`legal.txt` remains ignored and owner-local. Runtime input requires a version
header, valid UTF-8, at most 132000 bytes/32768 scalars, and excludes control
characters except newline. Wrapping is cached on asset load. Signed text origins
and explicit clipping preserve viewport boundaries without unbounded surfaces.

System controls and restore manipulate only in-memory host preference state.
Restore requires explicit Yes while Park/services are valid; loss of either
cancels completion. Equipment fixtures and read-only observations are preserved.
There is no filesystem deletion, OS reset, network, credential read, device write
or OEM reset command. Version strings use unavailable service states instead of
inferring installed firmware. Disclosure URLs are plain text, never opened or
fetched. No dependencies, lockfile changes or Docker components were added.

## Owner language dictionaries and native selection

The language increment reads six app dictionaries for the three explicit NA host
locales, the existing cached language declarations, `framework/js/Localization.js`
and one original voice icon as data. Source identities are checked against the
original extraction inventory. The new helper shares the inert literal reader;
no JavaScript, firmware program, language engine or updater is run. Locale paths
are restricted to the three validated mappings, dictionary values must be literal
strings and support flags must be booleans. Proprietary dictionaries are converted
to ignored `languages.bin`, not embedded in public source or uploaded.

Runtime decoding bounds the file at 4 MB, 64 unique locales, 4096 entries per
locale and 4096 scalars/16384 bytes per string. App names are allowlisted; IDs,
codes, UTF-8, booleans, counts, duplicate keys and exact EOF are validated. Failed
loads do not replace existing data. Text is never evaluated, followed as a URL or
used as a filesystem path. App/string identity avoids confusing different uses
of the same English word; ambiguous aliases do not guess a translation.

Language changes affect only session state. Confirmation defaults to No; change
completion/failure/retry is a deterministic host fixture, and Park loss cancels a
pending change. No macOS locale call, network request, credential read, speech
engine, OEM message or device write is added. Dialog word fitting prevents
oversized localized text from underflowing centering arithmetic. The paragraph
cache holds at most 16 entries with 4096 input scalars each; existing text/glyph
bounds and disabled font hinting remain. No crates, lockfile changes, new native
libraries, Docker services or vehicle integration are introduced. This focused
review does not certify all translations, scripts/fonts or the complete firmware.


## Music database UI increment

Reviewed the new Rust state/renderer modules, desktop fixture parsing, importer
allowlist changes and deterministic render helper. Eight allowlisted fixture
names select synthetic results; filenames, URLs and update payloads are not
accepted. All operations are session-only; the new modules add no filesystem,
network, process, credential, USB, vehicle or OS-setting calls. The default reports
no media. Source labels mentioning vehicle restart do not call any restart API.
Loss of Park/readiness cancels pending host work before the mock version changes.
Timers saturate and enter at most one context per advance. Existing image/text
bounds remain. Four PNGs were rehashed against the extraction inventory and
converted as data; the source APNG contributes only its default PNG frame.

No dependencies or lockfile changes, unsafe blocks, new CI/install commands or
Docker services were added. Owner art, dictionaries, firmware and native app
packages remain local and ignored. This focused code/static-data review does not
certify the firmware's real updater, service protocols, APNG frames or complete
firmware behavior. The host mock is deliberately unsuitable as an installer.


## Devices/Bluetooth host increment

Reviewed the Rust session, renderer, shared dialog typography extraction, fixed
fixture parser, seven-app dictionary allowlist, raw-image additions and new static
GIF decoder. All Bluetooth operations mutate finite host records only. No adapter,
phone/contact, network, process, OS-setting or vehicle I/O is added. Unknown fixture
paths are rejected. Failed/cancelled operations retain records; successful unpair
requires explicit Yes. Park/readiness loss revokes pending operations and consumes
an in-flight event before it can select a parent control. PIN input is four ASCII
digits in session memory; fixture values are not real credentials.

The GIF parser accepts one full noninterlaced frame with bounded dimensions/input,
valid palette indices and bounded LZW dictionary/output, explicit terminators and
exact EOF. Synthetic tests cover transparency, repeat codes, truncation, invalid
codes and expansion bounds. The original single-frame icon matched Pillow pixel
for pixel. APNG animation is still not decoded. CRLF normalization applies only to
dictionary values; lone CR and other disallowed controls still fail. All 161
selected source identities were rehashed against the initial extraction inventory.
No source script, native firmware program or installer was executed.

No Cargo dependency/lockfile, workflow, license or Docker service change is made.
Owner firmware, art, dictionaries, captures, PIN state and native packages remain
local. This focused review does not certify the full firmware Bluetooth stack,
protocols, legacy/active-projection/emergency-call behavior or on-car integration.


## Legacy pairing follow-up

Reviewed the added finite `legacy` CLI fixture, pending-context routing, Cancel-only
button mapping, 30-second timeout and source-dictionary rendering. The displayed
four-digit value is fixed synthetic data, separate from the PIN editor. It cannot
create a paired device, confirm on behalf of a phone or access a real adapter.
No dependency, lockfile, workflow, Docker service or vehicle/OS/network I/O changed.
The source's external Connect/completion/invalid-code messages remain unimplemented;
the timeout-to-error parent route is a documented host simulation choice.
