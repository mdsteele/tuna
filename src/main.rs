// TODO:
// - Undo/redo
// - Icons for tool pickers
// - Select/move/cut/copy/paste
// - Unsaved changes indicator

extern crate ahi;
extern crate sdl2;

use ahi::Image;
use sdl2::event::Event;
use sdl2::keyboard;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::rect::Rect;
use sdl2::render::Renderer;
use std::io;
use std::fs::File;

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
  filepath: String,
  images: Vec<Image>,
  current_image: usize,
  tool: Tool,
  prev_tool: Tool,
}

impl EditorState {
  fn new(filepath: String, mut images: Vec<Image>) -> EditorState {
    if images.is_empty() {
      images.push(Image::new(32, 32));
    }
    EditorState {
      color: 1,
      filepath: filepath,
      images: images,
      current_image: 0,
      tool: Tool::Pencil,
      prev_tool: Tool::Pencil,
    }
  }

  fn image(&self) -> &Image {
    return &self.images[self.current_image];
  }

  fn image_mut(&mut self) -> &mut Image {
    return &mut self.images[self.current_image];
  }

  fn image_at(&self, index: usize) -> &Image {
    return &self.images[index];
  }

  fn add_new_image(&mut self) {
    let (width, height) = self.image().size();
    self.current_image += 1;
    self.images.insert(self.current_image, Image::new(width, height));
  }

  fn try_delete_image(&mut self) -> bool {
    if self.images.len() > 1 {
      self.images.remove(self.current_image);
      if self.current_image == self.images.len() {
        self.current_image -= 1;
      }
      true
    } else { false }
  }

  fn save_to_file(&self) -> io::Result<()> {
    let mut file = try!(File::create(&self.filepath));
    Image::write(&mut file, &self.images)
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

  fn pick_tool(&self, state: &mut EditorState) -> bool {
    if state.tool == self.tool { return false; }
    state.prev_tool = state.tool;
    state.tool = self.tool;
    true
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
          return self.pick_tool(state);
        }
      },
      &Event::KeyDown{keycode: Some(key), ..} => {
        if key == self.key {
          return self.pick_tool(state);
        }
      },
      _ => {}
    }
    return false;
  }
}

/*===========================================================================*/

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
    } else { None }
  }

  fn pick(&self, state: &mut EditorState) -> bool {
    if let Some(index) = self.index(state) {
      state.current_image = index;
      true
    } else { false }
  }
}

impl GuiElement<EditorState> for ImagePicker {
  fn draw(&self, state: &EditorState, renderer: &mut Renderer) {
    if let Some(index) = self.index(state) {
      render_image(renderer, state.image_at(index),
                   self.left + 2, self.top + 2, 1);
      if self.delta == 0 {
        renderer.set_draw_color(Color::RGB(255, 255, 127));
      } else {
        renderer.set_draw_color(Color::RGB(127, 127, 63));
      }
    } else {
      renderer.set_draw_color(Color::RGB(0, 0, 0));
    }
    renderer.draw_rect(self.rect()).unwrap();
  }

  fn handle_event(&mut self, event: &Event, state: &mut EditorState) -> bool {
    match event {
      &Event::MouseButtonDown{x, y, ..} => {
        if self.rect().contains((x, y)) {
          return self.pick(state);
        }
      },
      _ => {}
    }
    return false;
  }
}

/*===========================================================================*/

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
    state.current_image = modulo((state.current_image as i32) + self.delta,
                                 state.images.len() as i32) as usize;
    true
  }
}

impl GuiElement<EditorState> for NextPrevImage {
  fn draw(&self, _: &EditorState, renderer: &mut Renderer) {
    renderer.set_draw_color(Color::RGB(63, 0, 127));
    renderer.fill_rect(self.rect()).unwrap();
  }

  fn handle_event(&mut self, event: &Event, state: &mut EditorState) -> bool {
    match event {
      &Event::MouseButtonDown{x, y, ..} => {
        if self.rect().contains((x, y)) {
          return self.increment(state);
        }
      },
      &Event::KeyDown{keycode: Some(key), ..} => {
        if key == self.key {
          return self.increment(state);
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
  max_size: u32,
}

impl Canvas {
  fn new(left: i32, top: i32, max_size: u32) -> Canvas {
    Canvas {
      left: left,
      top: top,
      max_size: max_size,
    }
  }

  fn scale(&self, state: &EditorState) -> u32 {
    let (width, height) = state.image().size();
    std::cmp::max(1, self.max_size / std::cmp::max(width, height))
  }

  fn rect(&self, state: &EditorState) -> Rect {
    let scale = self.scale(state);
    let (width, height) = state.image().size();
    Rect::new(self.left, self.top, width * scale, height * scale)
  }

  fn mouse_to_row_col(&self, x: i32, y: i32,
                      state: &EditorState) -> Option<(u32, u32)> {
    let scale = self.scale(state) as i32;
    let col = (x - self.left) / scale;
    let row = (y - self.top) / scale;
    let (width, height) = state.image().size();
    if col < 0 || col >= (width as i32) ||
       row < 0 || row >= (height as i32) { None }
    else { Some((col as u32, row as u32)) }
  }

  fn try_paint(&self, x: i32, y: i32, state: &mut EditorState) -> bool {
    if let Some((col, row)) = self.mouse_to_row_col(x, y, state) {
      state.image_mut()[(col, row)] = state.color;
      true
    } else { false }
  }

  fn try_eyedrop(&self, x: i32, y: i32, state: &mut EditorState) -> bool {
    if let Some((col, row)) = self.mouse_to_row_col(x, y, state) {
      state.color = state.image()[(col, row)];
      state.tool = state.prev_tool;
      true
    } else { false }
  }

  fn try_flood_fill(&self, x: i32, y: i32, state: &mut EditorState) -> bool {
    if let Some((col, row)) = self.mouse_to_row_col(x, y, state) {
      let to_color = state.color;
      let image = state.image_mut();
      let from_color = image[(col, row)];
      if from_color == to_color { return false; }
      image[(col, row)] = to_color;
      let mut stack: Vec<(u32, u32)> = vec![(col, row)];
      while let Some((col, row)) = stack.pop() {
        let mut next: Vec<(u32, u32)> = vec![];
        if col > 0 { next.push((col - 1, row)); }
        if col < 31 { next.push((col + 1, row)); }
        if row > 0 { next.push((col, row - 1)); }
        if row < 31 { next.push((col, row + 1)); }
        for coords in next {
          if image[coords] == from_color {
            image[coords] = to_color;
            stack.push(coords);
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
    renderer.draw_rect(expand(self.rect(state), 2)).unwrap();
    render_image(renderer, state.image(), self.left, self.top,
                 self.scale(state));
  }

  fn handle_event(&mut self, event: &Event, state: &mut EditorState) -> bool {
    match event {
      &Event::MouseButtonDown{x, y, ..} => {
        match state.tool {
          Tool::PaintBucket => {
            return self.try_flood_fill(x, y, state);
          },
          Tool::Pencil => {
            return self.try_paint(x, y, state);
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
              return self.try_paint(x, y, state);
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

fn modulo(a: i32, b: i32) -> i32 {
  if b == 0 { panic!(); }
  let remainder = a % b;
  if remainder == 0 { 0 }
  else if (a < 0) ^ (b < 0) { remainder + b }
  else { remainder }
}

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
  for row in 0..image.height() {
    for col in 0..image.width() {
      let pixel: u8 = image[(col, row)];
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

fn load_from_file(path: &String) -> io::Result<Vec<Image>> {
  let mut file = try!(File::open(path));
  Image::read(&mut file)
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

  let mut state = EditorState::new(filepath, images);
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
    Box::new(Canvas::new(8, 32, 256)),
    Box::new(Canvas::new(300, 32, 64)),
    Box::new(NextPrevImage::new(374, 8, -1, Keycode::Up)),
    Box::new(ImagePicker::new(372, 32, -2)),
    Box::new(ImagePicker::new(372, 70, -1)),
    Box::new(ImagePicker::new(372, 108, 0)),
    Box::new(ImagePicker::new(372, 146, 1)),
    Box::new(ImagePicker::new(372, 184, 2)),
    Box::new(NextPrevImage::new(374, 222, 1, Keycode::Down)),
  ];

  render_screen(&mut renderer, &state, &elements);

  let mut event_pump = sdl_context.event_pump().unwrap();
  loop {
    let mut needs_redraw = false;
    match event_pump.wait_event() {
      Event::Quit{..} => return,
      Event::KeyDown{keycode: Some(key), keymod: kmod, ..} if
          kmod.intersects(keyboard::LGUIMOD | keyboard::RGUIMOD) => {
        match key {
          Keycode::Backspace => {
            if state.try_delete_image() {
              needs_redraw = true;
            }
          },
          Keycode::N => {
            state.add_new_image();
            needs_redraw = true;
          },
          Keycode::S => { state.save_to_file().unwrap(); },
          _ => {}
        }
      },
      event => {
        for element in elements.iter_mut() {
          if element.handle_event(&event, &mut state) {
            needs_redraw = true;
          }
        }
      }
    }
    if needs_redraw {
      render_screen(&mut renderer, &state, &elements);
    }
  }
}
