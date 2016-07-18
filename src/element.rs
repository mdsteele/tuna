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

use sdl2::rect::Rect;
use super::canvas::Canvas;
use super::event::Event;

// ========================================================================= //

pub trait GuiElement<S> {
    fn draw(&self, state: &S, canvas: &mut Canvas);
    fn handle_event(&mut self, event: &Event, state: &mut S) -> bool;
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

    pub fn rect(&self) -> Rect {
        self.subrect
    }
}

impl<E, S> GuiElement<S> for SubrectElement<E>
    where E: GuiElement<S>
{
    fn draw(&self, state: &S, canvas: &mut Canvas) {
        let mut subcanvas = canvas.subcanvas(self.subrect);
        self.element.draw(state, &mut subcanvas);
    }

    fn handle_event(&mut self, event: &Event, state: &mut S) -> bool {
        match event {
            &Event::MouseDown(pt) => {
                if !self.subrect.contains(pt) {
                    return false;
                }
            }
            _ => {}
        }
        let event = event.translate(-self.subrect.x(), -self.subrect.y());
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

    fn handle_event(&mut self, event: &Event, state: &mut S) -> bool {
        let mut result = false;
        for element in self.elements.iter_mut() {
            result |= element.handle_event(event, state);
        }
        result
    }
}

// ========================================================================= //
