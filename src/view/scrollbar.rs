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
use crate::state::EditorState;
use num_integer::mod_floor;
use sdl2::rect::{Point, Rect};

//===========================================================================//

pub struct ImagesScrollbar {
    element: SubrectElement<AggregateElement<EditorState, ()>>,
}

impl ImagesScrollbar {
    pub fn new(left: i32, top: i32) -> ImagesScrollbar {
        let elements: Vec<Box<dyn GuiElement<EditorState, ()>>> = vec![
            ImagesScrollbar::arrow_button(2, -1, Keycode::Up),
            ImagesScrollbar::picker(20, -2),
            ImagesScrollbar::picker(58, -1),
            ImagesScrollbar::picker(96, 0),
            ImagesScrollbar::picker(134, 1),
            ImagesScrollbar::picker(172, 2),
            ImagesScrollbar::picker(210, 3),
            ImagesScrollbar::arrow_button(248, 1, Keycode::Down),
        ];
        ImagesScrollbar {
            element: SubrectElement::new(
                AggregateElement::new(elements),
                Rect::new(left, top, 38, 266),
            ),
        }
    }

    fn arrow_button(
        y: i32,
        delta: i32,
        key: Keycode,
    ) -> Box<dyn GuiElement<EditorState, ()>> {
        Box::new(SubrectElement::new(
            NextPrevImage::new(delta, key),
            Rect::new(2, y, 32, 16),
        ))
    }

    fn picker(y: i32, delta: i32) -> Box<dyn GuiElement<EditorState, ()>> {
        Box::new(SubrectElement::new(
            ImagePicker::new(delta),
            Rect::new(1, y, 36, 36),
        ))
    }
}

impl GuiElement<EditorState, ()> for ImagesScrollbar {
    fn draw(
        &self,
        state: &EditorState,
        resources: &Resources,
        canvas: &mut Canvas,
    ) {
        canvas.fill_rect((95, 95, 95, 255), self.element.rect());
        self.element.draw(state, resources, canvas);
    }

    fn on_event(
        &mut self,
        event: &Event,
        state: &mut EditorState,
    ) -> Action<()> {
        self.element.on_event(event, state)
    }
}

//===========================================================================//

struct ImagePicker {
    delta: i32,
}

impl ImagePicker {
    fn new(delta: i32) -> ImagePicker {
        ImagePicker { delta }
    }

    fn index(&self, state: &EditorState) -> Option<usize> {
        let index = (state.image_index() as i32) + self.delta;
        if index >= 0 && index < (state.num_images() as i32) {
            Some(index as usize)
        } else {
            None
        }
    }
}

impl GuiElement<EditorState, ()> for ImagePicker {
    fn draw(
        &self,
        state: &EditorState,
        _resources: &Resources,
        canvas: &mut Canvas,
    ) {
        let color = if let Some(index) = self.index(state) {
            canvas.draw_image(state.image_at(index), state.palette(), 2, 2, 1);
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

    fn on_event(
        &mut self,
        event: &Event,
        state: &mut EditorState,
    ) -> Action<()> {
        match event {
            &Event::MouseDown(_) => {
                if let Some(index) = self.index(state) {
                    state.set_image_index(index);
                    Action::redraw().and_stop()
                } else {
                    Action::ignore().and_stop()
                }
            }
            _ => Action::ignore(),
        }
    }
}

//===========================================================================//

struct NextPrevImage {
    delta: i32,
    key: Keycode,
}

impl NextPrevImage {
    fn new(delta: i32, key: Keycode) -> NextPrevImage {
        NextPrevImage { delta, key }
    }

    fn increment(&self, state: &mut EditorState) -> Action<()> {
        let new_index = mod_floor(
            (state.image_index() as i32) + self.delta,
            state.num_images() as i32,
        );
        state.set_image_index(new_index as usize);
        Action::redraw().and_stop()
    }
}

impl GuiElement<EditorState, ()> for NextPrevImage {
    fn draw(
        &self,
        _: &EditorState,
        resources: &Resources,
        canvas: &mut Canvas,
    ) {
        let icon = if self.delta > 0 {
            resources.arrow_down()
        } else {
            resources.arrow_up()
        };
        canvas.draw_sprite(icon, Point::new(0, 0));
    }

    fn on_event(
        &mut self,
        event: &Event,
        state: &mut EditorState,
    ) -> Action<()> {
        match event {
            &Event::MouseDown(_) => {
                return self.increment(state);
            }
            &Event::KeyDown(key, kmod) => {
                if key == self.key && kmod == NONE {
                    return self.increment(state);
                }
            }
            _ => {}
        }
        Action::ignore()
    }
}

//===========================================================================//
