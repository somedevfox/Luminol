// Copyright (C) 2023 Lily Lyons
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
use num_traits::{FromPrimitive, ToPrimitive};
use std::fmt::Display;
use strum::IntoEnumIterator;

/// Command editor for events
pub mod command_view;
/// Command view modals
pub mod command_view_modals;
/// Move route display
pub mod move_display;
/// Syntax highlighter
pub mod syntax_highlighting;
/// Toasts to be displayed for errors, information, etc.
pub mod toasts;
/// The toolbar for managing the project.
pub mod top_bar;

/// The tilemap.
pub mod tilemap {
    use crate::{data::rmxp_structs::rpg, UpdateInfo};

    cfg_if::cfg_if! {
           if #[cfg(feature = "generic-tilemap")] {
                  mod generic_tilemap;
                  pub use generic_tilemap::Tilemap;
           } else {
                  mod hardware_tilemap;
                  pub use hardware_tilemap::Tilemap;
           }
    }

    /// A trait defining how a tilemap should function.
    pub trait TilemapDef {
        /// Create a new tilemap.
        fn new(info: &'static UpdateInfo, id: i32) -> Self;

        /// Display the tilemap.
        fn ui(
            &mut self,
            ui: &mut egui::Ui,
            map: &rpg::Map,
            cursor_pos: &mut egui::Pos2,
            toggled_layers: &[bool],
            selected_layer: usize,
            dragging_event: bool,
        ) -> egui::Response;

        /// Display the tile picker.
        fn tilepicker(&self, ui: &mut egui::Ui, selected_tile: &mut i16);

        /// Check if the textures are loaded yet.
        fn textures_loaded(&self) -> bool;

        /// Return the result of loading the tilemap.
        fn load_result(&self) -> Result<(), String>;
    }
}

// btw there's a buncha places this could be used
// uhh in event edit there's an array of strings that gets itered over to do what this does lol
// TODO: Replace dropbox mechanism in event edit with this method

pub struct EnumMenuButton<T, F>
where
    T: Display + FromPrimitive + IntoEnumIterator,
    F: FnMut(T),
{
    current_value: i32,
    _enumeration: T,
    on_select: F,
}
impl<T: Display + FromPrimitive + IntoEnumIterator, F: FnMut(T)> EnumMenuButton<T, F> {
    pub fn new(current_value: i32, enumeration: T, on_select: F) -> Self {
        Self {
            current_value,
            _enumeration: enumeration,
            on_select,
        }
    }
}
impl<T: Display + FromPrimitive + IntoEnumIterator, F: FnMut(T)> egui::Widget
    for EnumMenuButton<T, F>
{
    fn ui(mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.menu_button(T::from_i32(self.current_value).unwrap().to_string(), |ui| {
            for enumeration_item in T::iter() {
                if ui.button(enumeration_item.to_string()).clicked() {
                    (self.on_select)(enumeration_item);
                    ui.close_menu();
                }
            }
        })
        .response
    }
}

pub struct Field<F>
where
    F: FnOnce(&mut egui::Ui),
{
    name: String,
    callback: F,
}
impl<F: FnOnce(&mut egui::Ui)> Field<F> {
    pub fn new(name: impl Into<String>, callback: F) -> Self {
        Self {
            name: name.into(),
            callback,
        }
    }
}
impl<F: FnOnce(&mut egui::Ui)> egui::Widget for Field<F> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.vertical(|ui| {
            ui.label(format!("{}:", self.name));
            (self.callback)(ui);
        })
        .response
    }
}
