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
use super::mirror::Mirror;
use super::mutation::Mutation;
use super::settings::Settings;
use super::snapshot::Snapshot;
use super::tool::Tool;
use ahi::{Collection, Color, Font, Image, Palette};
use sdl2::rect::{Point, Rect};
use std::io;

//===========================================================================//

const MAX_UNDOS: usize = 100;

//===========================================================================//

pub struct EditorState {
    settings: Settings,
    filepath: String,
    current: Snapshot,
    undo_stack: Vec<Snapshot>,
    redo_stack: Vec<Snapshot>,
    persistent_mutation_active: bool,
}

impl EditorState {
    pub fn new(filepath: String, collection: Collection) -> EditorState {
        EditorState {
            settings: Settings::default(),
            filepath,
            current: Snapshot::with_data(Data::from_collection(collection)),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            persistent_mutation_active: false,
        }
    }

    pub fn is_unsaved(&self) -> bool {
        self.current.unsaved
    }

    pub fn filepath(&self) -> &str {
        &self.filepath
    }

    pub fn swap_filepath(&mut self, path: String) -> String {
        std::mem::replace(&mut self.filepath, path)
    }

    pub fn color(&self) -> Color {
        self.settings.color
    }

    pub fn set_color(&mut self, color: Color) {
        self.settings.color = color;
    }

    pub fn grid(&self) -> (u32, u32) {
        self.settings.grid
    }

    pub fn set_grid(&mut self, horz: u32, vert: u32) {
        self.settings.grid = (horz, vert);
    }

    pub fn zoom(&self) -> (u32, u32) {
        self.current.zoom
    }

    pub fn set_zoom(&mut self, new_zoom: (u32, u32)) {
        self.current.zoom = new_zoom;
    }

    pub fn tool(&self) -> Tool {
        self.settings.tool
    }

    pub fn set_tool(&mut self, tool: Tool) {
        if self.settings.tool != tool {
            self.unselect_if_necessary();
            self.settings.prev_tool = self.settings.tool;
            self.settings.tool = tool;
        }
    }

    pub fn mirror(&self) -> Mirror {
        self.settings.mirror
    }

    pub fn set_mirror(&mut self, mirror: Mirror) {
        self.settings.mirror = mirror;
    }

    pub fn mirror_positions(&self, (x, y): (u32, u32)) -> Vec<(u32, u32)> {
        self.mirror().positions(self.image_size(), (x, y))
    }

    pub fn allows_mixed_sizes(&self) -> bool {
        self.current.data.allows_mixed_sizes()
    }

    pub fn test_sentence(&self) -> &str {
        &self.settings.test_sentence
    }

    pub fn set_test_sentence(&mut self, text: String) {
        self.settings.test_sentence = text;
    }

    pub fn eyedrop_at(&mut self, position: (u32, u32)) {
        self.settings.color = self.image()[position];
        if self.settings.tool == Tool::Eyedropper {
            self.settings.tool = if self.settings.prev_tool == Tool::Select {
                Tool::Pencil
            } else {
                self.settings.prev_tool
            };
        }
    }

    pub fn num_palettes(&self) -> usize {
        self.current.data.num_palettes()
    }

    pub fn palette_index(&self) -> usize {
        self.current.data.palette_index()
    }

    pub fn set_palette_index(&mut self, index: usize) {
        self.current.data.set_palette_index(index);
    }

    pub fn palette(&self) -> &Palette {
        self.current.data.palette()
    }

    pub fn num_images(&self) -> usize {
        self.current.data.num_images()
    }

    pub fn image_index(&self) -> usize {
        self.current.data.image_index()
    }

    pub fn set_image_index(&mut self, index: usize) {
        self.unselect_if_necessary();
        self.current.data.set_image_index(index);
    }

    pub fn go_to(&mut self, name: &str) -> bool {
        self.unselect_if_necessary();
        self.current.data.go_to_image(name)
    }

    pub fn image_name(&self) -> String {
        self.current.data.image_name()
    }

    pub fn glyph_metrics(&self) -> Option<(i32, i32, i32)> {
        self.current.data.glyph_metrics()
    }

    pub fn image_size(&self) -> (u32, u32) {
        self.current.data.image_size()
    }

    pub fn image(&self) -> &Image {
        self.current.data.image()
    }

    pub fn image_at(&self, index: usize) -> &Image {
        self.current.data.image_at(index)
    }

    pub fn font(&self) -> Option<&Font> {
        self.current.data.font()
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
        Mutation { settings: &mut self.settings, snapshot: &mut self.current }
    }

    pub fn persistent_mutation<'a>(&'a mut self) -> Mutation<'a> {
        if !self.persistent_mutation_active {
            self.push_change();
            self.persistent_mutation_active = true;
        }
        self.current.unsaved = true;
        Mutation { settings: &mut self.settings, snapshot: &mut self.current }
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
            std::mem::swap(&mut snapshot, &mut self.current);
            self.redo_stack.push(snapshot);
            if self.current.selection.is_some() {
                self.settings.tool = Tool::Select;
            }
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some(mut snapshot) = self.redo_stack.pop() {
            std::mem::swap(&mut snapshot, &mut self.current);
            self.undo_stack.push(snapshot);
            if self.current.selection.is_some() {
                self.settings.tool = Tool::Select;
            }
            true
        } else {
            false
        }
    }

    pub fn save_to_file(&mut self) -> io::Result<()> {
        self.unselect_if_necessary();
        self.current.data.save_to_file(&self.filepath)?;
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
        self.load_data(path, Data::from_font(font));
    }

    fn load_data(&mut self, path: String, data: Data) {
        self.filepath = path;
        self.current = Snapshot::with_data(data);
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.persistent_mutation_active = false;
    }
}

//===========================================================================//
