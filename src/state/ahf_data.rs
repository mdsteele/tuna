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

use ahi::{Font, Glyph, Image};

//===========================================================================//

#[derive(Clone)]
pub(super) struct AhfData {
    pub current_char: Option<char>,
    pub font: Font,
}

impl AhfData {
    pub fn image_index(&self) -> usize {
        if let Some(current) = self.current_char {
            for (index, chr) in self.font.chars().enumerate() {
                if chr == current {
                    return index + 1;
                }
            }
        }
        0
    }

    pub fn set_image_index(&mut self, index: usize) {
        if index == 0 {
            self.current_char = None;
        } else {
            let chr = self.font.chars().nth(index - 1).unwrap();
            self.current_char = Some(chr);
        }
    }

    /// If no glyph exists for the given character, adds one, switches to it,
    /// and returns true.  If a glyph for that character already exists,
    /// switches to it and returns false.
    pub fn add_new_glyph(
        &mut self,
        chr: char,
        width: u32,
        height: u32,
    ) -> bool {
        self.current_char = Some(chr);
        if self.font.get_char_glyph(chr).is_none() {
            let glyph =
                Glyph::new(Image::new(width, height), 0, 1 + width as i32);
            self.font.set_char_glyph(chr, glyph);
            true
        } else {
            false
        }
    }

    /// Deletes the current glyph and returns true, or returns false if the
    /// current image couldn't be deleted (because it is the default glyph).
    pub fn delete_current_glyph(&mut self) -> bool {
        if let Some(chr) = self.current_char {
            self.font.remove_char_glyph(chr);
            self.current_char = None;
            true
        } else {
            false
        }
    }

    pub fn resize_current_glyph(&mut self, new_width: u32, new_height: u32) {
        if new_height != self.font.glyph_height() {
            let mut font = Font::with_glyph_height(new_height);
            {
                let glyph = self.font.default_glyph();
                let new_glyph = Glyph::new(
                    glyph.image().crop(glyph.image().width(), new_height),
                    glyph.left_edge(),
                    glyph.right_edge(),
                );
                font.set_default_glyph(new_glyph);
            }
            for chr in self.font.chars() {
                let glyph = &self.font[chr];
                let new_glyph = Glyph::new(
                    glyph.image().crop(glyph.image().width(), new_height),
                    glyph.left_edge(),
                    glyph.right_edge(),
                );
                font.set_char_glyph(chr, new_glyph);
            }
            self.font = font;
        }
        match self.current_char {
            Some(chr) => {
                let glyph = &self.font[chr];
                let new_glyph = Glyph::new(
                    glyph.image().crop(new_width, glyph.image().height()),
                    glyph.left_edge(),
                    glyph.right_edge(),
                );
                self.font.set_char_glyph(chr, new_glyph);
            }
            None => {
                let glyph = self.font.default_glyph();
                let new_glyph = Glyph::new(
                    glyph.image().crop(new_width, glyph.image().height()),
                    glyph.left_edge(),
                    glyph.right_edge(),
                );
                self.font.set_default_glyph(new_glyph);
            }
        }
    }
}

//===========================================================================//
