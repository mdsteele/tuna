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

// TODO:
// - Image resizing
// - Open-file/Save-as

extern crate ahi;
extern crate sdl2;

use ahi::Image;
use sdl2::event::Event;
use sdl2::keyboard::{self, Keycode};
use sdl2::mouse::Mouse;
use sdl2::rect::{Point, Rect};
use std::fs::File;
use std::io;
use std::rc::Rc;

mod canvas;
use self::canvas::{Canvas, Sprite};

mod element;
use self::element::GuiElement;

mod palette;
use self::palette::ColorPalette;

mod state;
use self::state::{EditorState, Tool};

// ========================================================================= //

struct ToolPicker {
    tool: Tool,
    key: Keycode,
    left: i32,
    top: i32,
    icon: Sprite,
}

impl ToolPicker {
    fn new(left: i32,
           top: i32,
           tool: Tool,
           key: Keycode,
           icon: Sprite)
           -> ToolPicker {
        ToolPicker {
            tool: tool,
            key: key,
            left: left,
            top: top,
            icon: icon,
        }
    }

    fn rect(&self) -> Rect {
        Rect::new(self.left, self.top, 20, 20)
    }

    fn pick_tool(&self, state: &mut EditorState) -> bool {
        if state.tool == self.tool {
            return false;
        }
        state.unselect();
        state.prev_tool = state.tool;
        state.tool = self.tool;
        true
    }
}

impl GuiElement<EditorState> for ToolPicker {
    fn draw(&self, state: &EditorState, canvas: &mut Canvas) {
        let mut canvas = canvas.subcanvas(self.rect());
        if state.tool == self.tool {
            canvas.clear((255, 255, 255, 255));
        } else {
            canvas.clear((95, 95, 95, 255));
        }
        canvas.draw_sprite(&self.icon, Point::new(2, 2));
    }

    fn handle_event(&mut self,
                    event: &Event,
                    state: &mut EditorState)
                    -> bool {
        match event {
            &Event::MouseButtonDown { mouse_btn: Mouse::Left, x, y, .. } => {
                if self.rect().contains((x, y)) {
                    return self.pick_tool(state);
                }
            }
            &Event::KeyDown { keycode: Some(key), .. } => {
                if key == self.key {
                    return self.pick_tool(state);
                }
            }
            _ => {}
        }
        false
    }
}

// ========================================================================= //

struct ImagePicker {
    left: i32,
    top: i32,
    delta: i32,
}

impl ImagePicker {
    fn new(left: i32, top: i32, delta: i32) -> ImagePicker {
        ImagePicker {
            left: left,
            top: top,
            delta: delta,
        }
    }

    fn rect(&self) -> Rect {
        Rect::new(self.left, self.top, 36, 36)
    }

    fn index(&self, state: &EditorState) -> Option<usize> {
        let index = (state.current_image as i32) + self.delta;
        if index >= 0 && index < (state.images.len() as i32) {
            Some(index as usize)
        } else {
            None
        }
    }

    fn pick(&self, state: &mut EditorState) -> bool {
        if let Some(index) = self.index(state) {
            state.current_image = index;
            true
        } else {
            false
        }
    }
}

impl GuiElement<EditorState> for ImagePicker {
    fn draw(&self, state: &EditorState, canvas: &mut Canvas) {
        let color = if let Some(index) = self.index(state) {
            render_image(canvas,
                         state.image_at(index),
                         self.left + 2,
                         self.top + 2,
                         1);
            if self.delta == 0 {
                (255, 255, 127, 255)
            } else {
                (127, 127, 63, 255)
            }
        } else {
            (0, 0, 0, 255)
        };
        canvas.draw_rect(color, self.rect());
    }

    fn handle_event(&mut self,
                    event: &Event,
                    state: &mut EditorState)
                    -> bool {
        match event {
            &Event::MouseButtonDown { mouse_btn: Mouse::Left, x, y, .. } => {
                if self.rect().contains((x, y)) {
                    return self.pick(state);
                }
            }
            _ => {}
        }
        false
    }
}

// ========================================================================= //

struct NextPrevImage {
    left: i32,
    top: i32,
    delta: i32,
    key: Keycode,
}

impl NextPrevImage {
    fn new(left: i32, top: i32, delta: i32, key: Keycode) -> NextPrevImage {
        NextPrevImage {
            left: left,
            top: top,
            delta: delta,
            key: key,
        }
    }

    fn rect(&self) -> Rect {
        Rect::new(self.left, self.top, 32, 16)
    }

    fn increment(&self, state: &mut EditorState) -> bool {
        state.unselect();
        state.current_image =
            modulo((state.current_image as i32) + self.delta,
                   state.images.len() as i32) as usize;
        true
    }
}

impl GuiElement<EditorState> for NextPrevImage {
    fn draw(&self, _: &EditorState, canvas: &mut Canvas) {
        canvas.fill_rect((63, 0, 127, 255), self.rect());
    }

    fn handle_event(&mut self,
                    event: &Event,
                    state: &mut EditorState)
                    -> bool {
        match event {
            &Event::MouseButtonDown { mouse_btn: Mouse::Left, x, y, .. } => {
                if self.rect().contains((x, y)) {
                    return self.increment(state);
                }
            }
            &Event::KeyDown { keycode: Some(key), .. } => {
                if key == self.key {
                    return self.increment(state);
                }
            }
            _ => {}
        }
        false
    }
}

// ========================================================================= //

struct UnsavedIndicator {
    left: i32,
    top: i32,
}

impl UnsavedIndicator {
    fn new(left: i32, top: i32) -> UnsavedIndicator {
        UnsavedIndicator {
            left: left,
            top: top,
        }
    }

    fn rect(&self) -> Rect {
        Rect::new(self.left, self.top, 16, 16)
    }
}

impl GuiElement<EditorState> for UnsavedIndicator {
    fn draw(&self, state: &EditorState, canvas: &mut Canvas) {
        if state.is_unsaved() {
            canvas.fill_rect((255, 127, 0, 255), self.rect());
        }
    }

    fn handle_event(&mut self, _: &Event, _: &mut EditorState) -> bool {
        false
    }
}

// ========================================================================= //

struct FilePathTextBox {
    left: i32,
    top: i32,
    font: Rc<Vec<Sprite>>,
}

impl FilePathTextBox {
    fn new(left: i32, top: i32, font: Rc<Vec<Sprite>>) -> FilePathTextBox {
        FilePathTextBox {
            left: left,
            top: top,
            font: font,
        }
    }

    fn rect(&self) -> Rect {
        Rect::new(self.left, self.top, 324, 20)
    }
}

impl GuiElement<EditorState> for FilePathTextBox {
    fn draw(&self, state: &EditorState, canvas: &mut Canvas) {
        let rect = self.rect();
        render_string(canvas,
                      &self.font,
                      rect.x() + 2,
                      rect.y() + 2,
                      &state.filepath);
        canvas.draw_rect((255, 255, 255, 255), rect);
    }

    fn handle_event(&mut self, _: &Event, _: &mut EditorState) -> bool {
        false
    }
}

// ========================================================================= //

struct ImageCanvasDrag {
    from_selection: (i32, i32),
    from_pixel: (i32, i32),
    to_pixel: (i32, i32),
}

struct ImageCanvas {
    left: i32,
    top: i32,
    max_size: u32,
    drag_from_to: Option<ImageCanvasDrag>,
}

impl ImageCanvas {
    fn new(left: i32, top: i32, max_size: u32) -> ImageCanvas {
        ImageCanvas {
            left: left,
            top: top,
            max_size: max_size,
            drag_from_to: None,
        }
    }

    fn scale(&self, state: &EditorState) -> u32 {
        let (width, height) = state.image_size();
        std::cmp::max(1, self.max_size / std::cmp::max(width, height))
    }

    fn rect(&self, state: &EditorState) -> Rect {
        let scale = self.scale(state);
        let (width, height) = state.image_size();
        Rect::new(self.left, self.top, width * scale, height * scale)
    }

    fn dragged_rect(&self, state: &EditorState) -> Option<Rect> {
        if let Some(ref drag) = self.drag_from_to {
            let (fpx, fpy) = drag.from_pixel;
            let (tpx, tpy) = drag.to_pixel;
            let (from_col, from_row) = self.clamp_mouse_to_row_col(fpx,
                                                                   fpy,
                                                                   state);
            let (to_col, to_row) = self.clamp_mouse_to_row_col(tpx,
                                                               tpy,
                                                               state);
            let x = std::cmp::min(from_col, to_col) as i32;
            let y = std::cmp::min(from_row, to_row) as i32;
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
        (std::cmp::max(0, std::cmp::min(col, width as i32)) as u32,
         std::cmp::max(0, std::cmp::min(row, height as i32)) as u32)
    }

    fn try_paint(&self, x: i32, y: i32, state: &mut EditorState) -> bool {
        if let Some((col, row)) = self.mouse_to_row_col(x, y, state) {
            state.image_mut()[(col, row)] = state.color;
            true
        } else {
            false
        }
    }

    fn try_eyedrop(&self, x: i32, y: i32, state: &mut EditorState) -> bool {
        if let Some((col, row)) = self.mouse_to_row_col(x, y, state) {
            state.color = state.image()[(col, row)];
            state.tool = state.prev_tool;
            true
        } else {
            false
        }
    }

    fn try_flood_fill(&self, x: i32, y: i32, state: &mut EditorState) -> bool {
        if let Some((col, row)) = self.mouse_to_row_col(x, y, state) {
            let to_color = state.color;
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
        canvas.draw_rect((255, 255, 255, 255), expand(self.rect(state), 2));
        render_image(canvas, state.image(), self.left, self.top, scale);
        if let Some((ref selected, x, y)) = state.selection {
            let left = self.left + x * (scale as i32);
            let top = self.top + y * (scale as i32);
            render_image(canvas, selected, left, top, scale);
            canvas.draw_rect((255, 191, 255, 255),
                             Rect::new(left,
                                       top,
                                       selected.width() * scale,
                                       selected.height() * scale));
        } else if let Some(rect) = self.dragged_rect(state) {
            canvas.draw_rect((255, 255, 191, 255),
                             Rect::new(self.left + rect.x() * (scale as i32),
                                       self.top + rect.y() * (scale as i32),
                                       rect.width() * scale,
                                       rect.height() * scale));
        }
    }

    fn handle_event(&mut self,
                    event: &Event,
                    state: &mut EditorState)
                    -> bool {
        match event {
            &Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                return state.try_unselect_with_undo();
            }
            &Event::MouseButtonDown { mouse_btn: Mouse::Left, x, y, .. } => {
                if self.rect(state).contains((x, y)) {
                    match state.tool {
                        Tool::Eyedropper => {
                            return self.try_eyedrop(x, y, state);
                        }
                        Tool::PaintBucket => {
                            state.push_change();
                            return self.try_flood_fill(x, y, state);
                        }
                        Tool::Pencil => {
                            state.push_change();
                            return self.try_paint(x, y, state);
                        }
                        Tool::Select => {
                            let rect = if let Some((ref selected, x, y)) =
                                              state.selection {
                                Some(Rect::new(x,
                                               y,
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
                                    state.push_selection_move();
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
                            return true;
                        }
                    }
                } else {
                    self.drag_from_to = None;
                }
            }
            &Event::MouseButtonUp { mouse_btn: Mouse::Left, .. } => {
                match state.tool {
                    Tool::Select => {
                        if state.selection.is_none() {
                            if let Some(rect) = self.dragged_rect(state) {
                                state.select_with_undo(&rect);
                                self.drag_from_to = None;
                                return true;
                            }
                        }
                    }
                    _ => {}
                }
                self.drag_from_to = None;
            }
            &Event::MouseMotion { x, y, mousestate, .. } => {
                if mousestate.left() {
                    match state.tool {
                        Tool::Pencil => {
                            return self.try_paint(x, y, state);
                        }
                        Tool::Select => {
                            let scale = self.scale(state) as i32;
                            if let Some(ref mut drag) = self.drag_from_to {
                                drag.to_pixel = (x, y);
                                if let Some((_, ref mut sx, ref mut sy)) =
                                       state.selection {
                                    let (fsx, fsy) = drag.from_selection;
                                    let (fpx, fpy) = drag.from_pixel;
                                    *sx = fsx + (x - fpx) / scale;
                                    *sy = fsy + (y - fpy) / scale;
                                }
                                return true;
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        return false;
    }
}

// ========================================================================= //

fn modulo(a: i32, b: i32) -> i32 {
    if b == 0 {
        panic!();
    }
    let remainder = a % b;
    if remainder == 0 {
        0
    } else if (a < 0) ^ (b < 0) {
        remainder + b
    } else {
        remainder
    }
}

fn expand(rect: Rect, by: i32) -> Rect {
    Rect::new(rect.x() - by,
              rect.y() - by,
              ((rect.width() as i32) + 2 * by) as u32,
              ((rect.height() as i32) + 2 * by) as u32)
}

fn render_image(canvas: &mut Canvas,
                image: &Image,
                left: i32,
                top: i32,
                scale: u32) {
    for row in 0..image.height() {
        for col in 0..image.width() {
            let pixel = image[(col, row)];
            if pixel != ahi::Color::Transparent {
                canvas.fill_rect(pixel.rgba(),
                                 Rect::new(left + (scale * col) as i32,
                                           top + (scale * row) as i32,
                                           scale,
                                           scale));
            }
        }
    }
}

fn render_string(canvas: &mut Canvas,
                 font: &Vec<Sprite>,
                 left: i32,
                 top: i32,
                 string: &str) {
    let mut x = left;
    let mut y = top;
    for ch in string.chars() {
        if ch == '\n' {
            x = left;
            y += 24;
        } else {
            if ch >= '!' {
                let index = ch as usize - '!' as usize;
                if index < font.len() {
                    canvas.draw_sprite(&font[index], Point::new(x, y));
                }
            }
            x += 16;
        }
    }
}

fn render_screen(canvas: &mut Canvas,
                 state: &EditorState,
                 elements: &Vec<Box<GuiElement<EditorState>>>) {
    canvas.clear((64, 64, 64, 255));
    for element in elements {
        element.draw(state, canvas);
    }
    canvas.present();
}

fn load_from_file(path: &String) -> io::Result<Vec<Image>> {
    let mut file = try!(File::open(path));
    Image::read_all(&mut file)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (filepath, images) = if args.len() >= 2 {
        let filepath = &args[1];
        (filepath.clone(), load_from_file(filepath).unwrap())
    } else {
        ("out.ahi".to_string(), vec![])
    };

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let width: u32 = 32 * 15;
    let height: u32 = 32 * 10;

    let window = video_subsystem.window("AHI Editor", width, height)
      .position_centered()
      //.fullscreen_desktop()
      .build().unwrap();

    let mut renderer = window.renderer().build().unwrap();
    renderer.set_logical_size(width, height).unwrap();
    let mut canvas = Canvas::from_renderer(&mut renderer);

    let tool_icons = load_from_file(&"data/tool_icons.ahi".to_string())
                         .unwrap();
    let font: Rc<Vec<Sprite>> = Rc::new(load_from_file(&"data/font.ahi"
                                                            .to_string())
                                            .unwrap()
                                            .iter()
                                            .map(|image| {
                                                canvas.new_sprite(image)
                                            })
                                            .collect());

    let mut state = EditorState::new(filepath, images);
    let mut elements: Vec<Box<GuiElement<EditorState>>> = vec![
    Box::new(UnsavedIndicator::new(462, 2)),
    Box::new(ColorPalette::new(4, 4)),
    // Toolbox:
    Box::new(ToolPicker::new( 4, 296, Tool::Pencil,      Keycode::P,
                             canvas.new_sprite(&tool_icons[0]))),
    Box::new(ToolPicker::new(28, 296, Tool::PaintBucket, Keycode::K,
                             canvas.new_sprite(&tool_icons[1]))),
    Box::new(ToolPicker::new(52, 296, Tool::Eyedropper,  Keycode::Y,
                             canvas.new_sprite(&tool_icons[2]))),
    Box::new(ToolPicker::new(76, 296, Tool::Select,      Keycode::S,
                             canvas.new_sprite(&tool_icons[3]))),
    // Text box:
    Box::new(FilePathTextBox::new(152, 296, font.clone())),
    // Image canvases:
    Box::new(ImageCanvas::new(48, 32, 256)),
    Box::new(ImageCanvas::new(320, 32, 64)),
    // Images scrollbar:
    Box::new(NextPrevImage::new(414, 8, -1, Keycode::Up)),
    Box::new(ImagePicker::new(412, 32, -2)),
    Box::new(ImagePicker::new(412, 70, -1)),
    Box::new(ImagePicker::new(412, 108, 0)),
    Box::new(ImagePicker::new(412, 146, 1)),
    Box::new(ImagePicker::new(412, 184, 2)),
    Box::new(NextPrevImage::new(414, 222, 1, Keycode::Down)),
  ];

    render_screen(&mut canvas, &state, &elements);

    let mut event_pump = sdl_context.event_pump().unwrap();
    loop {
        let mut needs_redraw = false;
        match event_pump.wait_event() {
            Event::Quit { .. } => return,
            Event::KeyDown { keycode: Some(key), keymod: kmod, .. }
                if kmod.intersects(keyboard::LGUIMOD |
                                   keyboard::RGUIMOD) => {
                match key {
                    Keycode::Backspace => {
                        if state.try_delete_image() {
                            needs_redraw = true;
                        }
                    }
                    Keycode::A => {
                        state.select_all_with_undo();
                        needs_redraw = true;
                    }
                    Keycode::C => {
                        state.copy_selection();
                    }
                    Keycode::H => {
                        if kmod.intersects(keyboard::LSHIFTMOD |
                                           keyboard::RSHIFTMOD) {
                            state.flip_image_horz();
                            needs_redraw = true;
                        }
                    }
                    Keycode::N => {
                        state.add_new_image();
                        needs_redraw = true;
                    }
                    Keycode::S => {
                        state.save_to_file().unwrap();
                        needs_redraw = true;
                    }
                    Keycode::V => {
                        if kmod.intersects(keyboard::LSHIFTMOD |
                                           keyboard::RSHIFTMOD) {
                            state.flip_image_vert();
                        } else {
                            state.paste_selection();
                        }
                        needs_redraw = true;
                    }
                    Keycode::X => {
                        state.cut_selection();
                        needs_redraw = true;
                    }
                    Keycode::Z => {
                        if kmod.intersects(keyboard::LSHIFTMOD |
                                           keyboard::RSHIFTMOD) {
                            if state.redo() {
                                needs_redraw = true;
                            }
                        } else if state.undo() {
                            needs_redraw = true;
                        }
                    }
                    _ => {}
                }
            }
            event => {
                for element in elements.iter_mut() {
                    if element.handle_event(&event, &mut state) {
                        needs_redraw = true;
                    }
                }
            }
        }
        if needs_redraw {
            render_screen(&mut canvas, &state, &elements);
        }
    }
}

// ========================================================================= //
