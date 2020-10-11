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
use crate::event::{Event, Keycode, NONE};
use crate::state::{EditorState, Tool};
use sdl2::rect::{Point, Rect};

//===========================================================================//

pub struct Toolbox {
    element: SubrectElement<AggregateElement<Tool>>,
}

impl Toolbox {
    const WIDTH: u32 = 72;
    const HEIGHT: u32 = 96;

    pub fn new(left: i32, top: i32) -> Toolbox {
        let elements: Vec<Box<dyn GuiElement<Tool>>> = vec![
            Toolbox::picker(2, 2, Tool::Pencil, Keycode::P),
            Toolbox::picker(26, 2, Tool::PaintBucket, Keycode::K),
            Toolbox::picker(50, 2, Tool::PaletteReplace, Keycode::V),
            Toolbox::picker(2, 26, Tool::Watercolor, Keycode::W),
            Toolbox::picker(26, 26, Tool::Checkerboard, Keycode::H),
            Toolbox::picker(50, 26, Tool::PaletteSwap, Keycode::X),
            Toolbox::picker(2, 50, Tool::Line, Keycode::I),
            Toolbox::picker(26, 50, Tool::Rectangle, Keycode::R),
            Toolbox::picker(50, 50, Tool::Oval, Keycode::O),
            Toolbox::picker(2, 74, Tool::Eyedropper, Keycode::Y),
            Toolbox::picker(26, 74, Tool::Select, Keycode::S),
            Toolbox::picker(50, 74, Tool::Lasso, Keycode::L),
        ];
        Toolbox {
            element: SubrectElement::new(
                AggregateElement::new(elements),
                Rect::new(left, top, Toolbox::WIDTH, Toolbox::HEIGHT),
            ),
        }
    }

    fn picker(
        x: i32,
        y: i32,
        tool: Tool,
        key: Keycode,
    ) -> Box<dyn GuiElement<Tool>> {
        Box::new(SubrectElement::new(
            ToolPicker::new(tool, key),
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
    icon: ToolIcon,
}

impl ToolPicker {
    fn new(tool: Tool, key: Keycode) -> ToolPicker {
        let icon = match tool {
            Tool::Checkerboard => ToolIcon::Checkerboard,
            Tool::Eyedropper => ToolIcon::Eyedropper,
            Tool::Lasso => ToolIcon::Lasso,
            Tool::Line => ToolIcon::Line,
            Tool::Oval => ToolIcon::Oval,
            Tool::PaintBucket => ToolIcon::PaintBucket,
            Tool::PaletteReplace => ToolIcon::PaletteReplace,
            Tool::PaletteSwap => ToolIcon::PaletteSwap,
            Tool::Pencil => ToolIcon::Pencil,
            Tool::Rectangle => ToolIcon::Rectangle,
            Tool::Select => ToolIcon::Select,
            Tool::Watercolor => ToolIcon::Watercolor,
        };
        ToolPicker { tool, key, icon }
    }
}

impl GuiElement<Tool> for ToolPicker {
    fn draw(&self, tool: &Tool, resources: &Resources, canvas: &mut Canvas) {
        if *tool == self.tool {
            canvas.clear((255, 255, 255, 255));
        } else {
            canvas.clear((95, 95, 95, 255));
        }
        canvas.draw_sprite(resources.tool_icon(self.icon), Point::new(2, 2));
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
