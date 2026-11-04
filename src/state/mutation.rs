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

use super::data::Data;
use super::settings::Settings;
use super::snapshot::Snapshot;
use super::tool::Tool;
use crate::util::{self, Rgba};
use ahi::{Color, Image, Palette};
use sdl2::rect::{Point, Rect};
use std::rc::Rc;

//===========================================================================//

pub struct Mutation<'a> {
    pub(super) settings: &'a mut Settings,
    pub(super) snapshot: &'a mut Snapshot,
}

impl<'a> Mutation<'a> {
    fn data(&self) -> &Data {
        &self.snapshot.data
    }

    fn data_mut(&mut self) -> &mut Data {
        &mut self.snapshot.data
    }

    fn image(&self) -> &Image {
        self.data().image()
    }

    pub fn image_mut(&mut self) -> &mut Image {
        self.data_mut().image_mut()
    }

    fn image_clone(&self) -> Rc<Image> {
        self.snapshot.data.image_clone()
    }

    pub fn color_pixel(&mut self, position: (u32, u32)) {
        let color = self.settings.color;
        let size = self.data().image_size();
        let positions = self.settings.mirror.positions(size, position);
        let image = self.image_mut();
        for pos in positions {
            image[pos] = color;
        }
    }

    pub fn add_new_palette(&mut self) -> bool {
        self.unselect();
        self.data_mut().add_new_palette()
    }

    pub fn delete_palette(&mut self) -> bool {
        self.unselect();
        self.data_mut().delete_palette()
    }

    pub fn set_palette_color(&mut self, color: Color, rgba: Rgba) -> bool {
        self.unselect();
        self.data_mut().set_palette_color(color, rgba)
    }

    pub fn add_new_image(&mut self, chr: char) -> bool {
        self.unselect();
        self.data_mut().add_new_image(chr)
    }

    pub fn add_images(&mut self, images: &[Image]) -> bool {
        for image in images {
            if !self.add_new_image('_') {
                return false;
            }
            self.image_mut().draw(image, 0, 0);
        }
        true
    }

    pub fn delete_image(&mut self) -> bool {
        self.unselect();
        self.data_mut().delete_image()
    }

    pub fn set_allows_mixed_sizes(&mut self, allow: bool) {
        self.unselect();
        self.data_mut().set_allows_mixed_sizes(allow);
    }

    pub fn resize_images(&mut self, new_width: u32, new_height: u32) {
        self.unselect();
        self.data_mut().resize_images(new_width, new_height);
    }

    pub fn set_metadata(&mut self, metadata: Vec<i16>) {
        self.data_mut().set_metadata(metadata);
    }

    pub fn set_glyph_metrics(
        &mut self,
        new_baseline: i32,
        new_left_edge: i32,
        new_right_edge: i32,
    ) {
        self.data_mut().set_glyph_metrics(
            new_baseline,
            new_left_edge,
            new_right_edge,
        );
    }

    pub fn set_tag(&mut self, tag: String) {
        self.data_mut().set_tag(tag);
    }

    pub fn lasso(&mut self, vertices: &[(u32, u32)]) {
        self.unselect();
        let min_x = vertices.iter().map(|&(x, _)| x).min().unwrap_or(0);
        let max_x = vertices.iter().map(|&(x, _)| x + 1).max().unwrap_or(0);
        let min_y = vertices.iter().map(|&(_, y)| y).min().unwrap_or(0);
        let max_y = vertices.iter().map(|&(_, y)| y + 1).max().unwrap_or(0);
        let mut selected = Image::new(max_x - min_x, max_y - min_y);
        let image = self.image_mut();
        for row in min_y..max_y {
            for col in min_x..max_x {
                if vertices.contains(&(col, row)) {
                    selected[(col - min_x, row - min_y)] = image[(col, row)];
                    image[(col, row)] = Color::C0;
                }
            }
        }
        self.snapshot.selection =
            Some((Rc::new(selected), Point::new(min_x as i32, min_y as i32)));
        self.settings.tool = Tool::Select;
    }

    pub fn select(&mut self, rect: &Rect) {
        self.unselect();
        let mut selected = Image::new(rect.width(), rect.height());
        selected.draw(self.image_mut(), -rect.x(), -rect.y());
        self.snapshot.selection = Some((Rc::new(selected), rect.top_left()));
        self.image_mut().fill_rect(
            rect.x(),
            rect.y(),
            rect.width(),
            rect.height(),
            Color::C0,
        );
        self.settings.tool = Tool::Select;
    }

    pub fn select_all(&mut self) {
        let (width, height) = self.data_mut().image_size();
        self.select(&Rect::new(0, 0, width, height));
    }

    pub fn unselect(&mut self) {
        if let Some((image, position)) = self.snapshot.selection.take() {
            self.image_mut().draw(&image, position.x(), position.y());
        }
    }

    pub fn flip_selection_horz(&mut self) {
        if let Some((ref mut image, _)) = self.snapshot.selection {
            *image = Rc::new(image.flip_horz());
        } else {
            *self.image_mut() = self.image().flip_horz();
        }
    }

    pub fn flip_selection_vert(&mut self) {
        if let Some((ref mut image, _)) = self.snapshot.selection {
            *image = Rc::new(image.flip_vert());
        } else {
            *self.image_mut() = self.image().flip_vert();
        }
    }

    pub fn rotate_selection_clockwise(&mut self) {
        if let Some((ref mut image, _)) = self.snapshot.selection {
            *image = Rc::new(image.rotate_cw());
        } else {
            let rotated = self.image_mut().rotate_cw();
            self.image_mut().clear();
            self.image_mut().draw(&rotated, 0, 0);
        }
    }

    pub fn rotate_selection_counterclockwise(&mut self) {
        if let Some((ref mut image, _)) = self.snapshot.selection {
            *image = Rc::new(image.rotate_ccw());
        } else {
            let rotated = self.image_mut().rotate_ccw();
            self.image_mut().clear();
            self.image_mut().draw(&rotated, 0, 0);
        }
    }

    pub fn scale_selection_up(&mut self, by: u32) {
        if let Some((ref mut image, _)) = self.snapshot.selection {
            *image = Rc::new(scale_up(image, by));
        } else {
            let scaled = scale_up(self.image_mut(), by);
            self.image_mut().clear();
            self.image_mut().draw(&scaled, 0, 0);
        }
    }

    pub fn scale_selection_down(&mut self, by: u32) {
        let palette = self.data().palette().clone();
        if let Some((ref mut image, _)) = self.snapshot.selection {
            *image = Rc::new(scale_down(image, &palette, by));
        } else {
            let scaled = scale_down(self.image_mut(), &palette, by);
            self.image_mut().clear();
            self.image_mut().draw(&scaled, 0, 0);
        }
    }

    pub fn delete_selection(&mut self) {
        self.snapshot.selection = None;
    }

    pub fn cut_selection(&mut self) {
        let selection = self.snapshot.selection.take();
        self.settings.clipboard = selection.or_else(|| {
            let image = self.image_clone();
            self.image_mut().clear();
            Some((image, Point::new(0, 0)))
        });
    }

    pub fn copy_selection(&mut self) {
        let selection = self.snapshot.selection.clone();
        self.settings.clipboard =
            selection.or_else(|| Some((self.image_clone(), Point::new(0, 0))));
    }

    pub fn paste_selection(&mut self) {
        if self.settings.clipboard.is_some() {
            self.unselect();
            self.snapshot.selection = self.settings.clipboard.clone();
            self.settings.tool = Tool::Select;
        }
    }

    pub fn reposition_selection(&mut self, new_position: Point) {
        if let Some((_, ref mut position)) = self.snapshot.selection {
            *position = new_position;
        }
    }
}

//===========================================================================//

fn scale_up(image: &Image, by: u32) -> Image {
    let mut scaled = Image::new(image.width() * by, image.height() * by);
    for row in 0..image.height() {
        for col in 0..image.width() {
            let color = image[(col, row)];
            scaled.fill_rect(
                (by * col) as i32,
                (by * row) as i32,
                by,
                by,
                color,
            );
        }
    }
    scaled
}

fn scale_down(image: &Image, palette: &Palette, by: u32) -> Image {
    let mut scaled =
        Image::new((image.width() / by).max(1), (image.height() / by).max(1));
    for row in 0..scaled.height() {
        for col in 0..scaled.width() {
            let mut total_r = 0u32;
            let mut total_g = 0u32;
            let mut total_b = 0u32;
            let mut total_a = 0u32;
            for x in 0..by {
                for y in 0..by {
                    let color = image[(col * by + x, row * by + y)];
                    let rgba = palette[color];
                    total_r += rgba.0 as u32;
                    total_g += rgba.1 as u32;
                    total_b += rgba.2 as u32;
                    total_a += rgba.3 as u32;
                }
            }
            let avg_rgba = (
                (total_r / (by * by)) as u8,
                (total_g / (by * by)) as u8,
                (total_b / (by * by)) as u8,
                (total_a / (by * by)) as u8,
            );
            scaled[(col, row)] = util::nearest_color(palette, avg_rgba);
        }
    }
    scaled
}

//===========================================================================//
