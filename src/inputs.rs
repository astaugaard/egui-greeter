use std::{path::PathBuf, rc::Rc};

use crate::{
    app::DisplayState,
    auth_thread::{self, Handle, InputType},
    search_selector::search_selector,
};
use anyhow::Result;
use egui::{Button, Color32, Frame, RichText, TextEdit, Ui, Vec2};

fn fancy_button(ui: &mut Ui, text: impl Into<String>) -> egui::Response {
    ui.add(
        Button::new(RichText::new(text).color(Color32::from_rgb(36, 39, 58)))
            .frame(false)
            .fill(Color32::from_rgb(183, 189, 248))
            .corner_radius(10.0)
            .min_size(Vec2::new(220.0 + 10.0, 20.0 + 18.0)),
    )
}

pub fn text_edit_frame<F, A>(ui: &mut Ui, f: F) -> egui::InnerResponse<A>
where
    F: FnOnce(&mut Ui) -> A,
{
    Frame::new()
        .inner_margin(10)
        .outer_margin(5)
        .fill(Color32::from_rgb(36, 39, 58))
        .corner_radius(10.0)
        .show(ui, f)
}

pub fn basic_center_input(
    state: &mut DisplayState,
    sessions: &[(Option<PathBuf>, (String, String))],
    handle: &mut Handle,
    ui: &mut Ui,
) -> Result<()> {
    search_selector(
        ui.make_persistent_id("session_selector"),
        &mut state.session_input,
        &mut state.session,
        sessions.iter().map(|(_, (name, command))| {
            (Rc::new(name.to_string()), (name.clone(), command.clone()))
        }),
        &mut state.search_cache,
        200.0,
        ui,
    );

    if let Some(i) = &state.input_type {
        match i {
            InputType::None => {
                if ui.button("next").clicked() {
                    state.input_type = None;
                    handle.send_command(auth_thread::Command::Next)?
                }
            }
            InputType::Password => {
                let mut enter = text_edit_frame(ui, |ui| {
                    ui.add(
                        TextEdit::singleline(&mut state.input)
                            .password(true)
                            .text_color(Color32::from_rgb(198, 160, 246))
                            .desired_width(200.0)
                            .frame(false),
                    )
                    .lost_focus()
                })
                .inner;

                ui.add_space(7.0);

                enter |= fancy_button(ui, "submit").clicked();

                if enter {
                    state.input_type = None;
                    handle.send_command(auth_thread::Command::Entered(std::mem::take(
                        &mut state.input,
                    )))?
                }
            }
            InputType::Visible => {
                let mut enter = text_edit_frame(ui, |ui| {
                    ui.add(
                        TextEdit::singleline(&mut state.input)
                            .text_color(Color32::from_rgb(198, 160, 246))
                            .desired_width(200.0)
                            .frame(false),
                    )
                    .lost_focus()
                })
                .inner;

                ui.add_space(7.0);

                enter |= fancy_button(ui, "submit").clicked();

                if enter {
                    state.input_type = None;
                    handle.send_command(auth_thread::Command::Entered(std::mem::take(
                        &mut state.input,
                    )))?
                }
            }
        }
    }

    Ok(())
}
