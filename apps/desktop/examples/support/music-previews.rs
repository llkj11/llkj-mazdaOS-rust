use super::{save, CommanderEvent, MockMazda, StockAssets, UiModel};
use mazda_ui::MusicUpdateFixture;
use std::{io, path::Path, time::Duration};

fn fixture(artwork: &StockAssets, code: &str, scenario: MusicUpdateFixture) -> UiModel {
    let mut ui = UiModel::from_source(&MockMazda::demo());
    ui.set_assets(artwork.clone());
    assert!(ui.set_host_language(code));
    ui.set_music_update_fixture(scenario);
    for _ in 0..2 {
        ui.handle_commander(CommanderEvent::RotateClockwise);
    }
    ui.handle_commander(CommanderEvent::Select);
    for _ in 0..6 {
        ui.handle_commander(CommanderEvent::RotateClockwise);
    }
    ui.handle_commander(CommanderEvent::Select);
    for _ in 0..3 {
        ui.handle_commander(CommanderEvent::RotateClockwise);
    }
    ui.handle_commander(CommanderEvent::Select);
    ui
}
pub fn render(directory: &Path, artwork: &StockAssets) -> io::Result<()> {
    for code in ["en_US", "es_MX", "fr_CN"] {
        success(directory, artwork, code)?;
        failures(directory, artwork, code)?;
    }
    Ok(())
}
fn success(directory: &Path, artwork: &StockAssets, code: &str) -> io::Result<()> {
    let mut ui = fixture(artwork, code, MusicUpdateFixture::Available);
    let capture = |ui: &UiModel, name: &str| save(ui, directory, &format!("music-{code}-{name}"));
    capture(&ui, "search")?;
    ui.handle_commander(CommanderEvent::Select);
    capture(&ui, "searching")?;
    ui.advance_simulation(Duration::from_millis(1200));
    capture(&ui, "packages-current")?;
    ui.handle_commander(CommanderEvent::RotateClockwise);
    capture(&ui, "packages-update")?;
    ui.handle_commander(CommanderEvent::Select);
    capture(&ui, "confirm")?;
    ui.handle_commander(CommanderEvent::Select);
    capture(&ui, "preparing")?;
    ui.advance_simulation(Duration::from_millis(800));
    ui.advance_simulation(Duration::from_millis(2000));
    capture(&ui, "progress")?;
    ui.advance_simulation(Duration::from_millis(2000));
    capture(&ui, "success")?;
    ui.handle_commander(CommanderEvent::Select);
    ui.handle_commander(CommanderEvent::Select);
    ui.handle_commander(CommanderEvent::Select);
    ui.advance_simulation(Duration::from_millis(1200));
    capture(&ui, "current-simulated")?;
    Ok(())
}
fn failures(directory: &Path, artwork: &StockAssets, code: &str) -> io::Result<()> {
    for (name, scenario) in [
        ("no-media", MusicUpdateFixture::NoMedia),
        ("search-failed", MusicUpdateFixture::SearchFails),
        ("needs-failsafe", MusicUpdateFixture::NeedsFailsafe),
        ("validation-failed", MusicUpdateFixture::Invalid),
        ("unavailable", MusicUpdateFixture::Unavailable),
        ("install-failed", MusicUpdateFixture::InstallFails),
        ("retry-validation-failed", MusicUpdateFixture::RetryInvalid),
    ] {
        let mut ui = fixture(artwork, code, scenario);
        ui.handle_commander(CommanderEvent::Select);
        ui.advance_simulation(Duration::from_millis(1200));
        if matches!(
            scenario,
            MusicUpdateFixture::Invalid
                | MusicUpdateFixture::Unavailable
                | MusicUpdateFixture::InstallFails
                | MusicUpdateFixture::RetryInvalid
        ) {
            ui.handle_commander(CommanderEvent::RotateClockwise);
            ui.handle_commander(CommanderEvent::Select);
            ui.handle_commander(CommanderEvent::Select);
            ui.advance_simulation(Duration::from_millis(800));
        }
        if matches!(
            scenario,
            MusicUpdateFixture::InstallFails | MusicUpdateFixture::RetryInvalid
        ) {
            ui.advance_simulation(Duration::from_millis(4000));
            save(&ui, directory, &format!("music-{code}-{name}-first"))?;
            ui.handle_commander(CommanderEvent::Select);
            ui.advance_simulation(Duration::from_millis(2000));
            save(&ui, directory, &format!("music-{code}-{name}-retrying"))?;
            ui.advance_simulation(Duration::from_millis(2000));
        }
        save(&ui, directory, &format!("music-{code}-{name}"))?;
    }
    Ok(())
}
