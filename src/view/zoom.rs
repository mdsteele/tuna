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
use crate::event::{COMMAND, Event, Keycode};
use crate::state::EditorState;
use sdl2::rect::{Point, Rect};

//===========================================================================//

const MAX_ZOOM: u32 = 64;

type ZoomElement = Box<dyn GuiElement<(u32, u32), ZoomAction>>;

//===========================================================================//

pub enum ZoomAction {
    EditZoom,
}

//===========================================================================//

pub struct ZoomControls {
    element: SubrectElement<AggregateElement<(u32, u32), ZoomAction>>,
}

impl ZoomControls {
    const WIDTH: u32 = 90;
    const HEIGHT: u32 = 18;

    pub fn new(left: i32, top: i32) -> Self {
        let elements: Vec<ZoomElement> = vec![
            Self::button(1, 1, Keycode::Minus, false),
            Self::display(Rect::new(20, 1, 50, 16)),
            Self::button(73, 1, Keycode::Equals, true),
        ];
        Self {
            element: SubrectElement::new(
                AggregateElement::new(elements),
                Rect::new(left, top, Self::WIDTH, Self::HEIGHT),
            ),
        }
    }

    fn button(left: i32, top: i32, key: Keycode, inc: bool) -> ZoomElement {
        Box::new(SubrectElement::new(
            ZoomButton::new(key, inc),
            Rect::new(left, top, 16, 16),
        ))
    }

    fn display(rect: Rect) -> ZoomElement {
        Box::new(ZoomDisplay::new(rect))
    }
}

impl GuiElement<EditorState, ZoomAction> for ZoomControls {
    fn draw(
        &self,
        state: &EditorState,
        resources: &Resources,
        canvas: &mut Canvas,
    ) {
        canvas.fill_rect((95, 95, 95, 255), self.element.rect());
        self.element.draw(&state.zoom(), resources, canvas);
    }

    fn on_event(
        &mut self,
        event: &Event,
        state: &mut EditorState,
    ) -> Action<ZoomAction> {
        let mut new_zoom = state.zoom();
        let action = self.element.on_event(event, &mut new_zoom);
        if new_zoom != state.zoom() {
            state.set_zoom(new_zoom);
        }
        action
    }
}

//===========================================================================//

struct ZoomButton {
    key: Keycode,
    inc: bool,
}

impl ZoomButton {
    fn new(key: Keycode, inc: bool) -> Self {
        Self { key, inc }
    }

    fn update_zoom(&self, zoom: &mut (u32, u32)) {
        let (horz, vert) = *zoom;
        if self.inc {
            if horz.max(vert) <= MAX_ZOOM / 2 {
                *zoom = (horz * 2, vert * 2);
            }
        } else {
            if horz % 2 == 0 && vert % 2 == 0 {
                *zoom = (horz / 2, vert / 2);
            }
        }
    }
}

impl GuiElement<(u32, u32), ZoomAction> for ZoomButton {
    fn draw(
        &self,
        _: &(u32, u32),
        resources: &Resources,
        canvas: &mut Canvas,
    ) {
        let icon = if self.inc { ToolIcon::ZoomIn } else { ToolIcon::ZoomOut };
        canvas.draw_sprite(resources.tool_icon(icon), Point::new(0, 0));
    }

    fn on_event(
        &mut self,
        event: &Event,
        zoom: &mut (u32, u32),
    ) -> Action<ZoomAction> {
        match *event {
            Event::MouseDown(_) => {
                self.update_zoom(zoom);
                Action::redraw().and_stop()
            }
            Event::KeyDown(key, kmod)
                if key == self.key && kmod == COMMAND =>
            {
                self.update_zoom(zoom);
                Action::redraw().and_stop()
            }
            _ => Action::ignore(),
        }
    }
}

//===========================================================================//

struct ZoomDisplay {
    rect: Rect,
    double_click_counter: i32,
}

impl ZoomDisplay {
    fn new(rect: Rect) -> Self {
        Self { rect, double_click_counter: 0 }
    }
}

impl GuiElement<(u32, u32), ZoomAction> for ZoomDisplay {
    fn draw(
        &self,
        zoom: &(u32, u32),
        resources: &Resources,
        canvas: &mut Canvas,
    ) {
        let mut subcanvas = canvas.subcanvas(self.rect);
        let (horz, vert) = *zoom;
        let left = if horz < 10 { 8 } else { 1 };
        let string = format!("{horz}x/{vert}x");
        subcanvas.draw_string(resources.font(), left, 3, &string);
    }

    fn on_event(
        &mut self,
        event: &Event,
        _: &mut (u32, u32),
    ) -> Action<ZoomAction> {
        match *event {
            Event::ClockTick => {
                if self.double_click_counter > 0 {
                    self.double_click_counter -= 1;
                }
            }
            Event::MouseDown(pt) if self.rect.contains_point(pt) => {
                if self.double_click_counter > 0 {
                    return Action::redraw().and_return(ZoomAction::EditZoom);
                } else {
                    self.double_click_counter = 4;
                    return Action::ignore().and_stop();
                }
            }
            _ => {}
        }
        Action::ignore()
    }
}

//===========================================================================//
