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

mod draw;
mod mutate;
mod shape;

use self::shape::Shape;
use crate::canvas::{Canvas, Resources};
use crate::element::{Action, GuiElement};
use crate::event::{Event, Keycode};
use crate::state::{EditorState, Tool};
use crate::util::Rgba;
use num_integer::{gcd, mod_floor};
use sdl2::rect::{Point, Rect};

//===========================================================================//

const LASSO_PREVIEW_RGBA: Rgba = (192, 192, 64, 255);
const SELECTION_LABEL_RGBA: Rgba = (255, 255, 255, 255);
const SHAPE_PREVIEW_RGBA: Rgba = (192, 64, 192, 255);

//===========================================================================//

struct ImageCanvasDrag {
    from_selection: Point,
    from_pixel: Point,
    to_pixel: Point,
}

//===========================================================================//

pub struct ImageCanvas {
    top_left: Point,
    max_size: u32,
    autoscale: bool,
    drag_from_to: Option<ImageCanvasDrag>,
    lasso_points: Vec<(u32, u32)>,
    selection_animation_counter: i32,
    watercolor_parity: u32,
}

impl ImageCanvas {
    pub fn new(left: i32, top: i32, max_size: u32, autoscale: bool) -> Self {
        Self {
            top_left: Point::new(left, top),
            max_size,
            autoscale,
            drag_from_to: None,
            lasso_points: Vec::new(),
            selection_animation_counter: 0,
            watercolor_parity: 0,
        }
    }

    fn scale(&self, state: &EditorState) -> (u32, u32) {
        if !self.autoscale {
            return state.zoom();
        }
        // Try to maintain the zoom aspect ratio, but then scale to fit
        // self.max_size.
        let (horz, vert) = state.zoom();
        let div = gcd(horz, vert);
        let (horz, vert) = (horz / div, vert / div);
        let (width, height) = state.image_size();
        let scale = self.max_size / (width * horz).max(height * vert);
        if scale > 0 {
            return (scale * horz, scale * vert);
        }
        // If the zoom aspect ratio is impossible to satisfy and still
        // fit in max_size, then just use a 1:1 aspect ratio.
        let scale = (self.max_size / width.max(height)).max(1);
        (scale, scale)
    }

    fn scroll(&self, state: &EditorState) -> Point {
        let (x, y) = if self.autoscale { (0, 0) } else { state.scroll() };
        Point::new(x, y)
    }

    fn screen_rect(&self, state: &EditorState) -> Rect {
        let (horz_scale, vert_scale) = self.scale(state);
        let (image_width, image_height) = state.image_size();
        let screen_width = self.max_size.min(image_width * horz_scale);
        let screen_height = self.max_size.min(image_height * vert_scale);
        let screen_x =
            self.top_left.x() + ((self.max_size - screen_width) / 2) as i32;
        let screen_y =
            self.top_left.y() + ((self.max_size - screen_height) / 2) as i32;
        Rect::new(screen_x, screen_y, screen_width, screen_height)
    }

    fn dragged_points(
        &self,
        state: &EditorState,
    ) -> Option<((i32, i32), (i32, i32))> {
        if let Some(ref drag) = self.drag_from_to {
            let (x0, y0) = self.clamp_mouse_to_row_col(drag.from_pixel, state);
            let (x1, y1) = self.clamp_mouse_to_row_col(drag.to_pixel, state);
            Some(((x0 as i32, y0 as i32), (x1 as i32, y1 as i32)))
        } else {
            None
        }
    }

    fn dragged_image_subrect(&self, state: &EditorState) -> Option<Rect> {
        if let Some(((from_col, from_row), (to_col, to_row))) =
            self.dragged_points(state)
        {
            let x = from_col.min(to_col);
            let y = from_row.min(to_row);
            let w = ((from_col - to_col).abs() + 1) as u32;
            let h = ((from_row - to_row).abs() + 1) as u32;
            Some(Rect::new(x, y, w, h))
        } else {
            None
        }
    }

    fn mouse_to_row_col(
        &self,
        mouse: Point,
        state: &EditorState,
    ) -> Option<(u32, u32)> {
        let rect = self.screen_rect(state);
        if !rect.contains_point(mouse) {
            return None;
        }
        let position = unscaled(mouse - rect.top_left(), self.scale(state))
            + self.scroll(state);
        let (width, height) = state.image_size();
        if position.x() < 0
            || position.x() >= (width as i32)
            || position.y() < 0
            || position.y() >= (height as i32)
        {
            None
        } else {
            Some((position.x() as u32, position.y() as u32))
        }
    }

    fn clamp_mouse_to_row_col(
        &self,
        mouse: Point,
        state: &EditorState,
    ) -> (u32, u32) {
        let rect = self.screen_rect(state);
        let mouse = Point::new(
            mouse.x().max(rect.x()).min(rect.x() + rect.width() as i32 - 1),
            mouse.y().max(rect.y()).min(rect.y() + rect.height() as i32 - 1),
        );
        let position = unscaled(mouse - rect.top_left(), self.scale(state))
            + self.scroll(state);
        let (width, height) = state.image_size();
        (
            position.x().max(0).min(width as i32 - 1) as u32,
            position.y().max(0).min(height as i32 - 1) as u32,
        )
    }

    fn try_pencil(&self, mouse: Point, state: &mut EditorState) -> bool {
        if let Some(position) = self.mouse_to_row_col(mouse, state) {
            state.persistent_mutation().color_pixel(position);
            return true;
        }
        false
    }

    fn start_watercolor(&mut self, mouse: Point, state: &mut EditorState) {
        if let Some(position) = self.mouse_to_row_col(mouse, state) {
            self.watercolor_parity = (position.0 + position.1) % 2;
        }
    }

    fn try_watercolor(&self, mouse: Point, state: &mut EditorState) -> bool {
        if let Some(position) = self.mouse_to_row_col(mouse, state)
            && self.watercolor_parity == (position.0 + position.1) % 2
        {
            state.persistent_mutation().color_pixel(position);
            return true;
        }
        false
    }

    fn try_eyedrop(&self, mouse: Point, state: &mut EditorState) -> bool {
        if let Some(position) = self.mouse_to_row_col(mouse, state) {
            state.eyedrop_at(position);
            return true;
        }
        false
    }

    fn try_lasso(&mut self, mouse: Point, state: &mut EditorState) -> bool {
        if let Some(position) = self.mouse_to_row_col(mouse, state)
            && !self.lasso_points.contains(&position)
        {
            self.lasso_points.push(position);
            return true;
        }
        false
    }

    fn try_draw_shape(
        &mut self,
        shape: Shape,
        state: &mut EditorState,
    ) -> bool {
        if let Some(endpoints) = self.dragged_points(state) {
            mutate::draw_shape(state, shape, endpoints);
            self.drag_from_to = None;
            return true;
        }
        false
    }

    fn try_checker_fill(&self, mouse: Point, state: &mut EditorState) -> bool {
        if let Some(start) = self.mouse_to_row_col(mouse, state) {
            return mutate::checker_fill(state, start);
        }
        false
    }

    fn try_flood_fill(&self, mouse: Point, state: &mut EditorState) -> bool {
        if let Some(start) = self.mouse_to_row_col(mouse, state) {
            return mutate::flood_fill(state, start);
        }
        false
    }

    fn try_palette_replace(
        &self,
        mouse: Point,
        state: &mut EditorState,
        swap: bool,
    ) -> bool {
        if let Some(start) = self.mouse_to_row_col(mouse, state) {
            return mutate::replace_color(state, start, swap);
        }
        false
    }

    fn on_clock_tick(&mut self, state: &mut EditorState) -> Action<()> {
        if state.selection().is_some() {
            self.selection_animation_counter = mod_floor(
                self.selection_animation_counter + 1,
                draw::MARQUEE_ANIMATION_MODULUS,
            );
            Action::redraw()
        } else {
            Action::ignore()
        }
    }

    fn on_key_down(
        &mut self,
        key: Keycode,
        state: &mut EditorState,
    ) -> Action<()> {
        match key {
            Keycode::Backspace => {
                if state.selection().is_some() {
                    state.mutation().delete_selection();
                    Action::redraw().and_stop()
                } else {
                    Action::ignore()
                }
            }
            Keycode::Escape => {
                if state.selection().is_some() {
                    state.mutation().unselect();
                    Action::redraw().and_stop()
                } else {
                    Action::ignore()
                }
            }
            _ => Action::ignore(),
        }
    }

    fn on_mouse_down(
        &mut self,
        pt: Point,
        state: &mut EditorState,
    ) -> Action<()> {
        let canvas_rect = self.screen_rect(state);
        if !canvas_rect.contains_point(pt) {
            self.drag_from_to = None;
            return Action::ignore();
        }
        match state.tool() {
            Tool::Checkerboard => {
                let changed = self.try_checker_fill(pt, state);
                Action::redraw_if(changed).and_stop()
            }
            Tool::Eyedropper => {
                let changed = self.try_eyedrop(pt, state);
                Action::redraw_if(changed).and_stop()
            }
            Tool::Lasso => {
                let changed = self.try_lasso(pt, state);
                Action::redraw_if(changed).and_stop()
            }
            Tool::Line | Tool::Oval | Tool::Rectangle => {
                self.drag_from_to = Some(ImageCanvasDrag {
                    from_selection: Point::new(0, 0),
                    from_pixel: pt,
                    to_pixel: pt,
                });
                Action::redraw().and_stop()
            }
            Tool::PaintBucket => {
                let changed = self.try_flood_fill(pt, state);
                Action::redraw_if(changed).and_stop()
            }
            Tool::PaletteReplace => {
                let changed = self.try_palette_replace(pt, state, false);
                Action::redraw_if(changed).and_stop()
            }
            Tool::PaletteSwap => {
                let changed = self.try_palette_replace(pt, state, true);
                Action::redraw_if(changed).and_stop()
            }
            Tool::Pencil => {
                state.reset_persistent_mutation();
                let changed = self.try_pencil(pt, state);
                Action::redraw_if(changed).and_stop()
            }
            Tool::Select => {
                let sel_rect = state.selection_rect();
                if let Some(sr) = sel_rect {
                    let scale = self.scale(state);
                    let screen_topleft = canvas_rect.top_left()
                        + scaled(sr.top_left() - self.scroll(state), scale);
                    let screen_rect = Rect::new(
                        screen_topleft.x(),
                        screen_topleft.y(),
                        sr.width() * scale.0,
                        sr.height() * scale.1,
                    );
                    if !screen_rect.contains_point(pt) {
                        state.mutation().unselect();
                        return Action::redraw().and_stop();
                    } else {
                        state.reset_persistent_mutation();
                    }
                }
                self.drag_from_to = Some(ImageCanvasDrag {
                    from_selection: if let Some(sr) = sel_rect {
                        sr.top_left()
                    } else {
                        Point::new(0, 0)
                    },
                    from_pixel: pt,
                    to_pixel: pt,
                });
                Action::redraw().and_stop()
            }
            Tool::Watercolor => {
                state.reset_persistent_mutation();
                self.start_watercolor(pt, state);
                let changed = self.try_watercolor(pt, state);
                Action::redraw_if(changed).and_stop()
            }
        }
    }

    fn on_mouse_up(&mut self, state: &mut EditorState) -> Action<()> {
        match state.tool() {
            Tool::Lasso => {
                if !self.lasso_points.is_empty() {
                    if state.selection().is_none() {
                        state.mutation().lasso(&self.lasso_points);
                        self.selection_animation_counter = 0;
                    }
                    self.lasso_points.clear();
                    return Action::redraw();
                }
            }
            Tool::Line => {
                let changed = self.try_draw_shape(Shape::Line, state);
                return Action::redraw_if(changed);
            }
            Tool::Oval => {
                let changed = self.try_draw_shape(Shape::Oval, state);
                return Action::redraw_if(changed);
            }
            Tool::Rectangle => {
                let changed = self.try_draw_shape(Shape::Rect, state);
                return Action::redraw_if(changed);
            }
            Tool::Select => {
                if state.selection().is_none()
                    && let Some(image_subrect) =
                        self.dragged_image_subrect(state)
                {
                    state.mutation().select(&image_subrect);
                    self.drag_from_to = None;
                    self.selection_animation_counter = 0;
                    return Action::redraw();
                }
            }
            _ => {}
        }
        self.drag_from_to = None;
        Action::ignore()
    }

    fn on_mouse_drag(
        &mut self,
        pt: Point,
        state: &mut EditorState,
    ) -> Action<()> {
        match state.tool() {
            Tool::Lasso => Action::redraw_if(self.try_lasso(pt, state)),
            Tool::Line | Tool::Oval | Tool::Rectangle => {
                let Some(ref mut drag) = self.drag_from_to else {
                    return Action::ignore();
                };
                drag.to_pixel = pt;
                Action::redraw()
            }
            Tool::Pencil => Action::redraw_if(self.try_pencil(pt, state)),
            Tool::Select => {
                let scale = self.scale(state);
                let Some(ref mut drag) = self.drag_from_to else {
                    return Action::ignore();
                };
                drag.to_pixel = pt;
                if state.selection().is_some() {
                    let position = drag.from_selection
                        + unscaled(pt - drag.from_pixel, scale);
                    state.persistent_mutation().reposition_selection(position);
                }
                Action::redraw()
            }
            Tool::Watercolor => {
                Action::redraw_if(self.try_watercolor(pt, state))
            }
            _ => Action::ignore(),
        }
    }
}

impl GuiElement<EditorState, ()> for ImageCanvas {
    fn draw(
        &self,
        state: &EditorState,
        resources: &Resources,
        canvas: &mut Canvas,
    ) {
        let scale = self.scale(state);
        let scroll = self.scroll(state);
        let canvas_rect = self.screen_rect(state);
        let data = draw::DrawData {
            max_rect: Rect::new(
                self.top_left.x(),
                self.top_left.y(),
                self.max_size,
                self.max_size,
            ),
            canvas_rect,
            scale,
            scroll,
        };
        data.draw_border(canvas);
        data.draw_image(canvas, state);
        data.draw_grid(canvas, state);
        data.draw_glyph_metrics(canvas, state);
        {
            let mut canvas = canvas.subcanvas(canvas_rect);
            if let Some((selected, topleft)) = state.selection() {
                let screen_topleft = scaled(topleft - scroll, scale);
                canvas.draw_image_scaled(
                    selected,
                    state.palette(),
                    screen_topleft.x(),
                    screen_topleft.y(),
                    scale,
                );
                let marquee_screen_rect = Rect::new(
                    screen_topleft.x(),
                    screen_topleft.y(),
                    selected.width() * scale.0,
                    selected.height() * scale.1,
                );
                draw::draw_marquee(
                    &mut canvas,
                    marquee_screen_rect,
                    self.selection_animation_counter,
                );
            } else if let Some(shape) = Shape::from_tool(state.tool()) {
                if let Some(((col1, row1), (col2, row2))) =
                    self.dragged_points(state)
                {
                    let (width, height) = state.image_size();
                    for (x, y) in shape.pixels(col1, row1, col2, row2) {
                        if x >= 0
                            && x <= (width as i32)
                            && y >= 0
                            && y < (height as i32)
                        {
                            for (col, row) in
                                state.mirror_positions((x as u32, y as u32))
                            {
                                let x = ((col as i32) - scroll.x())
                                    * scale.0 as i32;
                                let y = ((row as i32) - scroll.y())
                                    * scale.1 as i32;
                                canvas.draw_rect(
                                    SHAPE_PREVIEW_RGBA,
                                    Rect::new(x, y, scale.0, scale.1),
                                );
                            }
                        }
                    }
                }
            } else if let Some(rect) = self.dragged_image_subrect(state) {
                let marquee_rect = Rect::new(
                    (rect.x() - scroll.x()) * (scale.0 as i32),
                    (rect.y() - scroll.y()) * (scale.1 as i32),
                    rect.width() * scale.0,
                    rect.height() * scale.1,
                );
                draw::draw_marquee(&mut canvas, marquee_rect, 0);
            } else if state.tool() == Tool::Lasso {
                for &(x, y) in self.lasso_points.iter() {
                    canvas.draw_rect(
                        LASSO_PREVIEW_RGBA,
                        Rect::new(
                            (x * scale.0) as i32,
                            (y * scale.1) as i32,
                            scale.0,
                            scale.1,
                        ),
                    );
                }
            }
        }
        if let Some(rect) = self.dragged_image_subrect(state) {
            let (topleft, label) = if let Some((_, to)) = state.selection() {
                let from = self.drag_from_to.as_ref().unwrap().from_selection;
                let delta_x = to.x() - from.x();
                let delta_y = to.y() - from.y();
                let label = format!("{},{}", delta_x.abs(), delta_y.abs());
                (to, label)
            } else {
                let label = format!("{}x{}", rect.width(), rect.height());
                (Point::new(rect.x(), rect.y()), label)
            };
            let topleft = topleft - scroll;
            let topleft = Point::new(
                canvas_rect.x() + (topleft.x() * (scale.0 as i32)).max(0),
                canvas_rect.y() + (topleft.y() * (scale.1 as i32)).max(0),
            );
            canvas.fill_rect(
                SELECTION_LABEL_RGBA,
                Rect::new(
                    topleft.x() + 1,
                    topleft.y() - 11,
                    (resources.font().text_width(&label) + 1) as u32,
                    10,
                ),
            );
            canvas.draw_string(
                resources.font(),
                topleft.x() + 2,
                topleft.y() - 11,
                &label,
            );
        }
    }

    fn on_event(
        &mut self,
        event: &Event,
        state: &mut EditorState,
    ) -> Action<()> {
        match *event {
            Event::ClockTick => self.on_clock_tick(state),
            Event::KeyDown(key, _) => self.on_key_down(key, state),
            Event::MouseDown(pt) => self.on_mouse_down(pt, state),
            Event::MouseUp => self.on_mouse_up(state),
            Event::MouseDrag(pt) => self.on_mouse_drag(pt, state),
            _ => Action::ignore(),
        }
    }
}

//===========================================================================//

fn scaled(point: Point, scale: (u32, u32)) -> Point {
    Point::new(point.x() * scale.0 as i32, point.y() * scale.1 as i32)
}

fn unscaled(point: Point, scale: (u32, u32)) -> Point {
    Point::new(point.x() / scale.0 as i32, point.y() / scale.1 as i32)
}

//===========================================================================//
