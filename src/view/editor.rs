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

use super::menu::{MenuAction, MenuView};
use super::metadata::MetadataView;
use super::mirrors::Mirrors;
use super::palette::{PaletteAction, PaletteView};
use super::scrollbar::ImagesScrollbar;
use super::textbox::{ModalTextBox, Mode};
use super::tiles::TileView;
use super::toolbox::Toolbox;
use super::unsaved::UnsavedIndicator;
use crate::canvas::{Canvas, Resources};
use crate::element::{Action, AggregateElement, GuiElement, SubrectElement};
use crate::event::{Event, Keycode, COMMAND, SHIFT};
use crate::paint::ImageCanvas;
use crate::state::EditorState;
use crate::util;
use ahi::Color;
use sdl2::rect::{Point, Rect};

//===========================================================================//

pub struct EditorView {
    aggregate: AggregateElement<EditorState, ()>,
    menu: MenuView,
    palette: PaletteView,
    textbox: ModalTextBox,
}

impl EditorView {
    pub const WIDTH: u32 = 480;
    pub const HEIGHT: u32 = 320;

    pub fn new(offset: Point) -> SubrectElement<EditorView> {
        let elements: Vec<Box<dyn GuiElement<EditorState, ()>>> = vec![
            Box::new(UnsavedIndicator::new(4, 11)),
            Box::new(Toolbox::new(3, 34)),
            Box::new(Mirrors::new(3, 134)),
            Box::new(ImagesScrollbar::new(440, 34)),
            Box::new(ImageCanvas::new(80, 36, 256)),
            Box::new(ImageCanvas::new(348, 36, 64)),
            Box::new(TileView::new(341, 126, 96, 96)),
            Box::new(MetadataView::new(348, 230)),
        ];
        SubrectElement::new(
            EditorView {
                aggregate: AggregateElement::new(elements),
                menu: MenuView::new(8, 297),
                palette: PaletteView::new(3, 188),
                textbox: ModalTextBox::new(20, 10),
            },
            Rect::new(
                offset.x(),
                offset.y(),
                EditorView::WIDTH,
                EditorView::HEIGHT,
            ),
        )
    }

    fn set_textbox_mode(&mut self, mode: Mode, text: String) {
        self.menu.close();
        self.textbox.set_mode(mode, text);
    }

    fn begin_export(&mut self, state: &mut EditorState) -> bool {
        if self.textbox.mode() == Mode::Edit {
            state.unselect_if_necessary();
            self.set_textbox_mode(
                Mode::Export,
                format!("{}.{}.png", state.filepath(), state.image_index()),
            );
            true
        } else {
            false
        }
    }

    fn begin_import(&mut self, state: &mut EditorState) -> bool {
        if state.font().is_some() {
            false
        } else if self.textbox.mode() == Mode::Edit {
            state.unselect_if_necessary();
            let mut dir_path = state.filepath().to_string();
            while !dir_path.is_empty() && !dir_path.ends_with("/") {
                dir_path.pop();
            }
            self.set_textbox_mode(Mode::Import, dir_path);
            true
        } else {
            false
        }
    }

    fn begin_new_image(&mut self, state: &mut EditorState) -> bool {
        if state.font().is_some() {
            if self.textbox.mode() == Mode::Edit {
                state.unselect_if_necessary();
                self.set_textbox_mode(Mode::NewGlyph, String::new());
                true
            } else {
                false
            }
        } else {
            state.mutation().add_new_image('_')
        }
    }

    fn begin_set_color(
        &mut self,
        state: &mut EditorState,
        color: Color,
    ) -> bool {
        if state.palette_index() < state.num_palettes() {
            state.unselect_if_necessary();
            let (r, g, b, a) = state.palette()[color];
            let text = format!("{:02X}{:02X}{:02X}{:02X}", r, g, b, a);
            self.set_textbox_mode(Mode::SetColor(color), text);
            true
        } else {
            false
        }
    }

    fn begin_goto(&mut self, state: &mut EditorState) -> bool {
        if self.textbox.mode() == Mode::Edit {
            state.unselect_if_necessary();
            self.set_textbox_mode(Mode::Goto, String::new());
            true
        } else {
            false
        }
    }

    fn begin_set_grid(&mut self, state: &mut EditorState) -> bool {
        if self.textbox.mode() == Mode::Edit {
            let (horz, vert) = state.grid();
            let text = format!("{}x{}", horz, vert);
            self.set_textbox_mode(Mode::SetGrid, text);
            true
        } else {
            false
        }
    }

    fn begin_load_file(&mut self, state: &mut EditorState) -> bool {
        if self.textbox.mode() == Mode::Edit {
            state.unselect_if_necessary();
            self.set_textbox_mode(
                Mode::LoadFile,
                state.filepath().to_string(),
            );
            true
        } else {
            false
        }
    }

    fn begin_resize(&mut self, state: &mut EditorState) -> bool {
        if self.textbox.mode() == Mode::Edit {
            state.unselect_if_necessary();
            self.set_textbox_mode(
                Mode::Resize,
                format!(
                    "{}x{}",
                    state.image().width(),
                    state.image().height()
                ),
            );
            true
        } else {
            false
        }
    }

    fn begin_save_as(&mut self, state: &mut EditorState) -> bool {
        if self.textbox.mode() == Mode::Edit {
            state.unselect_if_necessary();
            self.set_textbox_mode(Mode::SaveAs, state.filepath().to_string());
            true
        } else {
            false
        }
    }

    fn begin_set_metadata(&mut self, state: &mut EditorState) -> bool {
        if self.textbox.mode() == Mode::Edit {
            state.unselect_if_necessary();
            self.set_textbox_mode(
                Mode::SetMetadata,
                state
                    .image()
                    .metadata()
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(","),
            );
            true
        } else {
            false
        }
    }

    fn begin_set_metrics(&mut self, state: &mut EditorState) -> bool {
        if self.textbox.mode() == Mode::Edit {
            if let Some((bl, le, re)) = state.image_metrics() {
                state.unselect_if_necessary();
                self.set_textbox_mode(
                    Mode::SetMetrics,
                    format!("{}/{}/{}", bl, le, re),
                );
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn begin_set_tag(&mut self, state: &mut EditorState) -> bool {
        if self.textbox.mode() == Mode::Edit {
            state.unselect_if_necessary();
            self.set_textbox_mode(
                Mode::SetTag,
                state.image().tag().to_string(),
            );
            true
        } else {
            false
        }
    }

    fn begin_set_test_sentence(&mut self, state: &mut EditorState) -> bool {
        if self.textbox.mode() == Mode::Edit && state.font().is_some() {
            self.set_textbox_mode(
                Mode::TestSentence,
                state.test_sentence().to_string(),
            );
            true
        } else {
            false
        }
    }

    fn finish_mode(
        &mut self,
        state: &mut EditorState,
        mode: Mode,
        text: String,
    ) -> bool {
        match mode {
            Mode::Edit => false,
            Mode::Export => {
                match util::save_png_to_file(
                    state.image(),
                    state.palette(),
                    &text,
                ) {
                    Ok(()) => true,
                    Err(error) => {
                        println!("Error saving PNG: {}", error);
                        false
                    }
                }
            }
            Mode::Goto => state.go_to(&text),
            Mode::Import => {
                match util::load_png_from_file(state.palette(), &text) {
                    Ok(image) => state.mutation().add_images(&[image]),
                    Err(error) => {
                        println!("Error loading PNG: {}", error);
                        false
                    }
                }
            }
            Mode::LoadFile => match util::load_ahi_from_file(&text) {
                Ok(collection) => {
                    state.load_collection(text, collection);
                    true
                }
                Err(_) => match util::load_ahf_from_file(&text) {
                    Ok(font) => {
                        state.load_font(text, font);
                        true
                    }
                    Err(_) => false,
                },
            },
            Mode::NewGlyph => {
                let chars: Vec<char> = text.chars().collect();
                chars.len() == 1 && state.mutation().add_new_image(chars[0])
            }
            Mode::Resize => {
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
                state.mutation().resize_images(new_width, new_height);
                true
            }
            Mode::SaveAs => {
                let old = state.swap_filepath(text);
                match state.save_to_file() {
                    Ok(()) => true,
                    Err(_) => {
                        state.swap_filepath(old);
                        false
                    }
                }
            }
            Mode::SetColor(color) => {
                let rgba = match (text.len(), u32::from_str_radix(&text, 16)) {
                    (0, _) => (0, 0, 0, 0),
                    (1, Ok(v)) => {
                        let gray = (0x11 * v) as u8;
                        (gray, gray, gray, 255)
                    }
                    (2, Ok(v)) => {
                        let gray = v as u8;
                        (gray, gray, gray, 255)
                    }
                    (3, Ok(v)) => {
                        let r = (0x11 * (0xf & (v >> 8))) as u8;
                        let g = (0x11 * (0xf & (v >> 4))) as u8;
                        let b = (0x11 * (0xf & v)) as u8;
                        (r, g, b, 255)
                    }
                    (4, Ok(v)) => {
                        let r = (0x11 * (0xf & (v >> 12))) as u8;
                        let g = (0x11 * (0xf & (v >> 8))) as u8;
                        let b = (0x11 * (0xf & (v >> 4))) as u8;
                        let a = (0x11 * (0xf & v)) as u8;
                        (r, g, b, a)
                    }
                    (6, Ok(v)) => {
                        let r = (0xff & (v >> 16)) as u8;
                        let g = (0xff & (v >> 8)) as u8;
                        let b = (0xff & v) as u8;
                        (r, g, b, 255)
                    }
                    (8, Ok(v)) => {
                        let r = (0xff & (v >> 24)) as u8;
                        let g = (0xff & (v >> 16)) as u8;
                        let b = (0xff & (v >> 8)) as u8;
                        let a = (0xff & v) as u8;
                        (r, g, b, a)
                    }
                    _ => return false,
                };
                state.mutation().set_palette_color(color, rgba)
            }
            Mode::SetGrid => {
                let pieces: Vec<&str> = text.split('x').collect();
                if pieces.len() != 2 {
                    return false;
                }
                let new_horz = match pieces[0].parse::<u32>() {
                    Ok(horz) => horz,
                    Err(_) => return false,
                };
                let new_vert = match pieces[1].parse::<u32>() {
                    Ok(vert) => vert,
                    Err(_) => return false,
                };
                state.set_grid(new_horz, new_vert);
                true
            }
            Mode::SetMetadata => {
                let result = if text.is_empty() {
                    Ok(vec![])
                } else {
                    text.split(",").map(|s| s.parse::<i16>()).collect()
                };
                match result {
                    Ok(metadata) => {
                        state.mutation().set_metadata(metadata);
                        true
                    }
                    Err(_) => false,
                }
            }
            Mode::SetMetrics => {
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
                state.mutation().set_metrics(
                    new_baseline,
                    new_left_edge,
                    new_right_edge,
                );
                true
            }
            Mode::SetTag => {
                state.mutation().set_tag(text);
                true
            }
            Mode::TestSentence => {
                state.set_test_sentence(text);
                true
            }
        }
    }

    fn chop_col_major(&mut self, state: &mut EditorState) -> bool {
        let (grid_width, grid_height) = state.grid();
        let chop_width = if grid_width == 0 { 8 } else { grid_width };
        let chop_height = if grid_height == 0 { 8 } else { grid_height };
        let chop_cols = state.image().width() / chop_width;
        let chop_rows = state.image().height() / chop_height;
        let mut chopped = Vec::<ahi::Image>::new();
        for col in 0..chop_cols {
            let dx = (col * chop_width) as i32;
            for row in 0..chop_rows {
                let dy = (row * chop_height) as i32;
                let mut image = ahi::Image::new(chop_width, chop_height);
                image.draw(state.image(), -dx, -dy);
                chopped.push(image);
            }
        }
        state.mutation().add_images(&chopped)
    }

    fn chop_row_major(&mut self, state: &mut EditorState) -> bool {
        let (grid_width, grid_height) = state.grid();
        let chop_width = if grid_width == 0 { 8 } else { grid_width };
        let chop_height = if grid_height == 0 { 8 } else { grid_height };
        let chop_cols = state.image().width() / chop_width;
        let chop_rows = state.image().height() / chop_height;
        let mut chopped = Vec::<ahi::Image>::new();
        for row in 0..chop_rows {
            let dy = (row * chop_height) as i32;
            for col in 0..chop_cols {
                let dx = (col * chop_width) as i32;
                let mut image = ahi::Image::new(chop_width, chop_height);
                image.draw(state.image(), -dx, -dy);
                chopped.push(image);
            }
        }
        state.mutation().add_images(&chopped)
    }

    fn perform(
        &mut self,
        state: &mut EditorState,
        menu_action: MenuAction,
    ) -> Action<()> {
        let action = match menu_action {
            MenuAction::ChopColMajor => {
                self.chop_col_major(state);
                Action::redraw()
            }
            MenuAction::ChopRowMajor => {
                self.chop_row_major(state);
                Action::redraw()
            }
            MenuAction::ExportPng => {
                Action::redraw_if(self.begin_export(state))
            }
            MenuAction::FlipHorz => {
                state.mutation().flip_selection_horz();
                Action::redraw()
            }
            MenuAction::FlipVert => {
                state.mutation().flip_selection_vert();
                Action::redraw()
            }
            MenuAction::ImportPng => {
                Action::redraw_if(self.begin_import(state))
            }
            MenuAction::Resize => Action::redraw_if(self.begin_resize(state)),
            MenuAction::RotateLeft => {
                state.mutation().rotate_selection_counterclockwise();
                Action::redraw()
            }
            MenuAction::RotateRight => {
                state.mutation().rotate_selection_clockwise();
                Action::redraw()
            }
            MenuAction::Scale2x => {
                state.mutation().scale_selection_up(2);
                Action::redraw()
            }
            MenuAction::ScaleHalf => {
                state.mutation().scale_selection_down(2);
                Action::redraw()
            }
        };
        action.and_stop()
    }
}

impl GuiElement<EditorState, ()> for EditorView {
    fn draw(
        &self,
        state: &EditorState,
        resources: &Resources,
        canvas: &mut Canvas,
    ) {
        canvas.clear((64, 64, 64, 255));
        let rect = canvas.rect();
        canvas.draw_rect((127, 127, 127, 127), rect);
        self.aggregate.draw(state, resources, canvas);
        self.palette.draw(state, resources, canvas);
        self.menu.draw(state, resources, canvas);
        self.textbox.draw(state, resources, canvas);
    }

    fn on_event(
        &mut self,
        event: &Event,
        state: &mut EditorState,
    ) -> Action<()> {
        match event {
            &Event::KeyDown(Keycode::Backspace, kmod) if kmod == COMMAND => {
                Action::redraw_if(state.mutation().delete_image()).and_stop()
            }
            &Event::KeyDown(Keycode::A, kmod) if kmod == COMMAND => {
                state.mutation().select_all();
                Action::redraw().and_stop()
            }
            &Event::KeyDown(Keycode::B, kmod) if kmod == COMMAND | SHIFT => {
                Action::redraw_if(self.begin_set_metrics(state)).and_stop()
            }
            &Event::KeyDown(Keycode::C, kmod) if kmod == COMMAND => {
                state.mutation().copy_selection();
                Action::ignore().and_stop()
            }
            &Event::KeyDown(Keycode::G, kmod) if kmod == COMMAND => {
                Action::redraw_if(self.begin_goto(state)).and_stop()
            }
            &Event::KeyDown(Keycode::G, kmod) if kmod == COMMAND | SHIFT => {
                Action::redraw_if(self.begin_set_grid(state)).and_stop()
            }
            &Event::KeyDown(Keycode::H, kmod) if kmod == COMMAND | SHIFT => {
                self.perform(state, MenuAction::FlipHorz)
            }
            &Event::KeyDown(Keycode::L, kmod) if kmod == COMMAND | SHIFT => {
                self.perform(state, MenuAction::RotateLeft)
            }
            &Event::KeyDown(Keycode::M, kmod) if kmod == COMMAND | SHIFT => {
                Action::redraw_if(self.begin_set_metadata(state)).and_stop()
            }
            &Event::KeyDown(Keycode::N, kmod) if kmod == COMMAND => {
                Action::redraw_if(self.begin_new_image(state)).and_stop()
            }
            &Event::KeyDown(Keycode::O, kmod) if kmod == COMMAND => {
                Action::redraw_if(self.begin_load_file(state)).and_stop()
            }
            &Event::KeyDown(Keycode::R, kmod) if kmod == COMMAND => {
                self.perform(state, MenuAction::Resize)
            }
            &Event::KeyDown(Keycode::R, kmod) if kmod == COMMAND | SHIFT => {
                self.perform(state, MenuAction::RotateRight)
            }
            &Event::KeyDown(Keycode::S, kmod) if kmod == COMMAND => {
                state.save_to_file().unwrap();
                Action::redraw().and_stop()
            }
            &Event::KeyDown(Keycode::S, kmod) if kmod == COMMAND | SHIFT => {
                Action::redraw_if(self.begin_save_as(state)).and_stop()
            }
            &Event::KeyDown(Keycode::T, kmod) if kmod == COMMAND => {
                Action::redraw_if(self.begin_set_tag(state)).and_stop()
            }
            &Event::KeyDown(Keycode::T, kmod) if kmod == COMMAND | SHIFT => {
                Action::redraw_if(self.begin_set_test_sentence(state))
                    .and_stop()
            }
            &Event::KeyDown(Keycode::V, kmod) if kmod == COMMAND => {
                state.mutation().paste_selection();
                Action::redraw().and_stop()
            }
            &Event::KeyDown(Keycode::V, kmod) if kmod == COMMAND | SHIFT => {
                self.perform(state, MenuAction::FlipVert)
            }
            &Event::KeyDown(Keycode::X, kmod) if kmod == COMMAND => {
                state.mutation().cut_selection();
                Action::redraw().and_stop()
            }
            &Event::KeyDown(Keycode::Z, kmod) if kmod == COMMAND => {
                Action::redraw_if(state.undo()).and_stop()
            }
            &Event::KeyDown(Keycode::Z, kmod) if kmod == COMMAND | SHIFT => {
                Action::redraw_if(state.redo()).and_stop()
            }
            &Event::KeyDown(Keycode::Num2, kmod) if kmod == COMMAND => {
                state.mutation().scale_selection_up(2);
                Action::redraw().and_stop()
            }
            &Event::KeyDown(Keycode::Num3, kmod) if kmod == COMMAND => {
                state.mutation().scale_selection_up(3);
                Action::redraw().and_stop()
            }
            _ => {
                let mut action = Action::ignore();
                {
                    let mut subaction = self.textbox.on_event(event, state);
                    if let Some((mode, text)) = subaction.take_value() {
                        if self.finish_mode(state, mode, text) {
                            self.textbox.clear_mode();
                            subaction.also_redraw();
                        }
                    }
                    action.merge(subaction.but_no_value());
                }
                if !action.should_stop() {
                    let mut subaction = self.menu.on_event(event, state);
                    if let Some(menu_action) = subaction.take_value() {
                        subaction.merge(
                            self.perform(state, menu_action).but_no_value(),
                        );
                    }
                    action.merge(subaction.but_no_value());
                }
                if !action.should_stop() {
                    let mut subaction = self.palette.on_event(event, state);
                    match subaction.take_value() {
                        Some(PaletteAction::EditColor(color)) => {
                            if self.begin_set_color(state, color) {
                                subaction.also_redraw();
                            }
                        }
                        None => {}
                    }
                    action.merge(subaction.but_no_value());
                }
                if !action.should_stop() {
                    action.merge(self.aggregate.on_event(event, state));
                }
                action
            }
        }
    }
}

//===========================================================================//
