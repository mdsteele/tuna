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

use crate::state::Tool;

//===========================================================================//

#[derive(Clone, Copy)]
pub enum Shape {
    Line,
    Oval,
    Rect,
}

impl Shape {
    pub fn from_tool(tool: Tool) -> Option<Self> {
        match tool {
            Tool::Line => Some(Self::Line),
            Tool::Oval => Some(Self::Oval),
            Tool::Rectangle => Some(Self::Rect),
            _ => None,
        }
    }

    pub fn pixels(
        self,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
    ) -> Vec<(i32, i32)> {
        match self {
            Self::Line => bresenham_line(x1, y1, x2, y2),
            Self::Oval => bresenham_oval(x1, y1, x2, y2),
            Self::Rect => bresenham_rect(x1, y1, x2, y2),
        }
    }
}

//===========================================================================//

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
    let (mut x0, mut x1) = (x0.min(x1), x0.max(x1));
    let (mut y0, mut y1) = (y0.min(y1), y0.max(y1));
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
    let (x0, x1) = (x0.min(x1), x0.max(x1));
    let (y0, y1) = (y0.min(y1), y0.max(y1));
    let mut output = Vec::new();
    if x0 == x1 {
        for y in y0..=y1 {
            output.push((x0, y));
        }
    } else if y0 == y1 {
        for x in x0..=x1 {
            output.push((x, y0));
        }
    } else {
        for x in x0..x1 {
            output.push((x, y0));
            output.push((x + 1, y1));
        }
        for y in y0..y1 {
            output.push((x0, y + 1));
            output.push((x1, y));
        }
    }
    output
}

//===========================================================================//
