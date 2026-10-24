use super::{save, CommanderEvent, MockMazda, StockAssets, UiModel};
use mazda_ui::BluetoothFixture;
use std::{io, path::Path, time::Duration};
fn rotate(ui: &mut UiModel, count: usize) {
    for _ in 0..count {
        ui.handle_commander(CommanderEvent::RotateClockwise);
    }
}
fn select(ui: &mut UiModel) {
    ui.handle_commander(CommanderEvent::Select);
}
fn advance(ui: &mut UiModel, ms: u64) {
    ui.advance_simulation(Duration::from_millis(ms));
}
fn fixture(assets: &StockAssets, code: &str, scenario: BluetoothFixture) -> UiModel {
    let mut ui = UiModel::from_source(&MockMazda::demo());
    ui.set_assets(assets.clone());
    assert!(ui.set_host_language(code));
    ui.set_bluetooth_fixture(scenario);
    rotate(&mut ui, 2);
    select(&mut ui);
    rotate(&mut ui, 5);
    select(&mut ui);
    ui
}
pub fn render(directory: &Path, assets: &StockAssets) -> io::Result<()> {
    for code in ["en_US", "es_MX", "fr_CN"] {
        roots(directory, assets, code)?;
        paired(directory, assets, code)?;
        pairing(directory, assets, code)?;
        legacy(directory, assets, code)?;
        failures(directory, assets, code)?;
    }
    Ok(())
}
fn roots(directory: &Path, assets: &StockAssets, code: &str) -> io::Result<()> {
    let mut ui = fixture(assets, code, BluetoothFixture::Empty);
    let capture =
        |ui: &UiModel, name: &str| save(ui, directory, &format!("bluetooth-{code}-{name}"));
    capture(&ui, "devices-root")?;
    select(&mut ui);
    capture(&ui, "manager-empty")?;
    select(&mut ui);
    capture(&ui, "manager-off")?;
    select(&mut ui);
    rotate(&mut ui, 1);
    select(&mut ui);
    capture(&ui, "settings")?;
    select(&mut ui);
    capture(&ui, "information")?;
    select(&mut ui);
    rotate(&mut ui, 1);
    select(&mut ui);
    capture(&ui, "keypad")?;
    ui.handle_commander(CommanderEvent::RotateCounterClockwise);
    select(&mut ui);
    capture(&ui, "keypad-disabled-ok")?;
    ui.handle_commander(CommanderEvent::Home);
    ui = fixture(assets, code, BluetoothFixture::Empty);
    ui.set_projection_fixture(true);
    rotate(&mut ui, 1);
    select(&mut ui);
    rotate(&mut ui, 1);
    select(&mut ui);
    capture(&ui, "projection-enabled")?;
    select(&mut ui);
    capture(&ui, "carplay-disable")?;
    Ok(())
}
fn paired(directory: &Path, assets: &StockAssets, code: &str) -> io::Result<()> {
    let mut ui = fixture(assets, code, BluetoothFixture::Paired);
    select(&mut ui);
    let capture =
        |ui: &UiModel, name: &str| save(ui, directory, &format!("bluetooth-{code}-{name}"));
    capture(&ui, "manager-paired")?;
    rotate(&mut ui, 3);
    select(&mut ui);
    capture(&ui, "phone-profiles")?;
    ui.handle_commander(CommanderEvent::Back);
    rotate(&mut ui, 1);
    select(&mut ui);
    capture(&ui, "audio-only-profiles")?;
    select(&mut ui);
    capture(&ui, "connecting")?;
    advance(&mut ui, 1500);
    capture(&ui, "connected")?;
    advance(&mut ui, 3000);
    capture(&ui, "separate-phone-audio")?;
    ui = fixture(assets, code, BluetoothFixture::Paired);
    select(&mut ui);
    rotate(&mut ui, 3);
    select(&mut ui);
    rotate(&mut ui, 4);
    select(&mut ui);
    capture(&ui, "unpair-no")?;
    rotate(&mut ui, 1);
    capture(&ui, "unpair-yes")?;
    select(&mut ui);
    capture(&ui, "unpairing")?;
    advance(&mut ui, 1500);
    capture(&ui, "unpaired")?;
    ui = fixture(assets, code, BluetoothFixture::Paired);
    select(&mut ui);
    rotate(&mut ui, 1);
    select(&mut ui);
    rotate(&mut ui, 2);
    select(&mut ui);
    capture(&ui, "unpair-list")?;
    rotate(&mut ui, 2);
    select(&mut ui);
    capture(&ui, "unpair-all-confirm")?;
    Ok(())
}
fn pairing(directory: &Path, assets: &StockAssets, code: &str) -> io::Result<()> {
    let mut ui = fixture(assets, code, BluetoothFixture::Ssp);
    select(&mut ui);
    rotate(&mut ui, 2);
    select(&mut ui);
    let capture =
        |ui: &UiModel, name: &str| save(ui, directory, &format!("bluetooth-{code}-{name}"));
    capture(&ui, "discoverable")?;
    advance(&mut ui, 1500);
    capture(&ui, "numeric-confirm")?;
    select(&mut ui);
    capture(&ui, "pairing")?;
    advance(&mut ui, 1500);
    capture(&ui, "pair-success")?;
    advance(&mut ui, 3000);
    advance(&mut ui, 1500);
    capture(&ui, "pair-connected")?;
    ui = fixture(assets, code, BluetoothFixture::Empty);
    select(&mut ui);
    rotate(&mut ui, 2);
    select(&mut ui);
    ui.handle_commander(CommanderEvent::Back);
    capture(&ui, "pair-cancelling")?;
    Ok(())
}
fn failures(directory: &Path, assets: &StockAssets, code: &str) -> io::Result<()> {
    for (scenario, name, row) in [
        (BluetoothFixture::ConnectFails, "connect-failed", 2),
        (BluetoothFixture::DisconnectFails, "disconnect-failed", 3),
        (BluetoothFixture::DeleteFails, "delete-failed", 4),
    ] {
        let mut ui = fixture(assets, code, scenario);
        select(&mut ui);
        rotate(&mut ui, 3);
        select(&mut ui);
        if row == 2 {
            ui.handle_commander(CommanderEvent::Back);
            rotate(&mut ui, 1);
            select(&mut ui);
        } else {
            rotate(&mut ui, row);
        }
        select(&mut ui);
        if row == 4 {
            rotate(&mut ui, 1);
            select(&mut ui);
        }
        advance(&mut ui, 1500);
        save(&ui, directory, &format!("bluetooth-{code}-{name}"))?;
    }
    let mut ui = fixture(assets, code, BluetoothFixture::PairFails);
    select(&mut ui);
    rotate(&mut ui, 2);
    select(&mut ui);
    advance(&mut ui, 1500);
    select(&mut ui);
    advance(&mut ui, 1500);
    save(&ui, directory, &format!("bluetooth-{code}-pair-failed"))?;
    ui = fixture(assets, code, BluetoothFixture::DisconnectFails);
    select(&mut ui);
    rotate(&mut ui, 2);
    select(&mut ui);
    save(&ui, directory, &format!("bluetooth-{code}-preparing-pair"))?;
    advance(&mut ui, 1500);
    save(&ui, directory, &format!("bluetooth-{code}-pre-pair-failed"))?;
    Ok(())
}

fn legacy(directory: &Path, assets: &StockAssets, code: &str) -> io::Result<()> {
    let mut ui = fixture(assets, code, BluetoothFixture::Legacy);
    select(&mut ui);
    rotate(&mut ui, 2);
    select(&mut ui);
    advance(&mut ui, 1500);
    save(&ui, directory, &format!("bluetooth-{code}-legacy-confirm"))?;
    select(&mut ui);
    save(&ui, directory, &format!("bluetooth-{code}-legacy-cancel"))?;
    ui = fixture(assets, code, BluetoothFixture::Legacy);
    select(&mut ui);
    rotate(&mut ui, 2);
    select(&mut ui);
    advance(&mut ui, 1500);
    advance(&mut ui, 30_000);
    save(&ui, directory, &format!("bluetooth-{code}-legacy-timeout"))?;
    Ok(())
}
