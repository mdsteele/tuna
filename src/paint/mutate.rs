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

use super::shape::Shape;
use crate::state::EditorState;

//===========================================================================//

pub fn checker_fill(state: &mut EditorState, start: (u32, u32)) -> bool {
    let to_color = state.color();
    let from_color = state.image()[start];
    if from_color == to_color {
        return false;
    }
    let mut mutation = state.mutation();
    let image = mutation.image_mut();
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
}

pub fn draw_shape(
    state: &mut EditorState,
    shape: Shape,
    endpoints: ((i32, i32), (i32, i32)),
) {
    let ((col1, row1), (col2, row2)) = endpoints;
    let (width, height) = state.image_size();
    let mut mutation = state.mutation();
    for (x, y) in shape.pixels(col1, row1, col2, row2) {
        if x >= 0 && y >= 0 {
            let x = x as u32;
            let y = y as u32;
            if x < width && y < height {
                mutation.color_pixel((x, y));
            }
        }
    }
}

pub fn flood_fill(state: &mut EditorState, start: (u32, u32)) -> bool {
    let to_color = state.color();
    let from_color = state.image()[start];
    if from_color == to_color {
        return false;
    }
    let mut mutation = state.mutation();
    let image = mutation.image_mut();
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
}

pub fn replace_color(
    state: &mut EditorState,
    start: (u32, u32),
    swap: bool,
) -> bool {
    let to_color = state.color();
    let from_color = state.image()[start];
    if from_color == to_color {
        return false;
    }
    state.set_color(from_color);
    let mut mutation = state.mutation();
    let image = mutation.image_mut();
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
}

//===========================================================================//
