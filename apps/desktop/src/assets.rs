//! Fixed-name, size-bounded reads of the owner's offline-converted artwork.
use mazda_ui::{Bitmap, StockAssets};
use std::{fs, io, path::Path};
pub fn load(directory: &Path) -> io::Result<StockAssets> {
    let mut assets = StockAssets::default();
    for name in StockAssets::NAMES {
        let path = directory.join(format!("{name}.rgba"));
        if fs::metadata(&path)?.len() > 6_144_016 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "oversized image",
            ));
        }
        let bitmap = Bitmap::decode(&fs::read(path)?).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid converted firmware artwork",
            )
        })?;
        assets.insert(name, bitmap);
    }
    let zones = directory.join("clock-zones.txt");
    if fs::metadata(&zones)?.len() > 132_000 || !assets.load_clock_zones(&fs::read(zones)?) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid original time-zone text",
        ));
    }
    let legal = directory.join("legal.txt");
    if fs::metadata(&legal)?.len() > 132_000 || !assets.load_legal_text(&fs::read(legal)?) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid original disclosure text",
        ));
    }
    Ok(assets)
}
