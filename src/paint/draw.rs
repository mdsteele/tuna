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

use crate::canvas::Canvas;
use crate::state::EditorState;
use crate::util::Rgba;
use num_integer::mod_floor;
use sdl2::rect::{Point, Rect};

//===========================================================================//

const BORDER_RGBA: Rgba = (255, 255, 255, 255);
const GLYPH_BASELINE_RGBA: Rgba = (0, 127, 255, 255);
const GLYPH_LEFT_EDGE_RGBA: Rgba = (127, 255, 0, 255);
const GLYPH_RIGHT_EDGE_RGBA: Rgba = (255, 0, 127, 255);
const GRID_LINES_RGBA: Rgba = (192, 0, 255, 255);
const MARQUEE_1_RGBA: Rgba = (255, 255, 255, 255);
const MARQUEE_2_RGBA: Rgba = (0, 0, 0, 255);

pub const MARQUEE_ANIMATION_MODULUS: i32 = 10;

//===========================================================================//

pub(super) struct DrawData {
    pub max_rect: Rect,
    pub canvas_rect: Rect,
    pub scale: (u32, u32),
    pub scroll: Point,
}

impl DrawData {
    pub fn draw_border(&self, canvas: &mut Canvas) {
        canvas.draw_rect(BORDER_RGBA, expand(self.canvas_rect, 2));
    }

    pub fn draw_glyph_metrics(
        &self,
        canvas: &mut Canvas,
        state: &EditorState,
    ) {
        let Some((baseline, left_edge, right_edge)) = state.glyph_metrics()
        else {
            return;
        };
        let offset = self.canvas_rect.top_left() - self.max_rect.top_left();
        let mut subcanvas = canvas.subcanvas(self.max_rect);
        let (width, height) = subcanvas.size();
        subcanvas.draw_rect(
            GLYPH_BASELINE_RGBA,
            Rect::new(
                0,
                offset.y()
                    + (baseline - self.scroll.y()) * self.scale.1 as i32,
                width,
                1,
            ),
        );
        subcanvas.draw_rect(
            GLYPH_LEFT_EDGE_RGBA,
            Rect::new(
                offset.x()
                    + (left_edge - self.scroll.x()) * self.scale.0 as i32
                    - 1,
                0,
                1,
                height,
            ),
        );
        subcanvas.draw_rect(
            GLYPH_RIGHT_EDGE_RGBA,
            Rect::new(
                offset.x()
                    + (right_edge - self.scroll.x()) * self.scale.0 as i32,
                0,
                1,
                height,
            ),
        );
    }

    pub fn draw_grid(&self, canvas: &mut Canvas, state: &EditorState) {
        let mut subcanvas = canvas.subcanvas(self.canvas_rect);
        let (canvas_width, canvas_height) = subcanvas.size();
        let (grid_horz, grid_vert) = state.grid();
        if grid_horz > 0 {
            let image_width = state.image().width() as i32;
            let mut x = grid_horz as i32;
            while x < image_width {
                subcanvas.draw_rect(
                    GRID_LINES_RGBA,
                    Rect::new(
                        (x - self.scroll.x()) * self.scale.0 as i32,
                        0,
                        1,
                        canvas_height,
                    ),
                );
                x += grid_horz as i32;
            }
        }
        if grid_vert > 0 {
            let image_height = state.image().height() as i32;
            let mut y = grid_vert as i32;
            while y < image_height {
                subcanvas.draw_rect(
                    GRID_LINES_RGBA,
                    Rect::new(
                        0,
                        (y - self.scroll.y()) * self.scale.1 as i32,
                        canvas_width,
                        1,
                    ),
                );
                y += grid_vert as i32;
            }
        }
    }

    pub fn draw_image(&self, canvas: &mut Canvas, state: &EditorState) {
        canvas.subcanvas(self.canvas_rect).draw_image_scaled(
            state.image(),
            state.palette(),
            -self.scroll.x() * (self.scale.0 as i32),
            -self.scroll.y() * (self.scale.1 as i32),
            self.scale,
        );
    }
}

//===========================================================================//

pub fn draw_marquee(canvas: &mut Canvas, rect: Rect, anim: i32) {
    canvas.draw_rect(MARQUEE_1_RGBA, rect);
    for x in 0..(rect.width() as i32) {
        if mod_floor(x - anim, MARQUEE_ANIMATION_MODULUS)
            < MARQUEE_ANIMATION_MODULUS / 2
        {
            canvas.draw_pixel(
                MARQUEE_2_RGBA,
                Point::new(rect.left() + x, rect.top()),
            );
        }
        if mod_floor(x + anim, MARQUEE_ANIMATION_MODULUS)
            < MARQUEE_ANIMATION_MODULUS / 2
        {
            canvas.draw_pixel(
                MARQUEE_2_RGBA,
                Point::new(rect.left() + x, rect.bottom() - 1),
            );
        }
    }
    for y in 0..(rect.height() as i32) {
        if mod_floor(y + anim, MARQUEE_ANIMATION_MODULUS)
            >= MARQUEE_ANIMATION_MODULUS / 2
        {
            canvas.draw_pixel(
                MARQUEE_2_RGBA,
                Point::new(rect.left(), rect.top() + y),
            );
        }
        if mod_floor(y - anim, MARQUEE_ANIMATION_MODULUS)
            >= MARQUEE_ANIMATION_MODULUS / 2
        {
            canvas.draw_pixel(
                MARQUEE_2_RGBA,
                Point::new(rect.right() - 1, rect.top() + y),
            );
        }
    }
}

//===========================================================================//

fn expand(rect: Rect, by: i32) -> Rect {
    Rect::new(
        rect.x() - by,
        rect.y() - by,
        ((rect.width() as i32) + 2 * by) as u32,
        ((rect.height() as i32) + 2 * by) as u32,
    )
}

//===========================================================================//
