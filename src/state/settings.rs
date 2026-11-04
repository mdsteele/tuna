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

use super::mirror::Mirror;
use super::tool::Tool;
use ahi::Color;
use ahi::Image;
use sdl2::rect::Point;
use std::rc::Rc;

//===========================================================================//

const DEFAULT_TEST_SENTENCE: &str =
    "The quick, brown fox jumps over a ``lazy'' dog.";

//===========================================================================//

/// Editor state that isn't undoable and doesn't change when a new file is
/// loaded.
pub(super) struct Settings {
    pub color: Color,
    pub clipboard: Option<(Rc<Image>, Point)>,
    pub tool: Tool,
    pub prev_tool: Tool,
    pub mirror: Mirror,
    pub grid: (u32, u32),
    pub test_sentence: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            color: Color::C1,
            clipboard: None,
            tool: Tool::Pencil,
            prev_tool: Tool::Pencil,
            mirror: Mirror::None,
            grid: (0, 0),
            test_sentence: DEFAULT_TEST_SENTENCE.to_string(),
        }
    }
}

//===========================================================================//
