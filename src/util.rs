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

use ahi::{Color, Font, Image};
use sdl2::rect::Rect;
use std::fs::File;
use std::io;
use super::canvas::Canvas;

// ========================================================================= //

pub fn modulo(a: i32, b: i32) -> i32 {
    if b == 0 {
        panic!();
    }
    let remainder = a % b;
    if remainder == 0 {
        0
    } else if (a < 0) ^ (b < 0) {
        remainder + b
    } else {
        remainder
    }
}

// ========================================================================= //

pub fn load_ahf_from_file(path: &String) -> io::Result<Font> {
    let mut file = try!(File::open(path));
    Font::read(&mut file)
}

pub fn load_ahi_from_file(path: &String) -> io::Result<Vec<Image>> {
    let mut file = try!(File::open(path));
    Image::read_all(&mut file)
}

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
