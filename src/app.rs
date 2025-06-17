use std::path::PathBuf;

use eframe::CreationContext;
use egui::{
    Align, Align2, Color32, Direction, Frame, Stroke, ViewportCommand,
    epaint::text::{FontInsert, InsertFontFamily},
};
use egui_toast::{Toast, ToastKind, ToastOptions, Toasts};

use crate::{
    auth_thread::{self, Handle, InputType},
    inputs::basic_center_input,
    power_area::power_area,
    search_selector::{SelectorCache, SingleCache},
    sessions::get_sessions,
    settings::Settings,
    time_area::time_area,
};

pub struct DisplayState {
    pub session_input: String,
    pub search_cache: SelectorCache<(String, String)>,
    pub input: String,
    pub input_type: Option<InputType>,
    pub session: (String, String),
}

impl DisplayState {
    pub fn new(default: String, default_command: String) -> Self {
        Self {
            input: String::new(),
            input_type: None,
            session: (default.clone(), default_command),
            session_input: default,
            search_cache: SingleCache::default(),
        }
    }
}

pub struct DisplayManager<'a> {
    pub handle: &'a mut Handle,
    pub sessions: Vec<(Option<PathBuf>, (String, String))>,
    pub state: DisplayState,
}

impl<'a> DisplayManager<'a> {
    pub fn new(settings: Settings, handle: &'a mut Handle, cc: &CreationContext) -> Self {
        let sessions = get_sessions(
            settings.default_session_name.clone(),
            settings.default_session_command.clone(),
        );

        let ctx = &cc.egui_ctx;

        catppuccin_egui::set_theme(ctx, catppuccin_egui::MACCHIATO);

        ctx.add_font(FontInsert::new(
            "FiraCode",
            egui::FontData::from_static(include_bytes!(
                "../nerd-fonts/patched-fonts/FiraCode/Regular/FiraCodeNerdFontMono-Regular.ttf"
            )),
            vec![
                InsertFontFamily {
                    family: egui::FontFamily::Monospace,
                    priority: egui::epaint::text::FontPriority::Highest,
                },
                InsertFontFamily {
                    family: egui::FontFamily::Proportional,
                    priority: egui::epaint::text::FontPriority::Lowest,
                },
            ],
        ));

        ctx.style_mut(|style| {
            style.visuals.widgets.hovered.bg_stroke = Stroke::new(0.0, Color32::from_rgb(0, 0, 0));
            style.visuals.widgets.active.bg_stroke = Stroke::new(0.0, Color32::from_rgb(0, 0, 0));
            style.visuals.widgets.inactive.bg_stroke = Stroke::new(0.0, Color32::from_rgb(0, 0, 0));
            style.visuals.widgets.open.bg_stroke = Stroke::new(0.0, Color32::from_rgb(0, 0, 0));
            style.visuals.widgets.noninteractive.bg_stroke =
                Stroke::new(0.0, Color32::from_rgb(0, 0, 0));
            style.visuals.window_stroke = Stroke::new(0.0, Color32::from_rgb(0, 0, 0));

            style.override_font_id = Some(egui::FontId {
                size: 18.0,
                family: egui::FontFamily::Monospace,
            });
        });

        Self {
            handle,
            sessions,
            state: DisplayState::new(
                settings.default_session_name,
                settings.default_session_command,
            ),
        }
    }
}

impl eframe::App for DisplayManager<'_> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut toasts = Toasts::new()
            .anchor(Align2::CENTER_TOP, (0.0, 10.0))
            .direction(Direction::TopDown);

        while let Some(mes) = self.handle.get_response() {
            match mes {
                auth_thread::Responce::Success => ctx.send_viewport_cmd(ViewportCommand::Close),
                auth_thread::Responce::Error(err) => {
                    toasts.add(Toast {
                        kind: ToastKind::Error,
                        text: err.into(),
                        options: ToastOptions::default()
                            .duration_in_seconds(5.0)
                            .show_progress(true),
                        ..Default::default()
                    });
                }
                auth_thread::Responce::Message(mes) => {
                    toasts.add(Toast {
                        kind: ToastKind::Info,
                        text: mes.into(),
                        options: ToastOptions::default()
                            .duration_in_seconds(5.0)
                            .show_progress(true),
                        ..Default::default()
                    });
                }
                auth_thread::Responce::GetInput(t) => self.state.input_type = Some(t),
                auth_thread::Responce::GetSession => match self
                    .handle
                    .send_command(auth_thread::Command::Session(self.state.session.1.clone()))
                {
                    Ok(()) => {}
                    Err(err) => {
                        toasts.add(Toast {
                            kind: ToastKind::Error,
                            text: err.to_string().into(),
                            options: ToastOptions::default()
                                .duration_in_seconds(5.0)
                                .show_progress(true),
                            ..Default::default()
                        });
                    }
                },
            }
        }

        egui::CentralPanel::default().show(ctx, |_| {});

        egui::Area::new(egui::Id::new("center input"))
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                Frame::new()
                    .inner_margin(20)
                    .fill(Color32::from_rgb(54, 58, 79))
                    .corner_radius(20.0)
                    .show(ui, |ui| {
                        ui.with_layout(egui::Layout::top_down(Align::Center), |ui| {
                            match basic_center_input(
                                &mut self.state,
                                &self.sessions,
                                self.handle,
                                ui,
                            ) {
                                Ok(()) => {}
                                Err(_) => {
                                    panic!("invalid state occurred")
                                }
                            }
                        });
                    });
            });

        egui::Area::new(egui::Id::new("power"))
            .anchor(Align2::RIGHT_BOTTOM, [-20.0, 10.0])
            .show(ctx, |ui| match power_area(ui) {
                Ok(()) => {}
                Err(err) => {
                    toasts.add(Toast {
                        kind: ToastKind::Error,
                        text: err.to_string().into(),
                        options: ToastOptions::default()
                            .duration_in_seconds(5.0)
                            .show_progress(true),
                        ..Default::default()
                    });
                }
            });

        egui::Area::new(egui::Id::new("time"))
            .anchor(Align2::LEFT_BOTTOM, [20.0, 10.0])
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::top_down(Align::Center), time_area)
            });

        toasts.show(ctx);

        ctx.request_repaint_after_secs(1.0);
    }
}
