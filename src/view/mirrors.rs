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

use crate::canvas::{Canvas, Resources, ToolIcon};
use crate::element::{Action, AggregateElement, GuiElement, SubrectElement};
use crate::event::Event;
use crate::state::{EditorState, Mirror};
use sdl2::rect::{Point, Rect};

//===========================================================================//

pub struct Mirrors {
    element: SubrectElement<AggregateElement<Mirror>>,
}

impl Mirrors {
    const WIDTH: u32 = 72;
    const HEIGHT: u32 = 48;

    pub fn new(left: i32, top: i32) -> Mirrors {
        let elements: Vec<Box<dyn GuiElement<Mirror>>> = vec![
            Mirrors::picker(2, 2, Mirror::None),
            Mirrors::picker(26, 2, Mirror::Horz),
            Mirrors::picker(50, 2, Mirror::Rot2),
            Mirrors::picker(2, 26, Mirror::Vert),
            Mirrors::picker(26, 26, Mirror::Both),
            Mirrors::picker(50, 26, Mirror::Rot4),
        ];
        Mirrors {
            element: SubrectElement::new(
                AggregateElement::new(elements),
                Rect::new(left, top, Mirrors::WIDTH, Mirrors::HEIGHT),
            ),
        }
    }

    fn picker(x: i32, y: i32, mirror: Mirror) -> Box<dyn GuiElement<Mirror>> {
        Box::new(SubrectElement::new(
            MirrorPicker::new(mirror),
            Rect::new(x, y, 20, 20),
        ))
    }
}

impl GuiElement<EditorState> for Mirrors {
    fn draw(
        &self,
        state: &EditorState,
        resources: &Resources,
        canvas: &mut Canvas,
    ) {
        canvas.fill_rect((95, 95, 95, 255), self.element.rect());
        self.element.draw(&state.mirror(), resources, canvas);
    }

    fn handle_event(
        &mut self,
        event: &Event,
        state: &mut EditorState,
    ) -> Action {
        let mut new_mirror = state.mirror();
        let action = self.element.handle_event(event, &mut new_mirror);
        if new_mirror != state.mirror() {
            state.set_mirror(new_mirror);
        }
        action
    }
}

//===========================================================================//

struct MirrorPicker {
    mirror: Mirror,
    icon: ToolIcon,
}

impl MirrorPicker {
    fn new(mirror: Mirror) -> MirrorPicker {
        let icon = match mirror {
            Mirror::None => ToolIcon::MirrorNone,
            Mirror::Horz => ToolIcon::MirrorHorz,
            Mirror::Vert => ToolIcon::MirrorVert,
            Mirror::Both => ToolIcon::MirrorBoth,
            Mirror::Rot2 => ToolIcon::MirrorRot2,
            Mirror::Rot4 => ToolIcon::MirrorRot4,
        };
        MirrorPicker { mirror, icon }
    }
}

impl GuiElement<Mirror> for MirrorPicker {
    fn draw(
        &self,
        mirror: &Mirror,
        resources: &Resources,
        canvas: &mut Canvas,
    ) {
        if *mirror == self.mirror {
            canvas.clear((255, 255, 255, 255));
        } else {
            canvas.clear((95, 95, 95, 255));
        }
        canvas.draw_sprite(resources.tool_icon(self.icon), Point::new(2, 2));
    }

    fn handle_event(&mut self, event: &Event, mirror: &mut Mirror) -> Action {
        match event {
            &Event::MouseDown(_) => {
                *mirror = self.mirror;
                return Action::redraw().and_stop();
            }
            _ => {}
        }
        Action::ignore().and_continue()
    }
}

//===========================================================================//
