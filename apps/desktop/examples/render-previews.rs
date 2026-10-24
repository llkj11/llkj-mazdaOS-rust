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
    clock_previews(directory, &artwork)?;
    system_previews(directory, &artwork)?;
    language_previews(directory, &artwork)?;
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

fn language_previews(directory: &Path, artwork: &StockAssets) -> io::Result<()> {
    for code in ["en_US", "es_MX", "fr_CN"] {
        let mock = MockMazda::demo();
        let mut ui = UiModel::from_source(&mock);
        ui.set_assets(artwork.clone());
        assert!(ui.set_host_language(code));
        save(&ui, directory, &format!("locale-{code}-home"))?;
        locale_domain_previews(directory, &ui, code)?;
        for _ in 0..2 {
            ui.handle_commander(CommanderEvent::RotateClockwise);
        }
        ui.handle_commander(CommanderEvent::Select);
        ui.handle_commander(CommanderEvent::Select);
        save(&ui, directory, &format!("locale-{code}-display"))?;
        ui.handle_commander(CommanderEvent::Back);
        for _ in 0..2 {
            ui.handle_commander(CommanderEvent::RotateClockwise);
        }
        ui.handle_commander(CommanderEvent::Select);
        save(&ui, directory, &format!("locale-{code}-sound"))?;
        ui.handle_commander(CommanderEvent::Back);
        ui.handle_commander(CommanderEvent::RotateClockwise);
        ui.handle_commander(CommanderEvent::Select);
        save(&ui, directory, &format!("locale-{code}-clock"))?;
        ui.handle_commander(CommanderEvent::Back);
        for _ in 0..3 {
            ui.handle_commander(CommanderEvent::RotateClockwise);
        }
        ui.handle_commander(CommanderEvent::Select);
        save(&ui, directory, &format!("locale-{code}-system"))?;
        for _ in 0..2 {
            ui.handle_commander(CommanderEvent::RotateClockwise);
        }
        ui.handle_commander(CommanderEvent::Select);
        save(&ui, directory, &format!("locale-{code}-languages"))?;
    }
    language_flow_previews(directory, artwork)
}
fn locale_domain_previews(directory: &Path, source: &UiModel, code: &str) -> io::Result<()> {
    let mut ui = source.clone();
    for _ in 0..2 {
        ui.handle_commander(CommanderEvent::RotateCounterClockwise);
    }
    ui.handle_commander(CommanderEvent::Select);
    save(&ui, directory, &format!("locale-{code}-applications"))?;
    ui.handle_commander(CommanderEvent::Home);
    for _ in 0..2 {
        ui.handle_commander(CommanderEvent::RotateClockwise);
    }
    ui.handle_commander(CommanderEvent::Select);
    save(&ui, directory, &format!("locale-{code}-communication"))?;
    ui.handle_commander(CommanderEvent::Music);
    save(&ui, directory, &format!("locale-{code}-entertainment"))?;
    ui.handle_commander(CommanderEvent::Navigation);
    save(&ui, directory, &format!("locale-{code}-compass"))?;
    Ok(())
}

fn language_flow_previews(directory: &Path, artwork: &StockAssets) -> io::Result<()> {
    let mock = MockMazda::demo();
    let mut ui = UiModel::from_source(&mock);
    ui.set_assets(artwork.clone());
    for _ in 0..2 {
        ui.handle_commander(CommanderEvent::RotateClockwise);
    }
    ui.handle_commander(CommanderEvent::Select);
    for _ in 0..6 {
        ui.handle_commander(CommanderEvent::RotateClockwise);
    }
    ui.handle_commander(CommanderEvent::Select);
    for _ in 0..2 {
        ui.handle_commander(CommanderEvent::RotateClockwise);
    }
    ui.handle_commander(CommanderEvent::Select);
    ui.handle_commander(CommanderEvent::RotateClockwise);
    ui.handle_commander(CommanderEvent::Select);
    save(&ui, directory, "language-confirm-no")?;
    ui.handle_commander(CommanderEvent::RotateClockwise);
    save(&ui, directory, "language-confirm-yes")?;
    ui.handle_commander(CommanderEvent::Select);
    save(&ui, directory, "language-progress")?;
    ui.advance_simulation(std::time::Duration::from_millis(1500));
    save(&ui, directory, "language-spanish-selected")?;
    ui.handle_commander(CommanderEvent::RotateClockwise);
    ui.handle_commander(CommanderEvent::Select);
    ui.handle_commander(CommanderEvent::RotateClockwise);
    ui.handle_commander(CommanderEvent::Select);
    ui.set_language_change_failure(true);
    ui.advance_simulation(std::time::Duration::from_millis(1500));
    save(&ui, directory, "language-error-no")?;
    ui.handle_commander(CommanderEvent::RotateClockwise);
    save(&ui, directory, "language-error-retry")?;
    ui.handle_commander(CommanderEvent::Select);
    ui.set_language_change_failure(false);
    ui.advance_simulation(std::time::Duration::from_millis(1500));
    save(&ui, directory, "language-french-selected")?;
    ui.handle_commander(CommanderEvent::Back);
    for _ in 0..3 {
        ui.handle_commander(CommanderEvent::RotateClockwise);
    }
    ui.handle_commander(CommanderEvent::Select);
    save(&ui, directory, "locale-fr_CN-about")?;
    ui.handle_commander(CommanderEvent::Select);
    save(&ui, directory, "locale-fr_CN-disclosure")?;
    ui.handle_commander(CommanderEvent::Back);
    ui.handle_commander(CommanderEvent::RotateClockwise);
    ui.handle_commander(CommanderEvent::Select);
    save(&ui, directory, "locale-fr_CN-version")?;
    Ok(())
}

fn system_previews(directory: &Path, artwork: &StockAssets) -> io::Result<()> {
    let mut mock = MockMazda::demo();
    let mut ui = UiModel::from_source(&mock);
    ui.set_assets(artwork.clone());
    for _ in 0..2 {
        ui.handle_commander(CommanderEvent::RotateClockwise);
    }
    ui.handle_commander(CommanderEvent::Select);
    for _ in 0..6 {
        ui.handle_commander(CommanderEvent::RotateClockwise);
    }
    ui.handle_commander(CommanderEvent::Select);
    save(&ui, directory, "system-root")?;
    ui.handle_commander(CommanderEvent::Select);
    save(&ui, directory, "system-tooltips")?;
    ui.set_system_equipment(mazda_ui::SystemEquipment {
        temperature_units: true,
        distance_units: true,
        ..Default::default()
    });
    for _ in 0..3 {
        ui.handle_commander(CommanderEvent::RotateClockwise);
    }
    ui.handle_commander(CommanderEvent::Select);
    save(&ui, directory, "system-temperature")?;
    ui.handle_commander(CommanderEvent::RotateClockwise);
    ui.handle_commander(CommanderEvent::Select);
    save(&ui, directory, "system-distance")?;
    for _ in 0..3 {
        ui.handle_commander(CommanderEvent::RotateClockwise);
    }
    ui.handle_commander(CommanderEvent::Select);
    save(&ui, directory, "system-about")?;
    ui.handle_commander(CommanderEvent::Select);
    save(&ui, directory, "system-legal-start")?;
    for _ in 0..300 {
        ui.handle_commander(CommanderEvent::RotateClockwise);
    }
    save(&ui, directory, "system-legal-end")?;
    ui.handle_commander(CommanderEvent::Back);
    ui.handle_commander(CommanderEvent::RotateClockwise);
    ui.handle_commander(CommanderEvent::Select);
    save(&ui, directory, "system-version")?;
    ui.handle_commander(CommanderEvent::Select);
    ui.handle_commander(CommanderEvent::Back);
    ui.handle_commander(CommanderEvent::RotateCounterClockwise);
    ui.handle_commander(CommanderEvent::Select);
    save(&ui, directory, "system-reset-no")?;
    ui.handle_commander(CommanderEvent::RotateClockwise);
    save(&ui, directory, "system-reset-yes")?;
    ui.handle_commander(CommanderEvent::Select);
    save(&ui, directory, "system-reset-progress")?;
    ui.advance_simulation(std::time::Duration::from_millis(1500));
    save(&ui, directory, "system-reset-complete")?;
    ui.set_system_equipment(mazda_ui::SystemEquipment {
        temperature_units: true,
        distance_units: true,
        services_ready: false,
        new_cmu: true,
    });
    save(&ui, directory, "system-services-unavailable")?;
    mock.set_scenario(DemoScenario::Driving);
    ui.refresh(&mock);
    save(&ui, directory, "system-driving")?;
    Ok(())
}

fn clock_previews(directory: &Path, artwork: &StockAssets) -> io::Result<()> {
    let mut mock = MockMazda::demo();
    let mut ui = UiModel::from_source(&mock);
    ui.set_assets(artwork.clone());
    for _ in 0..2 {
        ui.handle_commander(CommanderEvent::RotateClockwise);
    }
    ui.handle_commander(CommanderEvent::Select);
    for _ in 0..3 {
        ui.handle_commander(CommanderEvent::RotateClockwise);
    }
    ui.handle_commander(CommanderEvent::Select);
    save(&ui, directory, "clock-root")?;
    ui.handle_commander(CommanderEvent::Select);
    save(&ui, directory, "clock-adjust-12h")?;
    for _ in 0..2 {
        ui.handle_commander(CommanderEvent::RotateClockwise);
    }
    ui.handle_commander(CommanderEvent::Select);
    save(&ui, directory, "clock-hour-plus")?;
    ui.handle_commander(CommanderEvent::Back);
    for _ in 0..2 {
        ui.handle_commander(CommanderEvent::RotateClockwise);
    }
    ui.handle_commander(CommanderEvent::Select);
    save(&ui, directory, "clock-format-24h")?;
    for _ in 0..2 {
        ui.handle_commander(CommanderEvent::RotateCounterClockwise);
    }
    ui.handle_commander(CommanderEvent::Select);
    save(&ui, directory, "clock-adjust-24h")?;
    ui.handle_commander(CommanderEvent::Back);
    for _ in 0..3 {
        ui.handle_commander(CommanderEvent::RotateClockwise);
    }
    ui.handle_commander(CommanderEvent::Select);
    save(&ui, directory, "clock-time-zones")?;
    for _ in 0..128 {
        ui.handle_commander(CommanderEvent::RotateClockwise);
    }
    ui.handle_commander(CommanderEvent::Select);
    save(&ui, directory, "clock-last-time-zone")?;
    ui.handle_commander(CommanderEvent::Back);
    ui.handle_commander(CommanderEvent::RotateClockwise);
    ui.handle_commander(CommanderEvent::Select);
    save(&ui, directory, "clock-dst")?;
    ui.set_clock_gps_sync(true);
    ui.set_clock_navigation_available(true);
    save(&ui, directory, "clock-gps-navigation")?;
    mock.set_scenario(DemoScenario::Driving);
    ui.refresh(&mock);
    save(&ui, directory, "clock-driving")?;
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
