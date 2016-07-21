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
use std::cmp;
use super::canvas::Canvas;
use super::element::{Action, GuiElement};
use super::event::{Event, Keycode};
use super::state::{EditorState, Tool};
use super::util;

// ========================================================================= //

struct ImageCanvasDrag {
    from_selection: (i32, i32),
    from_pixel: (i32, i32),
    to_pixel: (i32, i32),
}

pub struct ImageCanvas {
    left: i32,
    top: i32,
    max_size: u32,
    drag_from_to: Option<ImageCanvasDrag>,
}

impl ImageCanvas {
    pub fn new(left: i32, top: i32, max_size: u32) -> ImageCanvas {
        ImageCanvas {
            left: left,
            top: top,
            max_size: max_size,
            drag_from_to: None,
        }
    }

    fn scale(&self, state: &EditorState) -> u32 {
        let (width, height) = state.image_size();
        cmp::max(1, self.max_size / cmp::max(width, height))
    }

    fn rect(&self, state: &EditorState) -> Rect {
        let scale = self.scale(state);
        let (width, height) = state.image_size();
        Rect::new(self.left, self.top, width * scale, height * scale)
    }

    fn dragged_points(&self,
                      state: &EditorState)
                      -> Option<((u32, u32), (u32, u32))> {
        if let Some(ref drag) = self.drag_from_to {
            let (fpx, fpy) = drag.from_pixel;
            let (tpx, tpy) = drag.to_pixel;
            let from_point = self.clamp_mouse_to_row_col(fpx, fpy, state);
            let to_point = self.clamp_mouse_to_row_col(tpx, tpy, state);
            Some((from_point, to_point))
        } else {
            None
        }
    }

    fn dragged_rect(&self, state: &EditorState) -> Option<Rect> {
        if let Some(((from_col, from_row), (to_col, to_row))) =
               self.dragged_points(state) {
            let x = cmp::min(from_col, to_col) as i32;
            let y = cmp::min(from_row, to_row) as i32;
            let w = ((from_col as i32 - to_col as i32).abs() + 1) as u32;
            let h = ((from_row as i32 - to_row as i32).abs() + 1) as u32;
            Some(Rect::new(x, y, w, h))
        } else {
            None
        }
    }

    fn mouse_to_row_col(&self,
                        x: i32,
                        y: i32,
                        state: &EditorState)
                        -> Option<(u32, u32)> {
        if x < self.left || y < self.top {
            return None;
        }
        let scale = self.scale(state) as i32;
        let col = (x - self.left) / scale;
        let row = (y - self.top) / scale;
        let (width, height) = state.image_size();
        if col < 0 || col >= (width as i32) || row < 0 ||
           row >= (height as i32) {
            None
        } else {
            Some((col as u32, row as u32))
        }
    }

    fn clamp_mouse_to_row_col(&self,
                              x: i32,
                              y: i32,
                              state: &EditorState)
                              -> (u32, u32) {
        let scale = self.scale(state) as i32;
        let col = (x - self.left) / scale;
        let row = (y - self.top) / scale;
        let (width, height) = state.image_size();
        (cmp::max(0, cmp::min(col, width as i32 - 1)) as u32,
         cmp::max(0, cmp::min(row, height as i32 - 1)) as u32)
    }

    fn try_paint(&self, x: i32, y: i32, state: &mut EditorState) -> bool {
        if let Some((col, row)) = self.mouse_to_row_col(x, y, state) {
            state.image_mut()[(col, row)] = state.color();
            true
        } else {
            false
        }
    }

    fn try_eyedrop(&self, x: i32, y: i32, state: &mut EditorState) -> bool {
        if let Some(position) = self.mouse_to_row_col(x, y, state) {
            state.eyedrop_at(position);
            true
        } else {
            false
        }
    }

    fn try_draw_line(&mut self, state: &mut EditorState) -> bool {
        if let Some(((col1, row1), (col2, row2))) =
               self.dragged_points(state) {
            state.push_change();
            for coords in bresenham_points(col1, row1, col2, row2) {
                state.image_mut()[coords] = state.color();
            }
            self.drag_from_to = None;
            return true;
        }
        false
    }

    fn try_flood_fill(&self, x: i32, y: i32, state: &mut EditorState) -> bool {
        if let Some((col, row)) = self.mouse_to_row_col(x, y, state) {
            let to_color = state.color();
            let image = state.image_mut();
            let width = image.width();
            let height = image.height();
            let from_color = image[(col, row)];
            if from_color == to_color {
                return false;
            }
            image[(col, row)] = to_color;
            let mut stack: Vec<(u32, u32)> = vec![(col, row)];
            while let Some((col, row)) = stack.pop() {
                let mut next: Vec<(u32, u32)> = vec![];
                if col > 0 {
                    next.push((col - 1, row));
                }
                if col < width - 1 {
                    next.push((col + 1, row));
                }
                if row > 0 {
                    next.push((col, row - 1));
                }
                if row < height - 1 {
                    next.push((col, row + 1));
                }
                for coords in next {
                    if image[coords] == from_color {
                        image[coords] = to_color;
                        stack.push(coords);
                    }
                }
            }
            true
        } else {
            false
        }
    }
}

impl GuiElement<EditorState> for ImageCanvas {
    fn draw(&self, state: &EditorState, canvas: &mut Canvas) {
        let scale = self.scale(state);
        let canvas_rect = self.rect(state);
        canvas.draw_rect((255, 255, 255, 255), expand(canvas_rect, 2));
        let mut canvas = canvas.subcanvas(canvas_rect);
        util::render_image(&mut canvas, state.image(), 0, 0, scale);
        if let Some((ref selected, topleft)) = state.selection() {
            let left = topleft.x() * (scale as i32);
            let top = topleft.y() * (scale as i32);
            util::render_image(&mut canvas, selected, left, top, scale);
            canvas.draw_rect((255, 191, 255, 255),
                             Rect::new(left,
                                       top,
                                       selected.width() * scale,
                                       selected.height() * scale));
        } else if state.tool() == Tool::Line {
            if let Some(((col1, row1), (col2, row2))) =
                   self.dragged_points(state) {
                for (x, y) in bresenham_points(col1, row1, col2, row2) {
                    canvas.draw_rect((191, 191, 191, 255),
                                     Rect::new((x * scale) as i32,
                                               (y * scale) as i32,
                                               scale,
                                               scale));
                }
            }
        } else if let Some(rect) = self.dragged_rect(state) {
            canvas.draw_rect((255, 255, 191, 255),
                             Rect::new(rect.x() * (scale as i32),
                                       rect.y() * (scale as i32),
                                       rect.width() * scale,
                                       rect.height() * scale));
        }
    }

    fn handle_event(&mut self,
                    event: &Event,
                    state: &mut EditorState)
                    -> Action {
        match event {
            &Event::KeyDown(Keycode::Escape, _) => {
                if state.try_unselect_with_undo() {
                    return Action::redraw().and_stop();
                } else {
                    return Action::ignore().and_continue();
                }
            }
            &Event::MouseDown(pt) => {
                if self.rect(state).contains(pt) {
                    let (x, y) = (pt.x(), pt.y());
                    match state.tool() {
                        Tool::Eyedropper => {
                            let changed = self.try_eyedrop(x, y, state);
                            return Action::redraw_if(changed).and_stop();
                        }
                        Tool::Line => {
                            self.drag_from_to = Some(ImageCanvasDrag {
                                from_selection: (0, 0),
                                from_pixel: (x, y),
                                to_pixel: (x, y),
                            });
                            return Action::redraw().and_stop();
                        }
                        Tool::PaintBucket => {
                            state.push_change();
                            let changed = self.try_flood_fill(x, y, state);
                            return Action::redraw_if(changed).and_stop();
                        }
                        Tool::Pencil => {
                            state.push_change();
                            let changed = self.try_paint(x, y, state);
                            return Action::redraw_if(changed).and_stop();
                        }
                        Tool::Select => {
                            let rect = if let Some((ref selected, topleft)) =
                                              state.selection() {
                                Some(Rect::new(topleft.x(),
                                               topleft.y(),
                                               selected.width(),
                                               selected.height()))
                            } else {
                                None
                            };
                            if let Some(rect) = rect {
                                let scale = self.scale(state);
                                if !Rect::new(self.left +
                                              rect.x() * (scale as i32),
                                              self.top +
                                              rect.y() * (scale as i32),
                                              rect.width() * scale,
                                              rect.height() * scale)
                                        .contains((x, y)) {
                                    state.try_unselect_with_undo();
                                } else {
                                    state.push_change();
                                }
                            }
                            self.drag_from_to = Some(ImageCanvasDrag {
                                from_selection: if let Some(r) = rect {
                                    (r.x(), r.y())
                                } else {
                                    (0, 0)
                                },
                                from_pixel: (x, y),
                                to_pixel: (x, y),
                            });
                            return Action::redraw().and_stop();
                        }
                    }
                } else {
                    self.drag_from_to = None;
                }
            }
            &Event::MouseUp => {
                match state.tool() {
                    Tool::Line => {
                        let changed = self.try_draw_line(state);
                        return Action::redraw_if(changed).and_continue();
                    }
                    Tool::Select => {
                        if state.selection().is_none() {
                            if let Some(rect) = self.dragged_rect(state) {
                                state.select_with_undo(&rect);
                                self.drag_from_to = None;
                                return Action::redraw().and_continue();
                            }
                        }
                    }
                    _ => {}
                }
                self.drag_from_to = None;
            }
            &Event::MouseDrag(pt) => {
                let (x, y) = (pt.x(), pt.y());
                match state.tool() {
                    Tool::Line => {
                        if let Some(ref mut drag) = self.drag_from_to {
                            drag.to_pixel = (x, y);
                            return Action::redraw().and_continue();
                        }
                    }
                    Tool::Pencil => {
                        let changed = self.try_paint(x, y, state);
                        return Action::redraw_if(changed).and_continue();
                    }
                    Tool::Select => {
                        let scale = self.scale(state) as i32;
                        if let Some(ref mut drag) = self.drag_from_to {
                            drag.to_pixel = (x, y);
                            if state.selection().is_some() {
                                let (fsx, fsy) = drag.from_selection;
                                let (fpx, fpy) = drag.from_pixel;
                                state.reposition_selection(fsx +
                                                           (x - fpx) / scale,
                                                           fsy +
                                                           (y - fpy) / scale);
                            }
                            return Action::redraw().and_continue();
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        return Action::ignore().and_continue();
    }
}

// ========================================================================= //

fn bresenham_points(x1: u32, y1: u32, x2: u32, y2: u32) -> Vec<(u32, u32)> {
    let (x1, y1, x2, y2) = (x1 as i32, y1 as i32, x2 as i32, y2 as i32);
    let dx = (x2 - x1).abs();
    let dy = (y2 - y1).abs();
    let steep = dy > dx;
    let (dx, dy, x1, y1, x2, y2) = if steep {
        (dy, dx, y1, x1, y2, x2)
    } else {
        (dx, dy, x1, y1, x2, y2)
    };
    let reversed = x1 > x2;
    let (x1, y1, x2, y2) = if reversed {
        (x2, y2, x1, y1)
    } else {
        (x1, y1, x2, y2)
    };
    let y_step = if y1 < y2 {
        1
    } else {
        -1
    };
    let mut x = x1;
    let mut y = y1;
    let mut err = dx / 2;
    let mut output = vec![];
    while x <= x2 {
        output.push(if steep {
            (y as u32, x as u32)
        } else {
            (x as u32, y as u32)
        });
        x += 1;
        err -= dy;
        if err < 0 {
            y += y_step;
            err += dx;
        }
    }
    if reversed {
        output.reverse();
    }
    output
}

fn expand(rect: Rect, by: i32) -> Rect {
    Rect::new(rect.x() - by,
              rect.y() - by,
              ((rect.width() as i32) + 2 * by) as u32,
              ((rect.height() as i32) + 2 * by) as u32)
}

// ========================================================================= //
