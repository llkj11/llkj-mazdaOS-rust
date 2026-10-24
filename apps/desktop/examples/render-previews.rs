//! Deterministic native renders using only the owner's offline-extracted artwork.
use mazda_core::CommanderEvent;
use mazda_mock::{DemoScenario, MockMazda};
use mazda_ui::{Framebuffer, SoundProfile, StockAssets, UiModel, DISPLAY_HEIGHT, DISPLAY_WIDTH};
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
    sound_previews(directory, &artwork)?;
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
            for _ in 0..2 {
                ui.handle_commander(CommanderEvent::RotateClockwise);
            }
            ui.handle_commander(CommanderEvent::Select);
            save(&ui, directory, "display-day")?;
            ui.handle_commander(CommanderEvent::RotateClockwise);
            ui.handle_commander(CommanderEvent::Select);
            for _ in 0..3 {
                ui.handle_commander(CommanderEvent::RotateClockwise);
            }
            save(&ui, directory, "display-brightness")?;
            ui.handle_commander(CommanderEvent::Select);
            ui.handle_commander(CommanderEvent::RotateClockwise);
            ui.handle_commander(CommanderEvent::Select);
            for _ in 0..2 {
                ui.handle_commander(CommanderEvent::RotateCounterClockwise);
            }
            save(&ui, directory, "display-contrast")?;
            ui.handle_commander(CommanderEvent::Select);
            ui.handle_commander(CommanderEvent::RotateClockwise);
            ui.handle_commander(CommanderEvent::Select);
            save(&ui, directory, "display-reset-no")?;
            ui.handle_commander(CommanderEvent::RotateClockwise);
            save(&ui, directory, "display-reset-yes")?;
            ui.handle_commander(CommanderEvent::Select);
            save(&ui, directory, "display-reset-done")?;
            for _ in 0..4 {
                ui.handle_commander(CommanderEvent::RotateCounterClockwise);
            }
            ui.handle_commander(CommanderEvent::Select);
            save(&ui, directory, "display-off-clock")?;
            ui.handle_commander(CommanderEvent::Select);
            ui.handle_commander(CommanderEvent::RotateCounterClockwise);
            ui.handle_commander(CommanderEvent::Select);
            save(&ui, directory, "display-off")?;
            ui.handle_commander(CommanderEvent::Select);
        }
        ui.handle_commander(CommanderEvent::Home);
        ui.handle_commander(CommanderEvent::RotateClockwise);
    }
    Ok(())
}

fn sound_previews(directory: &Path, artwork: &StockAssets) -> io::Result<()> {
    for (name, profile) in [
        ("standard", SoundProfile::Standard),
        ("bose", SoundProfile::Bose),
        ("bose-j12", SoundProfile::BoseJ12),
    ] {
        let mut mock = MockMazda::demo();
        let mut ui = UiModel::from_source(&mock);
        ui.set_assets(artwork.clone());
        ui.set_sound_profile(profile);
        for _ in 0..2 {
            ui.handle_commander(CommanderEvent::RotateClockwise);
        }
        ui.handle_commander(CommanderEvent::Select);
        for _ in 0..2 {
            ui.handle_commander(CommanderEvent::RotateClockwise);
        }
        ui.handle_commander(CommanderEvent::Select);
        save(&ui, directory, &format!("sound-{name}"))?;
        ui.handle_commander(CommanderEvent::Select);
        for _ in 0..3 {
            ui.handle_commander(CommanderEvent::RotateCounterClockwise);
        }
        save(&ui, directory, &format!("sound-{name}-bass-minus3"))?;
        ui.handle_commander(CommanderEvent::Select);
        for _ in 0..20 {
            ui.handle_commander(CommanderEvent::RotateClockwise);
        }
        ui.handle_commander(CommanderEvent::Select);
        save(&ui, directory, &format!("sound-{name}-beep"))?;
        ui.set_guidance_available(false);
        ui.handle_commander(CommanderEvent::RotateCounterClockwise);
        save(&ui, directory, &format!("sound-{name}-guidance-disabled"))?;
        mock.set_scenario(DemoScenario::Driving);
        ui.refresh(&mock);
        save(&ui, directory, &format!("sound-{name}-driving"))?;
    }
    Ok(())
}
