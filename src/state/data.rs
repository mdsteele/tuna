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

use super::ahf_data::AhfData;
use super::ahi_data::AhiData;
use crate::util::Rgba;
use ahi::{Collection, Color, Font, Image, Palette};
use std::fs::File;
use std::io::{self, Write};
use std::rc::Rc;

//===========================================================================//

#[derive(Clone)]
pub(super) enum Data {
    Ahi(AhiData),
    Ahf(AhfData),
}

impl Data {
    pub fn from_collection(collection: Collection) -> Self {
        Self::Ahi(AhiData::new(collection))
    }

    pub fn from_font(font: Font) -> Self {
        Self::Ahf(AhfData::new(font))
    }

    pub fn allows_mixed_sizes(&self) -> bool {
        match self {
            Self::Ahi(ahi) => ahi.mixed_sizes,
            Self::Ahf(_) => true,
        }
    }

    pub fn num_palettes(&self) -> usize {
        match self {
            Self::Ahi(ahi) => ahi.palettes.len(),
            Self::Ahf(_) => 0,
        }
    }

    pub fn palette_index(&self) -> usize {
        match self {
            Self::Ahi(ahi) => ahi.palette_index,
            Self::Ahf(_) => 0,
        }
    }

    pub fn set_palette_index(&mut self, index: usize) {
        match self {
            Self::Ahi(ahi) => {
                ahi.palette_index = index % (1 + ahi.palettes.len());
            }
            Self::Ahf(_) => {}
        }
    }

    pub fn palette(&self) -> &Palette {
        match self {
            Self::Ahi(ahi) => ahi.current_palette(),
            Self::Ahf(_) => Palette::default(),
        }
    }

    pub fn num_images(&self) -> usize {
        match self {
            Self::Ahi(ahi) => ahi.images.len(),
            Self::Ahf(ahf) => 1 + ahf.font.chars().len(),
        }
    }

    pub fn image_index(&self) -> usize {
        match self {
            Self::Ahi(ahi) => ahi.image_index,
            Self::Ahf(ahf) => ahf.image_index(),
        }
    }

    pub fn set_image_index(&mut self, index: usize) {
        match self {
            Self::Ahi(ahi) => {
                debug_assert!(!ahi.images.is_empty());
                ahi.image_index = index % ahi.images.len();
            }
            Self::Ahf(ahf) => ahf.set_image_index(index),
        }
    }

    pub fn image_name(&self) -> String {
        match self {
            Self::Ahi(ahi) => format!("{}", ahi.image_index),
            Self::Ahf(ahf) => {
                if let Some(current) = ahf.current_char {
                    let mut name = "'".to_string();
                    for chr in current.escape_default() {
                        name.push(chr);
                    }
                    name.push('\'');
                    name
                } else {
                    "def".to_string()
                }
            }
        }
    }

    pub fn go_to_image(&mut self, text: &str) -> bool {
        match self {
            Self::Ahi(ahi) => match text.parse::<usize>() {
                Ok(index) if index < ahi.images.len() => {
                    ahi.image_index = index;
                    true
                }
                _ => false,
            },
            Self::Ahf(ahf) => {
                if text == "def" {
                    ahf.current_char = None;
                    true
                } else {
                    let chars: Vec<char> = text.chars().collect();
                    if chars.len() == 1 {
                        ahf.current_char = Some(chars[0]);
                        true
                    } else {
                        false
                    }
                }
            }
        }
    }

    pub fn glyph_metrics(&self) -> Option<(i32, i32, i32)> {
        match self {
            Self::Ahi(_) => None,
            Self::Ahf(ahf) => {
                let glyph = match ahf.current_char {
                    Some(chr) => &ahf.font[chr],
                    None => ahf.font.default_glyph(),
                };
                let tuple = (
                    ahf.font.baseline(),
                    glyph.left_edge(),
                    glyph.right_edge(),
                );
                Some(tuple)
            }
        }
    }

    pub fn image_size(&self) -> (u32, u32) {
        let image = self.image();
        (image.width(), image.height())
    }

    pub fn image(&self) -> &Image {
        match self {
            Self::Ahi(ahi) => &ahi.images[ahi.image_index],
            Self::Ahf(ahf) => match ahf.current_char {
                Some(chr) => ahf.font[chr].image(),
                None => ahf.font.default_glyph().image(),
            },
        }
    }

    pub fn image_mut(&mut self) -> &mut Image {
        match self {
            Self::Ahi(ahi) => Rc::make_mut(&mut ahi.images[ahi.image_index]),
            Self::Ahf(ahf) => match ahf.current_char {
                Some(chr) => ahf.font[chr].image_mut(),
                None => ahf.font.default_glyph_mut().image_mut(),
            },
        }
    }

    pub fn image_clone(&self) -> Rc<Image> {
        match self {
            Self::Ahi(ahi) => ahi.images[ahi.image_index].clone(),
            Self::Ahf(ahf) => Rc::new(match ahf.current_char {
                Some(chr) => ahf.font[chr].image().clone(),
                None => ahf.font.default_glyph().image().clone(),
            }),
        }
    }

    pub fn image_at(&self, index: usize) -> &Image {
        match self {
            Self::Ahi(ahi) => &ahi.images[index],
            Self::Ahf(ahf) => {
                if index == 0 {
                    ahf.font.default_glyph().image()
                } else {
                    let chr = ahf.font.chars().nth(index - 1).unwrap();
                    ahf.font[chr].image()
                }
            }
        }
    }

    pub fn font(&self) -> Option<&Font> {
        match self {
            Self::Ahi(_) => None,
            Self::Ahf(ahf) => Some(&ahf.font),
        }
    }

    pub fn add_new_palette(&mut self) -> bool {
        match self {
            Self::Ahi(ahi) => {
                ahi.add_new_palette();
                true
            }
            Self::Ahf(_) => false,
        }
    }

    pub fn delete_palette(&mut self) -> bool {
        match self {
            Self::Ahi(ahi) => ahi.delete_current_palette(),
            Self::Ahf(_) => false,
        }
    }

    pub fn set_palette_color(&mut self, color: Color, rgba: Rgba) -> bool {
        match self {
            Self::Ahi(ahi) => ahi.set_palette_color(color, rgba),
            Self::Ahf(_) => false,
        }
    }

    pub fn add_new_image(&mut self, chr: char) -> bool {
        let (width, height) = self.image_size();
        match self {
            Self::Ahi(ahi) => {
                ahi.add_new_image(width, height);
                true
            }
            Self::Ahf(ahf) => ahf.add_new_glyph(chr, width, height),
        }
    }

    pub fn delete_image(&mut self) -> bool {
        match self {
            Self::Ahi(ahi) => ahi.delete_current_image(),
            Self::Ahf(ahf) => ahf.delete_current_glyph(),
        }
    }

    pub fn set_allows_mixed_sizes(&mut self, allow: bool) {
        let (width, height) = self.image_size();
        match self {
            Self::Ahi(ahi) => {
                ahi.mixed_sizes = allow;
                if !allow {
                    ahi.resize_all_images(width, height)
                }
            }
            Self::Ahf(_) => {}
        }
    }

    pub fn resize_images(&mut self, new_width: u32, new_height: u32) {
        match self {
            Self::Ahi(ahi) => {
                if ahi.mixed_sizes {
                    ahi.resize_current_image(new_width, new_height)
                } else {
                    ahi.resize_all_images(new_width, new_height)
                }
            }
            Self::Ahf(ahf) => ahf.resize_current_glyph(new_width, new_height),
        }
    }

    pub fn set_metadata(&mut self, data: Vec<i16>) {
        if let Self::Ahi(ahi) = self {
            Rc::make_mut(&mut ahi.images[ahi.image_index]).set_metadata(data);
        }
    }

    pub fn set_glyph_metrics(
        &mut self,
        new_baseline: i32,
        new_left_edge: i32,
        new_right_edge: i32,
    ) {
        if let Self::Ahf(ahf) = self {
            ahf.font.set_baseline(new_baseline);
            let glyph = match ahf.current_char {
                Some(chr) => &mut ahf.font[chr],
                None => ahf.font.default_glyph_mut(),
            };
            glyph.set_left_edge(new_left_edge);
            glyph.set_right_edge(new_right_edge);
        }
    }

    pub fn set_tag(&mut self, tag: String) {
        if let Self::Ahi(ahi) = self {
            Rc::make_mut(&mut ahi.images[ahi.image_index]).set_tag(tag);
        }
    }

    pub fn save_to_file(&self, filepath: &str) -> io::Result<()> {
        let mut file = File::create(filepath)?;
        match self {
            Self::Ahi(ahi) => {
                let images: Vec<Image> = ahi
                    .images
                    .iter()
                    .map(|rc| Rc::as_ref(rc).clone())
                    .collect();
                let palettes: Vec<Palette> = ahi
                    .palettes
                    .iter()
                    .map(|rc| Rc::as_ref(rc).clone())
                    .collect();
                let collection = Collection { images, palettes };
                collection.write(&mut file)?;
            }
            Self::Ahf(ahf) => ahf.font.write(&mut file)?,
        }
        file.flush()
    }
}

//===========================================================================//
