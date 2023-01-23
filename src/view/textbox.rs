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
use crate::element::{Action, AggregateElement, GuiElement, SubrectElement};
use crate::event::{Event, Keycode, ALT};
use crate::state::EditorState;
use ahi::Color;
use sdl2::rect::Rect;
use std::cmp;
use std::ffi::OsStr;
use std::io;
use std::path::Path;

//===========================================================================//

const CURSOR_ON_FRAMES: u32 = 3;
const CURSOR_OFF_FRAMES: u32 = 3;

const LABEL_WIDTH: i32 = 50;

//===========================================================================//

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Mode {
    Edit,
    Export,
    Goto,
    LoadFile,
    NewGlyph,
    Resize,
    SaveAs,
    SetColor(Color),
    SetGrid,
    SetMetadata,
    SetMetrics,
    SetTag,
    TestSentence,
}

impl Mode {
    fn tab_completion(self) -> Option<TabCompletion> {
        match self {
            Mode::LoadFile => Some(TabCompletion::LoadableFiles),
            Mode::Export | Mode::SaveAs => Some(TabCompletion::AllFiles),
            _ => None,
        }
    }
}

//===========================================================================//

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum TabCompletion {
    AllFiles,
    LoadableFiles,
}

impl TabCompletion {
    fn allow(self, file_name: &str) -> bool {
        match self {
            TabCompletion::AllFiles => true,
            TabCompletion::LoadableFiles => {
                file_name.ends_with(".ahi") || file_name.ends_with(".ahf")
            }
        }
    }
}

//===========================================================================//

struct TextBox {
    byte_index: usize,
    cursor_blink: u32,
    text: String,
}

impl TextBox {
    pub fn new() -> TextBox {
        TextBox { byte_index: 0, cursor_blink: 0, text: String::new() }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, text: String) {
        self.byte_index = text.len();
        self.text = text;
        self.cursor_blink = 0;
    }
}

impl GuiElement<(), ()> for TextBox {
    fn draw(&self, _: &(), resources: &Resources, canvas: &mut Canvas) {
        let rect = canvas.rect();
        let rect_width = rect.width() as i32;
        let font = resources.font();
        let text_width = font.text_width(&self.text);
        let text_left = cmp::min(4, rect_width - 4 - text_width);
        canvas.fill_rect((128, 128, 128, 255), rect);
        canvas.draw_string(font, text_left, 4, &self.text);
        canvas.draw_rect((255, 255, 255, 255), rect);
        if self.cursor_blink < CURSOR_ON_FRAMES {
            let cursor_x =
                text_left + font.text_width(&self.text[..self.byte_index]);
            let cursor_rect =
                Rect::new(cursor_x, rect.y() + 3, 1, rect.height() - 6);
            canvas.fill_rect((255, 255, 0, 255), cursor_rect);
        }
    }

    fn on_event(&mut self, event: &Event, _: &mut ()) -> Action<()> {
        match event {
            &Event::ClockTick => {
                let was_on = self.cursor_blink < CURSOR_ON_FRAMES;
                self.cursor_blink = (self.cursor_blink + 1)
                    % (CURSOR_ON_FRAMES + CURSOR_OFF_FRAMES);
                let is_on = self.cursor_blink < CURSOR_ON_FRAMES;
                Action::redraw_if(was_on != is_on)
            }
            &Event::KeyDown(Keycode::Backspace, keymod) => {
                if self.byte_index > 0 {
                    let rest = self.text.split_off(self.byte_index);
                    if keymod == ALT {
                        let mut popped_non_slash = false;
                        loop {
                            match self.text.pop() {
                                None => break,
                                Some('/') => {
                                    if popped_non_slash {
                                        self.text.push('/');
                                        break;
                                    }
                                }
                                Some(_) => {
                                    popped_non_slash = true;
                                }
                            }
                        }
                    } else {
                        self.text.pop();
                    }
                    self.byte_index = self.text.len();
                    self.text.push_str(&rest);
                    self.cursor_blink = 0;
                    Action::redraw().and_stop()
                } else {
                    Action::ignore()
                }
            }
            &Event::KeyDown(Keycode::Up, _) => {
                if self.byte_index > 0 {
                    self.byte_index = 0;
                    self.cursor_blink = 0;
                    Action::redraw().and_stop()
                } else {
                    Action::ignore()
                }
            }
            &Event::KeyDown(Keycode::Down, _) => {
                if self.byte_index < self.text.len() {
                    self.byte_index = self.text.len();
                    self.cursor_blink = 0;
                    Action::redraw().and_stop()
                } else {
                    Action::ignore()
                }
            }
            &Event::KeyDown(Keycode::Left, _) => {
                if self.byte_index > 0 {
                    let mut new_byte_index = self.byte_index - 1;
                    while !self.text.is_char_boundary(new_byte_index) {
                        new_byte_index -= 1;
                    }
                    self.byte_index = new_byte_index;
                    self.cursor_blink = 0;
                    Action::redraw().and_stop()
                } else {
                    Action::ignore()
                }
            }
            &Event::KeyDown(Keycode::Right, _) => {
                if self.byte_index < self.text.len() {
                    let mut new_byte_index = self.byte_index + 1;
                    while !self.text.is_char_boundary(new_byte_index) {
                        new_byte_index += 1;
                    }
                    self.byte_index = new_byte_index;
                    self.cursor_blink = 0;
                    Action::redraw().and_stop()
                } else {
                    Action::ignore()
                }
            }
            &Event::KeyDown(_, _) => Action::ignore().and_stop(),
            &Event::TextInput(ref input) => {
                self.text.insert_str(self.byte_index, input);
                self.byte_index += input.len();
                self.cursor_blink = 0;
                Action::redraw().and_stop()
            }
            _ => Action::ignore(),
        }
    }
}

//===========================================================================//

struct RgbaSwatch {
    rgba: (u8, u8, u8, u8),
}

impl RgbaSwatch {
    fn new(rgba: (u8, u8, u8, u8)) -> RgbaSwatch {
        RgbaSwatch { rgba }
    }
}

impl GuiElement<(), (u8, u8, u8, u8)> for RgbaSwatch {
    fn draw(&self, _: &(), _: &Resources, canvas: &mut Canvas) {
        let rect = canvas.rect();
        let inner = shrink(rect, 2);
        let a = self.rgba.3;
        if a < u8::MAX {
            canvas.draw_rect((0, 0, 0, 255), inner);
            canvas.draw_rect((0, 0, 0, 255), shrink(inner, 2));
            canvas.draw_rect((0, 0, 0, 255), shrink(inner, 4));
        }
        if a > 0 {
            canvas.fill_rect(self.rgba, inner);
        }
        canvas.draw_rect((255, 255, 255, 255), shrink(rect, 1));
    }

    fn on_event(
        &mut self,
        event: &Event,
        _: &mut (),
    ) -> Action<(u8, u8, u8, u8)> {
        match event {
            &Event::MouseDown(_) => Action::redraw().and_return(self.rgba),
            _ => Action::ignore(),
        }
    }
}

//===========================================================================//

struct RgbaPanel {
    swatches: AggregateElement<(), (u8, u8, u8, u8)>,
}

impl RgbaPanel {
    const SWATCH_SIZE: i32 = 18;
    const MARGIN: i32 = 6;
    const NUM_COLS: i32 = 16;
    const NUM_ROWS: i32 = 6;
    const WIDTH: u32 = (RgbaPanel::SWATCH_SIZE * RgbaPanel::NUM_COLS
        + RgbaPanel::MARGIN * 2) as u32;
    const HEIGHT: u32 = (RgbaPanel::SWATCH_SIZE * RgbaPanel::NUM_ROWS
        + RgbaPanel::MARGIN * 2) as u32;

    fn new() -> RgbaPanel {
        let elements: Vec<Box<dyn GuiElement<(), (u8, u8, u8, u8)>>> = vec![
            // Default palette:
            RgbaPanel::swatch(0, 0, (0, 0, 0, 0)),
            RgbaPanel::swatch(1, 0, (0, 0, 0, 255)),
            RgbaPanel::swatch(2, 0, (127, 0, 0, 255)),
            RgbaPanel::swatch(3, 0, (255, 0, 0, 255)),
            RgbaPanel::swatch(4, 0, (0, 127, 0, 255)),
            RgbaPanel::swatch(5, 0, (0, 255, 0, 255)),
            RgbaPanel::swatch(6, 0, (127, 127, 0, 255)),
            RgbaPanel::swatch(7, 0, (255, 255, 0, 255)),
            RgbaPanel::swatch(8, 0, (0, 0, 127, 255)),
            RgbaPanel::swatch(9, 0, (0, 0, 255, 255)),
            RgbaPanel::swatch(10, 0, (127, 0, 127, 255)),
            RgbaPanel::swatch(11, 0, (255, 0, 255, 255)),
            RgbaPanel::swatch(12, 0, (0, 127, 127, 255)),
            RgbaPanel::swatch(13, 0, (0, 255, 255, 255)),
            RgbaPanel::swatch(14, 0, (127, 127, 127, 255)),
            RgbaPanel::swatch(15, 0, (255, 255, 255, 255)),
            // NES palette:
            RgbaPanel::swatch(0, 2, (84, 84, 84, 255)),
            RgbaPanel::swatch(1, 2, (0, 30, 116, 255)),
            RgbaPanel::swatch(2, 2, (8, 16, 144, 255)),
            RgbaPanel::swatch(3, 2, (48, 0, 136, 255)),
            RgbaPanel::swatch(4, 2, (68, 0, 100, 255)),
            RgbaPanel::swatch(5, 2, (92, 0, 48, 255)),
            RgbaPanel::swatch(6, 2, (84, 4, 0, 255)),
            RgbaPanel::swatch(7, 2, (60, 24, 0, 255)),
            RgbaPanel::swatch(8, 2, (32, 42, 0, 255)),
            RgbaPanel::swatch(9, 2, (8, 58, 0, 255)),
            RgbaPanel::swatch(10, 2, (0, 64, 0, 255)),
            RgbaPanel::swatch(11, 2, (0, 60, 0, 255)),
            RgbaPanel::swatch(12, 2, (0, 50, 60, 255)),
            RgbaPanel::swatch(0, 3, (152, 150, 152, 255)),
            RgbaPanel::swatch(1, 3, (8, 76, 196, 255)),
            RgbaPanel::swatch(2, 3, (48, 50, 236, 255)),
            RgbaPanel::swatch(3, 3, (92, 30, 228, 255)),
            RgbaPanel::swatch(4, 3, (136, 20, 176, 255)),
            RgbaPanel::swatch(5, 3, (160, 20, 100, 255)),
            RgbaPanel::swatch(6, 3, (152, 34, 32, 255)),
            RgbaPanel::swatch(7, 3, (120, 60, 0, 255)),
            RgbaPanel::swatch(8, 3, (84, 90, 0, 255)),
            RgbaPanel::swatch(9, 3, (40, 114, 0, 255)),
            RgbaPanel::swatch(10, 3, (8, 124, 0, 255)),
            RgbaPanel::swatch(11, 3, (0, 118, 40, 255)),
            RgbaPanel::swatch(12, 3, (0, 102, 120, 255)),
            RgbaPanel::swatch(13, 3, (0, 0, 0, 255)),
            RgbaPanel::swatch(0, 4, (236, 238, 236, 255)),
            RgbaPanel::swatch(1, 4, (76, 154, 236, 255)),
            RgbaPanel::swatch(2, 4, (120, 124, 236, 255)),
            RgbaPanel::swatch(3, 4, (176, 98, 236, 255)),
            RgbaPanel::swatch(4, 4, (228, 84, 236, 255)),
            RgbaPanel::swatch(5, 4, (236, 88, 180, 255)),
            RgbaPanel::swatch(6, 4, (236, 106, 100, 255)),
            RgbaPanel::swatch(7, 4, (212, 136, 32, 255)),
            RgbaPanel::swatch(8, 4, (160, 170, 0, 255)),
            RgbaPanel::swatch(9, 4, (116, 196, 0, 255)),
            RgbaPanel::swatch(10, 4, (76, 208, 32, 255)),
            RgbaPanel::swatch(11, 4, (56, 204, 108, 255)),
            RgbaPanel::swatch(12, 4, (56, 180, 204, 255)),
            RgbaPanel::swatch(13, 4, (60, 60, 60, 255)),
            RgbaPanel::swatch(1, 5, (168, 204, 236, 255)),
            RgbaPanel::swatch(2, 5, (188, 188, 236, 255)),
            RgbaPanel::swatch(3, 5, (212, 178, 236, 255)),
            RgbaPanel::swatch(4, 5, (236, 174, 236, 255)),
            RgbaPanel::swatch(5, 5, (236, 174, 212, 255)),
            RgbaPanel::swatch(6, 5, (236, 180, 176, 255)),
            RgbaPanel::swatch(7, 5, (228, 196, 144, 255)),
            RgbaPanel::swatch(8, 5, (204, 210, 120, 255)),
            RgbaPanel::swatch(9, 5, (180, 222, 120, 255)),
            RgbaPanel::swatch(10, 5, (168, 226, 144, 255)),
            RgbaPanel::swatch(11, 5, (152, 226, 180, 255)),
            RgbaPanel::swatch(12, 5, (160, 214, 228, 255)),
            RgbaPanel::swatch(13, 5, (160, 162, 160, 255)),
            // Game Boy palette:
            RgbaPanel::swatch(15, 2, (208, 224, 64, 255)),
            RgbaPanel::swatch(15, 3, (160, 168, 48, 255)),
            RgbaPanel::swatch(15, 4, (96, 112, 40, 255)),
            RgbaPanel::swatch(15, 5, (56, 72, 40, 255)),
        ];
        RgbaPanel { swatches: AggregateElement::new(elements) }
    }

    fn swatch(
        col: i32,
        row: i32,
        rgba: (u8, u8, u8, u8),
    ) -> Box<dyn GuiElement<(), (u8, u8, u8, u8)>> {
        let left = RgbaPanel::MARGIN + RgbaPanel::SWATCH_SIZE * col;
        let top = RgbaPanel::MARGIN + RgbaPanel::SWATCH_SIZE * row;
        Box::new(SubrectElement::new(
            RgbaSwatch::new(rgba),
            Rect::new(
                left,
                top,
                RgbaPanel::SWATCH_SIZE as u32,
                RgbaPanel::SWATCH_SIZE as u32,
            ),
        ))
    }
}
impl GuiElement<(), (u8, u8, u8, u8)> for RgbaPanel {
    fn draw(&self, state: &(), resources: &Resources, canvas: &mut Canvas) {
        let rect = canvas.rect();
        canvas.fill_rect((128, 128, 128, 255), rect);
        canvas.draw_rect((255, 255, 255, 255), rect);
        self.swatches.draw(state, resources, canvas);
    }

    fn on_event(
        &mut self,
        event: &Event,
        state: &mut (),
    ) -> Action<(u8, u8, u8, u8)> {
        self.swatches.on_event(event, state)
    }
}

//===========================================================================//

struct FileMatch {
    file_name: String,
    file_path: String,
}

impl FileMatch {
    fn new(file_name: String, file_path: String) -> FileMatch {
        FileMatch { file_name, file_path }
    }
}

impl GuiElement<(), String> for FileMatch {
    fn draw(&self, _: &(), resources: &Resources, canvas: &mut Canvas) {
        let rect = canvas.rect();
        canvas.draw_string(
            resources.font(),
            rect.left(),
            rect.top(),
            &self.file_name,
        );
    }

    fn on_event(&mut self, event: &Event, _: &mut ()) -> Action<String> {
        match event {
            &Event::MouseDown(_) => {
                Action::redraw().and_return(self.file_path.clone())
            }
            _ => Action::ignore(),
        }
    }
}

//===========================================================================//

struct MatchesPanel {
    left: i32,
    top: i32,
    matches: AggregateElement<(), String>,
}

impl MatchesPanel {
    const MATCH_HEIGHT: u32 = 14;
    const MARGIN: u32 = 4;
    const WIDTH: u32 = 360;

    fn new(left: i32, top: i32) -> MatchesPanel {
        MatchesPanel { left, top, matches: AggregateElement::empty() }
    }

    fn is_empty(&self) -> bool {
        self.matches.is_empty()
    }

    fn set_matches(&mut self, matches: Vec<(String, String)>) {
        let elements = matches
            .into_iter()
            .enumerate()
            .map(|(row, (file_name, file_path))| {
                MatchesPanel::make_match(row, file_name, file_path)
            })
            .collect();
        self.matches = AggregateElement::new(elements);
    }

    fn clear_matches(&mut self) {
        self.matches = AggregateElement::empty()
    }

    fn make_match(
        row: usize,
        file_name: String,
        file_path: String,
    ) -> Box<dyn GuiElement<(), String>> {
        let left = MatchesPanel::MARGIN as i32;
        let top = (MatchesPanel::MARGIN as i32)
            + (MatchesPanel::MATCH_HEIGHT as i32) * (row as i32);
        let width = MatchesPanel::WIDTH - MatchesPanel::MARGIN * 2;
        let height = MatchesPanel::MATCH_HEIGHT;
        Box::new(SubrectElement::new(
            FileMatch::new(file_name, file_path),
            Rect::new(left, top, width, height),
        ))
    }
}

impl GuiElement<(), String> for MatchesPanel {
    fn draw(&self, state: &(), resources: &Resources, canvas: &mut Canvas) {
        if !self.matches.is_empty() {
            let rect = Rect::new(
                self.left,
                self.top,
                MatchesPanel::WIDTH,
                MatchesPanel::MARGIN
                    + MatchesPanel::MATCH_HEIGHT * (self.matches.len() as u32),
            );
            canvas.fill_rect((128, 128, 128, 255), rect);
            canvas.draw_rect((255, 255, 255, 255), rect);
            let mut subcanvas = canvas.subcanvas(rect);
            self.matches.draw(state, resources, &mut subcanvas);
        }
    }

    fn on_event(&mut self, event: &Event, state: &mut ()) -> Action<String> {
        self.matches.on_event(&event.translate(-self.left, -self.top), state)
    }
}

//===========================================================================//

pub struct ModalTextBox {
    left: i32,
    top: i32,
    mode: Mode,
    textbox: SubrectElement<TextBox>,
    rgba_panel: SubrectElement<RgbaPanel>,
    matches_panel: MatchesPanel,
}

impl ModalTextBox {
    pub fn new(left: i32, top: i32) -> ModalTextBox {
        ModalTextBox {
            left,
            top,
            mode: Mode::Edit,
            textbox: SubrectElement::new(
                TextBox::new(),
                Rect::new(
                    left + LABEL_WIDTH,
                    top,
                    (440 - LABEL_WIDTH) as u32,
                    18,
                ),
            ),
            rgba_panel: SubrectElement::new(
                RgbaPanel::new(),
                Rect::new(
                    left + LABEL_WIDTH,
                    top + 20,
                    RgbaPanel::WIDTH,
                    RgbaPanel::HEIGHT,
                ),
            ),
            matches_panel: MatchesPanel::new(left + LABEL_WIDTH, top + 20),
        }
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: Mode, text: String) {
        self.mode = mode;
        self.textbox.inner_mut().set_text(text);
        self.matches_panel.clear_matches();
    }

    pub fn clear_mode(&mut self) {
        self.mode = Mode::Edit;
        self.textbox.inner_mut().set_text(String::new());
        self.matches_panel.clear_matches();
    }

    fn tab_complete(&mut self) -> Action<(Mode, String)> {
        if let Some(tab_completion) = self.mode.tab_completion() {
            match tab_complete_path(
                tab_completion,
                self.textbox.inner().text(),
            ) {
                Ok((path, matches)) => {
                    self.textbox.inner_mut().set_text(path);
                    if matches.len() > 1 {
                        self.matches_panel.set_matches(matches);
                    } else {
                        self.matches_panel.clear_matches();
                    }
                    Action::redraw().and_stop()
                }
                Err(_) => Action::ignore().and_stop(),
            }
        } else {
            Action::ignore()
        }
    }
}

impl GuiElement<EditorState, (Mode, String)> for ModalTextBox {
    fn draw(
        &self,
        state: &EditorState,
        resources: &Resources,
        canvas: &mut Canvas,
    ) {
        if self.mode == Mode::Edit {
            let font = resources.font();
            canvas.draw_string(
                font,
                self.left + LABEL_WIDTH + 4,
                self.top + 4,
                state.filepath(),
            );
        } else {
            self.textbox.draw(&(), resources, canvas);
            if let Mode::SetColor(_) = self.mode {
                self.rgba_panel.draw(&(), resources, canvas);
            } else if self.mode.tab_completion().is_some() {
                self.matches_panel.draw(&(), resources, canvas);
            }
        }
        let label = match self.mode {
            Mode::Edit => "Path:",
            Mode::Export => "Export:",
            Mode::Goto => "Goto:",
            Mode::LoadFile => "Load:",
            Mode::NewGlyph => "Char:",
            Mode::Resize => "Size:",
            Mode::SaveAs => "Save:",
            Mode::SetColor(_) => "Color:",
            Mode::SetGrid => "Grid:",
            Mode::SetMetadata => "Meta:",
            Mode::SetMetrics => "Metrics:",
            Mode::SetTag => "Tag:",
            Mode::TestSentence => "Text:",
        };
        let font = resources.font();
        let text_width = font.text_width(label);
        canvas.draw_string(
            font,
            self.left + LABEL_WIDTH - text_width - 2,
            self.top + 4,
            label,
        );
    }

    fn on_event(
        &mut self,
        event: &Event,
        _: &mut EditorState,
    ) -> Action<(Mode, String)> {
        if self.mode == Mode::Edit {
            return Action::ignore();
        }
        let mut action = match event {
            &Event::KeyDown(Keycode::Escape, _) => {
                self.clear_mode();
                Action::redraw().and_stop()
            }
            &Event::KeyDown(Keycode::Return, _) => {
                let text = self.textbox.inner().text().to_string();
                Action::redraw().and_return((self.mode, text))
            }
            &Event::KeyDown(Keycode::Tab, _) => self.tab_complete(),
            _ => Action::ignore(),
        };
        if !action.should_stop() {
            let subaction = self.textbox.on_event(event, &mut ());
            action.merge(subaction.but_no_value());
        }
        if !action.should_stop() {
            if !self.matches_panel.is_empty() {
                let mut subaction =
                    self.matches_panel.on_event(event, &mut ());
                if let Some(file_path) = subaction.take_value() {
                    self.textbox.inner_mut().set_text(file_path);
                    self.matches_panel.clear_matches();
                    action.merge(Action::redraw().and_stop());
                } else {
                    action.merge(subaction.but_no_value());
                }
            }
        }
        if !action.should_stop() {
            if let Mode::SetColor(_) = self.mode {
                let mut subaction = self.rgba_panel.on_event(event, &mut ());
                if let Some((r, g, b, a)) = subaction.take_value() {
                    let text = format!("{:02X}{:02X}{:02X}{:02X}", r, g, b, a);
                    action
                        .merge(Action::redraw().and_return((self.mode, text)));
                } else {
                    action.merge(subaction.but_no_value());
                }
            }
        }
        if !action.should_stop() {
            action = action.and_stop();
        }
        action
    }
}

//===========================================================================//

fn shrink(rect: Rect, by: i32) -> Rect {
    Rect::new(
        rect.x() + by,
        rect.y() + by,
        cmp::max((rect.width() as i32) - 2 * by, 0) as u32,
        cmp::max((rect.height() as i32) - 2 * by, 0) as u32,
    )
}

fn join_to_string(dir: &Path, file_name: &str) -> io::Result<String> {
    let mut file_path = dir.join(file_name);
    if file_path.is_dir() {
        file_path.push("");
    }
    file_path
        .into_os_string()
        .into_string()
        .map_err(|_| io::Error::new(io::ErrorKind::Other, ""))
}

fn tab_complete_path(
    tab_completion: TabCompletion,
    path_string: &str,
) -> io::Result<(String, Vec<(String, String)>)> {
    let path = Path::new(path_string);
    let (dir, prefix): (&Path, &str) = if path_string.ends_with('/') {
        (path, "")
    } else {
        (
            path.parent()
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, ""))?,
            path.file_name().map(OsStr::to_str).unwrap_or(None).unwrap_or(""),
        )
    };

    let mut file_names_and_paths = Vec::<(String, String)>::new();
    for entry_result in dir.read_dir()? {
        let entry = entry_result?;
        let file_name = entry.file_name().to_str().unwrap_or("").to_string();
        if file_name.starts_with(prefix) {
            if entry.file_type()?.is_dir() || tab_completion.allow(&file_name)
            {
                let file_path = join_to_string(dir, &file_name)?;
                file_names_and_paths.push((file_name, file_path));
            }
        }
    }
    file_names_and_paths.sort();

    if let Some((first, _)) = file_names_and_paths.first() {
        let mut completed = String::new();
        for chr in first.chars() {
            let mut candidate = completed.clone();
            candidate.push(chr);
            if !file_names_and_paths
                .iter()
                .all(|(name, _)| name.starts_with(&candidate))
            {
                break;
            }
            completed = candidate;
        }
        Ok((join_to_string(dir, &completed)?, file_names_and_paths))
    } else {
        Err(io::Error::new(io::ErrorKind::Other, ""))
    }
}

//===========================================================================//
