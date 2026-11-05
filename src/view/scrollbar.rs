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
use crate::element::{Action, GuiElement};
use crate::event::Event;
use crate::state::{EditorState, MAIN_CANVAS_SIZE};
use crate::util::Rgba;
use sdl2::rect::{Point, Rect};

//===========================================================================//

const SCROLLBAR_BACKGROUND_RGBA: Rgba = (95, 95, 95, 255);
const SCROLLBAR_DRAGGING_RGBA: Rgba = (128, 255, 255, 255);
const SCROLLBAR_ENABLED_RGBA: Rgba = (96, 192, 192, 255);
const SCROLLBAR_DISABLED_RGBA: Rgba = (64, 128, 128, 255);

//===========================================================================//

struct ScrollbarDrag {
    from_scroll: i32,
    from_point: Point,
    to_point: Point,
}

//===========================================================================//

pub struct Scrollbar {
    rect: Rect,
    vert: bool,
    drag: Option<ScrollbarDrag>,
}

impl Scrollbar {
    pub fn new(
        left: i32,
        top: i32,
        width: u32,
        height: u32,
        vert: bool,
    ) -> Self {
        Self { rect: Rect::new(left, top, width, height), vert, drag: None }
    }

    fn full_length(&self) -> u32 {
        if self.vert { self.rect.height() } else { self.rect.width() }
    }

    fn scroll(&self, state: &EditorState) -> i32 {
        let (x, y) = state.scroll();
        if self.vert { y } else { x }
    }

    fn set_scroll(&mut self, state: &mut EditorState, new: i32) {
        let (x, y) = state.scroll();
        state.set_scroll(if self.vert { (x, new) } else { (new, y) });
    }

    fn zoom(&self, state: &EditorState) -> u32 {
        let (horz, vert) = state.zoom();
        if self.vert { vert } else { horz }
    }

    fn image_size(&self, state: &EditorState) -> u32 {
        let (width, height) = state.image_size();
        if self.vert { height } else { width }
    }

    fn handle_rect(&self, state: &EditorState) -> Option<Rect> {
        let size = self.image_size(state);
        let zoom = self.zoom(state);
        if size * zoom <= MAIN_CANVAS_SIZE {
            return None;
        }
        let full_length = self.full_length();
        let handle_length =
            (full_length * MAIN_CANVAS_SIZE / (size * zoom)).max(1);
        let handle_offset =
            full_length as i32 * self.scroll(state) / size as i32;
        let handle_rect = if self.vert {
            Rect::new(
                self.rect.x(),
                self.rect.y() + handle_offset,
                self.rect.width(),
                handle_length,
            )
        } else {
            Rect::new(
                self.rect.x() + handle_offset,
                self.rect.y(),
                handle_length,
                self.rect.height(),
            )
        };
        Some(handle_rect)
    }
}

impl GuiElement<EditorState, ()> for Scrollbar {
    fn draw(&self, state: &EditorState, _: &Resources, canvas: &mut Canvas) {
        canvas.fill_rect(SCROLLBAR_BACKGROUND_RGBA, self.rect);
        if let Some(rect) = self.handle_rect(state) {
            let rgba = if self.drag.is_some() {
                SCROLLBAR_DRAGGING_RGBA
            } else {
                SCROLLBAR_ENABLED_RGBA
            };
            canvas.draw_rect(rgba, rect);
        } else {
            canvas.draw_rect(SCROLLBAR_DISABLED_RGBA, self.rect);
        }
    }

    fn on_event(
        &mut self,
        event: &Event,
        state: &mut EditorState,
    ) -> Action<()> {
        match *event {
            Event::MouseDown(pt)
                if self.rect.contains_point(pt)
                    && let Some(handle_rect) = self.handle_rect(state) =>
            {
                if handle_rect.contains_point(pt) {
                    self.drag = Some(ScrollbarDrag {
                        from_scroll: self.scroll(state),
                        from_point: pt,
                        to_point: pt,
                    });
                    return Action::redraw().and_stop();
                }
                self.drag = None;
                let delta =
                    (MAIN_CANVAS_SIZE / self.zoom(state)).max(1) as i32;
                if pt.x() < handle_rect.x() || pt.y() < handle_rect.y() {
                    self.set_scroll(state, self.scroll(state) - delta);
                } else {
                    self.set_scroll(state, self.scroll(state) + delta);
                }
                return Action::redraw().and_stop();
            }
            Event::MouseDrag(pt) if let Some(mut drag) = self.drag.take() => {
                drag.to_point = pt;
                let drag_delta = if self.vert {
                    drag.to_point.y() - drag.from_point.y()
                } else {
                    drag.to_point.x() - drag.from_point.x()
                };
                let image_size = self.image_size(state) as i32;
                let full_length = self.full_length() as i32;
                let scroll_delta = drag_delta * image_size / full_length;
                self.set_scroll(state, drag.from_scroll + scroll_delta);
                self.drag = Some(drag);
                return Action::redraw();
            }
            Event::MouseUp if self.drag.is_some() => {
                self.drag = None;
                return Action::redraw();
            }
            _ => {}
        }
        Action::ignore()
    }
}

//===========================================================================//
