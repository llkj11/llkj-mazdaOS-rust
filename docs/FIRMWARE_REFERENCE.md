# Owner-provided firmware package reference

Read-only inventory on 2026-10-24 of the owner's Downloads folder. Original files
were not modified, mounted, installed, decrypted or executed. Firmware bytes are
not included in this repository or uploaded to GitHub.

| Package | Bytes | SHA-256 |
| --- | ---: | --- |
| `cmu150_NA_70.00.100A_failsafe.up` | 7,088,769 | `ef9964509d81d47c1fcae35748096fe7021a9ea15b670acf5675a48679ddbccf` |
| `cmu150_NA_70.00.100A_reinstall.up` | 925,592,239 | `5938019df7e333447c7bf60aa8f6a7e7f9cd50d6eeb7b6e33605800b85bd4ad0` |

The local failsafe hash matches an independently downloaded copy from
[Mazda's official service CDN](https://s3.amazonaws.com/tsd.mazdausa.com/MAZDA_CONNECT/cmu150_NA_70.00.100A_failsafe.up).
The larger reinstall file was hashed and its ZIP central directory inventoried,
but it was not independently compared with a fresh vendor download.

Both files are ZIP containers with readable directory names and encrypted payload
entries, including their version/instruction files. The reinstall directory names
include Linux/system components; filenames alone do not reveal their ABI. No ZIP
password was attempted, no payload instructions were run, and signatures were not
validated. The visible package identity is NA 70.00.100A; the installed firmware
and exact CMU hardware remain unverified.

The matching failsafe package identity does not make the upstream CX-5 USB launcher
appropriate for the Mazda6. This evidence is a future comparison reference only.
