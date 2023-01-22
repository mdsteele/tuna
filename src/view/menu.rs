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
use crate::event::{Event, Keycode};
use crate::state::EditorState;
use sdl2::rect::Rect;

//===========================================================================//

#[derive(Clone, Copy)]
pub enum MenuAction {
    FlipHorz,
    FlipVert,
    Resize,
    RotateLeft,
    RotateRight,
}

impl MenuAction {
    pub fn label(&self) -> &'static str {
        match *self {
            MenuAction::FlipHorz => "Flip horizontally",
            MenuAction::FlipVert => "Flip vertically",
            MenuAction::Resize => "Resize images",
            MenuAction::RotateLeft => "Rotate left (CCW)",
            MenuAction::RotateRight => "Rotate right (CW)",
        }
    }

    pub fn shortcut(&self) -> &'static str {
        match *self {
            MenuAction::FlipHorz => "CS-H",
            MenuAction::FlipVert => "CS-V",
            MenuAction::Resize => "C-R",
            MenuAction::RotateLeft => "CS-L",
            MenuAction::RotateRight => "CS-R",
        }
    }

    pub fn all() -> Vec<MenuAction> {
        vec![
            MenuAction::FlipHorz,
            MenuAction::FlipVert,
            MenuAction::Resize,
            MenuAction::RotateLeft,
            MenuAction::RotateRight,
        ]
    }
}

//===========================================================================//

pub struct MenuView {
    button: SubrectElement<MenuButton>,
    items: MenuItems,
    is_open: bool,
}

impl MenuView {
    pub fn new(left: i32, top: i32) -> MenuView {
        let button_rect = Rect::new(left, top, 60, 18);
        let button = SubrectElement::new(MenuButton::new(), button_rect);
        let items = MenuItems::new(left, top - 2);
        MenuView { button, items, is_open: false }
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }
}

impl GuiElement<EditorState, MenuAction> for MenuView {
    fn draw(
        &self,
        _: &EditorState,
        resources: &Resources,
        canvas: &mut Canvas,
    ) {
        self.button.draw(&(), resources, canvas);
        if self.is_open {
            self.items.draw(&(), resources, canvas);
        }
    }

    fn on_event(
        &mut self,
        event: &Event,
        _: &mut EditorState,
    ) -> Action<MenuAction> {
        let mut action = Action::ignore();
        match event {
            &Event::KeyDown(Keycode::Escape, _) => {
                if self.is_open {
                    self.close();
                    action.merge(Action::redraw().and_stop());
                }
            }
            _ => {}
        }
        if !action.should_stop() && self.is_open {
            let subaction = self.items.on_event(event, &mut ());
            if subaction.has_value() {
                self.close();
            }
            action.merge(subaction);
        }
        if !action.should_stop() {
            let mut subaction = self.button.on_event(event, &mut ());
            if let Some(()) = subaction.take_value() {
                self.is_open = !self.is_open;
                subaction.also_redraw();
            }
            action.merge(subaction.but_no_value());
        }
        action
    }
}

//===========================================================================//

struct MenuButton {}

impl MenuButton {
    pub fn new() -> MenuButton {
        MenuButton {}
    }
}

impl GuiElement<(), ()> for MenuButton {
    fn draw(&self, _: &(), resources: &Resources, canvas: &mut Canvas) {
        let rect = canvas.rect();
        let width = rect.width() as i32;
        let font = resources.font();
        let text = "Menu";
        let text_width = font.text_width(text);
        canvas.fill_rect((160, 160, 160, 255), rect);
        canvas.draw_string(font, (width - text_width) / 2, 4, text);
        canvas.draw_rect((128, 128, 128, 255), rect);
    }

    fn on_event(&mut self, event: &Event, _: &mut ()) -> Action<()> {
        match event {
            &Event::MouseDown(_) => Action::ignore().and_return(()),
            _ => Action::ignore(),
        }
    }
}

//===========================================================================//

struct MenuItems {
    items: AggregateElement<(), MenuAction>,
    rect: Rect,
}

impl MenuItems {
    const MARGIN: u32 = 4;
    const WIDTH: u32 = 200;
    const ITEM_WIDTH: u32 = MenuItems::WIDTH - MenuItems::MARGIN * 2;
    const ITEM_HEIGHT: u32 = 14;

    fn new(left: i32, bottom: i32) -> MenuItems {
        let items = AggregateElement::new(
            MenuAction::all()
                .into_iter()
                .enumerate()
                .map(|(row, action)| MenuItems::item(row, action))
                .collect(),
        );
        let width = MenuItems::WIDTH;
        let height =
            MenuItems::MARGIN + MenuItems::ITEM_HEIGHT * (items.len() as u32);
        let top = bottom - (height as i32);
        let rect = Rect::new(left, top, width, height);
        MenuItems { items, rect }
    }

    fn item(
        row: usize,
        action: MenuAction,
    ) -> Box<dyn GuiElement<(), MenuAction>> {
        let rect = Rect::new(
            MenuItems::MARGIN as i32,
            (MenuItems::MARGIN as i32)
                + (MenuItems::ITEM_HEIGHT as i32) * (row as i32),
            MenuItems::ITEM_WIDTH,
            MenuItems::ITEM_HEIGHT,
        );
        Box::new(SubrectElement::new(MenuItem::new(action), rect))
    }
}

impl GuiElement<(), MenuAction> for MenuItems {
    fn draw(&self, state: &(), resources: &Resources, canvas: &mut Canvas) {
        canvas.fill_rect((128, 128, 128, 255), self.rect);
        canvas.draw_rect((255, 255, 255, 255), self.rect);
        let mut subcanvas = canvas.subcanvas(self.rect);
        self.items.draw(state, resources, &mut subcanvas);
    }

    fn on_event(
        &mut self,
        event: &Event,
        state: &mut (),
    ) -> Action<MenuAction> {
        let mut action = self.items.on_event(
            &event.translate(-self.rect.left(), -self.rect.top()),
            state,
        );
        if !action.should_stop() {
            match event {
                &Event::MouseDrag(pt) | &Event::MouseDown(pt) => {
                    if self.rect.contains_point(pt) {
                        action = action.and_stop();
                    }
                }
                _ => {}
            }
        }
        action
    }
}

//===========================================================================//

struct MenuItem {
    action: MenuAction,
}

impl MenuItem {
    pub fn new(action: MenuAction) -> MenuItem {
        MenuItem { action }
    }
}

impl GuiElement<(), MenuAction> for MenuItem {
    fn draw(&self, _: &(), resources: &Resources, canvas: &mut Canvas) {
        let font = resources.font();
        canvas.draw_string(font, 0, 0, self.action.label());
        let shortcut = self.action.shortcut();
        let shortcut_width = font.text_width(shortcut);
        let shortcut_left = (canvas.rect().width() as i32) - shortcut_width;
        canvas.draw_string(font, shortcut_left, 0, shortcut);
    }

    fn on_event(&mut self, event: &Event, _: &mut ()) -> Action<MenuAction> {
        match event {
            &Event::MouseDown(_) => Action::ignore().and_return(self.action),
            _ => Action::ignore(),
        }
    }
}

//===========================================================================//
