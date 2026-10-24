# Firmware language selection and North American host fixture

The native language path now implements `ChangeLanguage`, `LanguageConf`,
`LanguageChangeProgress` and `LanguageChangeError`. System → Language opens the
original List2 title/radio layout and original voice-support icon. Confirmation
and retry use full-screen Dialog3 No/Yes controls, defaulting to No. The selected
language changes only after a successful mock completion. Failure retains the
previous language; Yes retries and No returns to the list. Loss of the host Park
gate cancels a pending change. Back returns to the original System row.

## Source and fixture scope

`syssettingsApp.js` supplies the cached language list and its VR/TTS flags, list
construction, confirmation/progress/error declarations and callbacks.
`framework/js/Localization.js` maps enum IDs to GUI locale codes. Original
`common`, `system`, `syssettings`, `audiosettings`, `emnavi` and `sysupdate`
dictionaries supply text. Their hashes, and the original voice icon identity, are
recorded in `firmware-ui-evidence.json`.

The explicit host fixture uses the first three NA entries of the cached list:

| Source enum | GUI dictionary | Meaning |
| --- | --- | --- |
| LANGS_US_ENGLISH | en_US | US English |
| LANGS_NA_SPANISH | es_MX | North American Spanish |
| LANGS_CN_FRENCH | fr_CN | Canadian French, using the firmware's original spelling/code |

This is not a captured `LanguagesList` response from the owner's CMU. The source
cached declaration contains 42 global entries; the remaining languages,
`UseUKEnglish`, `UseKorean`, voice-language fallback, RTL scripts and broader font
coverage remain conversion work. Do not treat this increment as the complete
multilingual HMI or full Settings conversion. The microphone art reflects the
cached fixture flags, not a working speech-recognition service.

## Local data and text rendering

`tools/firmware_languages.py` reads literal objects/arrays with the existing inert
reader. It validates the three fixed ID/code mappings and boolean flags, retains
app/string IDs, and writes a 302082-byte owner-local `languages.bin`. It never
loads or evaluates firmware JavaScript. The dictionary bytes are proprietary and
remain ignored with the other owner assets.

The runtime accepts only the `MZLANG01` bounded data format: at most 4 MB, 1..64
unique locale/enum entries, 1..4096 rows per locale, fixed app names, UTF-8 strings
of at most 4096 scalars/16384 bytes, bounded metadata and boolean flags. Trailing,
truncated, duplicate or invalid data fails. Failed loads preserve existing data.
Lookups are app-scoped; known tabs/home captions use their exact source IDs.
Ambiguous English aliases fall back instead of guessing between different IDs.
Future converted controls should prefer explicit source IDs where aliases collide.

The existing ported screens now use the selected dictionaries before measuring,
fitting or centering their labels. This includes Home, root menus, Display, Sound,
Clock, System, About and version messages. The disclosure body remains the original
English `legalDisclosure.js`, as supplied by the firmware. Its translated heading
uses the original maximum of two 48px lines plus margins. Version text3 retains
its source one-line ellipsis. The longer Compass explanation is fitted to three
lines in the existing area with Inter at 16..22px; it does not cross the separator.
Paragraph fitting uses a bounded 16-entry cache, with at most 4096 input scalars
per entry, to avoid repeating width searches every frame. The existing 64-run,
512-glyph and 160px font-size bounds remain; font hinting is still disabled.

## Run and simulation boundary

```sh
cargo run --locked -p mazda-desktop -- --language en_US
cargo run --locked -p mazda-desktop -- --language es_MX
cargo run --locked -p mazda-desktop -- --language fr_CN
# Deterministic service failure to exercise retry:
cargo run --locked -p mazda-desktop -- --language-change-fails
```

A normal launch starts in English. A selection has a synthetic 1500ms completion,
not OEM service timing. Values stay in memory; restore resets the host language
to English while preserving equipment and failure fixtures. There is no macOS
language setting, OEM command, voice output, TTS engine, network call or vehicle
write. Success wink/SBN, meter animation, service messages, exact transition/touch
release timing and native accessibility remain incomplete. Native mouse activation
has not been established; keyboard and model routes are verified separately.
