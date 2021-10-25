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

use ahi::{Collection, Color, Font, Glyph, Image, Palette};
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

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Mirror {
    None,
    Horz,
    Vert,
    Both,
    Rot2,
    Rot4,
}

//===========================================================================//

#[derive(Clone)]
struct AhiData {
    palette_index: usize,
    palettes: Vec<Rc<Palette>>,
    image_index: usize,
    images: Vec<Rc<Image>>,
}

impl AhiData {
    fn new(mut collection: Collection) -> AhiData {
        if collection.images.is_empty() {
            collection.images.push(Image::new(32, 32));
        }
        AhiData {
            palette_index: 0,
            palettes: collection.palettes.drain(..).map(Rc::new).collect(),
            image_index: 0,
            images: collection.images.drain(..).map(Rc::new).collect(),
        }
    }
}

#[derive(Clone)]
struct AhfData {
    current_char: Option<char>,
    font: Font,
}

#[derive(Clone)]
enum Data {
    AHI(AhiData),
    AHF(AhfData),
}

impl Data {
    fn from_collection(collection: Collection) -> Data {
        Data::AHI(AhiData::new(collection))
    }
}

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
        let (width, height) = self.image_size();
        debug_assert!(x < width);
        debug_assert!(y < height);
        let mirror = self.mirror();
        let mut positions = vec![(x, y)];
        if mirror == Mirror::Horz || mirror == Mirror::Both {
            positions.push((width - x - 1, y));
        }
        if mirror == Mirror::Vert || mirror == Mirror::Both {
            positions.push((x, height - y - 1));
        }
        if mirror == Mirror::Both
            || mirror == Mirror::Rot2
            || mirror == Mirror::Rot4
        {
            positions.push((width - x - 1, height - y - 1));
        }
        if mirror == Mirror::Rot4 {
            let mut x1 = (height - y - 1) as i32;
            let mut y1 = x as i32;
            let mut x2 = y as i32;
            let mut y2 = (width - x - 1) as i32;
            if width > height {
                let diff = ((width - height) / 2) as i32;
                x1 += diff;
                x2 += diff;
                y1 -= diff;
                y2 -= diff;
            }
            if height > width {
                let diff = ((height - width) / 2) as i32;
                x1 -= diff;
                x2 -= diff;
                y1 += diff;
                y2 += diff;
            }
            if x1 >= 0
                && (x1 as u32) < width
                && y1 >= 0
                && (y1 as u32) < height
            {
                positions.push((x1 as u32, y1 as u32));
            }
            if x2 >= 0
                && (x2 as u32) < width
                && y2 >= 0
                && (y2 as u32) < height
            {
                positions.push((x2 as u32, y2 as u32));
            }
        }
        positions
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
        match self.current.data {
            Data::AHI(ref ahi) => ahi.palettes.len(),
            Data::AHF(_) => 1,
        }
    }

    pub fn palette_index(&self) -> usize {
        match self.current.data {
            Data::AHI(ref ahi) => ahi.palette_index,
            Data::AHF(_) => 0,
        }
    }

    pub fn set_palette_index(&mut self, index: usize) {
        match self.current.data {
            Data::AHI(ref mut ahi) => {
                ahi.palette_index = index % (1 + ahi.palettes.len());
            }
            Data::AHF(_) => {}
        }
    }

    pub fn palette(&self) -> &Palette {
        match self.current.data {
            Data::AHI(ref ahi) => {
                if ahi.palette_index < ahi.palettes.len() {
                    &ahi.palettes[ahi.palette_index]
                } else {
                    Palette::default()
                }
            }
            Data::AHF(_) => Palette::default(),
        }
    }

    pub fn num_images(&self) -> usize {
        match self.current.data {
            Data::AHI(ref ahi) => ahi.images.len(),
            Data::AHF(ref ahf) => 1 + ahf.font.chars().len(),
        }
    }

    pub fn image_index(&self) -> usize {
        match self.current.data {
            Data::AHI(ref ahi) => ahi.image_index,
            Data::AHF(ref ahf) => {
                if let Some(current) = ahf.current_char {
                    for (index, chr) in ahf.font.chars().enumerate() {
                        if chr == current {
                            return index + 1;
                        }
                    }
                }
                0
            }
        }
    }

    pub fn set_image_index(&mut self, index: usize) {
        self.unselect_if_necessary();
        match self.current.data {
            Data::AHI(ref mut ahi) => {
                debug_assert!(!ahi.images.is_empty());
                ahi.image_index = index % ahi.images.len();
            }
            Data::AHF(ref mut ahf) => {
                if index == 0 {
                    ahf.current_char = None;
                } else {
                    let chr = ahf.font.chars().skip(index - 1).next().unwrap();
                    ahf.current_char = Some(chr);
                }
            }
        }
    }

    pub fn go_to(&mut self, text: &str) -> bool {
        match self.current.data {
            Data::AHI(ref mut ahi) => match text.parse::<usize>() {
                Ok(index) if index < ahi.images.len() => {
                    ahi.image_index = index;
                    true
                }
                _ => false,
            },
            Data::AHF(ref mut ahf) => {
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
        match self.current.data {
            Data::AHI(ref ahi) => format!("{}", ahi.image_index),
            Data::AHF(ref ahf) => {
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
        match self.current.data {
            Data::AHI(_) => None,
            Data::AHF(ref ahf) => {
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
        match self.current.data {
            Data::AHI(ref ahi) => &ahi.images[ahi.image_index],
            Data::AHF(ref ahf) => match ahf.current_char {
                Some(chr) => ahf.font[chr].image(),
                None => ahf.font.default_glyph().image(),
            },
        }
    }

    pub fn image_at(&self, index: usize) -> &Image {
        match self.current.data {
            Data::AHI(ref ahi) => &ahi.images[index],
            Data::AHF(ref ahf) => {
                if index == 0 {
                    ahf.font.default_glyph().image()
                } else {
                    let chr = ahf.font.chars().skip(index - 1).next().unwrap();
                    ahf.font[chr].image()
                }
            }
        }
    }

    pub fn font(&self) -> Option<&Font> {
        match self.current.data {
            Data::AHI(_) => None,
            Data::AHF(ref ahf) => Some(&ahf.font),
        }
    }

    pub fn selection(&self) -> Option<(&Image, Point)> {
        match self.current.selection {
            Some((ref image, position)) => Some((&image, position)),
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

    pub fn mutation(&mut self) -> Mutation {
        self.push_change();
        self.current.unsaved = true;
        Mutation { state: self }
    }

    pub fn persistent_mutation(&mut self) -> Mutation {
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
        match self.current.data {
            Data::AHI(ref ahi) => {
                let images: Vec<Image> =
                    ahi.images.iter().map(|rc| rc.deref().clone()).collect();
                let palettes: Vec<Palette> =
                    ahi.palettes.iter().map(|rc| rc.deref().clone()).collect();
                let collection = Collection { images, palettes };
                collection.write(&mut file)?;
            }
            Data::AHF(ref ahf) => {
                ahf.font.write(file)?;
            }
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
        self.load_data(path, Data::AHF(AhfData { current_char: None, font }));
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
        match self.state.current.data {
            Data::AHI(ref ahi) => ahi.images[ahi.image_index].clone(),
            Data::AHF(ref ahf) => Rc::new(match ahf.current_char {
                Some(chr) => ahf.font[chr].image().clone(),
                None => ahf.font.default_glyph().image().clone(),
            }),
        }
    }

    pub fn image(&mut self) -> &mut Image {
        match self.state.current.data {
            Data::AHI(ref mut ahi) => {
                Rc::make_mut(&mut ahi.images[ahi.image_index])
            }
            Data::AHF(ref mut ahf) => match ahf.current_char {
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
        let new_palette = self.state.palette().clone();
        match self.state.current.data {
            Data::AHI(ref mut ahi) => {
                if ahi.palette_index < ahi.palettes.len() {
                    ahi.palette_index += 1;
                } else {
                    ahi.palette_index = 0;
                }
                let rc = Rc::new(new_palette);
                ahi.palettes.insert(ahi.palette_index, rc);
                true
            }
            Data::AHF(_) => false,
        }
    }

    pub fn delete_palette(&mut self) -> bool {
        self.unselect();
        match self.state.current.data {
            Data::AHI(ref mut ahi) => {
                if ahi.palette_index < ahi.palettes.len() {
                    ahi.palettes.remove(ahi.palette_index);
                    if ahi.palette_index > 0 {
                        ahi.palette_index -= 1;
                    } else {
                        ahi.palette_index = ahi.palettes.len();
                    }
                    true
                } else {
                    false
                }
            }
            Data::AHF(_) => false,
        }
    }

    pub fn set_palette_color(
        &mut self,
        color: Color,
        rgba: (u8, u8, u8, u8),
    ) -> bool {
        self.unselect();
        match self.state.current.data {
            Data::AHI(ref mut ahi) => {
                if ahi.palette_index < ahi.palettes.len() {
                    let mut palette =
                        Palette::clone(&ahi.palettes[ahi.palette_index]);
                    palette[color] = rgba;
                    ahi.palettes[ahi.palette_index] = Rc::new(palette);
                    true
                } else {
                    false
                }
            }
            Data::AHF(_) => false,
        }
    }

    pub fn add_new_image(&mut self, chr: char) -> bool {
        self.unselect();
        let (width, height) = self.state.image_size();
        match self.state.current.data {
            Data::AHI(ref mut ahi) => {
                ahi.image_index += 1;
                let rc = Rc::new(Image::new(width, height));
                ahi.images.insert(ahi.image_index, rc);
                true
            }
            Data::AHF(ref mut ahf) => {
                ahf.current_char = Some(chr);
                if ahf.font.get_char_glyph(chr).is_none() {
                    let glyph = Glyph::new(
                        Image::new(width, height),
                        0,
                        1 + width as i32,
                    );
                    ahf.font.set_char_glyph(chr, glyph);
                    true
                } else {
                    false
                }
            }
        }
    }

    pub fn delete_image(&mut self) -> bool {
        self.unselect();
        match self.state.current.data {
            Data::AHI(ref mut ahi) => {
                if ahi.images.len() > 1 {
                    let index = ahi.image_index;
                    ahi.images.remove(index);
                    if index == ahi.images.len() {
                        ahi.image_index -= 1;
                    }
                    true
                } else {
                    false
                }
            }
            Data::AHF(ref mut ahf) => {
                if let Some(chr) = ahf.current_char {
                    ahf.font.remove_char_glyph(chr);
                    ahf.current_char = None;
                    true
                } else {
                    false
                }
            }
        }
    }

    pub fn resize_images(&mut self, new_width: u32, new_height: u32) {
        self.unselect();
        match self.state.current.data {
            Data::AHI(ref mut ahi) => {
                ahi.images = ahi
                    .images
                    .iter()
                    .map(|image| Rc::new(image.crop(new_width, new_height)))
                    .collect();
            }
            Data::AHF(ref mut ahf) => {
                if new_height != ahf.font.glyph_height() {
                    let mut font = Font::with_glyph_height(new_height);
                    {
                        let glyph = ahf.font.default_glyph();
                        let new_glyph = Glyph::new(
                            glyph
                                .image()
                                .crop(glyph.image().width(), new_height),
                            glyph.left_edge(),
                            glyph.right_edge(),
                        );
                        font.set_default_glyph(new_glyph);
                    }
                    for chr in ahf.font.chars() {
                        let glyph = &ahf.font[chr];
                        let new_glyph = Glyph::new(
                            glyph
                                .image()
                                .crop(glyph.image().width(), new_height),
                            glyph.left_edge(),
                            glyph.right_edge(),
                        );
                        font.set_char_glyph(chr, new_glyph);
                    }
                    ahf.font = font;
                }
                match ahf.current_char {
                    Some(chr) => {
                        let new_glyph = {
                            let glyph = &ahf.font[chr];
                            Glyph::new(
                                glyph
                                    .image()
                                    .crop(new_width, glyph.image().height()),
                                glyph.left_edge(),
                                glyph.right_edge(),
                            )
                        };
                        ahf.font.set_char_glyph(chr, new_glyph);
                    }
                    None => {
                        let new_glyph = {
                            let glyph = ahf.font.default_glyph();
                            Glyph::new(
                                glyph
                                    .image()
                                    .crop(new_width, glyph.image().height()),
                                glyph.left_edge(),
                                glyph.right_edge(),
                            )
                        };
                        ahf.font.set_default_glyph(new_glyph);
                    }
                }
            }
        }
    }

    pub fn set_metadata(&mut self, data: Vec<i16>) {
        if let Data::AHI(ref mut ahi) = self.state.current.data {
            Rc::make_mut(&mut ahi.images[ahi.image_index]).set_metadata(data);
        }
    }

    pub fn set_metrics(
        &mut self,
        new_baseline: i32,
        new_left_edge: i32,
        new_right_edge: i32,
    ) {
        if let Data::AHF(ref mut ahf) = self.state.current.data {
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
        if let Data::AHI(ref mut ahi) = self.state.current.data {
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

    pub fn scale_selection_2x(&mut self) {
        if let Some((ref mut image, _)) = self.state.current.selection {
            *image = Rc::new(scale_2x(image));
        } else {
            let scaled = scale_2x(self.image());
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

fn scale_2x(image: &Image) -> Image {
    let mut scaled = Image::new(image.width() * 2, image.height() * 2);
    for row in 0..image.height() {
        for col in 0..image.width() {
            let color = image[(col, row)];
            scaled.fill_rect((2 * col) as i32, (2 * row) as i32, 2, 2, color);
        }
    }
    scaled
}

//===========================================================================//

const DEFAULT_TEST_SENTENCE: &'static str = "The quick, brown fox jumps over \
                                             a ``lazy'' dog.";

const MAX_UNDOS: usize = 100;

//===========================================================================//
