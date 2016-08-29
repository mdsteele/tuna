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

use ahi::{Color, Font, Glyph, Image};
use sdl2::rect::{Point, Rect};
use std::fs::File;
use std::io;
use std::mem;
use std::ops::Deref;
use std::rc::Rc;
use super::util;

// ========================================================================= //

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Tool {
    Checkerboard,
    Eyedropper,
    Line,
    Oval,
    PaintBucket,
    Pencil,
    Rectangle,
    Select,
}

#[derive(Clone, Eq, PartialEq)]
pub enum Mode {
    Edit,
    Goto(String),
    LoadFile(String),
    NewGlyph(String),
    Resize(String),
    SaveAs(String),
    SetMetrics(String),
    TestSentence,
}

// ========================================================================= //

#[derive(Clone)]
struct AhiData {
    image_index: usize,
    images: Vec<Rc<Image>>,
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

#[derive(Clone)]
struct Snapshot {
    data: Data,
    selection: Option<(Rc<Image>, Point)>,
    unsaved: bool,
}

// ========================================================================= //

pub struct EditorState {
    mode: Mode,
    color: Color,
    filepath: String,
    current: Snapshot,
    undo_stack: Vec<Snapshot>,
    redo_stack: Vec<Snapshot>,
    clipboard: Option<(Rc<Image>, Point)>,
    tool: Tool,
    prev_tool: Tool,
    persistent_mutation_active: bool,
    test_sentence: String,
}

impl EditorState {
    pub fn new(filepath: String, mut images: Vec<Image>) -> EditorState {
        if images.is_empty() {
            images.push(Image::new(32, 32));
        }
        EditorState {
            mode: Mode::Edit,
            color: Color::Black,
            filepath: filepath,
            current: Snapshot {
                data: Data::AHI(AhiData {
                    image_index: 0,
                    images: images.drain(..).map(Rc::new).collect(),
                }),
                selection: None,
                unsaved: false,
            },
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            clipboard: None,
            tool: Tool::Pencil,
            prev_tool: Tool::Pencil,
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

    pub fn mode(&self) -> &Mode {
        &self.mode
    }

    pub fn mode_mut(&mut self) -> &mut Mode {
        &mut self.mode
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

    pub fn test_sentence(&self) -> &String {
        &self.test_sentence
    }

    pub fn test_sentence_mut(&mut self) -> &mut String {
        &mut self.test_sentence
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
                Some((ahf.font.baseline(),
                      glyph.left_edge(),
                      glyph.right_edge()))
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
            Data::AHF(ref ahf) => {
                match ahf.current_char {
                    Some(chr) => ahf.font[chr].image(),
                    None => ahf.font.default_glyph().image(),
                }
            }
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

    fn unselect_if_necessary(&mut self) {
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
        let mut file = try!(File::create(&self.filepath));
        match self.current.data {
            Data::AHI(ref ahi) => {
                let images: Vec<Image> = ahi.images
                                            .iter()
                                            .map(|rc| rc.deref().clone())
                                            .collect();
                try!(Image::write_all(&mut file, &images));
            }
            Data::AHF(ref ahf) => {
                try!(ahf.font.write(file));
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

    pub fn begin_new_image(&mut self) -> bool {
        match self.current.data {
            Data::AHI(_) => self.mutation().add_new_image('_'),
            Data::AHF(_) => {
                if self.mode == Mode::Edit {
                    self.unselect_if_necessary();
                    self.mode = Mode::NewGlyph(String::new());
                    true
                } else {
                    false
                }
            }
        }
    }

    pub fn begin_goto(&mut self) -> bool {
        if self.mode == Mode::Edit {
            self.unselect_if_necessary();
            self.mode = Mode::Goto(String::new());
            true
        } else {
            false
        }
    }

    pub fn begin_load_file(&mut self) -> bool {
        if self.mode == Mode::Edit {
            self.unselect_if_necessary();
            self.mode = Mode::LoadFile(self.filepath.clone());
            true
        } else {
            false
        }
    }

    pub fn begin_resize(&mut self) -> bool {
        if self.mode == Mode::Edit {
            self.unselect_if_necessary();
            self.mode = Mode::Resize(format!("{}x{}",
                                             self.image().width(),
                                             self.image().height()));
            true
        } else {
            false
        }
    }

    pub fn begin_save_as(&mut self) -> bool {
        if self.mode == Mode::Edit {
            self.unselect_if_necessary();
            self.mode = Mode::SaveAs(self.filepath.clone());
            true
        } else {
            false
        }
    }

    pub fn begin_set_metrics(&mut self) -> bool {
        if self.mode == Mode::Edit {
            if let Some((bl, le, re)) = self.image_metrics() {
                self.unselect_if_necessary();
                self.mode = Mode::SetMetrics(format!("{}/{}/{}", bl, le, re));
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn begin_set_test_sentence(&mut self) -> bool {
        if self.mode == Mode::Edit && self.font().is_some() {
            self.mode = Mode::TestSentence;
            true
        } else {
            false
        }
    }

    pub fn mode_cancel(&mut self) -> bool {
        match self.mode {
            Mode::Edit => false,
            _ => {
                self.mode = Mode::Edit;
                true
            }
        }
    }

    pub fn mode_perform(&mut self) -> bool {
        match self.mode.clone() {
            Mode::Edit => false,
            Mode::Goto(ref text) => {
                match self.current.data {
                    Data::AHI(ref mut ahi) => {
                        match text.parse::<usize>() {
                            Ok(index) if index < ahi.images.len() => {
                                ahi.image_index = index;
                                self.mode = Mode::Edit;
                                true
                            }
                            _ => false,
                        }
                    }
                    Data::AHF(ref mut ahf) => {
                        if text == "def" {
                            ahf.current_char = None;
                            self.mode = Mode::Edit;
                            true
                        } else {
                            let chars: Vec<char> = text.chars().collect();
                            if chars.len() == 1 {
                                ahf.current_char = Some(chars[0]);
                                self.mode = Mode::Edit;
                                true
                            } else {
                                false
                            }
                        }
                    }
                }
            }
            Mode::LoadFile(path) => {
                match util::load_ahi_from_file(&path) {
                    Ok(mut images) => {
                        let data = Data::AHI(AhiData {
                            image_index: 0,
                            images: images.drain(..).map(Rc::new).collect(),
                        });
                        self.load_data(path, data);
                        true
                    }
                    Err(_) => {
                        match util::load_ahf_from_file(&path) {
                            Ok(font) => {
                                let data = Data::AHF(AhfData {
                                    current_char: None,
                                    font: font,
                                });
                                self.load_data(path, data);
                                true
                            }
                            Err(_) => false,
                        }
                    }
                }
            }
            Mode::NewGlyph(text) => {
                let chars: Vec<char> = text.chars().collect();
                if chars.len() == 1 &&
                   self.mutation().add_new_image(chars[0]) {
                    self.mode = Mode::Edit;
                    true
                } else {
                    false
                }
            }
            Mode::Resize(text) => {
                let pieces: Vec<&str> = text.split('x').collect();
                if pieces.len() != 2 {
                    return false;
                }
                let new_width = match pieces[0].parse::<u32>() {
                    Ok(width) => width,
                    Err(_) => return false,
                };
                let new_height = match pieces[1].parse::<u32>() {
                    Ok(height) => height,
                    Err(_) => return false,
                };
                self.mutation().resize_images(new_width, new_height);
                self.mode = Mode::Edit;
                true
            }
            Mode::SaveAs(mut path) => {
                mem::swap(&mut path, &mut self.filepath);
                match self.save_to_file() {
                    Ok(()) => {
                        self.mode = Mode::Edit;
                        true
                    }
                    Err(_) => {
                        mem::swap(&mut path, &mut self.filepath);
                        false
                    }
                }
            }
            Mode::SetMetrics(text) => {
                let pieces: Vec<&str> = text.split('/').collect();
                if pieces.len() != 3 {
                    return false;
                }
                let new_baseline = match pieces[0].parse::<i32>() {
                    Ok(baseline) => baseline,
                    Err(_) => return false,
                };
                let new_left_edge = match pieces[1].parse::<i32>() {
                    Ok(left_edge) => left_edge,
                    Err(_) => return false,
                };
                let new_right_edge = match pieces[2].parse::<i32>() {
                    Ok(right_edge) => right_edge,
                    Err(_) => return false,
                };
                self.mutation()
                    .set_metrics(new_baseline, new_left_edge, new_right_edge);
                self.mode = Mode::Edit;
                true
            }
            Mode::TestSentence => {
                self.mode = Mode::Edit;
                true
            }
        }
    }

    fn load_data(&mut self, path: String, data: Data) {
        self.mode = Mode::Edit;
        self.filepath = path;
        self.current = Snapshot {
            data: data,
            selection: None,
            unsaved: false,
        };
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.persistent_mutation_active = false;
    }
}

// ========================================================================= //

pub struct Mutation<'a> {
    state: &'a mut EditorState,
}

impl<'a> Mutation<'a> {
    fn image_rc(&self) -> Rc<Image> {
        match self.state.current.data {
            Data::AHI(ref ahi) => ahi.images[ahi.image_index].clone(),
            Data::AHF(ref ahf) => {
                Rc::new(match ahf.current_char {
                    Some(chr) => ahf.font[chr].image().clone(),
                    None => ahf.font.default_glyph().image().clone(),
                })
            }
        }
    }

    pub fn image(&mut self) -> &mut Image {
        match self.state.current.data {
            Data::AHI(ref mut ahi) => {
                Rc::make_mut(&mut ahi.images[ahi.image_index])
            }
            Data::AHF(ref mut ahf) => {
                match ahf.current_char {
                    Some(chr) => ahf.font[chr].image_mut(),
                    None => ahf.font.default_glyph_mut().image_mut(),
                }
            }
        }
    }

    fn add_new_image(&mut self, chr: char) -> bool {
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
                    let glyph = Glyph::new(Image::new(width, height),
                                           0,
                                           1 + width as i32);
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
                ahi.images = ahi.images
                                .iter()
                                .map(|image| {
                                    Rc::new(image.crop(new_width, new_height))
                                })
                                .collect();
            }
            Data::AHF(ref mut ahf) => {
                if new_height != ahf.font.glyph_height() {
                    let mut font = Font::with_glyph_height(new_height);
                    {
                        let glyph = ahf.font.default_glyph();
                        let new_glyph = Glyph::new(glyph.image()
                                                        .crop(glyph.image()
                                                                   .width(),
                                                              new_height),
                                                   glyph.left_edge(),
                                                   glyph.right_edge());
                        font.set_default_glyph(new_glyph);
                    }
                    for chr in ahf.font.chars() {
                        let glyph = &ahf.font[chr];
                        let new_glyph = Glyph::new(glyph.image()
                                                        .crop(glyph.image()
                                                                   .width(),
                                                              new_height),
                                                   glyph.left_edge(),
                                                   glyph.right_edge());
                        font.set_char_glyph(chr, new_glyph);
                    }
                    ahf.font = font;
                }
                match ahf.current_char {
                    Some(chr) => {
                        let new_glyph = {
                            let glyph = &ahf.font[chr];
                            Glyph::new(glyph.image().crop(new_width,
                                                          glyph.image()
                                                               .height()),
                                       glyph.left_edge(),
                                       glyph.right_edge())
                        };
                        ahf.font.set_char_glyph(chr, new_glyph);
                    }
                    None => {
                        let new_glyph = {
                            let glyph = ahf.font.default_glyph();
                            Glyph::new(glyph.image().crop(new_width,
                                                          glyph.image()
                                                               .height()),
                                       glyph.left_edge(),
                                       glyph.right_edge())
                        };
                        ahf.font.set_default_glyph(new_glyph);
                    }
                }
            }
        }
    }

    fn set_metrics(&mut self,
                   new_baseline: i32,
                   new_left_edge: i32,
                   new_right_edge: i32) {
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

    pub fn select(&mut self, rect: &Rect) {
        self.unselect();
        let mut selected = Image::new(rect.width(), rect.height());
        selected.draw(self.image(), -rect.x(), -rect.y());
        self.state.current.selection = Some((Rc::new(selected),
                                             rect.top_left()));
        self.image().fill_rect(rect.x(),
                               rect.y(),
                               rect.width(),
                               rect.height(),
                               Color::Transparent);
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

// ========================================================================= //

const DEFAULT_TEST_SENTENCE: &'static str = "The quick, brown fox jumps over \
                                             a ``lazy'' dog.";

const MAX_UNDOS: usize = 100;

// ========================================================================= //
