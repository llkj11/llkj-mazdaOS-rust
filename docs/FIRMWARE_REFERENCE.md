# Owner-provided firmware package reference

Read-only inventory on 2026-10-24 of the owner's Downloads folder. Original files
were not modified, mounted, installed or executed. The later offline UI
extraction described below reads/decrypts selected archive content as data. Firmware bytes are
not included in this repository or uploaded to GitHub.

| Package | Bytes | SHA-256 |
| --- | ---: | --- |
| `cmu150_NA_70.00.100A_failsafe.up` | 7,088,769 | `ef9964509d81d47c1fcae35748096fe7021a9ea15b670acf5675a48679ddbccf` |
| `cmu150_NA_70.00.100A_reinstall.up` | 925,592,239 | `5938019df7e333447c7bf60aa8f6a7e7f9cd50d6eeb7b6e33605800b85bd4ad0` |

The local failsafe hash matches an independently downloaded copy from
[Mazda's official service CDN](https://s3.amazonaws.com/tsd.mazdausa.com/MAZDA_CONNECT/cmu150_NA_70.00.100A_failsafe.up).
The larger reinstall file was hashed and its ZIP central directory inventoried,
but it was not independently compared with a fresh vendor download.

Both files are ZIP containers with encrypted payload entries. Initial work only
inventoried them. The subsequent, explicitly requested offline UI extraction
opened the reinstall archive using the publicly documented archive password and
verified its pinned whole-file SHA-256 before reading. Its rootfs payload contains
a gzip wrapper around a gzip tar; 4,499 GUI files and 58 font files were copied as
non-executable regular files to an ignored local directory. No update instruction,
firmware script, native executable or extracted JavaScript was run.

See [the stock UI port evidence and boundaries](STOCK_UI_PORT.md). The original
archives in Downloads are unchanged. Package signature validation, installed car
firmware and exact CMU hardware remain unverified. No firmware bytes or proprietary
UI assets are committed or uploaded to GitHub.

The matching failsafe package identity does not make the upstream CX-5 USB launcher
appropriate for the Mazda6. This evidence is a future comparison reference only.
