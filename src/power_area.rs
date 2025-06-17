use std::process::Command;

use anyhow::Result;
use egui::{Color32, Label, Response, RichText, Sense, Ui};

fn power_button(name: &'static str, ui: &mut Ui) -> Response {
    ui.add(
        Label::new(
            RichText::new(name)
                .size(64.0)
                .color(Color32::from_rgb(139, 213, 202)),
        )
        .sense(Sense::click()),
    )
}

pub fn power_area(ui: &mut Ui) -> Result<()> {
    ui.horizontal(|ui| -> Result<()> {
        if power_button("", ui).clicked() {
            Command::new("systemctl").arg("reboot").spawn()?;
        }

        ui.add_space(20.0);

        if power_button("⏻", ui).clicked() {
            Command::new("systemctl").arg("poweroff").spawn()?;
        }

        Ok(())
    })
    .inner?;

    Ok(())
}
