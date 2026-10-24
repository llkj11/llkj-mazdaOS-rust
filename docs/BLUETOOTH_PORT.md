# Devices and Bluetooth: firmware UI, host-only services

The NA Devices tab and 25 btpairing contexts now have Rust host views and routes.
This uses the owner's firmware layout/artwork and Inter text. It does not access
the Mac's Bluetooth adapter, discover real phones, read contacts, pair devices,
change macOS settings or send any Mazda service/vehicle request.

## Source basis and equipment

Static review used `syssettingsApp.js` DevicesTab declarations, dynamic row
construction and enablement; `btpairingApp.js` declarations, list builders, ready
callbacks, profile flags, timeouts and UI event callbacks; Dialog3/List2 JS/CSS;
and DialPad2 control/template JS/CSS. The original NA dictionaries and thirteen
additional images provide the text and artwork. All 161 selected source identities
were rehashed against the original extraction inventory.

The North American source excludes Network Management/Wi-Fi. Android Auto and
Apple CarPlay start hidden until the system reports capabilities. Default host
launch therefore shows Bluetooth only. `--projection-fixture` exposes both
projection preference rows for testing; it does not detect the owner's USB hub,
retrofit, licensing, phone or installed CMU. Independent capability/disable flags
and projection-active restrictions remain work.

## Native context mapping

| Firmware context | Host view/behavior |
| --- | --- |
| syssettings.DevicesTab | NA list, optional projection checkbox fixture |
| syssettings.ConfirmDisableCarPlay | Original full-screen, one-Yes acknowledgement after disabling; Back leaves disabled |
| BTConnectionManager | Bluetooth checkbox, Custom Settings, Add New Device, mock devices |
| BluetoothSettings | Information, Change Pairing Code, Unpair All Paired Devices |
| BluetoothInformation | Original style17; Mazda name and unavailable address, no real MAC read |
| ConnectionType | Phone and Audio / Phone Only / Audio Only radios, Disconnect, Unpair |
| Discoverable | Original style22 and 180-second source countdown |
| NumericConfirmSSP | Original style05, synthetic code, Cancel/No/Yes; Yes initially focused |
| NumericConfirmLegacy | Original style05, synthetic four-digit incoming code, Cancel only and 30-second timeout |
| PairingInProcess | Original progress text and meter image |
| PairSuccess | Paired record exists; connection is a separate simulated operation |
| ConnectingDevice / ConnectSuccess | Source progress/result dialogs |
| DisconnectingDevice / DisconnectSuccess | Source progress/result dialogs |
| DeleteWhichDevice | Device list plus All, original GIF remove icons |
| ConfirmDelete | No initially focused, one device or All |
| DeleteDevice / DeleteComplete | Source progress and timed result |
| PairCancelling | Source cancellation progress |
| DisconnectingToPair / NoPairDisconnectFail | Pre-pair disconnection and failure/retry |
| PairExitPopUp / ConnectFailed / DisconnectFailed / DeleteFailed | Source layouts, error text, Retry initially focused |
| ChangePairCode | DialPad2 style03, four holders, original keypad/clear/cancel/OK artwork |

The normal list retains 64px rows, five-row scrolling, source indentation,
checkbox/radio/profile icon cells and reserved title-indicator space. Dialog3
styles 02/03/05/13/14/17/22 retain their source boxes. The PIN keypad uses the
source positions and rotary order: Clear, 1–9, Cancel, 0, OK; it starts on 1.
Clear removes one digit, OK requires four digits, Cancel/Back retains the prior
session code. Disabled OK uses source half-opacity styling. Number keys 1–4 in
the desktop remain vehicle-scenario test shortcuts; use arrows/Enter or the keypad
buttons for PIN entry.

## Finite service fixtures

After the offline import:

```sh
cargo run --locked -p mazda-desktop -- --bluetooth-fixture paired
cargo run --locked -p mazda-desktop -- --bluetooth-fixture ssp
```

Allowed names: `empty` (default), `paired`, `ssp`, `pair-fails`, `connect-fails`,
`disconnect-fails`, `delete-fails`, `legacy`. Unknown names or paths are rejected. Paired
fixtures contain only Simulated Phone and Simulated Player; the latter supports
audio only. No personal device data is imported. The source's independent phone
and audio ownership is modeled: switching the player to Audio Only preserves the
phone profile on the first device. Bluetooth Off retains paired records and clears
connections; the manager hides those records until Bluetooth is enabled again.

`ssp`/`pair-fails` create a synthetic incoming request after 1500ms. Code `123456`
and initial PIN `0000` are synthetic fixture values. No actual security credential
is generated, transmitted or persisted. Pair/connect/disconnect/delete operations
take 1500ms of host simulation time. Result dialogs use the source 3000ms timeout;
SSP and legacy confirmation use the source 30000ms timeout. Every advance can enter only one
context. UI callbacks send events to external MMUI in the firmware; these host
service completions and parent routes are finite simulation choices, not proof of
the complete OEM state machine.

Failed operations preserve pair records and profile ownership. Unpair requires
explicit Yes. Loss of confirmed Park or service readiness revokes pending host
operations before their next mutation; the triggering event is consumed rather
than applied to a newly exposed parent screen. Global domain shortcuts cancel
pending host work. These host rules do not establish the actual vehicle's
in-motion service policy or real-operation cancellation guarantees.

## Data handling and limits

The fixed image set is now 94. The original 37×37 unpair GIF is a single frame;
its decoded RGBA pixels independently matched Pillow. The public importer uses a
bounded standard-library GIF decoder: at most 4 MB input, 1600×960 pixels, 4096
dictionary entries, one full noninterlaced frame, valid palette/pixel count, explicit end
codes and exact EOF. Extra frames/unsupported layouts are rejected. No Python
dependency or Rust crate was added.

The owner-local language catalog now covers seven apps and is 323541 bytes.
One original Spanish Bluetooth string uses CRLF; the importer normalizes CRLF to
one newline in dictionary values. Other control characters remain rejected, and
IDs/locale mappings are not normalized. The runtime retains its 4 MB and bounded
string/catalog validation. No source JavaScript is evaluated.

Remaining btpairing contexts: PartialSuccess, Mobile911Enable,
InvalidLegacyPIN, MaxDevices, DeleteiOS, TurnBTConnOn, RemoveOldEntry, DeviceSetUp,
DisconnectBtForAA, NoNaviCon. Also pending: active-projection policies, real service
schemas, profile capability variants, notifications, battery/signal indicators,
long-press PIN clearing, spinner APNG frames, native pointer activation (automated Mac clicks did not activate controls), full region/
equipment variants and integration with the System factory reset. The existing
factory-reset host flow does not yet reset these new Bluetooth/projection values.

Generated Rust screenshots and Mac keyboard tests establish host behavior only;
there is no independent Opera screenshot oracle or hardware-runtime proof. Full
Settings and the remaining frontend domains are unfinished. Follow the
[deployment gates](CMU_DEPLOYMENT_PLAN.md) before any separately approved bench or
vehicle work; this module has no car installation or update mechanism.

## Legacy pairing follow-up (0.11.1)

`--bluetooth-fixture legacy` enters Discoverable, then presents the source
NumericConfirmLegacy style05 screen after a synthetic incoming request at 1500ms.
Its incoming code `1234` is fixture data, separate from the user-editable pairing
PIN. The actual source takes this field from a Connect message and displays its
last four padded digits. No real Connect message or device credential is read.
The source offers Cancel only: Enter/Back cancel, and rotation cannot expose a
Yes action. It waits until the original 30-second deadline without adding a device;
then the host routes to its existing timeout failure/retry screen. That parent
route is a simulation choice, not a traced external MMUI transition. A real phone
completion, invalid legacy PIN and other legacy outcomes remain unimplemented.
