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

//===========================================================================//

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Mirror {
    None,
    Horz,
    Vert,
    Both,
    Rot2,
    Rot4,
}

impl Mirror {
    pub(super) fn positions(
        self,
        (width, height): (u32, u32),
        (x, y): (u32, u32),
    ) -> Vec<(u32, u32)> {
        assert!(x < width);
        assert!(y < height);
        let mut positions = vec![(x, y)];
        if self == Self::Horz || self == Self::Both {
            positions.push((width - x - 1, y));
        }
        if self == Self::Vert || self == Self::Both {
            positions.push((x, height - y - 1));
        }
        if self == Self::Both || self == Self::Rot2 || self == Self::Rot4 {
            positions.push((width - x - 1, height - y - 1));
        }
        if self == Self::Rot4 {
            let mut x1 = (height - y - 1) as i32;
            let mut y1 = x as i32;
            let mut x2 = y as i32;
            let mut y2 = (width - x - 1) as i32;
            if width > height {
                let diff = ((width - height) / 2) as i32;
                x1 += diff;
                x2 += diff;
                y1 -= diff;
                y2 -= diff;
            }
            if height > width {
                let diff = ((height - width) / 2) as i32;
                x1 -= diff;
                x2 -= diff;
                y1 += diff;
                y2 += diff;
            }
            if x1 >= 0
                && (x1 as u32) < width
                && y1 >= 0
                && (y1 as u32) < height
            {
                positions.push((x1 as u32, y1 as u32));
            }
            if x2 >= 0
                && (x2 as u32) < width
                && y2 >= 0
                && (y2 as u32) < height
            {
                positions.push((x2 as u32, y2 as u32));
            }
        }
        positions
    }
}

//===========================================================================//
