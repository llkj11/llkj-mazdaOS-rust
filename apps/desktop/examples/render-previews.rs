//! Deterministic native renders using only the owner's offline-extracted artwork.
use mazda_core::CommanderEvent;
use mazda_mock::MockMazda;
use mazda_ui::{Framebuffer, UiModel, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use std::{
    fs,
    io::{self, Write},
    path::Path,
};
#[path = "../src/assets.rs"]
mod assets;
fn save(ui: &UiModel, directory: &Path, name: &str) -> io::Result<()> {
    let mut frame = Framebuffer::mazda_connect();
    ui.render(&mut frame);
    let mut file = io::BufWriter::new(fs::File::create(directory.join(format!("{name}.ppm")))?);
    write!(file, "P6\n{DISPLAY_WIDTH} {DISPLAY_HEIGHT}\n255\n")?;
    for pixel in frame.pixels() {
        file.write_all(&pixel.to_be_bytes()[1..])?;
    }
    Ok(())
}
fn main() -> io::Result<()> {
    let arguments: Vec<_> = std::env::args_os().collect();
    let directory = arguments.get(1).map(Path::new).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "pass output directory, optionally asset directory",
        )
    })?;
    if let Some(font_path) = arguments.get(3) {
        let path = Path::new(font_path);
        if fs::metadata(path)?.len() > 32_000_000
            || !mazda_ui::install_reference_font(fs::read(path)?)
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid reference font",
            ));
        }
    }
    let artwork = assets::load(arguments.get(2).map_or_else(
        || Path::new("target/owner-firmware/render-assets"),
        Path::new,
    ))?;
    fs::create_dir_all(directory)?;
    let mock = MockMazda::demo();
    let mut ui = UiModel::from_source(&mock);
    ui.set_assets(artwork);
    save(&ui, directory, "home-communication")?;
    for _ in 0..2 {
        ui.handle_commander(CommanderEvent::RotateCounterClockwise);
    }
    for name in [
        "applications",
        "entertainment",
        "communication",
        "navigation",
        "settings",
    ] {
        save(&ui, directory, &format!("home-{name}"))?;
        ui.handle_commander(CommanderEvent::Select);
        save(&ui, directory, name)?;
        if name == "settings" {
            ui.handle_commander(CommanderEvent::Select);
            save(&ui, directory, "settings-display")?;
        }
        ui.handle_commander(CommanderEvent::Home);
        ui.handle_commander(CommanderEvent::RotateClockwise);
    }
    Ok(())
}
