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

use sdl2::event::Event;
use sdl2::rect::{Point, Rect};
use std::rc::Rc;
use super::canvas::{Canvas, Sprite};
use super::element::{GuiElement, SubrectElement};
use super::state::EditorState;

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

    fn handle_event(&mut self, _: &Event, _: &mut String) -> bool {
        false
    }
}

// ========================================================================= //

pub struct FilePathTextBox {
    element: SubrectElement<TextBox>,
}

impl FilePathTextBox {
    pub fn new(left: i32, top: i32, font: Rc<Vec<Sprite>>) -> FilePathTextBox {
        FilePathTextBox {
            element: SubrectElement::new(TextBox::new(font),
                                         Rect::new(left, top, 472, 20)),
        }
    }
}

impl GuiElement<EditorState> for FilePathTextBox {
    fn draw(&self, state: &EditorState, canvas: &mut Canvas) {
        self.element.draw(&state.filepath, canvas);
    }

    fn handle_event(&mut self, _: &Event, _: &mut EditorState) -> bool {
        false
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
