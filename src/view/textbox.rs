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

use crate::canvas::{Canvas, Resources};
use crate::element::{Action, GuiElement, SubrectElement};
use crate::event::{Event, Keycode};
use crate::state::EditorState;
use sdl2::rect::Rect;
use std::cmp;
use std::ffi::OsStr;
use std::io;
use std::path::{Path, PathBuf};

//===========================================================================//

const CURSOR_ON_FRAMES: u32 = 3;
const CURSOR_OFF_FRAMES: u32 = 3;

//===========================================================================//

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Mode {
    Edit,
    Goto,
    LoadFile,
    NewGlyph,
    Resize,
    SaveAs,
    SetMetadata,
    SetMetrics,
    SetTag,
    TestSentence,
}

//===========================================================================//

struct TextBox {
    byte_index: usize,
    cursor_blink: u32,
    text: String,
}

impl TextBox {
    pub fn new() -> TextBox {
        TextBox { byte_index: 0, cursor_blink: 0, text: String::new() }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, text: String) {
        self.byte_index = text.len();
        self.text = text;
    }
}

impl GuiElement<(), ()> for TextBox {
    fn draw(&self, _: &(), resources: &Resources, canvas: &mut Canvas) {
        let rect = canvas.rect();
        let rect_width = rect.width() as i32;
        let font = resources.font();
        let text_width = font.text_width(&self.text);
        let text_left = cmp::min(4, rect_width - 4 - text_width);
        canvas.fill_rect((128, 128, 128, 255), rect);
        canvas.draw_string(font, text_left, 4, &self.text);
        canvas.draw_rect((255, 255, 255, 255), rect);
        if self.cursor_blink < CURSOR_ON_FRAMES {
            let cursor_x =
                text_left + font.text_width(&self.text[..self.byte_index]);
            let cursor_rect =
                Rect::new(cursor_x, rect.y() + 3, 1, rect.height() - 6);
            canvas.fill_rect((255, 255, 0, 255), cursor_rect);
        }
    }

    fn on_event(&mut self, event: &Event, _: &mut ()) -> Action<()> {
        match event {
            &Event::ClockTick => {
                let was_on = self.cursor_blink < CURSOR_ON_FRAMES;
                self.cursor_blink = (self.cursor_blink + 1)
                    % (CURSOR_ON_FRAMES + CURSOR_OFF_FRAMES);
                let is_on = self.cursor_blink < CURSOR_ON_FRAMES;
                Action::redraw_if(was_on != is_on)
            }
            &Event::KeyDown(Keycode::Backspace, _) => {
                if self.byte_index > 0 {
                    let rest = self.text.split_off(self.byte_index);
                    self.text.pop();
                    self.byte_index = self.text.len();
                    self.text.push_str(&rest);
                    self.cursor_blink = 0;
                    Action::redraw().and_stop()
                } else {
                    Action::ignore()
                }
            }
            &Event::KeyDown(Keycode::Tab, _) => {
                match tab_complete_path(Path::new(&self.text)) {
                    Ok(path) => match path.into_os_string().into_string() {
                        Ok(string) => {
                            self.text = string;
                            self.byte_index = self.text.len();
                            self.cursor_blink = 0;
                            Action::redraw().and_stop()
                        }
                        Err(_) => Action::ignore().and_stop(),
                    },
                    Err(_) => Action::ignore().and_stop(),
                }
            }
            &Event::KeyDown(Keycode::Up, _) => {
                if self.byte_index > 0 {
                    self.byte_index = 0;
                    self.cursor_blink = 0;
                    Action::redraw().and_stop()
                } else {
                    Action::ignore()
                }
            }
            &Event::KeyDown(Keycode::Down, _) => {
                if self.byte_index < self.text.len() {
                    self.byte_index = self.text.len();
                    self.cursor_blink = 0;
                    Action::redraw().and_stop()
                } else {
                    Action::ignore()
                }
            }
            &Event::KeyDown(Keycode::Left, _) => {
                if self.byte_index > 0 {
                    let mut new_byte_index = self.byte_index - 1;
                    while !self.text.is_char_boundary(new_byte_index) {
                        new_byte_index -= 1;
                    }
                    self.byte_index = new_byte_index;
                    self.cursor_blink = 0;
                    Action::redraw().and_stop()
                } else {
                    Action::ignore()
                }
            }
            &Event::KeyDown(Keycode::Right, _) => {
                if self.byte_index < self.text.len() {
                    let mut new_byte_index = self.byte_index + 1;
                    while !self.text.is_char_boundary(new_byte_index) {
                        new_byte_index += 1;
                    }
                    self.byte_index = new_byte_index;
                    self.cursor_blink = 0;
                    Action::redraw().and_stop()
                } else {
                    Action::ignore()
                }
            }
            &Event::KeyDown(_, _) => Action::ignore().and_stop(),
            &Event::TextInput(ref input) => {
                self.text.insert_str(self.byte_index, input);
                self.byte_index += input.len();
                self.cursor_blink = 0;
                Action::redraw().and_stop()
            }
            _ => Action::ignore(),
        }
    }
}

fn tab_complete_path(path: &Path) -> io::Result<PathBuf> {
    let dir: &Path = if path.is_dir() {
        path
    } else {
        path.parent()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, ""))?
    };
    let prefix: &str =
        path.file_name().map(OsStr::to_str).unwrap_or(None).unwrap_or("");

    let mut file_names = Vec::<String>::new();
    for entry_result in dir.read_dir()? {
        let entry = entry_result?;
        let file_name = entry.file_name().to_str().unwrap_or("").to_string();
        if file_name.starts_with(prefix) {
            file_names.push(file_name);
        }
    }

    if let Some(first) = file_names.pop() {
        let mut completed = String::new();
        for chr in first.chars() {
            let mut candidate = completed.clone();
            candidate.push(chr);
            if !file_names.iter().all(|name| name.starts_with(&candidate)) {
                break;
            }
            completed = candidate;
        }
        let mut completed_path = dir.join(completed);
        if completed_path.is_dir() {
            completed_path.push("");
        }
        Ok(completed_path)
    } else {
        Err(io::Error::new(io::ErrorKind::Other, ""))
    }
}

//===========================================================================//

const LABEL_WIDTH: i32 = 50;

pub struct ModalTextBox {
    left: i32,
    top: i32,
    mode: Mode,
    element: SubrectElement<TextBox>,
}

impl ModalTextBox {
    pub fn new(left: i32, top: i32) -> ModalTextBox {
        ModalTextBox {
            left,
            top,
            mode: Mode::Edit,
            element: SubrectElement::new(
                TextBox::new(),
                Rect::new(
                    left + LABEL_WIDTH,
                    top,
                    (440 - LABEL_WIDTH) as u32,
                    18,
                ),
            ),
        }
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: Mode, text: String) {
        self.mode = mode;
        self.element.inner_mut().set_text(text);
    }

    pub fn clear_mode(&mut self) {
        self.mode = Mode::Edit;
        self.element.inner_mut().set_text(String::new());
    }
}

impl GuiElement<EditorState, (Mode, String)> for ModalTextBox {
    fn draw(
        &self,
        state: &EditorState,
        resources: &Resources,
        canvas: &mut Canvas,
    ) {
        if self.mode == Mode::Edit {
            let font = resources.font();
            canvas.draw_string(
                font,
                self.left + LABEL_WIDTH + 4,
                self.top + 4,
                state.filepath(),
            );
        } else {
            self.element.draw(&(), resources, canvas);
        }
        let label = match self.mode {
            Mode::Edit => "Path:",
            Mode::Goto => "Goto:",
            Mode::LoadFile => "Load:",
            Mode::NewGlyph => "Char:",
            Mode::Resize => "Size:",
            Mode::SaveAs => "Save:",
            Mode::SetMetadata => "Meta:",
            Mode::SetMetrics => "Metrics:",
            Mode::SetTag => "Tag:",
            Mode::TestSentence => "Text:",
        };
        let font = resources.font();
        let text_width = font.text_width(label);
        canvas.draw_string(
            font,
            self.left + LABEL_WIDTH - text_width - 2,
            self.top + 4,
            label,
        );
    }

    fn on_event(
        &mut self,
        event: &Event,
        _: &mut EditorState,
    ) -> Action<(Mode, String)> {
        if self.mode == Mode::Edit {
            Action::ignore()
        } else {
            match event {
                &Event::KeyDown(Keycode::Escape, _) => {
                    self.clear_mode();
                    Action::redraw().and_stop()
                }
                &Event::KeyDown(Keycode::Return, _) => Action::redraw()
                    .and_return((
                        self.mode,
                        self.element.inner().text().to_string(),
                    )),
                _ => self
                    .element
                    .on_event(event, &mut ())
                    .but_no_value()
                    .and_stop(),
            }
        }
    }
}

//===========================================================================//
