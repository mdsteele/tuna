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

use crate::util::Rgba;
use ahi::{Collection, Color, Image, Palette};
use std::rc::Rc;

//===========================================================================//

#[derive(Clone)]
pub(super) struct AhiData {
    pub palette_index: usize,
    pub palettes: Vec<Rc<Palette>>,
    pub image_index: usize,
    pub images: Vec<Rc<Image>>,
    pub mixed_sizes: bool,
}

impl AhiData {
    pub fn new(mut collection: Collection) -> AhiData {
        if collection.images.is_empty() {
            collection.images.push(Image::new(32, 32));
        }
        let mixed_sizes = has_mixed_sizes(&collection.images);
        AhiData {
            palette_index: 0,
            palettes: collection.palettes.drain(..).map(Rc::new).collect(),
            image_index: 0,
            images: collection.images.drain(..).map(Rc::new).collect(),
            mixed_sizes,
        }
    }

    pub fn current_palette(&self) -> &Palette {
        if self.palette_index < self.palettes.len() {
            &self.palettes[self.palette_index]
        } else {
            Palette::default()
        }
    }

    pub fn add_new_palette(&mut self) {
        if self.palette_index < self.palettes.len() {
            self.palette_index += 1;
        } else {
            self.palette_index = 0;
        }
        let rc = Rc::new(self.current_palette().clone());
        self.palettes.insert(self.palette_index, rc);
    }

    pub fn delete_current_palette(&mut self) -> bool {
        if self.palette_index < self.palettes.len() {
            self.palettes.remove(self.palette_index);
            if self.palette_index > 0 {
                self.palette_index -= 1;
            } else {
                self.palette_index = self.palettes.len();
            }
            true
        } else {
            false
        }
    }

    pub fn set_palette_color(&mut self, color: Color, rgba: Rgba) -> bool {
        if self.palette_index < self.palettes.len() {
            let mut palette =
                Palette::clone(&self.palettes[self.palette_index]);
            palette[color] = rgba;
            self.palettes[self.palette_index] = Rc::new(palette);
            true
        } else {
            false
        }
    }

    pub fn add_new_image(&mut self, width: u32, height: u32) {
        self.image_index += 1;
        let rc = Rc::new(Image::new(width, height));
        self.images.insert(self.image_index, rc);
    }

    pub fn delete_current_image(&mut self) -> bool {
        if self.images.len() > 1 {
            let index = self.image_index;
            self.images.remove(index);
            if index == self.images.len() {
                self.image_index -= 1;
            }
            true
        } else {
            false
        }
    }

    pub fn resize_current_image(&mut self, new_width: u32, new_height: u32) {
        let image = &mut self.images[self.image_index];
        *image = Rc::new(image.crop(new_width, new_height));
    }

    pub fn resize_all_images(&mut self, new_width: u32, new_height: u32) {
        self.images = self
            .images
            .iter()
            .map(|image| Rc::new(image.crop(new_width, new_height)))
            .collect();
    }
}

//===========================================================================//

fn has_mixed_sizes<'a, T: IntoIterator<Item = &'a Image>>(images: T) -> bool {
    let mut iter = images.into_iter();
    if let Some(first) = iter.next() {
        !iter.all(|image| {
            image.width() == first.width() && image.height() == first.height()
        })
    } else {
        false
    }
}

//===========================================================================//
