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

use super::canvas::{Canvas, Resources};
use super::event::Event;
use sdl2::rect::Rect;
use std::mem;

//===========================================================================//

#[derive(Debug, Eq, PartialEq)]
enum Value<A> {
    Continue,
    Stop,
    Return(A),
}

impl<A> Value<A> {
    fn merge(&mut self, other: Value<A>) {
        match other {
            Value::Continue => {}
            Value::Stop => match *self {
                Value::Continue => *self = other,
                _ => {}
            },
            Value::Return(_) => {
                *self = other;
            }
        }
    }
}

//===========================================================================//

pub struct Action<A> {
    redraw: bool,
    value: Value<A>,
}

impl<A> Action<A> {
    pub fn ignore() -> Action<A> {
        Action { redraw: false, value: Value::Continue }
    }

    pub fn redraw() -> Action<A> {
        Action { redraw: true, value: Value::Continue }
    }

    pub fn redraw_if(redraw: bool) -> Action<A> {
        Action { redraw, value: Value::Continue }
    }

    pub fn also_redraw(&mut self) {
        self.redraw = true;
    }

    pub fn and_stop(mut self) -> Action<A> {
        self.value = Value::Stop;
        self
    }

    pub fn and_return(mut self, value: A) -> Action<A> {
        self.value = Value::Return(value);
        self
    }

    pub fn but_no_value<B>(self) -> Action<B> {
        Action {
            redraw: self.redraw,
            value: match self.value {
                Value::Continue => Value::Continue,
                _ => Value::Stop,
            },
        }
    }

    pub fn should_redraw(&self) -> bool {
        self.redraw
    }

    pub fn should_stop(&self) -> bool {
        match self.value {
            Value::Continue => false,
            _ => true,
        }
    }

    pub fn take_value(&mut self) -> Option<A> {
        match self.value {
            Value::Continue | Value::Stop => return None,
            Value::Return(_) => {}
        }
        match mem::replace(&mut self.value, Value::Stop) {
            Value::Return(value) => Some(value),
            _ => unreachable!(),
        }
    }

    pub fn merge(&mut self, action: Action<A>) {
        self.redraw |= action.redraw;
        self.value.merge(action.value);
    }
}

//===========================================================================//

pub trait GuiElement<S, A> {
    fn draw(&self, state: &S, resources: &Resources, canvas: &mut Canvas);
    fn on_event(&mut self, event: &Event, state: &mut S) -> Action<A>;
}

//===========================================================================//

pub struct SubrectElement<E> {
    subrect: Rect,
    element: E,
}

impl<E> SubrectElement<E> {
    pub fn new(element: E, subrect: Rect) -> SubrectElement<E> {
        SubrectElement { subrect, element }
    }

    pub fn rect(&self) -> Rect {
        self.subrect
    }

    pub fn inner(&self) -> &E {
        &self.element
    }

    pub fn inner_mut(&mut self) -> &mut E {
        &mut self.element
    }
}

impl<E, S, A> GuiElement<S, A> for SubrectElement<E>
where
    E: GuiElement<S, A>,
{
    fn draw(&self, state: &S, resources: &Resources, canvas: &mut Canvas) {
        let mut subcanvas = canvas.subcanvas(self.subrect);
        self.element.draw(state, resources, &mut subcanvas);
    }

    fn on_event(&mut self, event: &Event, state: &mut S) -> Action<A> {
        match event {
            &Event::MouseDown(pt) => {
                if !self.subrect.contains_point(pt) {
                    return Action::ignore();
                }
            }
            _ => {}
        }
        let dx = self.subrect.x();
        let dy = self.subrect.y();
        let event = event.translate(-dx, -dy);
        self.element.on_event(&event, state)
    }
}

//===========================================================================//

pub struct AggregateElement<S, A> {
    elements: Vec<Box<dyn GuiElement<S, A>>>,
}

impl<S, A> AggregateElement<S, A> {
    pub fn new(
        elements: Vec<Box<dyn GuiElement<S, A>>>,
    ) -> AggregateElement<S, A> {
        AggregateElement { elements }
    }
}

impl<S, A> GuiElement<S, A> for AggregateElement<S, A> {
    fn draw(&self, state: &S, resources: &Resources, canvas: &mut Canvas) {
        for element in self.elements.iter().rev() {
            element.draw(state, resources, canvas);
        }
    }

    fn on_event(&mut self, event: &Event, state: &mut S) -> Action<A> {
        let mut action = Action::ignore();
        for element in self.elements.iter_mut() {
            action.merge(element.on_event(event, state));
            if action.should_stop() {
                break;
            }
        }
        action
    }
}

//===========================================================================//
