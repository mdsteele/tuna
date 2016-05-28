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

use sdl2::event::Event;
use sdl2::rect::Rect;
use super::canvas::Canvas;

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
        if event_outside_rect(event, self.subrect) {
            return false;
        }
        let event = translate_event(event, self.subrect.x(), self.subrect.y());
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

fn event_outside_rect(event: &Event, rect: Rect) -> bool {
    let maybe_pt = match *event {
        Event::MouseButtonDown { x, y, .. } => Some((x, y)),
        _ => None,
    };
    if let Some(pt) = maybe_pt {
        !rect.contains(pt)
    } else {
        false
    }
}

fn translate_event(event: &Event, x_offset: i32, y_offset: i32) -> Event {
    let mut event = event.clone();
    match event {
        Event::MouseButtonDown { ref mut x, ref mut y, .. } => {
            *x -= x_offset;
            *y -= y_offset;
        }
        Event::MouseButtonUp { ref mut x, ref mut y, .. } => {
            *x -= x_offset;
            *y -= y_offset;
        }
        Event::MouseMotion { ref mut x, ref mut y, .. } => {
            *x -= x_offset;
            *y -= y_offset;
        }
        Event::MouseWheel { ref mut x, ref mut y, .. } => {
            *x -= x_offset;
            *y -= y_offset;
        }
        _ => {}
    }
    event
}

// ========================================================================= //
