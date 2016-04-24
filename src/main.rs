// TODO:
// - Load/save
// - Undo/redo
// - Go back to prev tool after using eyedropper
// - Icons for tool pickers
// - Editing multiple images
// - Switching between images
// - Select/move/cut/copy/paste
// - Support non-32x32 images

extern crate ahi;
extern crate sdl2;

use ahi::Image;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::rect::Rect;
use sdl2::render::Renderer;

/*===========================================================================*/

trait GuiElement<S> {
  fn draw(&self, state: &S, renderer: &mut Renderer);
  fn handle_event(&mut self, event: &Event, state: &mut S) -> bool;
}

/*===========================================================================*/

#[derive(Clone, Copy, Eq, PartialEq)]
enum Tool {
  Eyedropper,
  PaintBucket,
  Pencil,
}

struct EditorState {
  color: u8,
  image: Image,
  tool: Tool,
}

impl EditorState {
  fn new() -> EditorState {
    EditorState {
      color: 1,
      image: Image::new(32, 32),
      tool: Tool::Pencil,
    }
  }
}

/*===========================================================================*/

struct ColorPicker {
  key: Keycode,
  left: i32,
  top: i32,
  color: u8,
}

impl ColorPicker {
  fn new(left: i32, top: i32, color: u8, key: Keycode) -> ColorPicker {
    ColorPicker {
      left: left,
      top: top,
      color: color,
      key: key,
    }
  }

  fn rect(&self) -> Rect {
    Rect::new(self.left, self.top, 16, 16)
  }
}

impl GuiElement<EditorState> for ColorPicker {
  fn draw(&self, state: &EditorState, renderer: &mut Renderer) {
    let inner = expand(self.rect(), -2);
    if self.color == 0 {
      renderer.set_draw_color(Color::RGB(0, 0, 0));
      renderer.draw_rect(inner).unwrap();
      renderer.draw_line(Point::new(inner.left(), inner.top()),
                         Point::new(inner.right() - 1,
                                    inner.bottom() - 1)).unwrap();
      renderer.draw_line(Point::new(inner.left(), inner.bottom() - 1),
                         Point::new(inner.right() - 1, inner.top())).unwrap();
    } else {
      renderer.set_draw_color(hex_pixel_to_sdl_color(self.color));
      renderer.fill_rect(inner).unwrap();
    }
    if state.color == self.color {
      renderer.set_draw_color(Color::RGB(255, 255, 255));
      renderer.draw_rect(self.rect()).unwrap();
    }
  }

  fn handle_event(&mut self, event: &Event, state: &mut EditorState) -> bool {
    match event {
      &Event::MouseButtonDown{x, y, ..} => {
        if self.rect().contains((x, y)) {
          state.color = self.color;
          return true;
        }
      },
      &Event::KeyDown{keycode: Some(key), ..} => {
        if key == self.key {
          state.color = self.color;
          return true;
        }
      },
      _ => {}
    }
    return false;
  }
}

/*===========================================================================*/

struct ToolPicker {
  tool: Tool,
  key: Keycode,
  left: i32,
  top: i32,
}

impl ToolPicker {
  fn new(left: i32, top: i32, tool: Tool, key: Keycode) -> ToolPicker {
    ToolPicker {
      tool: tool,
      key: key,
      left: left,
      top: top,
    }
  }

  fn rect(&self) -> Rect {
    Rect::new(self.left, self.top, 16, 16)
  }
}

impl GuiElement<EditorState> for ToolPicker {
  fn draw(&self, state: &EditorState, renderer: &mut Renderer) {
    renderer.set_draw_color(Color::RGB(127, 63, 0));
    renderer.fill_rect(self.rect()).unwrap();
    if state.tool == self.tool {
      renderer.set_draw_color(Color::RGB(255, 255, 255));
      renderer.draw_rect(self.rect()).unwrap();
    }
  }

  fn handle_event(&mut self, event: &Event, state: &mut EditorState) -> bool {
    match event {
      &Event::MouseButtonDown{x, y, ..} => {
        if self.rect().contains((x, y)) {
          state.tool = self.tool;
          return true;
        }
      },
      &Event::KeyDown{keycode: Some(key), ..} => {
        if key == self.key {
          state.tool = self.tool;
          return true;
        }
      },
      _ => {}
    }
    return false;
  }
}

/*===========================================================================*/

struct Canvas {
  left: i32,
  top: i32,
  scale: u32,
}

impl Canvas {
  fn new(left: i32, top: i32, scale: u32) -> Canvas {
    Canvas {
      left: left,
      top: top,
      scale: scale,
    }
  }

  fn rect(&self) -> Rect {
    Rect::new(self.left, self.top, 32 * self.scale, 32 * self.scale)
  }

  fn mouse_to_row_col(&self, x: i32, y: i32) -> Option<(u32, u32)> {
    let col = (x - self.left) / (self.scale as i32);
    let row = (y - self.top) / (self.scale as i32);
    if col < 0 || col >= 32 || row < 0 || row >= 32 { None }
    else { Some((col as u32, row as u32)) }
  }

  fn try_paint(&self, x: i32, y: i32, color: u8, image: &mut Image) -> bool {
    if let Some((col, row)) = self.mouse_to_row_col(x, y) {
      image.pixels[(row * 32 + col) as usize] = color;
      true
    } else { false }
  }

  fn try_eyedrop(&self, x: i32, y: i32, state: &mut EditorState) -> bool {
    if let Some((col, row)) = self.mouse_to_row_col(x, y) {
      state.color = state.image.pixels[(32 * row + col) as usize];
      true
    } else { false }
  }

  fn try_flood_fill(&self, x: i32, y: i32, to_color: u8,
                    image: &mut Image) -> bool {
    if let Some((col, row)) = self.mouse_to_row_col(x, y) {
      let from_color = image.pixels[(row * 32 + col) as usize];
      if from_color == to_color { return false; }
      image.pixels[(row * 32 + col) as usize] = to_color;
      let mut stack: Vec<(u32, u32)> = vec![(col, row)];
      while let Some((col, row)) = stack.pop() {
        let mut next: Vec<(u32, u32)> = vec![];
        if col > 0 { next.push((col - 1, row)); }
        if col < 31 { next.push((col + 1, row)); }
        if row > 0 { next.push((col, row - 1)); }
        if row < 31 { next.push((col, row + 1)); }
        for (col, row) in next {
          if image.pixels[(row * 32 + col) as usize] == from_color {
            image.pixels[(row * 32 + col) as usize] = to_color;
            stack.push((col, row));
          }
        }
      }
      true
    } else { false }
  }
}

impl GuiElement<EditorState> for Canvas {
  fn draw(&self, state: &EditorState, renderer: &mut Renderer) {
    renderer.set_draw_color(Color::RGB(255, 255, 255));
    renderer.draw_rect(expand(self.rect(), 2)).unwrap();
    render_image(renderer, &state.image, self.left, self.top, self.scale);
  }

  fn handle_event(&mut self, event: &Event, state: &mut EditorState) -> bool {
    match event {
      &Event::MouseButtonDown{x, y, ..} => {
        match state.tool {
          Tool::PaintBucket => {
            return self.try_flood_fill(x, y, state.color, &mut state.image);
          },
          Tool::Pencil => {
            return self.try_paint(x, y, state.color, &mut state.image);
          },
          Tool::Eyedropper => {
            return self.try_eyedrop(x, y, state);
          },
        }
      },
      &Event::MouseMotion{x, y, mousestate, ..} => {
        if mousestate.left() {
          match state.tool {
            Tool::Pencil => {
              return self.try_paint(x, y, state.color, &mut state.image);
            },
            _ => {}
          }
        }
      },
      _ => {}
    }
    return false;
  }
}

/*===========================================================================*/

fn expand(rect: Rect, by: i32) -> Rect {
  Rect::new(rect.x() - by, rect.y() - by,
            ((rect.width() as i32) + 2 * by) as u32,
            ((rect.height() as i32) + 2 * by) as u32)
}

fn hex_pixel_to_sdl_color(pixel: u8) -> Color {
  return match pixel {
    0x1 => Color::RGB(  0,   0,   0),
    0x2 => Color::RGB(127,   0,   0),
    0x3 => Color::RGB(255,   0,   0),
    0x4 => Color::RGB(  0, 127,   0),
    0x5 => Color::RGB(  0, 255,   0),
    0x6 => Color::RGB(127, 127,   0),
    0x7 => Color::RGB(255, 255,   0),
    0x8 => Color::RGB(  0,   0, 127),
    0x9 => Color::RGB(  0,   0, 255),
    0xA => Color::RGB(127,   0, 127),
    0xB => Color::RGB(255,   0, 255),
    0xC => Color::RGB(  0, 127, 127),
    0xD => Color::RGB(  0, 255, 255),
    0xE => Color::RGB(127, 127, 127),
    0xF => Color::RGB(255, 255, 255),
    _ => Color::RGBA(0, 0, 0, 0),
  }
}

fn render_image(renderer: &mut Renderer, image: &Image,
                left: i32, top: i32, scale: u32) {
  for row in 0..image.height {
    for col in 0..image.width {
      let pixel: u8 = image.pixels[(row * image.width + col) as usize];
      if pixel != 0 {
          renderer.set_draw_color(hex_pixel_to_sdl_color(pixel));
          renderer.fill_rect(Rect::new(left + (scale * col) as i32,
                                       top + (scale * row) as i32,
                                       scale, scale)).unwrap();
      }
    }
  }
}

fn render_screen(renderer: &mut Renderer, state: &EditorState,
                 elements: &Vec<Box<GuiElement<EditorState>>>) {
  renderer.set_draw_color(Color::RGB(64, 64, 64));
  renderer.clear();
  for element in elements {
    element.draw(state, renderer);
  }
  renderer.present();
}

fn main() {
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

  let mut state = EditorState::new();
  let mut elements: Vec<Box<GuiElement<EditorState>>> = vec![
    Box::new(ColorPicker::new(  2, 2, 0x0, Keycode::Num0)),
    Box::new(ColorPicker::new( 20, 2, 0x1, Keycode::Num1)),
    Box::new(ColorPicker::new( 38, 2, 0x2, Keycode::Num2)),
    Box::new(ColorPicker::new( 56, 2, 0x3, Keycode::Num3)),
    Box::new(ColorPicker::new( 74, 2, 0x4, Keycode::Num4)),
    Box::new(ColorPicker::new( 92, 2, 0x5, Keycode::Num5)),
    Box::new(ColorPicker::new(110, 2, 0x6, Keycode::Num6)),
    Box::new(ColorPicker::new(128, 2, 0x7, Keycode::Num7)),
    Box::new(ColorPicker::new(146, 2, 0x8, Keycode::Num8)),
    Box::new(ColorPicker::new(164, 2, 0x9, Keycode::Num9)),
    Box::new(ColorPicker::new(182, 2, 0xA, Keycode::A)),
    Box::new(ColorPicker::new(200, 2, 0xB, Keycode::B)),
    Box::new(ColorPicker::new(218, 2, 0xC, Keycode::C)),
    Box::new(ColorPicker::new(236, 2, 0xD, Keycode::D)),
    Box::new(ColorPicker::new(254, 2, 0xE, Keycode::E)),
    Box::new(ColorPicker::new(272, 2, 0xF, Keycode::F)),
    Box::new(ToolPicker::new( 2, 302, Tool::Pencil,      Keycode::P)),
    Box::new(ToolPicker::new(20, 302, Tool::PaintBucket, Keycode::K)),
    Box::new(ToolPicker::new(38, 302, Tool::Eyedropper,  Keycode::Y)),
    Box::new(Canvas::new(32, 32, 8)),
    Box::new(Canvas::new(300, 32, 2)),
    Box::new(Canvas::new(300, 128, 1)),
  ];

  render_screen(&mut renderer, &state, &elements);

  let mut event_pump = sdl_context.event_pump().unwrap();
  loop {
    match event_pump.wait_event() {
      Event::Quit{..} => return,
      event => {
        let mut needs_redraw = false;
        for element in elements.iter_mut() {
          if element.handle_event(&event, &mut state) {
            needs_redraw = true;
          }
        }
        if needs_redraw {
          render_screen(&mut renderer, &state, &elements);
        }
      }
    }
  }
}
