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
use sdl2::keyboard::Keycode;
use sdl2::mouse::Mouse;
use sdl2::rect::{Point, Rect};
use super::canvas::{Canvas, Sprite};
use super::element::{AggregateElement, GuiElement, SubrectElement};
use super::state::EditorState;
use super::util;

// ========================================================================= //

pub struct ImagesScrollbar {
    element: SubrectElement<AggregateElement<EditorState>>,
}

impl ImagesScrollbar {
    pub fn new(left: i32,
               top: i32,
               mut icons: Vec<Sprite>)
               -> ImagesScrollbar {
        icons.truncate(2);
        assert_eq!(icons.len(), 2);
        let down_icon = icons.pop().unwrap();
        let up_icon = icons.pop().unwrap();
        let elements: Vec<Box<GuiElement<EditorState>>> = vec![
 ImagesScrollbar::arrow_button(2, -1, Keycode::Up, up_icon),
 ImagesScrollbar::picker(20, -2),
 ImagesScrollbar::picker(58, -1),
 ImagesScrollbar::picker(96, 0),
 ImagesScrollbar::picker(134, 1),
 ImagesScrollbar::picker(172, 2),
 ImagesScrollbar::picker(210, 3),
 ImagesScrollbar::arrow_button(248, 1, Keycode::Down, down_icon),
        ];
        ImagesScrollbar {
            element: SubrectElement::new(AggregateElement::new(elements),
                                         Rect::new(left, top, 40, 266)),
        }
    }

    fn arrow_button(y: i32,
                    delta: i32,
                    key: Keycode,
                    icon: Sprite)
                    -> Box<GuiElement<EditorState>> {
        Box::new(SubrectElement::new(NextPrevImage::new(delta, key, icon),
                                     Rect::new(4, y, 32, 16)))
    }

    fn picker(y: i32, delta: i32) -> Box<GuiElement<EditorState>> {
        Box::new(SubrectElement::new(ImagePicker::new(delta),
                                     Rect::new(2, y, 36, 36)))
    }
}

impl GuiElement<EditorState> for ImagesScrollbar {
    fn draw(&self, state: &EditorState, canvas: &mut Canvas) {
        canvas.fill_rect((95, 95, 95, 255), self.element.rect());
        self.element.draw(state, canvas);
    }

    fn handle_event(&mut self,
                    event: &Event,
                    state: &mut EditorState)
                    -> bool {
        self.element.handle_event(event, state)
    }
}

// ========================================================================= //

struct ImagePicker {
    delta: i32,
}

impl ImagePicker {
    fn new(delta: i32) -> ImagePicker {
        ImagePicker { delta: delta }
    }

    fn index(&self, state: &EditorState) -> Option<usize> {
        let index = (state.current_image as i32) + self.delta;
        if index >= 0 && index < (state.images.len() as i32) {
            Some(index as usize)
        } else {
            None
        }
    }

    fn pick(&self, state: &mut EditorState) -> bool {
        if let Some(index) = self.index(state) {
            state.current_image = index;
            true
        } else {
            false
        }
    }
}

impl GuiElement<EditorState> for ImagePicker {
    fn draw(&self, state: &EditorState, canvas: &mut Canvas) {
        let color = if let Some(index) = self.index(state) {
            util::render_image(canvas, state.image_at(index), 2, 2, 1);
            if self.delta == 0 {
                (255, 255, 127, 255)
            } else {
                (127, 127, 63, 255)
            }
        } else {
            (0, 0, 0, 255)
        };
        let rect = canvas.rect();
        canvas.draw_rect(color, rect);
    }

    fn handle_event(&mut self,
                    event: &Event,
                    state: &mut EditorState)
                    -> bool {
        match event {
            &Event::MouseButtonDown { mouse_btn: Mouse::Left, .. } => {
                return self.pick(state);
            }
            _ => {}
        }
        false
    }
}

// ========================================================================= //

struct NextPrevImage {
    delta: i32,
    key: Keycode,
    icon: Sprite,
}

impl NextPrevImage {
    fn new(delta: i32, key: Keycode, icon: Sprite) -> NextPrevImage {
        NextPrevImage {
            delta: delta,
            key: key,
            icon: icon,
        }
    }

    fn increment(&self, state: &mut EditorState) -> bool {
        state.unselect();
        state.current_image =
            modulo((state.current_image as i32) + self.delta,
                   state.images.len() as i32) as usize;
        true
    }
}

impl GuiElement<EditorState> for NextPrevImage {
    fn draw(&self, _: &EditorState, canvas: &mut Canvas) {
        canvas.draw_sprite(&self.icon, Point::new(0, 0));
    }

    fn handle_event(&mut self,
                    event: &Event,
                    state: &mut EditorState)
                    -> bool {
        match event {
            &Event::MouseButtonDown { mouse_btn: Mouse::Left, .. } => {
                return self.increment(state);
            }
            &Event::KeyDown { keycode: Some(key), .. } => {
                if key == self.key {
                    return self.increment(state);
                }
            }
            _ => {}
        }
        false
    }
}

// ========================================================================= //

fn modulo(a: i32, b: i32) -> i32 {
    if b == 0 {
        panic!();
    }
    let remainder = a % b;
    if remainder == 0 {
        0
    } else if (a < 0) ^ (b < 0) {
        remainder + b
    } else {
        remainder
    }
}

// ========================================================================= //
