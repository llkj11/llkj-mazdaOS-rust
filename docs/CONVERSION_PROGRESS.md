# Conversion coverage and active work

The objective remains the rest of the native Rust conversion based on the actual
firmware UI, starting with full Settings screens/controls, preserving Mazda layout
and navigation, and changing typography only. Commits are pushed incrementally to
the owner fork. This ledger is not a claim that catalogued contexts are ported.

## Full Settings scope

Static declaration extraction currently finds these contexts:

| Firmware app | Declared contexts | Statically resolved list rows | Implementation status |
| --- | ---: | ---: | --- |
| syssettings | 28 | 115 | Catalogued; full native implementation and verification pending |
| audiosettings | 1 | 0 | Catalogued; full native implementation and verification pending |
| vehsettings | 81 | 176 | Catalogued; full native implementation and verification pending |
| btpairing | 35 | 12 | Catalogued; full native implementation and verification pending |
| netmgmt | 16 | 0 | Catalogued; full native implementation and verification pending |
| sysupdate | 31 | 2 | Catalogued; full native implementation and verification pending |

There are 192 contexts across these six apps, plus system display-off/clock contexts
and equipment-dependent list variants. Empty static row lists often mean that the
OEM services construct rows dynamically; they are not evidence that there is
nothing to port. The extractor retains unresolved expressions as inert data and
does not evaluate callbacks. All six context tables parsed without a declaration
error. A catalogued screen does not imply its routing, layout or behavior works.

## Work lanes

- Home: final static artwork states, five categories and basic routing ported.
  Focus/activation transitions and complete touch event timing remain.
- Settings: correct the initial Display skeleton to the original image header and
  three-row viewport; implement mode choices, bounded sliders, screen-off/clock,
  reset confirmation and host-state behavior. Then use the same control engine
  for all roots, submenus, dialogs and dynamic equipment variants.
- Remaining domains: media/phone/application/navigation context conversion and
  simulated behavior still pending; preserve original routes and artwork.
- Runtime/device boundary: follow [the CMU deployment plan](CMU_DEPLOYMENT_PLAN.md).
  ABI, native graphics/input, stock services, camera priority, fallback and an
  exact-car installation proposal remain unproven.

## Required proof before completion

Every context/control needs source evidence, implemented routes, rendered fidelity
checks and behavioral tests covering normal, unavailable and restricted states.
Owner assets stay local. CI must pass for pushed source, and the latest packaged
native app must be exercised. The complete goal is still active.
