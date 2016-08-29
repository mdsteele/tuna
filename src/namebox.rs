// +--------------------------------------------------------------------------+
// | Copyright 2016 Matthew D. Steele <mdsteele@alum.mit.edu>                 |
// |                                                                          |
// | This file is part of Tuna.                                               |
// |                                                                          |
// | Tuna is free software: you can redistribute it and/or modify it under    |
// | the terms of the GNU General Public License as published by the Free     |
// | Software Foundation, either version 3 of the License, or (at your        |
// | option) any later version.                                               |
// |                                                                          |
// | Tuna is distributed in the hope that it will be useful, but WITHOUT ANY  |
// | WARRANTY; without even the implied warranty of MERCHANTABILITY or        |
// | FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License    |
// | for details.                                                             |
// |                                                                          |
// | You should have received a copy of the GNU General Public License along  |
// | with Tuna.  If not, see <http://www.gnu.org/licenses/>.                  |
// +--------------------------------------------------------------------------+

use std::rc::Rc;
use super::canvas::{Canvas, Font};
use super::element::{Action, GuiElement};
use super::event::Event;
use super::state::EditorState;
use super::util;

// ========================================================================= //

pub struct ImageNameBox {
    left: i32,
    top: i32,
    font: Rc<Font>,
}

impl ImageNameBox {
    pub fn new(left: i32, top: i32, font: Rc<Font>) -> ImageNameBox {
        ImageNameBox {
            left: left,
            top: top,
            font: font,
        }
    }
}

impl GuiElement<EditorState> for ImageNameBox {
    fn draw(&self, state: &EditorState, canvas: &mut Canvas) {
        let text = state.image_name();
        util::render_string(canvas, &self.font, self.left, self.top, &text);
    }

    fn handle_event(&mut self, _: &Event, _: &mut EditorState) -> Action {
        Action::ignore().and_continue()
    }
}

// ========================================================================= //
