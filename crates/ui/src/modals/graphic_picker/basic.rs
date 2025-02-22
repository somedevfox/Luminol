// Copyright (C) 2024 Melody Madeline Lyons
//
// This file is part of Luminol.
//
// Luminol is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Luminol is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Luminol.  If not, see <http://www.gnu.org/licenses/>.
//
//     Additional permission under GNU GPL version 3 section 7
//
// If you modify this Program, or any covered work, by linking or combining
// it with Steamworks API by Valve Corporation, containing parts covered by
// terms of the Steamworks API by Valve Corporation, the licensors of this
// Program grant you additional permission to convey the resulting work.

use crate::components::UiExt;
use color_eyre::eyre::WrapErr;
use luminol_core::prelude::*;

use super::{ButtonSprite, Entry, PreviewSprite, Selected};

pub struct Modal {
    state: State,
    id_source: egui::Id,

    button_size: egui::Vec2,
    directory: camino::Utf8PathBuf, // do we make this &'static Utf8Path?

    button_sprite: Option<ButtonSprite>,

    scrolled_on_first_open: bool,
}

enum State {
    Closed,
    Open {
        entries: Vec<Entry>,
        filtered_entries: Vec<Entry>,
        search_text: String,

        selected: Selected,
    },
}

impl Modal {
    pub fn new(
        update_state: &UpdateState<'_>,
        directory: camino::Utf8PathBuf,
        path: Option<&camino::Utf8Path>,
        button_size: egui::Vec2,
        id_source: impl Into<egui::Id>,
    ) -> Self {
        let button_sprite = path.map(|path| {
            let texture = update_state
                .graphics
                .texture_loader
                .load_now_dir(update_state.filesystem, &directory, path)
                .unwrap(); // FIXME

            let button_viewport = Viewport::new(&update_state.graphics, Default::default());
            let sprite = Sprite::basic(&update_state.graphics, &texture, &button_viewport);
            ButtonSprite {
                sprite,
                sprite_size: texture.size_vec2(),
                viewport: button_viewport,
            }
        });

        Self {
            state: State::Closed,
            id_source: id_source.into(),
            button_size,
            directory,
            button_sprite,
            scrolled_on_first_open: false,
        }
    }
}

impl luminol_core::Modal for Modal {
    type Data<'m> = &'m mut Option<camino::Utf8PathBuf>;

    fn button<'m>(
        &'m mut self,
        data: Self::Data<'m>,
        update_state: &'m mut luminol_core::UpdateState<'_>,
    ) -> impl egui::Widget + 'm {
        |ui: &mut egui::Ui| {
            let desired_size = self.button_size + ui.spacing().button_padding * 2.0;
            let is_open = matches!(self.state, State::Open { .. });
            let mut response = ButtonSprite::ui(
                self.button_sprite.as_mut(),
                ui,
                update_state,
                is_open,
                desired_size,
            );

            if response.clicked() && !is_open {
                let selected = match data.clone() {
                    Some(path) => {
                        // FIXME error handling
                        let sprite =
                            Self::load_preview_sprite(update_state, &self.directory, &path)
                                .unwrap();
                        Selected::Entry { path, sprite }
                    }
                    None => Selected::None,
                };

                let entries = Entry::load(update_state, &self.directory);

                self.state = State::Open {
                    filtered_entries: entries.clone(),
                    entries,
                    search_text: String::new(),
                    selected,
                };
            }
            if self.show_window(update_state, ui.ctx(), data) {
                response.mark_changed();
            }

            response
        }
    }

    fn reset(&mut self, update_state: &mut luminol_core::UpdateState<'_>, data: Self::Data<'_>) {
        self.update_graphic(update_state, data); // we need to update the button sprite to prevent desyncs
        self.state = State::Closed;
        self.scrolled_on_first_open = false;
    }
}

impl Modal {
    fn load_preview_sprite(
        update_state: &luminol_core::UpdateState<'_>,
        directory: &camino::Utf8Path,
        path: &camino::Utf8Path,
    ) -> color_eyre::Result<PreviewSprite> {
        let texture = update_state
            .graphics
            .texture_loader
            .load_now_dir(update_state.filesystem, directory, path)
            .wrap_err("While loading a preview sprite")?;

        Ok(Self::create_preview_sprite_from_texture(
            update_state,
            &texture,
        ))
    }

    fn create_preview_sprite_from_texture(
        update_state: &luminol_core::UpdateState<'_>,
        texture: &Texture,
    ) -> PreviewSprite {
        let viewport = Viewport::new(
            &update_state.graphics,
            glam::vec2(texture.width() as f32, texture.height() as f32),
        );

        let sprite = Sprite::basic(&update_state.graphics, texture, &viewport);
        PreviewSprite {
            sprite,
            sprite_size: texture.size_vec2(),
            viewport,
        }
    }

    fn update_graphic(
        &mut self,
        update_state: &UpdateState<'_>,
        data: &Option<camino::Utf8PathBuf>,
    ) {
        self.button_sprite = data.as_ref().map(|path| {
            let texture = update_state
                .graphics
                .texture_loader
                .load_now_dir(update_state.filesystem, &self.directory, path)
                .unwrap(); // FIXME

            let button_viewport = Viewport::new(&update_state.graphics, Default::default());
            let sprite = Sprite::basic(&update_state.graphics, &texture, &button_viewport);
            ButtonSprite {
                sprite,
                sprite_size: texture.size_vec2(),
                viewport: button_viewport,
            }
        });
    }

    fn show_window(
        &mut self,
        update_state: &mut luminol_core::UpdateState<'_>,
        ctx: &egui::Context,
        data: &mut Option<camino::Utf8PathBuf>,
    ) -> bool {
        let mut win_open = true;
        let mut keep_open = true;
        let mut needs_save = false;

        let State::Open {
            entries,
            filtered_entries,
            search_text,
            selected,
        } = &mut self.state
        else {
            self.scrolled_on_first_open = false;
            return false;
        };

        egui::Window::new("Graphic Picker")
            .resizable(true)
            .open(&mut win_open)
            .id(self.id_source.with("window"))
            .show(ctx, |ui| {
                egui::SidePanel::left(self.id_source.with("sidebar")).show_inside(ui, |ui| {
                    let out = egui::TextEdit::singleline(search_text)
                        .hint_text("Search 🔎")
                        .show(ui);
                    if out.response.changed() {
                        *filtered_entries = Entry::filter(entries, search_text);
                    }

                    ui.separator();

                    // Get row height.
                    let row_height = ui.spacing().interact_size.y.max(
                        ui.text_style_height(&egui::TextStyle::Button)
                            + 2. * ui.spacing().button_padding.y,
                    );
                    ui.with_cross_justify(|ui| {
                        let mut scroll_area_output = egui::ScrollArea::vertical()
                            .auto_shrink([false, true])
                            .show_rows(
                                ui,
                                row_height,
                                filtered_entries.len() + 1,
                                |ui, mut rows| {
                                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);

                                    if rows.contains(&0) {
                                        let checked = matches!(selected, Selected::None);
                                        let res = ui.selectable_label(checked, "(None)");
                                        if res.clicked() && !matches!(selected, Selected::None) {
                                            *selected = Selected::None;
                                        }
                                    }

                                    // subtract 1 to account for (None)
                                    rows.start = rows.start.saturating_sub(1);
                                    rows.end = rows.end.saturating_sub(1);

                                    Entry::ui(
                                        filtered_entries,
                                        &self.directory,
                                        update_state,
                                        ui,
                                        rows,
                                        selected,
                                        |path| {
                                            Self::load_preview_sprite(
                                                update_state,
                                                &self.directory,
                                                path,
                                            )
                                            .unwrap()
                                        },
                                    )
                                },
                            );

                        // Scroll the selected item into view
                        if !self.scrolled_on_first_open {
                            let row = if matches!(selected, Selected::None) {
                                Some(0)
                            } else {
                                Entry::find_matching_entry(
                                    filtered_entries,
                                    &self.directory,
                                    update_state,
                                    selected,
                                )
                                .map(|i| i + 1)
                            };
                            if let Some(row) = row {
                                let spacing = ui.spacing().item_spacing.y;
                                let max = row as f32 * (row_height + spacing) + spacing;
                                let min = row as f32 * (row_height + spacing) + row_height
                                    - spacing
                                    - scroll_area_output.inner_rect.height();
                                if scroll_area_output.state.offset.y > max {
                                    scroll_area_output.state.offset.y = max;
                                    scroll_area_output
                                        .state
                                        .store(ui.ctx(), scroll_area_output.id);
                                } else if scroll_area_output.state.offset.y < min {
                                    scroll_area_output.state.offset.y = min;
                                    scroll_area_output
                                        .state
                                        .store(ui.ctx(), scroll_area_output.id);
                                }
                            }
                            self.scrolled_on_first_open = true;
                        }
                    });
                });

                egui::TopBottomPanel::bottom(self.id_source.with("bottom")).show_inside(ui, |ui| {
                    ui.add_space(ui.style().spacing.item_spacing.y);
                    crate::components::close_options_ui(ui, &mut keep_open, &mut needs_save);
                });

                egui::CentralPanel::default().show_inside(ui, |ui| {
                    egui::ScrollArea::both()
                        .auto_shrink([false, false])
                        .show_viewport(ui, |ui, viewport| match selected {
                            Selected::None => {}
                            Selected::Entry { sprite, .. } => {
                                sprite.ui(ui, viewport, update_state);
                            }
                        });
                });
            });

        if needs_save {
            match selected {
                Selected::None => *data = None,
                Selected::Entry { path, .. } => *data = Some(path.clone()),
            }

            self.update_graphic(update_state, data);
        }

        if !(win_open && keep_open) {
            self.state = State::Closed;
            self.scrolled_on_first_open = false;
        }

        needs_save
    }
}
