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

impl super::CommandView {
    pub fn command_ui<'i, I>(
        &mut self,
        _ui: &mut egui::Ui,
        _db: &luminol_config::command_db::CommandDB,
        (_index, _command): (usize, &'i mut luminol_data::rpg::EventCommand),
        _iter: &mut std::iter::Peekable<I>,
    ) where
        I: Iterator<Item = (usize, &'i mut luminol_data::rpg::EventCommand)>,
    {
        todo!()
    }
}
