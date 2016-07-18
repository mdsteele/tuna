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
use super::canvas::{Canvas, Sprite};
use super::element::{AggregateElement, GuiElement, SubrectElement};
use super::event::{Event, Keycode, NONE};
use super::state::{EditorState, Tool};

// ========================================================================= //

pub struct Toolbox {
    element: SubrectElement<AggregateElement<Tool>>,
}

impl Toolbox {
    pub fn new(left: i32, top: i32, mut icons: Vec<Sprite>) -> Toolbox {
        icons.truncate(5);
        assert_eq!(icons.len(), 5);
        let line_icon = icons.pop().unwrap();
        let select_icon = icons.pop().unwrap();
        let eyedrop_icon = icons.pop().unwrap();
        let bucket_icon = icons.pop().unwrap();
        let pencil_icon = icons.pop().unwrap();
        let elements: Vec<Box<GuiElement<Tool>>> = vec![
            Toolbox::picker(2, 2, Tool::Pencil, Keycode::P, pencil_icon),
            Toolbox::picker(2, 26, Tool::PaintBucket, Keycode::K, bucket_icon),
            Toolbox::picker(2, 50, Tool::Eyedropper, Keycode::Y, eyedrop_icon),
            Toolbox::picker(2, 74, Tool::Select, Keycode::S, select_icon),
            Toolbox::picker(2, 98, Tool::Line, Keycode::L, line_icon),
        ];
        Toolbox {
            element: SubrectElement::new(AggregateElement::new(elements),
                                         Rect::new(left, top, 24, 120)),
        }
    }

    fn picker(x: i32,
              y: i32,
              tool: Tool,
              key: Keycode,
              icon: Sprite)
              -> Box<GuiElement<Tool>> {
        Box::new(SubrectElement::new(ToolPicker::new(tool, key, icon),
                                     Rect::new(x, y, 20, 20)))
    }
}

impl GuiElement<EditorState> for Toolbox {
    fn draw(&self, state: &EditorState, canvas: &mut Canvas) {
        canvas.fill_rect((95, 95, 95, 255), self.element.rect());
        self.element.draw(&state.tool(), canvas);
    }

    fn handle_event(&mut self,
                    event: &Event,
                    state: &mut EditorState)
                    -> bool {
        let mut new_tool = state.tool();
        let result = self.element.handle_event(event, &mut new_tool);
        if new_tool != state.tool() {
            state.set_tool(new_tool);
        }
        result
    }
}

// ========================================================================= //

struct ToolPicker {
    tool: Tool,
    key: Keycode,
    icon: Sprite,
}

impl ToolPicker {
    fn new(tool: Tool, key: Keycode, icon: Sprite) -> ToolPicker {
        ToolPicker {
            tool: tool,
            key: key,
            icon: icon,
        }
    }
}

impl GuiElement<Tool> for ToolPicker {
    fn draw(&self, tool: &Tool, canvas: &mut Canvas) {
        if *tool == self.tool {
            canvas.clear((255, 255, 255, 255));
        } else {
            canvas.clear((95, 95, 95, 255));
        }
        canvas.draw_sprite(&self.icon, Point::new(2, 2));
    }

    fn handle_event(&mut self, event: &Event, tool: &mut Tool) -> bool {
        match event {
            &Event::MouseDown(_) => {
                *tool = self.tool;
                return true;
            }
            &Event::KeyDown(key, kmod) => {
                if key == self.key && kmod == NONE {
                    *tool = self.tool;
                    return true;
                }
            }
            _ => {}
        }
        false
    }
}

// ========================================================================= //
