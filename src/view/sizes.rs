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

use crate::canvas::{Canvas, Resources, ToolIcon};
use crate::element::{Action, GuiElement};
use crate::event::Event;
use crate::state::EditorState;
use sdl2::rect::{Point, Rect};

//===========================================================================//

pub(super) struct SizesToggle {
    rect: Rect,
}

impl SizesToggle {
    const WIDTH: u32 = 18;
    const HEIGHT: u32 = 18;

    pub fn new(left: i32, top: i32) -> SizesToggle {
        SizesToggle { rect: Rect::new(left, top, Self::WIDTH, Self::HEIGHT) }
    }
}

impl GuiElement<EditorState, ()> for SizesToggle {
    fn draw(
        &self,
        state: &EditorState,
        resources: &Resources,
        canvas: &mut Canvas,
    ) {
        let icon = if state.allows_mixed_sizes() {
            ToolIcon::MixedSizes
        } else {
            ToolIcon::EqualSizes
        };
        let mut subcanvas = canvas.subcanvas(self.rect);
        subcanvas.clear((95, 95, 95, 255));
        subcanvas.draw_sprite(resources.tool_icon(icon), Point::new(1, 1));
    }

    fn on_event(
        &mut self,
        event: &Event,
        state: &mut EditorState,
    ) -> Action<()> {
        match *event {
            Event::MouseDown(pt) if self.rect.contains_point(pt) => {
                state.set_allows_mixed_sizes(!state.allows_mixed_sizes());
                Action::redraw().and_stop()
            }
            _ => Action::ignore(),
        }
    }
}

//===========================================================================//
