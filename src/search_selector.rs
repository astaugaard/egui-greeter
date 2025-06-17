use std::{hash::Hash, rc::Rc};

use egui::{Button, Color32, RichText, TextEdit, Ui, popup_below_widget};

use crate::inputs::text_edit_frame;

#[derive(Debug, Default, Clone)]
pub struct SingleCache<K, V> {
    last_key: Option<K>,
    value: V,
    read: bool,
}

impl<K, V> SingleCache<K, V> {
    pub fn get<F>(&mut self, key: &K, f: F) -> &V
    where
        F: FnOnce() -> V,
        K: PartialEq + Clone,
    {
        self.read = true;
        if self.last_key.as_ref() != Some(key) {
            self.last_key = Some(key.clone());
            self.value = f();
        }

        &self.value
    }

    pub fn update(&mut self) {
        if !self.read {
            self.last_key = None;
        }
        self.read = false;
    }
}

fn selector_button(ui: &mut Ui, text: &str) -> egui::Response {
    ui.add(
        Button::new(RichText::new(text).color(Color32::from_rgb(245, 189, 230)))
            .frame(false)
            .fill(Color32::from_rgb(36, 39, 58)),
    )
}

pub type SelectorCache<A> = SingleCache<String, Vec<(Rc<String>, A)>>;

pub(crate) fn search_selector<A, I: Hash>(
    id: I,
    text: &mut String,
    selection: &mut A,
    options: impl Iterator<Item = (Rc<String>, A)>,
    cache: &mut SelectorCache<A>,
    width: f32,
    ui: &mut Ui,
) -> bool
where
    A: Clone,
{
    let edit = text_edit_frame(ui, |ui| {
        ui.add(
            TextEdit::singleline(text)
                .desired_width(width)
                .frame(false)
                .text_color(Color32::from_rgb(245, 189, 230)),
        )
    })
    .inner;

    let mut changed = false;

    let id = ui.make_persistent_id(id);

    if edit.gained_focus() {
        ui.memory_mut(|mem| mem.open_popup(id));
    }

    popup_below_widget(
        ui,
        id,
        &edit,
        egui::PopupCloseBehavior::CloseOnClickOutside,
        |ui| {
            text_edit_frame(ui, |ui| {
                let vals = cache.get(text, || {
                    options
                        .filter(|(name, _value)| name.contains(text.as_str()))
                        .take(10)
                        .collect::<Vec<_>>()
                });

                if vals.len() == 1 {
                    *selection = vals[0].1.clone();
                    changed = true;
                }

                for (name, value) in vals {
                    if selector_button(ui, name.as_str()).clicked() {
                        changed = true;
                        *selection = value.clone();
                        ui.memory_mut(|mem| mem.close_popup());
                        text.clear();

                        text.push_str(name.as_str());
                    }
                }
            });
        },
    );

    cache.update();

    changed
}
