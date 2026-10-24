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
| audiosettings | 1 | 0 | SoundTab host controls implemented for standard/Bose/Bose J12; real services, guidance SBN and detailed timing remain |
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
- Settings Display: source image/header and three-row viewport restored; mode choices,
  -5..5 sliders with keyboard/pointer drag, screen-off/clock, wake and No/Yes reset
  implemented in host state. Unit/render checks pass; native verification recorded
  in the validation log. Other roots, submenus, dialogs and equipment variants
  remain required. The 192-context ledger above is not completed by this component.
- Remaining domains: media/phone/application/navigation context conversion and
  simulated behavior still pending; preserve original routes and artwork.
- Runtime/device boundary: follow [the CMU deployment plan](CMU_DEPLOYMENT_PLAN.md).
  ABI, native graphics/input, stock services, camera priority, fallback and an
  exact-car installation proposal remain unproven.
- Settings Sound: source-defined rows, ranges, pivot/progress fills, ticks, labels,
  checkboxes and disabled Guidance Volume are implemented in session state for
  standard/Bose/Bose J12 profiles. The 2018 Touring host fixture defaults to
  standard audio without HUD, based on factory equipment. This is not live
  amplifier/equipment detection. Shared tab navigation now preserves directional
  positioning and first-offscreen arrow jumps. Native/model checks are recorded
  in the validation log; the full Settings objective remains active.

## Required proof before completion

Every context/control needs source evidence, implemented routes, rendered fidelity
checks and behavioral tests covering normal, unavailable and restricted states.
Owner assets stay local. CI must pass for pushed source, and the latest packaged
native app must be exercised. The complete goal is still active.
