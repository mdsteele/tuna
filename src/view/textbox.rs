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
use crate::state::{EditorState, Mode};
use sdl2::rect::Rect;
use std::cmp;
use std::ffi::OsStr;
use std::io;
use std::path::{Path, PathBuf};

//===========================================================================//

struct TextBox {}

impl TextBox {
    pub fn new() -> TextBox {
        TextBox {}
    }
}

impl GuiElement<String> for TextBox {
    fn draw(&self, text: &String, resources: &Resources, canvas: &mut Canvas) {
        let rect = canvas.rect();
        let rect_width = rect.width() as i32;
        let font = resources.font();
        let text_width = font.text_width(text);
        let text_left = cmp::min(4, rect_width - 4 - text_width);
        canvas.fill_rect((128, 128, 128, 255), rect);
        canvas.draw_string(font, text_left, 4, text);
        canvas.draw_rect((255, 255, 255, 255), rect);
    }

    fn handle_event(&mut self, event: &Event, text: &mut String) -> Action {
        match event {
            &Event::KeyDown(Keycode::Backspace, _) => {
                Action::redraw_if(text.pop().is_some()).and_stop()
            }
            &Event::KeyDown(Keycode::Tab, _) => {
                match tab_complete_path(Path::new(&text)) {
                    Ok(path) => match path.into_os_string().into_string() {
                        Ok(string) => {
                            *text = string;
                            Action::redraw().and_stop()
                        }
                        Err(_) => Action::ignore().and_stop(),
                    },
                    Err(_) => Action::ignore().and_stop(),
                }
            }
            &Event::KeyDown(_, _) => Action::ignore().and_stop(),
            &Event::TextInput(ref input) => {
                text.push_str(input);
                Action::redraw().and_stop()
            }
            _ => Action::ignore().and_continue(),
        }
    }
}

fn tab_complete_path(path: &Path) -> io::Result<PathBuf> {
    let dir: &Path = if path.is_dir() {
        path
    } else {
        path.parent().ok_or_else(|| io::Error::new(io::ErrorKind::Other, ""))?
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
    element: SubrectElement<TextBox>,
}

impl ModalTextBox {
    pub fn new(left: i32, top: i32) -> ModalTextBox {
        ModalTextBox {
            left,
            top,
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
}

impl GuiElement<EditorState> for ModalTextBox {
    fn draw(
        &self,
        state: &EditorState,
        resources: &Resources,
        canvas: &mut Canvas,
    ) {
        let label = match *state.mode() {
            Mode::Edit => {
                self.element.draw(state.filepath(), resources, canvas);
                "Path:"
            }
            Mode::Goto(ref text) => {
                self.element.draw(text, resources, canvas);
                "Goto:"
            }
            Mode::LoadFile(ref text) => {
                self.element.draw(text, resources, canvas);
                "Load:"
            }
            Mode::NewGlyph(ref text) => {
                self.element.draw(text, resources, canvas);
                "Char:"
            }
            Mode::Resize(ref text) => {
                self.element.draw(text, resources, canvas);
                "Size:"
            }
            Mode::SaveAs(ref text) => {
                self.element.draw(text, resources, canvas);
                "Save:"
            }
            Mode::SetMetadata(ref text) => {
                self.element.draw(text, resources, canvas);
                "Meta:"
            }
            Mode::SetMetrics(ref text) => {
                self.element.draw(text, resources, canvas);
                "Metrics:"
            }
            Mode::SetTag(ref text) => {
                self.element.draw(text, resources, canvas);
                "Tag:"
            }
            Mode::TestSentence => {
                self.element.draw(state.test_sentence(), resources, canvas);
                "Text:"
            }
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

    fn handle_event(
        &mut self,
        event: &Event,
        state: &mut EditorState,
    ) -> Action {
        match event {
            &Event::KeyDown(Keycode::Escape, _) => {
                if state.mode_cancel() {
                    Action::redraw().and_stop()
                } else {
                    Action::ignore().and_continue()
                }
            }
            &Event::KeyDown(Keycode::Return, _) => {
                if state.mode_perform() {
                    Action::redraw().and_stop()
                } else {
                    Action::ignore().and_continue()
                }
            }
            _ => {
                match *state.mode_mut() {
                    Mode::Edit => return Action::ignore().and_continue(),
                    Mode::Goto(ref mut text)
                    | Mode::LoadFile(ref mut text)
                    | Mode::NewGlyph(ref mut text)
                    | Mode::Resize(ref mut text)
                    | Mode::SaveAs(ref mut text)
                    | Mode::SetMetadata(ref mut text)
                    | Mode::SetMetrics(ref mut text)
                    | Mode::SetTag(ref mut text) => {
                        return self.element.handle_event(event, text)
                    }
                    Mode::TestSentence => {}
                }
                self.element.handle_event(event, state.test_sentence_mut())
            }
        }
    }
}

//===========================================================================//
