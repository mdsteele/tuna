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

use super::data::Data;
use ahi::Image;
use sdl2::rect::Point;
use std::rc::Rc;

//===========================================================================//

#[derive(Clone)]
pub(super) struct Snapshot {
    pub data: Data,
    pub selection: Option<(Rc<Image>, Point)>,
    pub zoom: (u32, u32),
    pub unsaved: bool,
}

impl Snapshot {
    pub fn with_data(data: Data) -> Self {
        let (width, height) = data.image_size();
        let scale = (256 / width.max(height)).max(1);
        Self { data, selection: None, zoom: (scale, scale), unsaved: false }
    }
}

//===========================================================================//
