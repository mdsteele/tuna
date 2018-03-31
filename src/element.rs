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

use super::canvas::Canvas;
use super::event::Event;
use sdl2::rect::Rect;

// ========================================================================= //

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Action {
    redraw: bool,
    stop: bool,
}

impl Action {
    pub fn ignore() -> ActionBuilder { ActionBuilder { redraw: false } }

    pub fn redraw() -> ActionBuilder { ActionBuilder { redraw: true } }

    pub fn redraw_if(should_redraw: bool) -> ActionBuilder {
        ActionBuilder { redraw: should_redraw }
    }

    pub fn should_redraw(&self) -> bool { self.redraw }

    pub fn should_stop(&self) -> bool { self.stop }

    fn merge(&mut self, action: Action) {
        self.redraw |= action.redraw;
        self.stop |= action.stop;
    }
}

#[derive(Clone, Copy)]
pub struct ActionBuilder {
    redraw: bool,
}

impl ActionBuilder {
    pub fn and_continue(&self) -> Action {
        Action {
            redraw: self.redraw,
            stop: false,
        }
    }

    pub fn and_stop(&self) -> Action {
        Action {
            redraw: self.redraw,
            stop: true,
        }
    }
}

// ========================================================================= //

pub trait GuiElement<S> {
    fn draw(&self, state: &S, canvas: &mut Canvas);
    fn handle_event(&mut self, event: &Event, state: &mut S) -> Action;
}

// ========================================================================= //

pub struct SubrectElement<E> {
    subrect: Rect,
    element: E,
}

impl<E> SubrectElement<E> {
    pub fn new(element: E, subrect: Rect) -> SubrectElement<E> {
        SubrectElement {
            subrect: subrect,
            element: element,
        }
    }

    pub fn rect(&self) -> Rect { self.subrect }
}

impl<E, S> GuiElement<S> for SubrectElement<E>
where
    E: GuiElement<S>,
{
    fn draw(&self, state: &S, canvas: &mut Canvas) {
        let mut subcanvas = canvas.subcanvas(self.subrect);
        self.element.draw(state, &mut subcanvas);
    }

    fn handle_event(&mut self, event: &Event, state: &mut S) -> Action {
        match event {
            &Event::MouseDown(pt) => {
                if !self.subrect.contains_point(pt) {
                    return Action::ignore().and_continue();
                }
            }
            _ => {}
        }
        let dx = self.subrect.x();
        let dy = self.subrect.y();
        let event = event.translate(-dx, -dy);
        self.element.handle_event(&event, state)
    }
}

// ========================================================================= //

pub struct AggregateElement<S> {
    elements: Vec<Box<GuiElement<S>>>,
}

impl<S> AggregateElement<S> {
    pub fn new(elements: Vec<Box<GuiElement<S>>>) -> AggregateElement<S> {
        AggregateElement { elements: elements }
    }
}

impl<S> GuiElement<S> for AggregateElement<S> {
    fn draw(&self, state: &S, canvas: &mut Canvas) {
        for element in self.elements.iter().rev() {
            element.draw(state, canvas);
        }
    }

    fn handle_event(&mut self, event: &Event, state: &mut S) -> Action {
        let mut action = Action::ignore().and_continue();
        for element in self.elements.iter_mut() {
            action.merge(element.handle_event(event, state));
            if action.should_stop() {
                break;
            }
        }
        action
    }
}

// ========================================================================= //
