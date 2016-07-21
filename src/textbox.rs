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

use sdl2::rect::{Point, Rect};
use std::rc::Rc;
use super::canvas::{Canvas, Sprite};
use super::element::{Action, GuiElement, SubrectElement};
use super::event::{Event, Keycode};
use super::state::{EditorState, Mode};

// ========================================================================= //

pub struct TextBox {
    font: Rc<Vec<Sprite>>,
}

impl TextBox {
    pub fn new(font: Rc<Vec<Sprite>>) -> TextBox {
        TextBox { font: font }
    }
}

impl GuiElement<String> for TextBox {
    fn draw(&self, text: &String, canvas: &mut Canvas) {
        let rect = canvas.rect();
        render_string(canvas, &self.font, 2, 2, text);
        canvas.draw_rect((255, 255, 255, 255), rect);
    }

    fn handle_event(&mut self, event: &Event, text: &mut String) -> Action {
        match event {
            &Event::KeyDown(Keycode::Backspace, _) => {
                Action::redraw_if(text.pop().is_some()).and_stop()
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

// ========================================================================= //

pub struct ModalTextBox {
    left: i32,
    top: i32,
    font: Rc<Vec<Sprite>>,
    element: SubrectElement<TextBox>,
}

impl ModalTextBox {
    pub fn new(left: i32, top: i32, font: Rc<Vec<Sprite>>) -> ModalTextBox {
        ModalTextBox {
            left: left,
            top: top,
            font: font.clone(),
            element: SubrectElement::new(TextBox::new(font),
                                         Rect::new(left + 70, top, 404, 20)),
        }
    }
}

impl GuiElement<EditorState> for ModalTextBox {
    fn draw(&self, state: &EditorState, canvas: &mut Canvas) {
        let label = match *state.mode() {
            Mode::Edit => {
                self.element.draw(state.filepath(), canvas);
                "Path:"
            }
            Mode::LoadFile(ref text) => {
                self.element.draw(text, canvas);
                "Load:"
            }
            Mode::Resize(ref text) => {
                self.element.draw(text, canvas);
                "Size:"
            }
            Mode::SaveAs(ref text) => {
                self.element.draw(text, canvas);
                "Save:"
            }
        };
        render_string(canvas, &self.font, self.left, self.top + 2, label);
    }

    fn handle_event(&mut self,
                    event: &Event,
                    state: &mut EditorState)
                    -> Action {
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
                    Mode::Edit => Action::ignore().and_continue(),
                    Mode::LoadFile(ref mut text) |
                    Mode::Resize(ref mut text) |
                    Mode::SaveAs(ref mut text) => {
                        self.element.handle_event(event, text)
                    }
                }
            }
        }
    }
}

// ========================================================================= //

fn render_string(canvas: &mut Canvas,
                 font: &Vec<Sprite>,
                 left: i32,
                 top: i32,
                 string: &str) {
    let mut x = left;
    let mut y = top;
    for ch in string.chars() {
        if ch == '\n' {
            x = left;
            y += 24;
        } else {
            if ch >= '!' {
                let index = ch as usize - '!' as usize;
                if index < font.len() {
                    canvas.draw_sprite(&font[index], Point::new(x, y));
                }
            }
            x += 14;
        }
    }
}

// ========================================================================= //
