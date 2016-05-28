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
use sdl2::rect::Rect;
use std::fs::File;
use std::io;
use std::mem;

// ========================================================================= //

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Tool {
    Eyedropper,
    PaintBucket,
    Pencil,
    Select,
}

const MAX_UNDOS: usize = 100;

enum Undo {
    AddImage(usize),
    ChangeImage(usize, Image),
    RemoveImage(usize, Image),
    SelectionBegin,
    SelectionCut(Image, i32, i32),
    SelectionEnd(Rect),
    SelectionMove(i32, i32),
    SelectionPaste,
}

enum Redo {
    AddImage(usize),
    ChangeImage(usize, Image),
    RemoveImage(usize),
    SelectionBegin(Rect),
    SelectionCut,
    SelectionEnd,
    SelectionMove(i32, i32),
    SelectionPaste(Image, i32, i32),
}

pub struct EditorState {
    pub color: Color,
    pub filepath: String,
    pub images: Vec<Image>,
    pub current_image: usize,
    pub selection: Option<(Image, i32, i32)>,
    pub clipboard: Option<(Image, i32, i32)>,
    tool: Tool,
    prev_tool: Tool,
    undo_stack: Vec<Undo>,
    redo_stack: Vec<Redo>,
    unsaved: bool,
}

impl EditorState {
    pub fn new(filepath: String, mut images: Vec<Image>) -> EditorState {
        if images.is_empty() {
            images.push(Image::new(32, 32));
        }
        EditorState {
            color: Color::Black,
            filepath: filepath,
            images: images,
            current_image: 0,
            selection: None,
            clipboard: None,
            tool: Tool::Pencil,
            prev_tool: Tool::Pencil,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            unsaved: false,
        }
    }

    pub fn is_unsaved(&self) -> bool {
        self.unsaved
    }

    pub fn tool(&self) -> Tool {
        self.tool
    }

    pub fn set_tool(&mut self, tool: Tool) {
        if self.tool != tool {
            self.unselect();
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

    pub fn image_size(&self) -> (u32, u32) {
        let image = self.image();
        (image.width(), image.height())
    }

    pub fn image(&self) -> &Image {
        &self.images[self.current_image]
    }

    pub fn image_mut(&mut self) -> &mut Image {
        self.unsaved = true;
        &mut self.images[self.current_image]
    }

    pub fn image_at(&self, index: usize) -> &Image {
        &self.images[index]
    }

    pub fn flip_image_horz(&mut self) {
        self.push_change();
        *self.image_mut() = self.image().flip_horz();
    }

    pub fn flip_image_vert(&mut self) {
        self.push_change();
        *self.image_mut() = self.image().flip_vert();
    }

    pub fn add_new_image(&mut self) {
        self.unselect();
        let (width, height) = self.image_size();
        self.current_image += 1;
        self.images.insert(self.current_image, Image::new(width, height));
        let undo = Undo::AddImage(self.current_image);
        self.push_undo(undo);
        self.unsaved = true;
    }

    pub fn try_delete_image(&mut self) -> bool {
        if self.images.len() > 1 {
            self.unselect();
            let image = self.images.remove(self.current_image);
            let undo = Undo::RemoveImage(self.current_image, image);
            self.push_undo(undo);
            if self.current_image == self.images.len() {
                self.current_image -= 1;
            }
            self.unsaved = true;
            true
        } else {
            false
        }
    }

    pub fn select_with_undo(&mut self, rect: &Rect) {
        self.select(rect);
        self.push_undo(Undo::SelectionBegin);
        self.tool = Tool::Select;
    }

    pub fn select_all_with_undo(&mut self) {
        let (width, height) = self.image_size();
        self.select_with_undo(&Rect::new(0, 0, width, height));
    }

    pub fn try_unselect_with_undo(&mut self) -> bool {
        if let Some(rect) = self.unselect() {
            self.push_undo(Undo::SelectionEnd(rect));
            true
        } else {
            false
        }
    }

    pub fn select(&mut self, rect: &Rect) {
        self.unselect();
        let mut selected = Image::new(rect.width(), rect.height());
        selected.draw(self.image(), -rect.x(), -rect.y());
        self.selection = Some((selected, rect.x(), rect.y()));
        self.image_mut().fill_rect(rect.x(),
                                   rect.y(),
                                   rect.width(),
                                   rect.height(),
                                   Color::Transparent);
    }

    pub fn unselect(&mut self) -> Option<Rect> {
        if let Some((selected, x, y)) = self.selection.take() {
            self.image_mut().draw(&selected, x, y);
            Some(Rect::new(x, y, selected.width(), selected.height()))
        } else {
            None
        }
    }

    pub fn cut_selection(&mut self) {
        if let Some((selected, x, y)) = self.selection.take() {
            self.push_undo(Undo::SelectionCut(selected.clone(), x, y));
            self.clipboard = Some((selected, x, y));
        }
    }

    pub fn copy_selection(&mut self) {
        if self.selection.is_some() {
            self.clipboard = self.selection.clone();
        } else {
            self.clipboard = Some((self.image().clone(), 0, 0));
        }
    }

    pub fn paste_selection(&mut self) {
        self.unselect();
        if self.clipboard.is_some() {
            self.selection = self.clipboard.clone();
            self.push_undo(Undo::SelectionPaste);
            self.tool = Tool::Select;
        }
    }

    fn push_undo(&mut self, undo: Undo) {
        self.undo_stack.push(undo);
        self.redo_stack.clear();
        if self.undo_stack.len() > MAX_UNDOS {
            self.undo_stack.remove(0);
        }
    }

    pub fn push_change(&mut self) {
        let image = self.image().clone();
        let undo = Undo::ChangeImage(self.current_image, image);
        self.push_undo(undo);
    }

    pub fn push_selection_move(&mut self) {
        let &(_, x, y) = self.selection.as_ref().unwrap();
        self.push_undo(Undo::SelectionMove(x, y));
    }

    pub fn undo(&mut self) -> bool {
        if let Some(undo) = self.undo_stack.pop() {
            match undo {
                Undo::AddImage(index) => {
                    self.images.remove(index);
                    self.redo_stack.push(Redo::AddImage(index));
                }
                Undo::ChangeImage(index, mut image) => {
                    mem::swap(&mut image, &mut self.images[index]);
                    self.redo_stack.push(Redo::ChangeImage(index, image));
                }
                Undo::RemoveImage(index, image) => {
                    self.images.insert(index, image);
                    self.redo_stack.push(Redo::RemoveImage(index));
                }
                Undo::SelectionBegin => {
                    let rect = {
                        let &(ref image, x, y) = self.selection
                                                     .as_ref()
                                                     .unwrap();
                        Rect::new(x, y, image.width(), image.height())
                    };
                    self.unselect();
                    self.redo_stack.push(Redo::SelectionBegin(rect));
                }
                Undo::SelectionCut(image, x, y) => {
                    self.selection = Some((image, x, y));
                    self.redo_stack.push(Redo::SelectionCut);
                }
                Undo::SelectionEnd(rect) => {
                    self.select(&rect);
                    self.redo_stack.push(Redo::SelectionEnd);
                }
                Undo::SelectionMove(old_x, old_y) => {
                    let (new_x, new_y) = {
                        let &mut (_, ref mut x, ref mut y) = self.selection
                                                                 .as_mut()
                                                                 .unwrap();
                        let new = (*x, *y);
                        *x = old_x;
                        *y = old_y;
                        new
                    };
                    self.redo_stack.push(Redo::SelectionMove(new_x, new_y));
                }
                Undo::SelectionPaste => {
                    let (image, x, y) = self.selection.take().unwrap();
                    self.redo_stack.push(Redo::SelectionPaste(image, x, y));
                }
            }
            self.unsaved = true;
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some(redo) = self.redo_stack.pop() {
            match redo {
                Redo::AddImage(index) => {
                    let (width, height) = self.image_size();
                    self.images.insert(index, Image::new(width, height));
                    self.undo_stack.push(Undo::AddImage(index));
                }
                Redo::ChangeImage(index, mut image) => {
                    mem::swap(&mut image, &mut self.images[index]);
                    self.undo_stack.push(Undo::ChangeImage(index, image));
                }
                Redo::RemoveImage(index) => {
                    let image = self.images.remove(index);
                    self.undo_stack.push(Undo::RemoveImage(index, image));
                }
                Redo::SelectionBegin(rect) => {
                    self.select(&rect);
                    self.undo_stack.push(Undo::SelectionBegin);
                }
                Redo::SelectionCut => {
                    let (image, x, y) = self.selection.take().unwrap();
                    self.undo_stack.push(Undo::SelectionCut(image, x, y));
                }
                Redo::SelectionEnd => {
                    let rect = {
                        let &(ref image, x, y) = self.selection
                                                     .as_ref()
                                                     .unwrap();
                        Rect::new(x, y, image.width(), image.height())
                    };
                    self.unselect();
                    self.undo_stack.push(Undo::SelectionEnd(rect));
                }
                Redo::SelectionMove(new_x, new_y) => {
                    let (old_x, old_y) = {
                        let &mut (_, ref mut x, ref mut y) = self.selection
                                                                 .as_mut()
                                                                 .unwrap();
                        let old = (*x, *y);
                        *x = new_x;
                        *y = new_y;
                        old
                    };
                    self.undo_stack.push(Undo::SelectionMove(old_x, old_y));
                }
                Redo::SelectionPaste(image, x, y) => {
                    self.selection = Some((image, x, y));
                    self.undo_stack.push(Undo::SelectionPaste);
                }
            }
            self.unsaved = true;
            true
        } else {
            false
        }
    }

    pub fn save_to_file(&mut self) -> io::Result<()> {
        self.unselect();
        let mut file = try!(File::create(&self.filepath));
        try!(Image::write_all(&mut file, &self.images));
        self.unsaved = false;
        Ok(())
    }
}

// ========================================================================= //
