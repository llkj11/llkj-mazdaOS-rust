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
