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
use sdl2::rect::{Point, Rect};

//===========================================================================//

pub struct Toolbox {
    element: SubrectElement<AggregateElement<Tool>>,
}

impl Toolbox {
    pub fn new(left: i32, top: i32) -> Toolbox {
        let elements: Vec<Box<dyn GuiElement<Tool>>> = vec![
            Toolbox::picker(2, 2, Tool::Pencil, Keycode::P, 0),
            Toolbox::picker(26, 2, Tool::Line, Keycode::L, 4),
            Toolbox::picker(2, 26, Tool::Oval, Keycode::O, 6),
            Toolbox::picker(26, 26, Tool::Rectangle, Keycode::R, 7),
            Toolbox::picker(2, 50, Tool::PaintBucket, Keycode::K, 1),
            Toolbox::picker(26, 50, Tool::Checkerboard, Keycode::H, 5),
            Toolbox::picker(2, 74, Tool::PaletteSwap, Keycode::W, 8),
            Toolbox::picker(26, 74, Tool::PaletteReplace, Keycode::Q, 9),
            Toolbox::picker(2, 98, Tool::Eyedropper, Keycode::Y, 2),
            Toolbox::picker(26, 98, Tool::Select, Keycode::S, 3),
        ];
        Toolbox {
            element: SubrectElement::new(
                AggregateElement::new(elements),
                Rect::new(left, top, 48, 120),
            ),
        }
    }

    fn picker(
        x: i32,
        y: i32,
        tool: Tool,
        key: Keycode,
        icon_index: usize,
    ) -> Box<dyn GuiElement<Tool>> {
        Box::new(SubrectElement::new(
            ToolPicker::new(tool, key, icon_index),
            Rect::new(x, y, 20, 20),
        ))
    }
}

impl GuiElement<EditorState> for Toolbox {
    fn draw(
        &self,
        state: &EditorState,
        resources: &Resources,
        canvas: &mut Canvas,
    ) {
        canvas.fill_rect((95, 95, 95, 255), self.element.rect());
        self.element.draw(&state.tool(), resources, canvas);
    }

    fn handle_event(
        &mut self,
        event: &Event,
        state: &mut EditorState,
    ) -> Action {
        let mut new_tool = state.tool();
        let action = self.element.handle_event(event, &mut new_tool);
        if new_tool != state.tool() {
            state.set_tool(new_tool);
        }
        action
    }
}

//===========================================================================//

struct ToolPicker {
    tool: Tool,
    key: Keycode,
    icon_index: usize,
}

impl ToolPicker {
    fn new(tool: Tool, key: Keycode, icon_index: usize) -> ToolPicker {
        ToolPicker { tool, key, icon_index }
    }
}

impl GuiElement<Tool> for ToolPicker {
    fn draw(&self, tool: &Tool, resources: &Resources, canvas: &mut Canvas) {
        if *tool == self.tool {
            canvas.clear((255, 255, 255, 255));
        } else {
            canvas.clear((95, 95, 95, 255));
        }
        canvas.draw_sprite(
            resources.tool_icon(self.icon_index),
            Point::new(2, 2),
        );
    }

    fn handle_event(&mut self, event: &Event, tool: &mut Tool) -> Action {
        match event {
            &Event::MouseDown(_) => {
                *tool = self.tool;
                return Action::redraw().and_stop();
            }
            &Event::KeyDown(key, kmod) => {
                if key == self.key && kmod == NONE {
                    *tool = self.tool;
                    return Action::redraw().and_stop();
                }
            }
            _ => {}
        }
        Action::ignore().and_continue()
    }
}

//===========================================================================//
