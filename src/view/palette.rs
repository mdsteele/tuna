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
use ahi::{self, Color};
use sdl2::rect::{Point, Rect};
use std::cmp;

//===========================================================================//

pub enum PaletteAction {
    EditColor(Color),
}

//===========================================================================//

pub struct PaletteView {
    element: SubrectElement<AggregateElement<EditorState, PaletteAction>>,
}

impl PaletteView {
    pub fn new(left: i32, top: i32) -> PaletteView {
        let elements: Vec<Box<dyn GuiElement<EditorState, PaletteAction>>> = vec![
            Box::new(ColorPalette::new(0, 0)),
            PaletteView::arrow(
                0,
                ColorPalette::HEIGHT as i32,
                -1,
                Keycode::Left,
            ),
            PaletteView::add_palette_button(
                NextPrevPalette::WIDTH as i32,
                ColorPalette::HEIGHT as i32,
            ),
            PaletteView::delete_palette_button(
                2 * (NextPrevPalette::WIDTH as i32),
                ColorPalette::HEIGHT as i32,
            ),
            PaletteView::arrow(
                3 * (NextPrevPalette::WIDTH as i32),
                ColorPalette::HEIGHT as i32,
                1,
                Keycode::Right,
            ),
            Box::new(PaletteInfoView::new(
                0,
                (ColorPalette::HEIGHT + NextPrevPalette::HEIGHT) as i32,
            )),
        ];
        PaletteView {
            element: SubrectElement::new(
                AggregateElement::new(elements),
                Rect::new(
                    left,
                    top,
                    ColorPalette::WIDTH,
                    ColorPalette::HEIGHT
                        + NextPrevPalette::HEIGHT
                        + PaletteInfoView::HEIGHT,
                ),
            ),
        }
    }

    fn arrow(
        x: i32,
        y: i32,
        delta: i32,
        key: Keycode,
    ) -> Box<dyn GuiElement<EditorState, PaletteAction>> {
        Box::new(SubrectElement::new(
            NextPrevPalette::new(delta, key),
            Rect::new(
                x,
                y,
                NextPrevPalette::WIDTH as u32,
                NextPrevPalette::HEIGHT as u32,
            ),
        ))
    }

    fn add_palette_button(
        x: i32,
        y: i32,
    ) -> Box<dyn GuiElement<EditorState, PaletteAction>> {
        Box::new(SubrectElement::new(
            AddPalettteButton::new(),
            Rect::new(
                x,
                y,
                NextPrevPalette::WIDTH as u32,
                NextPrevPalette::HEIGHT as u32,
            ),
        ))
    }

    fn delete_palette_button(
        x: i32,
        y: i32,
    ) -> Box<dyn GuiElement<EditorState, PaletteAction>> {
        Box::new(SubrectElement::new(
            DeletePalettteButton::new(),
            Rect::new(
                x,
                y,
                NextPrevPalette::WIDTH as u32,
                NextPrevPalette::HEIGHT as u32,
            ),
        ))
    }
}

impl GuiElement<EditorState, PaletteAction> for PaletteView {
    fn draw(
        &self,
        state: &EditorState,
        resources: &Resources,
        canvas: &mut Canvas,
    ) {
        self.element.draw(state, resources, canvas);
    }

    fn on_event(
        &mut self,
        event: &Event,
        state: &mut EditorState,
    ) -> Action<PaletteAction> {
        self.element.on_event(event, state)
    }
}

//===========================================================================//

struct ColorPalette {
    element: SubrectElement<AggregateElement<EditorState, PaletteAction>>,
}

impl ColorPalette {
    const WIDTH: u32 = 72;
    const HEIGHT: u32 = 72;

    fn new(left: i32, top: i32) -> ColorPalette {
        let elements: Vec<Box<dyn GuiElement<EditorState, PaletteAction>>> = vec![
            ColorPalette::picker(0, 0, Color::C0, Keycode::Num0),
            ColorPalette::picker(18, 0, Color::C1, Keycode::Num1),
            ColorPalette::picker(36, 0, Color::C2, Keycode::Num2),
            ColorPalette::picker(54, 0, Color::C3, Keycode::Num3),
            ColorPalette::picker(0, 18, Color::C4, Keycode::Num4),
            ColorPalette::picker(18, 18, Color::C5, Keycode::Num5),
            ColorPalette::picker(36, 18, Color::C6, Keycode::Num6),
            ColorPalette::picker(54, 18, Color::C7, Keycode::Num7),
            ColorPalette::picker(0, 36, Color::C8, Keycode::Num8),
            ColorPalette::picker(18, 36, Color::C9, Keycode::Num9),
            ColorPalette::picker(36, 36, Color::Ca, Keycode::A),
            ColorPalette::picker(54, 36, Color::Cb, Keycode::B),
            ColorPalette::picker(0, 54, Color::Cc, Keycode::C),
            ColorPalette::picker(18, 54, Color::Cd, Keycode::D),
            ColorPalette::picker(36, 54, Color::Ce, Keycode::E),
            ColorPalette::picker(54, 54, Color::Cf, Keycode::F),
        ];
        ColorPalette {
            element: SubrectElement::new(
                AggregateElement::new(elements),
                Rect::new(
                    left,
                    top,
                    ColorPalette::WIDTH,
                    ColorPalette::HEIGHT,
                ),
            ),
        }
    }

    fn picker(
        x: i32,
        y: i32,
        color: Color,
        key: Keycode,
    ) -> Box<dyn GuiElement<EditorState, PaletteAction>> {
        Box::new(SubrectElement::new(
            ColorPicker::new(color, key),
            Rect::new(x, y, 18, 18),
        ))
    }
}

impl GuiElement<EditorState, PaletteAction> for ColorPalette {
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
    ) -> Action<PaletteAction> {
        self.element.on_event(event, state)
    }
}

//===========================================================================//

struct ColorPicker {
    color: Color,
    key: Keycode,
    double_click_counter: i32,
}

impl ColorPicker {
    fn new(color: Color, key: Keycode) -> ColorPicker {
        ColorPicker { color, key, double_click_counter: 0 }
    }

    fn pick_color(&self, state: &mut EditorState) {
        if state.color() != self.color {
            state.set_color(self.color);
            if state.tool() == Tool::Select {
                state.set_tool(Tool::Pencil);
            }
        }
    }
}

impl GuiElement<EditorState, PaletteAction> for ColorPicker {
    fn draw(
        &self,
        state: &EditorState,
        _resources: &Resources,
        canvas: &mut Canvas,
    ) {
        let rect = canvas.rect();
        let inner = shrink(rect, 2);
        let (r, g, b, a) = state.palette()[self.color];
        if a < u8::MAX {
            canvas.draw_rect((0, 0, 0, 255), inner);
            canvas.draw_rect((0, 0, 0, 255), shrink(inner, 2));
            canvas.draw_rect((0, 0, 0, 255), shrink(inner, 4));
        }
        if a > 0 {
            canvas.fill_rect((r, g, b, a), inner);
        }
        if state.color() == self.color {
            canvas.draw_rect((255, 255, 255, 255), rect);
        }
    }

    fn on_event(
        &mut self,
        event: &Event,
        state: &mut EditorState,
    ) -> Action<PaletteAction> {
        match event {
            &Event::ClockTick => {
                if self.double_click_counter > 0 {
                    self.double_click_counter -= 1;
                }
            }
            &Event::MouseDown(_) => {
                if self.double_click_counter > 0 {
                    return Action::redraw()
                        .and_return(PaletteAction::EditColor(self.color));
                } else {
                    self.double_click_counter = 4;
                    self.pick_color(state);
                    return Action::redraw().and_stop();
                }
            }
            &Event::KeyDown(key, kmod) => {
                if key == self.key && kmod == NONE {
                    self.pick_color(state);
                    return Action::redraw().and_stop();
                }
            }
            _ => {}
        }
        Action::ignore()
    }
}

//===========================================================================//

struct NextPrevPalette {
    delta: i32,
    key: Keycode,
}

impl NextPrevPalette {
    const WIDTH: u32 = 18;
    const HEIGHT: u32 = 18;

    fn new(delta: i32, key: Keycode) -> NextPrevPalette {
        NextPrevPalette { delta, key }
    }

    fn increment(&self, state: &mut EditorState) -> Action<PaletteAction> {
        let new_index = ((state.palette_index() as i32) + self.delta)
            .rem_euclid((state.num_palettes() as i32) + 1);
        state.set_palette_index(new_index as usize);
        Action::redraw().and_stop()
    }
}

impl GuiElement<EditorState, PaletteAction> for NextPrevPalette {
    fn draw(
        &self,
        state: &EditorState,
        resources: &Resources,
        canvas: &mut Canvas,
    ) {
        if state.num_palettes() > 0 {
            let icon = if self.delta > 0 {
                resources.tool_icon(ToolIcon::ArrowRight)
            } else {
                resources.tool_icon(ToolIcon::ArrowLeft)
            };
            canvas.draw_sprite(icon, Point::new(1, 1));
        }
    }

    fn on_event(
        &mut self,
        event: &Event,
        state: &mut EditorState,
    ) -> Action<PaletteAction> {
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

struct AddPalettteButton {}

impl AddPalettteButton {
    fn new() -> AddPalettteButton {
        AddPalettteButton {}
    }
}

impl GuiElement<EditorState, PaletteAction> for AddPalettteButton {
    fn draw(
        &self,
        _: &EditorState,
        resources: &Resources,
        canvas: &mut Canvas,
    ) {
        let icon = resources.tool_icon(ToolIcon::AddPalette);
        canvas.draw_sprite(icon, Point::new(1, 1));
    }

    fn on_event(
        &mut self,
        event: &Event,
        state: &mut EditorState,
    ) -> Action<PaletteAction> {
        match event {
            &Event::MouseDown(_) => {
                state.mutation().add_new_palette();
                Action::redraw().and_stop()
            }
            _ => Action::ignore(),
        }
    }
}

//===========================================================================//

struct DeletePalettteButton {}

impl DeletePalettteButton {
    fn new() -> DeletePalettteButton {
        DeletePalettteButton {}
    }
}

impl GuiElement<EditorState, PaletteAction> for DeletePalettteButton {
    fn draw(
        &self,
        state: &EditorState,
        resources: &Resources,
        canvas: &mut Canvas,
    ) {
        if state.palette_index() < state.num_palettes() {
            let icon = resources.tool_icon(ToolIcon::DeletePalette);
            canvas.draw_sprite(icon, Point::new(1, 1));
        }
    }

    fn on_event(
        &mut self,
        event: &Event,
        state: &mut EditorState,
    ) -> Action<PaletteAction> {
        match event {
            &Event::MouseDown(_) => {
                state.mutation().delete_palette();
                Action::redraw().and_stop()
            }
            _ => Action::ignore(),
        }
    }
}

//===========================================================================//

pub struct PaletteInfoView {
    left: i32,
    top: i32,
}

impl PaletteInfoView {
    const HEIGHT: u32 = 18;

    fn new(left: i32, top: i32) -> PaletteInfoView {
        PaletteInfoView { left, top }
    }
}

impl GuiElement<EditorState, PaletteAction> for PaletteInfoView {
    fn draw(
        &self,
        state: &EditorState,
        resources: &Resources,
        canvas: &mut Canvas,
    ) {
        let palette_index = state.palette_index();
        let num_palettes = state.num_palettes();
        if palette_index < num_palettes {
            canvas.draw_string(
                resources.font(),
                self.left + 26,
                self.top + 2,
                &format!("{}/{}", palette_index, num_palettes),
            );
        } else {
            canvas.draw_string(
                resources.font(),
                self.left + 13,
                self.top + 2,
                &format!("def/{}", num_palettes),
            );
        }
    }

    fn on_event(
        &mut self,
        _: &Event,
        _: &mut EditorState,
    ) -> Action<PaletteAction> {
        Action::ignore()
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
