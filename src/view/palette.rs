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
use crate::element::{Action, AggregateElement, GuiElement, SubrectElement};
use crate::event::{Event, Keycode, NONE};
use crate::state::{EditorState, Tool};
use ahi::{self, Color};
use sdl2::rect::Rect;
use std::cmp;

//===========================================================================//

pub struct ColorPalette {
    element: SubrectElement<AggregateElement<Color>>,
}

impl ColorPalette {
    pub fn new(left: i32, top: i32) -> ColorPalette {
        let elements: Vec<Box<dyn GuiElement<Color>>> = vec![
            ColorPalette::picker(0, 0, Color::C0, Keycode::Num0),
            ColorPalette::picker(18, 0, Color::C1, Keycode::Num1),
            ColorPalette::picker(0, 18, Color::C2, Keycode::Num2),
            ColorPalette::picker(18, 18, Color::C3, Keycode::Num3),
            ColorPalette::picker(0, 36, Color::C4, Keycode::Num4),
            ColorPalette::picker(18, 36, Color::C5, Keycode::Num5),
            ColorPalette::picker(0, 54, Color::C6, Keycode::Num6),
            ColorPalette::picker(18, 54, Color::C7, Keycode::Num7),
            ColorPalette::picker(0, 72, Color::C8, Keycode::Num8),
            ColorPalette::picker(18, 72, Color::C9, Keycode::Num9),
            ColorPalette::picker(0, 90, Color::Ca, Keycode::A),
            ColorPalette::picker(18, 90, Color::Cb, Keycode::B),
            ColorPalette::picker(0, 108, Color::Cc, Keycode::C),
            ColorPalette::picker(18, 108, Color::Cd, Keycode::D),
            ColorPalette::picker(0, 126, Color::Ce, Keycode::E),
            ColorPalette::picker(18, 126, Color::Cf, Keycode::F),
        ];
        ColorPalette {
            element: SubrectElement::new(
                AggregateElement::new(elements),
                Rect::new(left, top, 36, 144),
            ),
        }
    }

    fn picker(
        x: i32,
        y: i32,
        color: Color,
        key: Keycode,
    ) -> Box<dyn GuiElement<Color>> {
        Box::new(SubrectElement::new(
            ColorPicker::new(color, key),
            Rect::new(x, y, 18, 18),
        ))
    }
}

impl GuiElement<EditorState> for ColorPalette {
    fn draw(
        &self,
        state: &EditorState,
        resources: &Resources,
        canvas: &mut Canvas,
    ) {
        canvas.fill_rect((95, 95, 95, 255), self.element.rect());
        self.element.draw(&state.color(), resources, canvas);
    }

    fn handle_event(
        &mut self,
        event: &Event,
        state: &mut EditorState,
    ) -> Action {
        let mut new_color = state.color();
        let result = self.element.handle_event(event, &mut new_color);
        if new_color != state.color() {
            state.set_color(new_color);
            if state.tool() == Tool::Select {
                state.set_tool(Tool::Pencil);
            }
        }
        result
    }
}

//===========================================================================//

struct ColorPicker {
    color: Color,
    key: Keycode,
}

impl ColorPicker {
    fn new(color: Color, key: Keycode) -> ColorPicker {
        ColorPicker { color, key }
    }
}

impl GuiElement<Color> for ColorPicker {
    fn draw(
        &self,
        state: &Color,
        _resources: &Resources,
        canvas: &mut Canvas,
    ) {
        let rect = canvas.rect();
        let inner = shrink(rect, 2);
        let (r, g, b, a) = ahi::Palette::default()[self.color];
        if a < u8::MAX {
            canvas.draw_rect((0, 0, 0, 255), inner);
            canvas.draw_rect((0, 0, 0, 255), shrink(inner, 2));
            canvas.draw_rect((0, 0, 0, 255), shrink(inner, 4));
        }
        if a > 0 {
            canvas.fill_rect((r, g, b, a), inner);
        }
        if *state == self.color {
            canvas.draw_rect((255, 255, 255, 255), rect);
        }
    }

    fn handle_event(&mut self, event: &Event, state: &mut Color) -> Action {
        match event {
            &Event::MouseDown(_) => {
                *state = self.color;
                return Action::redraw().and_stop();
            }
            &Event::KeyDown(key, kmod) => {
                if key == self.key && kmod == NONE {
                    *state = self.color;
                    return Action::redraw().and_stop();
                }
            }
            _ => {}
        }
        Action::ignore().and_continue()
    }
}

//===========================================================================//

fn shrink(rect: Rect, by: i32) -> Rect {
    Rect::new(
        rect.x() + by,
        rect.y() + by,
        cmp::max((rect.width() as i32) - 2 * by, 0) as u32,
        cmp::max((rect.height() as i32) - 2 * by, 0) as u32,
    )
}

//===========================================================================//
