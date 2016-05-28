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

use ahi::{Color, Image};
use super::canvas::Canvas;
use sdl2::rect::Rect;

// ========================================================================= //

pub fn render_image(canvas: &mut Canvas,
                    image: &Image,
                    left: i32,
                    top: i32,
                    scale: u32) {
    for row in 0..image.height() {
        for col in 0..image.width() {
            let pixel = image[(col, row)];
            if pixel != Color::Transparent {
                canvas.fill_rect(pixel.rgba(),
                                 Rect::new(left + (scale * col) as i32,
                                           top + (scale * row) as i32,
                                           scale,
                                           scale));
            }
        }
    }
}

// ========================================================================= //
