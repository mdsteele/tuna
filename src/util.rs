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

use ahi;
use std::fs::File;
use std::io;

//===========================================================================//

pub fn load_ahf_from_file(path: &String) -> io::Result<ahi::Font> {
    let mut file = File::open(path)?;
    ahi::Font::read(&mut file)
}

pub fn load_ahi_from_file(path: &String) -> io::Result<ahi::Collection> {
    let mut file = File::open(path)?;
    ahi::Collection::read(&mut file)
}

pub fn save_png_to_file(
    image: &ahi::Image,
    palette: &ahi::Palette,
    path: &String,
) -> io::Result<()> {
    let rgba_data = image.rgba_data(&palette);
    let output_file = File::create(path)?;
    let mut encoder =
        png::Encoder::new(output_file, image.width(), image.height());
    // TODO: Set palette and use ColorType::Indexed instead.
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(&rgba_data).map_err(|err| match err {
        png::EncodingError::IoError(err) => err,
        err => io::Error::new(io::ErrorKind::InvalidData, err.to_string()),
    })
}

//===========================================================================//
