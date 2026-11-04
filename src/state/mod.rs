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

mod ahf_data;
mod ahi_data;
mod mirror;

use self::ahf_data::AhfData;
use self::ahi_data::AhiData;
pub use self::mirror::Mirror;
use crate::util::{self, Rgba};
use ahi::{Collection, Color, Font, Image, Palette};
use sdl2::rect::{Point, Rect};
use std::fs::File;
use std::io;
use std::mem;
use std::ops::Deref;
use std::rc::Rc;

//===========================================================================//

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Tool {
    Checkerboard,
    Eyedropper,
    Lasso,
    Line,
    Oval,
    PaintBucket,
    PaletteReplace,
    PaletteSwap,
    Pencil,
    Rectangle,
    Select,
    Watercolor,
}

//===========================================================================//

#[derive(Clone)]
enum Data {
    Ahi(AhiData),
    Ahf(AhfData),
}

impl Data {
    fn from_collection(collection: Collection) -> Data {
        Data::Ahi(AhiData::new(collection))
    }
}

//===========================================================================//

#[derive(Clone)]
struct Snapshot {
    data: Data,
    selection: Option<(Rc<Image>, Point)>,
    unsaved: bool,
}

//===========================================================================//

pub struct EditorState {
    color: Color,
    filepath: String,
    current: Snapshot,
    undo_stack: Vec<Snapshot>,
    redo_stack: Vec<Snapshot>,
    clipboard: Option<(Rc<Image>, Point)>,
    tool: Tool,
    prev_tool: Tool,
    mirror: Mirror,
    persistent_mutation_active: bool,
    grid: (u32, u32),
    test_sentence: String,
}

impl EditorState {
    pub fn new(filepath: String, collection: Collection) -> EditorState {
        EditorState {
            color: Color::C1,
            filepath,
            current: Snapshot {
                data: Data::from_collection(collection),
                selection: None,
                unsaved: false,
            },
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            clipboard: None,
            tool: Tool::Pencil,
            prev_tool: Tool::Pencil,
            mirror: Mirror::None,
            persistent_mutation_active: false,
            grid: (0, 0),
            test_sentence: DEFAULT_TEST_SENTENCE.to_string(),
        }
    }

    pub fn is_unsaved(&self) -> bool {
        self.current.unsaved
    }

    pub fn filepath(&self) -> &String {
        &self.filepath
    }

    pub fn swap_filepath(&mut self, path: String) -> String {
        mem::replace(&mut self.filepath, path)
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    pub fn grid(&self) -> (u32, u32) {
        self.grid
    }

    pub fn set_grid(&mut self, horz: u32, vert: u32) {
        self.grid = (horz, vert);
    }

    pub fn tool(&self) -> Tool {
        self.tool
    }

    pub fn set_tool(&mut self, tool: Tool) {
        if self.tool != tool {
            self.unselect_if_necessary();
            self.prev_tool = self.tool;
            self.tool = tool;
        }
    }

    pub fn mirror(&self) -> Mirror {
        self.mirror
    }

    pub fn set_mirror(&mut self, mirror: Mirror) {
        self.mirror = mirror;
    }

    pub fn mirror_positions(&self, (x, y): (u32, u32)) -> Vec<(u32, u32)> {
        self.mirror().positions(self.image_size(), (x, y))
    }

    pub fn test_sentence(&self) -> &String {
        &self.test_sentence
    }

    pub fn set_test_sentence(&mut self, text: String) {
        self.test_sentence = text;
    }

    pub fn eyedrop_at(&mut self, position: (u32, u32)) {
        self.color = self.image()[position];
        if self.tool == Tool::Eyedropper {
            self.tool = if self.prev_tool == Tool::Select {
                Tool::Pencil
            } else {
                self.prev_tool
            };
        }
    }

    pub fn num_palettes(&self) -> usize {
        match &self.current.data {
            Data::Ahi(ahi) => ahi.palettes.len(),
            Data::Ahf(_) => 0,
        }
    }

    pub fn palette_index(&self) -> usize {
        match &self.current.data {
            Data::Ahi(ahi) => ahi.palette_index,
            Data::Ahf(_) => 0,
        }
    }

    pub fn set_palette_index(&mut self, index: usize) {
        match &mut self.current.data {
            Data::Ahi(ahi) => {
                ahi.palette_index = index % (1 + ahi.palettes.len());
            }
            Data::Ahf(_) => {}
        }
    }

    pub fn palette(&self) -> &Palette {
        match &self.current.data {
            Data::Ahi(ahi) => ahi.current_palette(),
            Data::Ahf(_) => Palette::default(),
        }
    }

    pub fn num_images(&self) -> usize {
        match &self.current.data {
            Data::Ahi(ahi) => ahi.images.len(),
            Data::Ahf(ahf) => 1 + ahf.font.chars().len(),
        }
    }

    pub fn image_index(&self) -> usize {
        match &self.current.data {
            Data::Ahi(ahi) => ahi.image_index,
            Data::Ahf(ahf) => ahf.image_index(),
        }
    }

    pub fn set_image_index(&mut self, index: usize) {
        self.unselect_if_necessary();
        match &mut self.current.data {
            Data::Ahi(ahi) => {
                debug_assert!(!ahi.images.is_empty());
                ahi.image_index = index % ahi.images.len();
            }
            Data::Ahf(ahf) => ahf.set_image_index(index),
        }
    }

    pub fn go_to(&mut self, text: &str) -> bool {
        match &mut self.current.data {
            Data::Ahi(ahi) => match text.parse::<usize>() {
                Ok(index) if index < ahi.images.len() => {
                    ahi.image_index = index;
                    true
                }
                _ => false,
            },
            Data::Ahf(ahf) => {
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

    pub fn image_name(&self) -> String {
        match &self.current.data {
            Data::Ahi(ahi) => format!("{}", ahi.image_index),
            Data::Ahf(ahf) => {
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

    pub fn image_metrics(&self) -> Option<(i32, i32, i32)> {
        match &self.current.data {
            Data::Ahi(_) => None,
            Data::Ahf(ahf) => {
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
        match &self.current.data {
            Data::Ahi(ahi) => &ahi.images[ahi.image_index],
            Data::Ahf(ahf) => match ahf.current_char {
                Some(chr) => ahf.font[chr].image(),
                None => ahf.font.default_glyph().image(),
            },
        }
    }

    pub fn image_at(&self, index: usize) -> &Image {
        match &self.current.data {
            Data::Ahi(ahi) => &ahi.images[index],
            Data::Ahf(ahf) => {
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
        match &self.current.data {
            Data::Ahi(_) => None,
            Data::Ahf(ahf) => Some(&ahf.font),
        }
    }

    pub fn selection(&self) -> Option<(&Image, Point)> {
        match self.current.selection {
            Some((ref image, position)) => Some((image, position)),
            None => None,
        }
    }

    pub fn selection_rect(&self) -> Option<Rect> {
        self.current.selection.as_ref().map(|&(ref img, pt)| {
            Rect::new(pt.x(), pt.y(), img.width(), img.height())
        })
    }

    pub fn unselect_if_necessary(&mut self) {
        self.reset_persistent_mutation();
        if self.selection().is_some() {
            self.mutation().unselect();
        }
    }

    pub fn mutation<'a>(&'a mut self) -> Mutation<'a> {
        self.push_change();
        self.current.unsaved = true;
        Mutation { state: self }
    }

    pub fn persistent_mutation<'a>(&'a mut self) -> Mutation<'a> {
        if !self.persistent_mutation_active {
            self.push_change();
            self.persistent_mutation_active = true;
        }
        self.current.unsaved = true;
        Mutation { state: self }
    }

    pub fn reset_persistent_mutation(&mut self) {
        self.persistent_mutation_active = false;
    }

    fn push_change(&mut self) {
        self.reset_persistent_mutation();
        self.redo_stack.clear();
        self.undo_stack.push(self.current.clone());
        if self.undo_stack.len() > MAX_UNDOS {
            self.undo_stack.remove(0);
        }
    }

    pub fn undo(&mut self) -> bool {
        if let Some(mut snapshot) = self.undo_stack.pop() {
            mem::swap(&mut snapshot, &mut self.current);
            self.redo_stack.push(snapshot);
            if self.current.selection.is_some() {
                self.tool = Tool::Select;
            }
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some(mut snapshot) = self.redo_stack.pop() {
            mem::swap(&mut snapshot, &mut self.current);
            self.undo_stack.push(snapshot);
            if self.current.selection.is_some() {
                self.tool = Tool::Select;
            }
            true
        } else {
            false
        }
    }

    pub fn save_to_file(&mut self) -> io::Result<()> {
        self.unselect_if_necessary();
        let mut file = File::create(&self.filepath)?;
        match &self.current.data {
            Data::Ahi(ahi) => {
                let images: Vec<Image> =
                    ahi.images.iter().map(|rc| rc.deref().clone()).collect();
                let palettes: Vec<Palette> =
                    ahi.palettes.iter().map(|rc| rc.deref().clone()).collect();
                let collection = Collection { images, palettes };
                collection.write(&mut file)?;
            }
            Data::Ahf(ahf) => ahf.font.write(file)?,
        }
        self.current.unsaved = false;
        for snapshot in self.undo_stack.iter_mut() {
            snapshot.unsaved = true;
        }
        for snapshot in self.redo_stack.iter_mut() {
            snapshot.unsaved = true;
        }
        Ok(())
    }

    pub fn load_collection(&mut self, path: String, collection: Collection) {
        self.load_data(path, Data::from_collection(collection));
    }

    pub fn load_font(&mut self, path: String, font: Font) {
        self.load_data(path, Data::Ahf(AhfData { current_char: None, font }));
    }

    fn load_data(&mut self, path: String, data: Data) {
        self.filepath = path;
        self.current = Snapshot { data, selection: None, unsaved: false };
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.persistent_mutation_active = false;
    }
}

//===========================================================================//

pub struct Mutation<'a> {
    state: &'a mut EditorState,
}

impl<'a> Mutation<'a> {
    fn image_rc(&self) -> Rc<Image> {
        match &self.state.current.data {
            Data::Ahi(ahi) => ahi.images[ahi.image_index].clone(),
            Data::Ahf(ahf) => Rc::new(match ahf.current_char {
                Some(chr) => ahf.font[chr].image().clone(),
                None => ahf.font.default_glyph().image().clone(),
            }),
        }
    }

    pub fn image(&mut self) -> &mut Image {
        match &mut self.state.current.data {
            Data::Ahi(ahi) => Rc::make_mut(&mut ahi.images[ahi.image_index]),
            Data::Ahf(ahf) => match ahf.current_char {
                Some(chr) => ahf.font[chr].image_mut(),
                None => ahf.font.default_glyph_mut().image_mut(),
            },
        }
    }

    pub fn color_pixel(&mut self, position: (u32, u32)) {
        let color = self.state.color();
        let positions = self.state.mirror_positions(position);
        let image = self.image();
        for pos in positions {
            image[pos] = color;
        }
    }

    pub fn add_new_palette(&mut self) -> bool {
        self.unselect();
        match &mut self.state.current.data {
            Data::Ahi(ahi) => {
                ahi.add_new_palette();
                true
            }
            Data::Ahf(_) => false,
        }
    }

    pub fn delete_palette(&mut self) -> bool {
        self.unselect();
        match &mut self.state.current.data {
            Data::Ahi(ahi) => ahi.delete_current_palette(),
            Data::Ahf(_) => false,
        }
    }

    pub fn set_palette_color(&mut self, color: Color, rgba: Rgba) -> bool {
        self.unselect();
        match &mut self.state.current.data {
            Data::Ahi(ahi) => ahi.set_palette_color(color, rgba),
            Data::Ahf(_) => false,
        }
    }

    pub fn add_new_image(&mut self, chr: char) -> bool {
        self.unselect();
        let (width, height) = self.state.image_size();
        match &mut self.state.current.data {
            Data::Ahi(ahi) => {
                ahi.add_new_image(width, height);
                true
            }
            Data::Ahf(ahf) => ahf.add_new_glyph(chr, width, height),
        }
    }

    pub fn add_images(&mut self, images: &[Image]) -> bool {
        for image in images {
            if !self.add_new_image('_') {
                return false;
            }
            self.image().draw(image, 0, 0);
        }
        true
    }

    pub fn delete_image(&mut self) -> bool {
        self.unselect();
        match &mut self.state.current.data {
            Data::Ahi(ahi) => ahi.delete_current_image(),
            Data::Ahf(ahf) => ahf.delete_current_glyph(),
        }
    }

    pub fn resize_images(&mut self, new_width: u32, new_height: u32) {
        self.unselect();
        match &mut self.state.current.data {
            Data::Ahi(ahi) => ahi.resize_all(new_width, new_height),
            Data::Ahf(ahf) => ahf.resize_current_glyph(new_width, new_height),
        }
    }

    pub fn set_metadata(&mut self, data: Vec<i16>) {
        if let Data::Ahi(ahi) = &mut self.state.current.data {
            Rc::make_mut(&mut ahi.images[ahi.image_index]).set_metadata(data);
        }
    }

    pub fn set_metrics(
        &mut self,
        new_baseline: i32,
        new_left_edge: i32,
        new_right_edge: i32,
    ) {
        if let Data::Ahf(ahf) = &mut self.state.current.data {
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
        if let Data::Ahi(ahi) = &mut self.state.current.data {
            Rc::make_mut(&mut ahi.images[ahi.image_index]).set_tag(tag);
        }
    }

    pub fn lasso(&mut self, vertices: &[(u32, u32)]) {
        self.unselect();
        let min_x = vertices.iter().map(|&(x, _)| x).min().unwrap_or(0);
        let max_x = vertices.iter().map(|&(x, _)| x + 1).max().unwrap_or(0);
        let min_y = vertices.iter().map(|&(_, y)| y).min().unwrap_or(0);
        let max_y = vertices.iter().map(|&(_, y)| y + 1).max().unwrap_or(0);
        let mut selected = Image::new(max_x - min_x, max_y - min_y);
        let image = self.image();
        for row in min_y..max_y {
            for col in min_x..max_x {
                if vertices.contains(&(col, row)) {
                    selected[(col - min_x, row - min_y)] = image[(col, row)];
                    image[(col, row)] = Color::C0;
                }
            }
        }
        self.state.current.selection =
            Some((Rc::new(selected), Point::new(min_x as i32, min_y as i32)));
        self.state.tool = Tool::Select;
    }

    pub fn select(&mut self, rect: &Rect) {
        self.unselect();
        let mut selected = Image::new(rect.width(), rect.height());
        selected.draw(self.image(), -rect.x(), -rect.y());
        self.state.current.selection =
            Some((Rc::new(selected), rect.top_left()));
        self.image().fill_rect(
            rect.x(),
            rect.y(),
            rect.width(),
            rect.height(),
            Color::C0,
        );
        self.state.tool = Tool::Select;
    }

    pub fn select_all(&mut self) {
        let (width, height) = self.state.image_size();
        self.select(&Rect::new(0, 0, width, height));
    }

    pub fn unselect(&mut self) {
        if let Some((image, position)) = self.state.current.selection.take() {
            self.image().draw(&image, position.x(), position.y());
        }
    }

    pub fn flip_selection_horz(&mut self) {
        if let Some((ref mut image, _)) = self.state.current.selection {
            *image = Rc::new(image.flip_horz());
        } else {
            *self.image() = self.state.image().flip_horz();
        }
    }

    pub fn flip_selection_vert(&mut self) {
        if let Some((ref mut image, _)) = self.state.current.selection {
            *image = Rc::new(image.flip_vert());
        } else {
            *self.image() = self.state.image().flip_vert();
        }
    }

    pub fn rotate_selection_clockwise(&mut self) {
        if let Some((ref mut image, _)) = self.state.current.selection {
            *image = Rc::new(image.rotate_cw());
        } else {
            let rotated = self.image().rotate_cw();
            self.image().clear();
            self.image().draw(&rotated, 0, 0);
        }
    }

    pub fn rotate_selection_counterclockwise(&mut self) {
        if let Some((ref mut image, _)) = self.state.current.selection {
            *image = Rc::new(image.rotate_ccw());
        } else {
            let rotated = self.image().rotate_ccw();
            self.image().clear();
            self.image().draw(&rotated, 0, 0);
        }
    }

    pub fn scale_selection_up(&mut self, by: u32) {
        if let Some((ref mut image, _)) = self.state.current.selection {
            *image = Rc::new(scale_up(image, by));
        } else {
            let scaled = scale_up(self.image(), by);
            self.image().clear();
            self.image().draw(&scaled, 0, 0);
        }
    }

    pub fn scale_selection_down(&mut self, by: u32) {
        let palette = self.state.palette().clone();
        if let Some((ref mut image, _)) = self.state.current.selection {
            *image = Rc::new(scale_down(image, &palette, by));
        } else {
            let scaled = scale_down(self.image(), &palette, by);
            self.image().clear();
            self.image().draw(&scaled, 0, 0);
        }
    }

    pub fn delete_selection(&mut self) {
        self.state.current.selection = None;
    }

    pub fn cut_selection(&mut self) {
        if self.state.current.selection.is_some() {
            self.state.clipboard = self.state.current.selection.take();
        } else {
            self.state.clipboard = Some((self.image_rc(), Point::new(0, 0)));
            self.image().clear();
        }
    }

    pub fn copy_selection(&mut self) {
        if self.state.current.selection.is_some() {
            self.state.clipboard = self.state.current.selection.clone();
        } else {
            self.state.clipboard = Some((self.image_rc(), Point::new(0, 0)));
        }
    }

    pub fn paste_selection(&mut self) {
        if self.state.clipboard.is_some() {
            self.unselect();
            self.state.current.selection = self.state.clipboard.clone();
            self.state.tool = Tool::Select;
        }
    }

    pub fn reposition_selection(&mut self, new_position: Point) {
        if let Some((_, ref mut position)) = self.state.current.selection {
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

const DEFAULT_TEST_SENTENCE: &str =
    "The quick, brown fox jumps over a ``lazy'' dog.";

const MAX_UNDOS: usize = 100;

//===========================================================================//
