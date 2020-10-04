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

use crate::canvas::{Canvas, Resources};
use crate::element::{Action, GuiElement};
use crate::event::Event;
use crate::state::EditorState;
use sdl2::rect::Rect;

//===========================================================================//

pub struct TileView {
    rect: Rect,
}

impl TileView {
    pub fn new(left: i32, top: i32, width: u32, height: u32) -> TileView {
        TileView { rect: Rect::new(left, top, width, height) }
    }
}

impl GuiElement<EditorState> for TileView {
    fn draw(
        &self,
        state: &EditorState,
        _resources: &Resources,
        canvas: &mut Canvas,
    ) {
        let mut canvas = canvas.subcanvas(self.rect);
        let (width, height) = self.rect.size();
        if let Some(font) = state.font() {
            let mut top: i32 = 0;
            let mut left: i32 = 0;
            for chr in state.test_sentence().chars() {
                let glyph = &font[chr];
                left -= glyph.left_edge();
                if left + (glyph.image().width() as i32) > (width as i32)
                    && left > 0
                {
                    top += font.glyph_height() as i32 + 1;
                    left = -glyph.left_edge();
                }
                canvas.draw_image(glyph.image(), left, top, 1);
                left += glyph.right_edge();
            }
        } else {
            let image = state.image();
            let mut top = 0;
            while top < height as i32 {
                let mut left = 0;
                while left < width as i32 {
                    canvas.draw_image(image, left, top, 1);
                    left += image.width() as i32;
                }
                top += image.height() as i32;
            }
        }
    }

    fn handle_event(&mut self, _: &Event, _: &mut EditorState) -> Action {
        Action::ignore().and_continue()
    }
}

//===========================================================================//
