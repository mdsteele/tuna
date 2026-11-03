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
    pub fn new(left: i32, top: i32, height: u32) -> Self {
        let elements: Vec<Box<dyn GuiElement<EditorState, ()>>> = vec![
            Self::arrow_button(2, -1, Keycode::Up),
            Self::multipicker(20, height - 40),
            Self::arrow_button(height as i32 - 18, 1, Keycode::Down),
        ];
        Self {
            element: SubrectElement::new(
                AggregateElement::new(elements),
                Rect::new(left, top, 38, height),
            ),
        }
    }

    fn arrow_button(
        y: i32,
        delta: isize,
        key: Keycode,
    ) -> Box<dyn GuiElement<EditorState, ()>> {
        Box::new(SubrectElement::new(
            NextPrevImage::new(delta, key),
            Rect::new(2, y, 32, 16),
        ))
    }

    fn multipicker(
        y: i32,
        height: u32,
    ) -> Box<dyn GuiElement<EditorState, ()>> {
        Box::new(SubrectElement::new(
            ImageMultipicker::new(36, height),
            Rect::new(1, y, 36, height),
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

struct ImageMultipicker {
    width: u32,
    height: u32,
}

impl ImageMultipicker {
    const BOX_MARGIN: i32 = 2;
    const BOX_SPACING: i32 = 2;

    fn new(width: u32, height: u32) -> Self {
        ImageMultipicker { width, height }
    }

    fn boxes(&self, state: &EditorState) -> Vec<(Rect, usize)> {
        let num_images = state.num_images();
        let current_image_index = state.image_index();

        let mut height_above: i32 = 0;
        for index in 0..current_image_index {
            let image_height = state.image_at(index).height() as i32;
            let box_height = image_height + Self::BOX_MARGIN * 2;
            height_above += Self::BOX_SPACING + box_height;
        }

        let mut height_below: i32 = 0;
        for index in (current_image_index + 1)..num_images {
            let image_height = state.image_at(index).height() as i32;
            let box_height = image_height + Self::BOX_MARGIN * 2;
            height_below += Self::BOX_SPACING + box_height;
        }

        let picker_height = self.height as i32;
        let current_image_height = state.image().height() as i32;
        let current_box_height = current_image_height + Self::BOX_MARGIN * 2;
        let mut box_y = (picker_height - current_box_height) / 2;
        if height_below < (picker_height - current_box_height) / 2 {
            box_y = picker_height - height_below - current_box_height;
        }
        box_y = box_y.min(height_above);

        let mut index = current_image_index;
        while index > 0 && box_y > Self::BOX_SPACING {
            index -= 1;
            let image_height = state.image_at(index).height() as i32;
            box_y -= Self::BOX_SPACING + image_height + Self::BOX_MARGIN * 2;
        }

        let mut boxes = Vec::<(Rect, usize)>::new();
        while index < num_images && box_y < picker_height {
            let image_height = state.image_at(index).height() as i32;
            let box_height = Self::BOX_MARGIN * 2 + image_height;
            let box_rect = Rect::new(0, box_y, self.width, box_height as u32);
            boxes.push((box_rect, index));
            index += 1;
            box_y += box_height + Self::BOX_SPACING;
        }
        boxes
    }
}

impl GuiElement<EditorState, ()> for ImageMultipicker {
    fn draw(
        &self,
        state: &EditorState,
        _resources: &Resources,
        canvas: &mut Canvas,
    ) {
        let current_image_index = state.image_index();
        for (subrect, index) in self.boxes(state) {
            canvas.draw_image(
                state.image_at(index),
                state.palette(),
                subrect.x() + Self::BOX_MARGIN,
                subrect.y() + Self::BOX_MARGIN,
                1,
            );
            let border_color = if index == current_image_index {
                (255, 255, 127, 255)
            } else {
                (127, 127, 63, 255)
            };
            canvas.draw_rect(border_color, subrect);
        }
    }

    fn on_event(
        &mut self,
        event: &Event,
        state: &mut EditorState,
    ) -> Action<()> {
        match event {
            &Event::MouseDown(pt) => {
                for (subrect, index) in self.boxes(state) {
                    if subrect.contains_point(pt) {
                        state.set_image_index(index);
                        return Action::redraw().and_stop();
                    }
                }
                Action::ignore().and_stop()
            }
            _ => Action::ignore(),
        }
    }
}

//===========================================================================//

struct NextPrevImage {
    delta: isize,
    key: Keycode,
}

impl NextPrevImage {
    fn new(delta: isize, key: Keycode) -> Self {
        Self { delta, key }
    }

    fn increment(&self, state: &mut EditorState) -> Action<()> {
        let new_index = mod_floor(
            (state.image_index() as isize) + self.delta,
            state.num_images() as isize,
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
        match *event {
            Event::MouseDown(_) => {
                return self.increment(state);
            }
            Event::KeyDown(key, kmod) if key == self.key && kmod == NONE => {
                return self.increment(state);
            }
            _ => {}
        }
        Action::ignore()
    }
}

//===========================================================================//
