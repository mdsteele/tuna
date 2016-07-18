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

use ahi::Image;
use sdl2::rect::Rect;
use super::canvas::Canvas;
use super::element::{GuiElement, SubrectElement};
use super::event::Event;
use super::state::EditorState;
use super::util;

// ========================================================================= //

pub struct TileView {
    element: SubrectElement<InnerTileView>,
}

impl TileView {
    pub fn new(left: i32, top: i32, width: u32, height: u32) -> TileView {
        TileView {
            element: SubrectElement::new(InnerTileView::new(),
                                         Rect::new(left, top, width, height)),
        }
    }
}

impl GuiElement<EditorState> for TileView {
    fn draw(&self, state: &EditorState, canvas: &mut Canvas) {
        self.element.draw(state.image(), canvas);
    }

    fn handle_event(&mut self, _: &Event, _: &mut EditorState) -> bool {
        false
    }
}

// ========================================================================= //

struct InnerTileView {
}

impl InnerTileView {
    fn new() -> InnerTileView {
        InnerTileView {}
    }
}

impl GuiElement<Image> for InnerTileView {
    fn draw(&self, image: &Image, canvas: &mut Canvas) {
        let rect = canvas.rect();
        let width = rect.width();
        let height = rect.height();
        let mut top = 0;
        while top < height {
            let mut left = 0;
            while left < width {
                util::render_image(canvas, image, left as i32, top as i32, 1);
                left += image.width();
            }
            top += image.height();
        }
    }

    fn handle_event(&mut self, _: &Event, _: &mut Image) -> bool {
        false
    }
}

// ========================================================================= //
