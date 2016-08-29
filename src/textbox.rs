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

use sdl2::rect::Rect;
use std::cmp;
use std::rc::Rc;
use super::canvas::{Canvas, Font};
use super::element::{Action, GuiElement, SubrectElement};
use super::event::{Event, Keycode};
use super::state::{EditorState, Mode};
use super::util;

// ========================================================================= //

pub struct TextBox {
    font: Rc<Font>,
}

impl TextBox {
    pub fn new(font: Rc<Font>) -> TextBox {
        TextBox { font: font }
    }
}

impl GuiElement<String> for TextBox {
    fn draw(&self, text: &String, canvas: &mut Canvas) {
        let rect = canvas.rect();
        let rect_width = rect.width() as i32;
        let text_width = self.font.text_width(text);
        let text_left = cmp::min(4, rect_width - 4 - text_width);
        canvas.fill_rect((128, 128, 128, 255), rect);
        util::render_string(canvas, &self.font, text_left, 4, text);
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

const LABEL_WIDTH: i32 = 50;

pub struct ModalTextBox {
    left: i32,
    top: i32,
    font: Rc<Font>,
    element: SubrectElement<TextBox>,
}

impl ModalTextBox {
    pub fn new(left: i32, top: i32, font: Rc<Font>) -> ModalTextBox {
        ModalTextBox {
            left: left,
            top: top,
            font: font.clone(),
            element:
                SubrectElement::new(TextBox::new(font),
                                    Rect::new(left + LABEL_WIDTH,
                                              top,
                                              (700 - LABEL_WIDTH) as u32,
                                              18)),
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
            Mode::Goto(ref text) => {
                self.element.draw(text, canvas);
                "Goto:"
            }
            Mode::LoadFile(ref text) => {
                self.element.draw(text, canvas);
                "Load:"
            }
            Mode::NewGlyph(ref text) => {
                self.element.draw(text, canvas);
                "Char:"
            }
            Mode::Resize(ref text) => {
                self.element.draw(text, canvas);
                "Size:"
            }
            Mode::SaveAs(ref text) => {
                self.element.draw(text, canvas);
                "Save:"
            }
            Mode::SetMetrics(ref text) => {
                self.element.draw(text, canvas);
                "Metrics:"
            }
            Mode::TestSentence => {
                self.element.draw(state.test_sentence(), canvas);
                "Text:"
            }
        };
        let text_width = self.font.text_width(label);
        util::render_string(canvas,
                            &self.font,
                            self.left + LABEL_WIDTH - text_width - 2,
                            self.top + 4,
                            label);
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
                    Mode::Edit => return Action::ignore().and_continue(),
                    Mode::Goto(ref mut text) |
                    Mode::LoadFile(ref mut text) |
                    Mode::NewGlyph(ref mut text) |
                    Mode::Resize(ref mut text) |
                    Mode::SaveAs(ref mut text) |
                    Mode::SetMetrics(ref mut text) => {
                        return self.element.handle_event(event, text)
                    }
                    Mode::TestSentence => {}
                }
                self.element.handle_event(event, state.test_sentence_mut())
            }
        }
    }
}

// ========================================================================= //
