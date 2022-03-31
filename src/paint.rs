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
use crate::event::{Event, Keycode};
use crate::state::{EditorState, Tool};
use num_integer::mod_floor;
use sdl2::rect::{Point, Rect};
use std::cmp;

//===========================================================================//

enum Shape {
    Line,
    Oval,
    Rect,
}

impl Shape {
    fn from_tool(tool: Tool) -> Option<Shape> {
        match tool {
            Tool::Line => Some(Shape::Line),
            Tool::Oval => Some(Shape::Oval),
            Tool::Rectangle => Some(Shape::Rect),
            _ => None,
        }
    }
}

struct ImageCanvasDrag {
    from_selection: Point,
    from_pixel: Point,
    to_pixel: Point,
}

pub struct ImageCanvas {
    top_left: Point,
    max_size: u32,
    drag_from_to: Option<ImageCanvasDrag>,
    lasso_points: Vec<(u32, u32)>,
    selection_animation_counter: i32,
    watercolor_parity: u32,
}

impl ImageCanvas {
    pub fn new(left: i32, top: i32, max_size: u32) -> ImageCanvas {
        ImageCanvas {
            top_left: Point::new(left, top),
            max_size,
            drag_from_to: None,
            lasso_points: Vec::new(),
            selection_animation_counter: 0,
            watercolor_parity: 0,
        }
    }

    fn scale(&self, state: &EditorState) -> u32 {
        let (width, height) = state.image_size();
        cmp::max(1, self.max_size / cmp::max(width, height))
    }

    fn rect(&self, state: &EditorState) -> Rect {
        let scale = self.scale(state);
        let (width, height) = state.image_size();
        Rect::new(
            self.top_left.x(),
            self.top_left.y(),
            width * scale,
            height * scale,
        )
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

    fn dragged_rect(&self, state: &EditorState) -> Option<Rect> {
        if let Some(((from_col, from_row), (to_col, to_row))) =
            self.dragged_points(state)
        {
            let x = cmp::min(from_col, to_col);
            let y = cmp::min(from_row, to_row);
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
        if mouse.x() < self.top_left.x() || mouse.y() < self.top_left.y() {
            return None;
        }
        let scaled = (mouse - self.top_left) / self.scale(state) as i32;
        let (width, height) = state.image_size();
        if scaled.x() < 0
            || scaled.x() >= (width as i32)
            || scaled.y() < 0
            || scaled.y() >= (height as i32)
        {
            None
        } else {
            Some((scaled.x() as u32, scaled.y() as u32))
        }
    }

    fn clamp_mouse_to_row_col(
        &self,
        mouse: Point,
        state: &EditorState,
    ) -> (u32, u32) {
        let scaled = (mouse - self.top_left) / self.scale(state) as i32;
        let (width, height) = state.image_size();
        (
            cmp::max(0, cmp::min(scaled.x(), width as i32 - 1)) as u32,
            cmp::max(0, cmp::min(scaled.y(), height as i32 - 1)) as u32,
        )
    }

    fn try_pencil(&self, mouse: Point, state: &mut EditorState) -> bool {
        if let Some(position) = self.mouse_to_row_col(mouse, state) {
            state.persistent_mutation().color_pixel(position);
            true
        } else {
            false
        }
    }

    fn start_watercolor(&mut self, mouse: Point, state: &mut EditorState) {
        if let Some(position) = self.mouse_to_row_col(mouse, state) {
            self.watercolor_parity = (position.0 + position.1) % 2;
        }
    }

    fn try_watercolor(&self, mouse: Point, state: &mut EditorState) -> bool {
        if let Some(position) = self.mouse_to_row_col(mouse, state) {
            if self.watercolor_parity == (position.0 + position.1) % 2 {
                state.persistent_mutation().color_pixel(position);
                return true;
            }
        }
        return false;
    }

    fn try_eyedrop(&self, mouse: Point, state: &mut EditorState) -> bool {
        if let Some(position) = self.mouse_to_row_col(mouse, state) {
            state.eyedrop_at(position);
            true
        } else {
            false
        }
    }

    fn try_lasso(&mut self, mouse: Point, state: &mut EditorState) -> bool {
        if let Some(position) = self.mouse_to_row_col(mouse, state) {
            if !self.lasso_points.contains(&position) {
                self.lasso_points.push(position);
                return true;
            }
        }
        return false;
    }

    fn try_draw_shape(
        &mut self,
        shape: Shape,
        state: &mut EditorState,
    ) -> bool {
        if let Some(((col1, row1), (col2, row2))) = self.dragged_points(state)
        {
            let (width, height) = state.image_size();
            let mut mutation = state.mutation();
            for (x, y) in bresenham_shape(shape, col1, row1, col2, row2) {
                if x >= 0 && y >= 0 {
                    let x = x as u32;
                    let y = y as u32;
                    if x < width && y < height {
                        mutation.color_pixel((x, y));
                    }
                }
            }
            self.drag_from_to = None;
            return true;
        }
        false
    }

    fn try_checker_fill(&self, mouse: Point, state: &mut EditorState) -> bool {
        if let Some(start) = self.mouse_to_row_col(mouse, state) {
            let to_color = state.color();
            let from_color = state.image()[start];
            if from_color == to_color {
                return false;
            }
            let mut mutation = state.mutation();
            let image = mutation.image();
            let width = image.width();
            let height = image.height();
            image[start] = to_color;
            let mut stack: Vec<(u32, u32)> = vec![start];
            while let Some((col, row)) = stack.pop() {
                let mut next: Vec<(u32, u32)> = vec![];
                if col > 0 && row > 0 {
                    next.push((col - 1, row - 1));
                }
                if col < width - 1 && row < height - 1 {
                    next.push((col + 1, row + 1));
                }
                if col < width - 1 && row > 0 {
                    next.push((col + 1, row - 1));
                }
                if col > 0 && row < height - 1 {
                    next.push((col - 1, row + 1));
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

    fn try_flood_fill(&self, mouse: Point, state: &mut EditorState) -> bool {
        if let Some(start) = self.mouse_to_row_col(mouse, state) {
            let to_color = state.color();
            let from_color = state.image()[start];
            if from_color == to_color {
                return false;
            }
            let mut mutation = state.mutation();
            let image = mutation.image();
            let width = image.width();
            let height = image.height();
            image[start] = to_color;
            let mut stack: Vec<(u32, u32)> = vec![start];
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

    fn try_palette_replace(
        &self,
        mouse: Point,
        state: &mut EditorState,
        swap: bool,
    ) -> bool {
        if let Some(start) = self.mouse_to_row_col(mouse, state) {
            let to_color = state.color();
            let from_color = state.image()[start];
            if from_color == to_color {
                return false;
            }
            state.set_color(from_color);
            let mut mutation = state.mutation();
            let image = mutation.image();
            let width = image.width();
            let height = image.height();
            for y in 0..height {
                for x in 0..width {
                    let color = image[(x, y)];
                    if color == from_color {
                        image[(x, y)] = to_color;
                    } else if swap && color == to_color {
                        image[(x, y)] = from_color;
                    }
                }
            }
            true
        } else {
            false
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
        let canvas_rect = self.rect(state);
        canvas.draw_rect((255, 255, 255, 255), expand(canvas_rect, 2));
        canvas.draw_image(
            state.image(),
            state.palette(),
            canvas_rect.x(),
            canvas_rect.y(),
            scale,
        );
        if let Some((baseline, left_edge, right_edge)) = state.image_metrics()
        {
            canvas.draw_rect(
                (0, 127, 255, 255),
                Rect::new(
                    canvas_rect.x(),
                    canvas_rect.y() + baseline * scale as i32,
                    canvas_rect.width(),
                    1,
                ),
            );
            canvas.draw_rect(
                (127, 255, 0, 255),
                Rect::new(
                    canvas_rect.x() + left_edge * scale as i32 - 1,
                    canvas_rect.y(),
                    1,
                    canvas_rect.height(),
                ),
            );
            canvas.draw_rect(
                (255, 0, 127, 255),
                Rect::new(
                    canvas_rect.x() + right_edge * scale as i32,
                    canvas_rect.y(),
                    1,
                    canvas_rect.height(),
                ),
            );
        }
        if let Some(rect) = self.dragged_rect(state) {
            let marquee_rect = Rect::new(
                canvas_rect.x() + rect.x() * (scale as i32),
                canvas_rect.y() + rect.y() * (scale as i32),
                rect.width() * scale,
                rect.height() * scale,
            );
            draw_marquee(canvas, marquee_rect, 0);
            let size_string = format!("{}x{}", rect.width(), rect.height());
            canvas.fill_rect(
                (255, 255, 255, 255),
                Rect::new(
                    marquee_rect.x() + 1,
                    marquee_rect.y() - 11,
                    (resources.font().text_width(&size_string) + 1) as u32,
                    10,
                ),
            );
            canvas.draw_string(
                resources.font(),
                marquee_rect.x() + 2,
                marquee_rect.y() - 11,
                &size_string,
            );
        }
        let mut canvas = canvas.subcanvas(canvas_rect);
        if let Some((ref selected, topleft)) = state.selection() {
            let left = topleft.x() * (scale as i32);
            let top = topleft.y() * (scale as i32);
            canvas.draw_image(selected, state.palette(), left, top, scale);
            let marquee_rect = Rect::new(
                left,
                top,
                selected.width() * scale,
                selected.height() * scale,
            );
            draw_marquee(
                &mut canvas,
                marquee_rect,
                self.selection_animation_counter,
            );
        } else if let Some(shape) = Shape::from_tool(state.tool()) {
            if let Some(((col1, row1), (col2, row2))) =
                self.dragged_points(state)
            {
                let (width, height) = state.image_size();
                for (x, y) in bresenham_shape(shape, col1, row1, col2, row2) {
                    if x >= 0
                        && x <= (width as i32)
                        && y >= 0
                        && y < (height as i32)
                    {
                        for (col, row) in
                            state.mirror_positions((x as u32, y as u32))
                        {
                            canvas.draw_rect(
                                (192, 64, 192, 255),
                                Rect::new(
                                    (col * scale) as i32,
                                    (row * scale) as i32,
                                    scale,
                                    scale,
                                ),
                            );
                        }
                    }
                }
            }
        } else if state.tool() == Tool::Lasso {
            for &(x, y) in self.lasso_points.iter() {
                canvas.draw_rect(
                    (192, 192, 64, 255),
                    Rect::new(
                        (x * scale) as i32,
                        (y * scale) as i32,
                        scale,
                        scale,
                    ),
                );
            }
        }
    }

    fn on_event(
        &mut self,
        event: &Event,
        state: &mut EditorState,
    ) -> Action<()> {
        match event {
            &Event::ClockTick => {
                if state.selection().is_some() {
                    self.selection_animation_counter = mod_floor(
                        self.selection_animation_counter + 1,
                        MARQUEE_ANIMATION_MODULUS,
                    );
                    return Action::redraw();
                } else {
                    return Action::ignore();
                }
            }
            &Event::KeyDown(Keycode::Backspace, _) => {
                if state.selection().is_some() {
                    state.mutation().delete_selection();
                    return Action::redraw().and_stop();
                } else {
                    return Action::ignore();
                }
            }
            &Event::KeyDown(Keycode::Escape, _) => {
                if state.selection().is_some() {
                    state.mutation().unselect();
                    return Action::redraw().and_stop();
                } else {
                    return Action::ignore();
                }
            }
            &Event::MouseDown(pt) => {
                if self.rect(state).contains_point(pt) {
                    match state.tool() {
                        Tool::Checkerboard => {
                            let changed = self.try_checker_fill(pt, state);
                            return Action::redraw_if(changed).and_stop();
                        }
                        Tool::Eyedropper => {
                            let changed = self.try_eyedrop(pt, state);
                            return Action::redraw_if(changed).and_stop();
                        }
                        Tool::Lasso => {
                            let changed = self.try_lasso(pt, state);
                            return Action::redraw_if(changed).and_stop();
                        }
                        Tool::Line | Tool::Oval | Tool::Rectangle => {
                            self.drag_from_to = Some(ImageCanvasDrag {
                                from_selection: Point::new(0, 0),
                                from_pixel: pt,
                                to_pixel: pt,
                            });
                            return Action::redraw().and_stop();
                        }
                        Tool::PaintBucket => {
                            let changed = self.try_flood_fill(pt, state);
                            return Action::redraw_if(changed).and_stop();
                        }
                        Tool::PaletteReplace => {
                            let changed =
                                self.try_palette_replace(pt, state, false);
                            return Action::redraw_if(changed).and_stop();
                        }
                        Tool::PaletteSwap => {
                            let changed =
                                self.try_palette_replace(pt, state, true);
                            return Action::redraw_if(changed).and_stop();
                        }
                        Tool::Pencil => {
                            state.reset_persistent_mutation();
                            let changed = self.try_pencil(pt, state);
                            return Action::redraw_if(changed).and_stop();
                        }
                        Tool::Select => {
                            let rect = state.selection_rect();
                            if let Some(rect) = rect {
                                let screen_topleft = self.top_left
                                    + rect.top_left()
                                        * self.scale(state) as i32;
                                let scale = self.scale(state);
                                let screen_rect = Rect::new(
                                    screen_topleft.x(),
                                    screen_topleft.y(),
                                    rect.width() * scale,
                                    rect.height() * scale,
                                );
                                if !screen_rect.contains_point(pt) {
                                    state.mutation().unselect();
                                    return Action::redraw().and_stop();
                                } else {
                                    state.reset_persistent_mutation();
                                }
                            }
                            self.drag_from_to = Some(ImageCanvasDrag {
                                from_selection: if let Some(r) = rect {
                                    r.top_left()
                                } else {
                                    Point::new(0, 0)
                                },
                                from_pixel: pt,
                                to_pixel: pt,
                            });
                            return Action::redraw().and_stop();
                        }
                        Tool::Watercolor => {
                            state.reset_persistent_mutation();
                            self.start_watercolor(pt, state);
                            let changed = self.try_watercolor(pt, state);
                            return Action::redraw_if(changed).and_stop();
                        }
                    }
                } else {
                    self.drag_from_to = None;
                }
            }
            &Event::MouseUp => {
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
                        if state.selection().is_none() {
                            if let Some(rect) = self.dragged_rect(state) {
                                state.mutation().select(&rect);
                                self.drag_from_to = None;
                                self.selection_animation_counter = 0;
                                return Action::redraw();
                            }
                        }
                    }
                    _ => {}
                }
                self.drag_from_to = None;
            }
            &Event::MouseDrag(pt) => match state.tool() {
                Tool::Lasso => {
                    let changed = self.try_lasso(pt, state);
                    return Action::redraw_if(changed);
                }
                Tool::Line | Tool::Oval | Tool::Rectangle => {
                    if let Some(ref mut drag) = self.drag_from_to {
                        drag.to_pixel = pt;
                        return Action::redraw();
                    }
                }
                Tool::Pencil => {
                    let changed = self.try_pencil(pt, state);
                    return Action::redraw_if(changed);
                }
                Tool::Select => {
                    let scale = self.scale(state) as i32;
                    if let Some(ref mut drag) = self.drag_from_to {
                        drag.to_pixel = pt;
                        if state.selection().is_some() {
                            let position = drag.from_selection
                                + (pt - drag.from_pixel) / scale;
                            state
                                .persistent_mutation()
                                .reposition_selection(position);
                        }
                        return Action::redraw();
                    }
                }
                Tool::Watercolor => {
                    let changed = self.try_watercolor(pt, state);
                    return Action::redraw_if(changed);
                }
                _ => {}
            },
            _ => {}
        }
        return Action::ignore();
    }
}

//===========================================================================//

fn bresenham_shape(
    shape: Shape,
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
) -> Vec<(i32, i32)> {
    match shape {
        Shape::Line => bresenham_line(x1, y1, x2, y2),
        Shape::Oval => bresenham_oval(x1, y1, x2, y2),
        Shape::Rect => bresenham_rect(x1, y1, x2, y2),
    }
}

fn bresenham_line(x0: i32, y0: i32, x1: i32, y1: i32) -> Vec<(i32, i32)> {
    // This function was adapted from the plotLine function in
    // http://members.chello.at/easyfilter/bresenham.js by Zingl Alois.
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let x_step = (x1 - x0).signum();
    let y_step = (y1 - y0).signum();
    let mut err = dx + dy;
    let (mut x, mut y) = (x0, y0);
    let mut output = vec![(x, y)];
    while x != x1 || y != y1 {
        let err2 = 2 * err;
        if err2 >= dy {
            err += dy;
            x += x_step;
        }
        if err2 <= dx {
            err += dx;
            y += y_step;
        }
        output.push((x, y));
    }
    output
}

fn bresenham_oval(x0: i32, y0: i32, x1: i32, y1: i32) -> Vec<(i32, i32)> {
    // This function was adapted from the plotEllipseRect function in
    // http://members.chello.at/easyfilter/bresenham.js by Zingl Alois.
    let (mut x0, mut x1) = (cmp::min(x0, x1), cmp::max(x0, x1));
    let (mut y0, mut y1) = (cmp::min(y0, y1), cmp::max(y0, y1));
    let width = x1 - x0;
    let height = y1 - y0;
    let h_parity = height & 1;
    y0 += (height + 1) / 2;
    y1 = y0 - h_parity;
    let mut dx = 4 * (1 - width) * height * height;
    let mut dy = 4 * (h_parity + 1) * width * width;
    let dx_step = 8 * height * height;
    let dy_step = 8 * width * width;
    let mut err = dx + dy + h_parity * width * width;
    let mut output = Vec::new();
    while x0 <= x1 {
        output.push((x0, y0));
        output.push((x0, y1));
        output.push((x1, y0));
        output.push((x1, y1));
        let err2 = 2 * err;
        if err2 <= dy {
            y0 += 1;
            y1 -= 1;
            dy += dy_step;
            err += dy;
        }
        if err2 >= dx || 2 * err > dy {
            x0 += 1;
            x1 -= 1;
            dx += dx_step;
            err += dx;
        }
    }
    while y0 - y1 <= height {
        output.push((x0 - 1, y0));
        output.push((x0 - 1, y1));
        output.push((x1 + 1, y0));
        output.push((x1 + 1, y1));
        y0 += 1;
        y1 -= 1;
    }
    output
}

fn bresenham_rect(x0: i32, y0: i32, x1: i32, y1: i32) -> Vec<(i32, i32)> {
    let (x0, x1) = (cmp::min(x0, x1), cmp::max(x0, x1));
    let (y0, y1) = (cmp::min(y0, y1), cmp::max(y0, y1));
    let mut output = Vec::new();
    for x in x0..x1 {
        output.push((x, y0));
        output.push((x + 1, y1));
    }
    for y in y0..y1 {
        output.push((x0, y + 1));
        output.push((x1, y));
    }
    output
}

fn expand(rect: Rect, by: i32) -> Rect {
    Rect::new(
        rect.x() - by,
        rect.y() - by,
        ((rect.width() as i32) + 2 * by) as u32,
        ((rect.height() as i32) + 2 * by) as u32,
    )
}

const MARQUEE_ANIMATION_MODULUS: i32 = 8;

fn draw_marquee(canvas: &mut Canvas, rect: Rect, anim: i32) {
    canvas.draw_rect((255, 255, 255, 255), rect);
    let color = (0, 0, 0, 255);
    for x in 0..(rect.width() as i32) {
        if mod_floor(x - anim, MARQUEE_ANIMATION_MODULUS) < 4 {
            canvas.draw_pixel(color, Point::new(rect.left() + x, rect.top()));
        }
        if mod_floor(x + anim, MARQUEE_ANIMATION_MODULUS) < 4 {
            canvas.draw_pixel(
                color,
                Point::new(rect.left() + x, rect.bottom() - 1),
            );
        }
    }
    for y in 0..(rect.height() as i32) {
        if mod_floor(y + anim, MARQUEE_ANIMATION_MODULUS) >= 4 {
            canvas.draw_pixel(color, Point::new(rect.left(), rect.top() + y));
        }
        if mod_floor(y - anim, MARQUEE_ANIMATION_MODULUS) >= 4 {
            canvas.draw_pixel(
                color,
                Point::new(rect.right() - 1, rect.top() + y),
            );
        }
    }
}

//===========================================================================//
