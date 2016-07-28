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

use ahi::{Color, Image};
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
    Eyedropper,
    Line,
    PaintBucket,
    Pencil,
    Select,
}

#[derive(Clone, Eq, PartialEq)]
pub enum Mode {
    Edit,
    LoadFile(String),
    Resize(String),
    SaveAs(String),
}

// ========================================================================= //

const MAX_UNDOS: usize = 100;

#[derive(Clone)]
struct Snapshot {
    image_index: usize,
    images: Vec<Rc<Image>>,
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
                image_index: 0,
                images: images.drain(..).map(Rc::new).collect(),
                selection: None,
                unsaved: false,
            },
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            clipboard: None,
            tool: Tool::Pencil,
            prev_tool: Tool::Pencil,
            persistent_mutation_active: false,
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

    pub fn eyedrop_at(&mut self, position: (u32, u32)) {
        self.color = self.image()[position];
        if self.tool == Tool::Eyedropper {
            self.tool = self.prev_tool;
        }

    }

    pub fn num_images(&self) -> usize {
        self.current.images.len()
    }

    pub fn image_index(&self) -> usize {
        self.current.image_index
    }

    pub fn set_image_index(&mut self, index: usize) {
        self.unselect_if_necessary();
        self.current.image_index = index % self.current.images.len();
    }

    pub fn image_size(&self) -> (u32, u32) {
        let image = self.image();
        (image.width(), image.height())
    }

    pub fn image(&self) -> &Image {
        &self.current.images[self.current.image_index]
    }

    pub fn image_at(&self, index: usize) -> &Image {
        &self.current.images[index]
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
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some(mut snapshot) = self.redo_stack.pop() {
            mem::swap(&mut snapshot, &mut self.current);
            self.undo_stack.push(snapshot);
            true
        } else {
            false
        }
    }

    pub fn save_to_file(&mut self) -> io::Result<()> {
        self.unselect_if_necessary();
        let mut file = try!(File::create(&self.filepath));
        let images: Vec<Image> = self.current
                                     .images
                                     .iter()
                                     .map(|rc| rc.deref().clone())
                                     .collect();
        try!(Image::write_all(&mut file, &images));
        self.current.unsaved = false;
        for snapshot in self.undo_stack.iter_mut() {
            snapshot.unsaved = true;
        }
        for snapshot in self.redo_stack.iter_mut() {
            snapshot.unsaved = true;
        }
        Ok(())
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
            Mode::LoadFile(path) => {
                match util::load_ahi_from_file(&path) {
                    Ok(mut images) => {
                        self.mode = Mode::Edit;
                        self.filepath = path;
                        self.current = Snapshot {
                            image_index: 0,
                            images: images.drain(..).map(Rc::new).collect(),
                            selection: None,
                            unsaved: false,
                        };
                        self.undo_stack.clear();
                        self.redo_stack.clear();
                        self.persistent_mutation_active = false;
                        true
                    }
                    Err(_) => false,
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
        }
    }
}

// ========================================================================= //

pub struct Mutation<'a> {
    state: &'a mut EditorState,
}

impl<'a> Mutation<'a> {
    fn image_rc(&self) -> Rc<Image> {
        self.state.current.images[self.state.current.image_index].clone()
    }

    pub fn image(&mut self) -> &mut Image {
        Rc::make_mut(&mut self.state.current.images[self.state
                                                        .current
                                                        .image_index])
    }

    pub fn add_new_image(&mut self) {
        self.unselect();
        let (width, height) = self.state.image_size();
        self.state.current.image_index += 1;
        let rc = Rc::new(Image::new(width, height));
        self.state.current.images.insert(self.state.current.image_index, rc);
    }

    pub fn delete_image(&mut self) -> bool {
        if self.state.current.images.len() > 1 {
            self.unselect();
            let index = self.state.current.image_index;
            self.state.current.images.remove(index);
            if index == self.state.num_images() {
                self.state.current.image_index -= 1;
            }
            true
        } else {
            false
        }
    }

    pub fn flip_image_horz(&mut self) {
        *self.image() = self.state.image().flip_horz();
    }

    pub fn flip_image_vert(&mut self) {
        *self.image() = self.state.image().flip_vert();
    }

    pub fn resize_images(&mut self, new_width: u32, new_height: u32) {
        self.unselect();
        self.state.current.images = self.state
                                        .current
                                        .images
                                        .iter()
                                        .map(|old_image| {
                                            let mut new_image =
                                                Image::new(new_width,
                                                           new_height);
                                            new_image.draw(&old_image, 0, 0);
                                            Rc::new(new_image)
                                        })
                                        .collect();
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
