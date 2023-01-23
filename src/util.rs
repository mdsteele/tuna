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

use std::fs::File;
use std::io;

//===========================================================================//

const COLORS: &[ahi::Color] = &[
    ahi::Color::C0,
    ahi::Color::C1,
    ahi::Color::C2,
    ahi::Color::C3,
    ahi::Color::C4,
    ahi::Color::C5,
    ahi::Color::C6,
    ahi::Color::C7,
    ahi::Color::C8,
    ahi::Color::C9,
    ahi::Color::Ca,
    ahi::Color::Cb,
    ahi::Color::Cc,
    ahi::Color::Cd,
    ahi::Color::Ce,
    ahi::Color::Cf,
];

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

pub fn load_png_from_file(
    palette: &ahi::Palette,
    path: &String,
) -> io::Result<ahi::Image> {
    let decoder = png::Decoder::new(File::open(path)?);
    let mut reader = decoder.read_info()?;
    let mut buffer = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buffer)?;
    let rgba_data = match info.color_type {
        png::ColorType::Rgba => buffer,
        png::ColorType::Rgb => {
            let num_pixels = buffer.len() / 3;
            let mut rgba = Vec::with_capacity(num_pixels * 4);
            for i in 0..num_pixels {
                rgba.extend_from_slice(&buffer[(3 * i)..][..3]);
                rgba.push(u8::MAX);
            }
            rgba
        }
        png::ColorType::GrayscaleAlpha => {
            let num_pixels = buffer.len() / 2;
            let mut rgba = Vec::with_capacity(num_pixels * 4);
            for i in 0..num_pixels {
                let gray = buffer[2 * i];
                let alpha = buffer[2 * i + 1];
                rgba.push(gray);
                rgba.push(gray);
                rgba.push(gray);
                rgba.push(alpha);
            }
            rgba
        }
        png::ColorType::Grayscale => {
            let mut rgba = Vec::with_capacity(buffer.len() * 4);
            for value in buffer.into_iter() {
                rgba.push(value);
                rgba.push(value);
                rgba.push(value);
                rgba.push(std::u8::MAX);
            }
            rgba
        }
        png::ColorType::Indexed => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unsupported PNG color type: {:?}", info.color_type),
            ));
        }
    };
    let mut image = ahi::Image::new(info.width, info.height);
    for row in 0..info.height {
        for col in 0..info.width {
            let start = ((row * info.width + col) as usize) * 4;
            let png_rgba: (u8, u8, u8, u8) = (
                rgba_data[start + 0],
                rgba_data[start + 1],
                rgba_data[start + 2],
                rgba_data[start + 3],
            );
            let mut best_color = ahi::Color::C0;
            let mut best_dist = i32::MAX;
            for &color in COLORS {
                let color_rgba: (u8, u8, u8, u8) = palette[color];
                let delta = (
                    (color_rgba.0 as i32) - (png_rgba.0 as i32),
                    (color_rgba.1 as i32) - (png_rgba.1 as i32),
                    (color_rgba.2 as i32) - (png_rgba.2 as i32),
                    (color_rgba.3 as i32) - (png_rgba.3 as i32),
                );
                let dist = delta.0 * delta.0
                    + delta.1 * delta.1
                    + delta.2 * delta.2
                    + delta.3 * delta.3;
                if dist < best_dist {
                    best_dist = dist;
                    best_color = color;
                }
            }
            image[(col, row)] = best_color;
        }
    }
    Ok(image)
}

//===========================================================================//
