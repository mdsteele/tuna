// TODO:
// - Image resizing
// - Open-file/Save-as

extern crate ahi;
extern crate sdl2;

use ahi::Image;
use sdl2::event::Event;
use sdl2::keyboard;
use sdl2::keyboard::Keycode;
use sdl2::mouse::Mouse;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Point;
use sdl2::rect::Rect;
use sdl2::render::Renderer;
use sdl2::render::Texture;
use sdl2::surface::Surface;
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
  Select,
}

const MAX_UNDOS: usize = 100;

enum Undo {
  AddImage(usize),
  ChangeImage(usize, Image),
  RemoveImage(usize, Image),
  SelectionBegin,
  SelectionCut(Image, i32, i32),
  SelectionEnd(Rect),
  SelectionMove(i32, i32),
  SelectionPaste,
}

enum Redo {
  AddImage(usize),
  ChangeImage(usize, Image),
  RemoveImage(usize),
  SelectionBegin(Rect),
  SelectionCut,
  SelectionEnd,
  SelectionMove(i32, i32),
  SelectionPaste(Image, i32, i32),
}

struct EditorState {
  color: u8,
  filepath: String,
  images: Vec<Image>,
  current_image: usize,
  selection: Option<(Image, i32, i32)>,
  clipboard: Option<(Image, i32, i32)>,
  tool: Tool,
  prev_tool: Tool,
  undo_stack: Vec<Undo>,
  redo_stack: Vec<Redo>,
  unsaved: bool,
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
      selection: None,
      clipboard: None,
      tool: Tool::Pencil,
      prev_tool: Tool::Pencil,
      undo_stack: Vec::new(),
      redo_stack: Vec::new(),
      unsaved: false,
    }
  }

  fn image(&self) -> &Image {
    &self.images[self.current_image]
  }

  fn image_mut(&mut self) -> &mut Image {
    self.unsaved = true;
    &mut self.images[self.current_image]
  }

  fn image_at(&self, index: usize) -> &Image {
    &self.images[index]
  }

  fn flip_image_horz(&mut self) {
    self.push_change();
    self.image_mut().flip_horz();
  }

  fn flip_image_vert(&mut self) {
    self.push_change();
    self.image_mut().flip_vert();
  }

  fn add_new_image(&mut self) {
    self.unselect();
    let (width, height) = self.image().size();
    self.current_image += 1;
    self.images.insert(self.current_image, Image::new(width, height));
    let undo = Undo::AddImage(self.current_image);
    self.push_undo(undo);
    self.unsaved = true;
  }

  fn try_delete_image(&mut self) -> bool {
    if self.images.len() > 1 {
      self.unselect();
      let image = self.images.remove(self.current_image);
      let undo = Undo::RemoveImage(self.current_image, image);
      self.push_undo(undo);
      if self.current_image == self.images.len() {
        self.current_image -= 1;
      }
      self.unsaved = true;
      true
    } else { false }
  }

  fn select_with_undo(&mut self, rect: &Rect) {
    self.select(rect);
    self.push_undo(Undo::SelectionBegin);
    self.tool = Tool::Select;
  }

  fn select_all_with_undo(&mut self) {
    let (width, height) = self.image().size();
    self.select_with_undo(&Rect::new(0, 0, width, height));
  }

  fn try_unselect_with_undo(&mut self) -> bool {
    if let Some(rect) = self.unselect() {
      self.push_undo(Undo::SelectionEnd(rect));
      true
    } else { false }
  }

  fn select(&mut self, rect: &Rect) {
    self.unselect();
    let mut selected = Image::new(rect.width(), rect.height());
    selected.draw(self.image(), -rect.x(), -rect.y());
    self.selection = Some((selected, rect.x(), rect.y()));
    self.image_mut().clear_rect(rect.x(), rect.y(),
                                rect.width(), rect.height());
  }

  fn unselect(&mut self) -> Option<Rect> {
    if let Some((selected, x, y)) = self.selection.take() {
      self.image_mut().draw(&selected, x, y);
      Some(Rect::new(x, y, selected.width(), selected.height()))
    } else { None }
  }

  fn cut_selection(&mut self) {
    if let Some((selected, x, y)) = self.selection.take() {
      self.push_undo(Undo::SelectionCut(selected.clone(), x, y));
      self.clipboard = Some((selected, x, y));
    }
  }

  fn copy_selection(&mut self) {
    if self.selection.is_some() {
      self.clipboard = self.selection.clone();
    } else {
      self.clipboard = Some((self.image().clone(), 0, 0));
    }
  }

  fn paste_selection(&mut self) {
    self.unselect();
    if self.clipboard.is_some() {
      self.selection = self.clipboard.clone();
      self.push_undo(Undo::SelectionPaste);
      self.tool = Tool::Select;
    }
  }

  fn push_undo(&mut self, undo: Undo) {
    self.undo_stack.push(undo);
    self.redo_stack.clear();
    if self.undo_stack.len() > MAX_UNDOS {
      self.undo_stack.remove(0);
    }
  }

  fn push_change(&mut self) {
    let image = self.image().clone();
    let undo = Undo::ChangeImage(self.current_image, image);
    self.push_undo(undo);
  }

  fn push_selection_move(&mut self) {
    let &(_, x, y) = self.selection.as_ref().unwrap();
    self.push_undo(Undo::SelectionMove(x, y));
  }

  fn undo(&mut self) -> bool {
    if let Some(undo) = self.undo_stack.pop() {
      match undo {
        Undo::AddImage(index) => {
          self.images.remove(index);
          self.redo_stack.push(Redo::AddImage(index));
        },
        Undo::ChangeImage(index, mut image) => {
          std::mem::swap(&mut image, &mut self.images[index]);
          self.redo_stack.push(Redo::ChangeImage(index, image));
        },
        Undo::RemoveImage(index, image) => {
          self.images.insert(index, image);
          self.redo_stack.push(Redo::RemoveImage(index));
        },
        Undo::SelectionBegin => {
          let rect = {
            let &(ref image, x, y) = self.selection.as_ref().unwrap();
            Rect::new(x, y, image.width(), image.height())
          };
          self.unselect();
          self.redo_stack.push(Redo::SelectionBegin(rect));
        },
        Undo::SelectionCut(image, x, y) => {
          self.selection = Some((image, x, y));
          self.redo_stack.push(Redo::SelectionCut);
        },
        Undo::SelectionEnd(rect) => {
          self.select(&rect);
          self.redo_stack.push(Redo::SelectionEnd);
        },
        Undo::SelectionMove(old_x, old_y) => {
          let (new_x, new_y) = {
            let &mut (_, ref mut x, ref mut y) =
                self.selection.as_mut().unwrap();
            let new = (*x, *y);
            *x = old_x;
            *y = old_y;
            new
          };
          self.redo_stack.push(Redo::SelectionMove(new_x, new_y));
        },
        Undo::SelectionPaste => {
          let (image, x, y) = self.selection.take().unwrap();
          self.redo_stack.push(Redo::SelectionPaste(image, x, y));
        },
      }
      self.unsaved = true;
      true
    } else { false }
  }

  fn redo(&mut self) -> bool {
    if let Some(redo) = self.redo_stack.pop() {
      match redo {
        Redo::AddImage(index) => {
          let (width, height) = self.image().size();
          self.images.insert(index, Image::new(width, height));
          self.undo_stack.push(Undo::AddImage(index));
        },
        Redo::ChangeImage(index, mut image) => {
          std::mem::swap(&mut image, &mut self.images[index]);
          self.undo_stack.push(Undo::ChangeImage(index, image));
        },
        Redo::RemoveImage(index) => {
          let image = self.images.remove(index);
          self.undo_stack.push(Undo::RemoveImage(index, image));
        },
        Redo::SelectionBegin(rect) => {
          self.select(&rect);
          self.undo_stack.push(Undo::SelectionBegin);
        },
        Redo::SelectionCut => {
          let (image, x, y) = self.selection.take().unwrap();
          self.undo_stack.push(Undo::SelectionCut(image, x, y));
        },
        Redo::SelectionEnd => {
          let rect = {
            let &(ref image, x, y) = self.selection.as_ref().unwrap();
            Rect::new(x, y, image.width(), image.height())
          };
          self.unselect();
          self.undo_stack.push(Undo::SelectionEnd(rect));
        },
        Redo::SelectionMove(new_x, new_y) => {
          let (old_x, old_y) = {
            let &mut (_, ref mut x, ref mut y) =
                self.selection.as_mut().unwrap();
            let old = (*x, *y);
            *x = new_x;
            *y = new_y;
            old
          };
          self.undo_stack.push(Undo::SelectionMove(old_x, old_y));
        },
        Redo::SelectionPaste(image, x, y) => {
          self.selection = Some((image, x, y));
          self.undo_stack.push(Undo::SelectionPaste);
        },
      }
      self.unsaved = true;
      true
    } else { false }
  }

  fn save_to_file(&mut self) -> io::Result<()> {
    self.unselect();
    let mut file = try!(File::create(&self.filepath));
    try!(Image::write(&mut file, &self.images));
    self.unsaved = false;
    Ok(())
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

  fn pick_color(&self, state: &mut EditorState) -> bool {
    state.unselect();
    state.color = self.color;
    if state.tool == Tool::Select {
      state.tool = Tool::Pencil;
    }
    true
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
      &Event::MouseButtonDown{mouse_btn: Mouse::Left, x, y, ..} => {
        if self.rect().contains((x, y)) {
          return self.pick_color(state);
        }
      },
      &Event::KeyDown{keycode: Some(key), ..} => {
        if key == self.key {
          return self.pick_color(state);
        }
      },
      _ => {}
    }
    false
  }
}

/*===========================================================================*/

struct ToolPicker {
  tool: Tool,
  key: Keycode,
  left: i32,
  top: i32,
  icon: Texture,
}

impl ToolPicker {
  fn new(left: i32, top: i32, tool: Tool, key: Keycode,
         icon: Texture) -> ToolPicker {
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
    if state.tool == self.tool { return false; }
    state.unselect();
    state.prev_tool = state.tool;
    state.tool = self.tool;
    true
  }
}

impl GuiElement<EditorState> for ToolPicker {
  fn draw(&self, state: &EditorState, renderer: &mut Renderer) {
    if state.tool == self.tool {
      renderer.set_draw_color(Color::RGB(255, 255, 255));
      renderer.fill_rect(self.rect()).unwrap();
    }
    renderer.copy(&self.icon, None, Some(expand(self.rect(), -2)));
    renderer.set_draw_color(Color::RGB(191, 191, 191));
    renderer.draw_rect(self.rect()).unwrap();
  }

  fn handle_event(&mut self, event: &Event, state: &mut EditorState) -> bool {
    match event {
      &Event::MouseButtonDown{mouse_btn: Mouse::Left, x, y, ..} => {
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
    false
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
      &Event::MouseButtonDown{mouse_btn: Mouse::Left, x, y, ..} => {
        if self.rect().contains((x, y)) {
          return self.pick(state);
        }
      },
      _ => {}
    }
    false
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
    state.unselect();
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
      &Event::MouseButtonDown{mouse_btn: Mouse::Left, x, y, ..} => {
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
    false
  }
}

/*===========================================================================*/

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
  fn draw(&self, state: &EditorState, renderer: &mut Renderer) {
    if state.unsaved {
      renderer.set_draw_color(Color::RGB(255, 127, 0));
      renderer.fill_rect(self.rect()).unwrap();
    }
  }

  fn handle_event(&mut self, _: &Event, _: &mut EditorState) -> bool {
    false
  }
}

/*===========================================================================*/

struct CanvasDrag {
  from_selection: (i32, i32),
  from_pixel: (i32, i32),
  to_pixel: (i32, i32),
}

struct Canvas {
  left: i32,
  top: i32,
  max_size: u32,
  drag_from_to: Option<CanvasDrag>,
}

impl Canvas {
  fn new(left: i32, top: i32, max_size: u32) -> Canvas {
    Canvas {
      left: left,
      top: top,
      max_size: max_size,
      drag_from_to: None,
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

  fn dragged_rect(&self, state: &EditorState) -> Option<Rect> {
    if let Some(ref drag) = self.drag_from_to {
      let (fpx, fpy) = drag.from_pixel;
      let (tpx, tpy) = drag.to_pixel;
      let (from_col, from_row) = self.clamp_mouse_to_row_col(fpx, fpy, state);
      let (to_col, to_row) = self.clamp_mouse_to_row_col(tpx, tpy, state);
      let x = std::cmp::min(from_col, to_col) as i32;
      let y = std::cmp::min(from_row, to_row) as i32;
      let w = ((from_col as i32 - to_col as i32).abs() + 1) as u32;
      let h = ((from_row as i32 - to_row as i32).abs() + 1) as u32;
      Some(Rect::new(x, y, w, h))
    } else { None }
  }

  fn mouse_to_row_col(&self, x: i32, y: i32,
                      state: &EditorState) -> Option<(u32, u32)> {
    if x < self.left || y < self.top { return None; }
    let scale = self.scale(state) as i32;
    let col = (x - self.left) / scale;
    let row = (y - self.top) / scale;
    let (width, height) = state.image().size();
    if col < 0 || col >= (width as i32) ||
       row < 0 || row >= (height as i32) { None }
    else { Some((col as u32, row as u32)) }
  }

  fn clamp_mouse_to_row_col(&self, x: i32, y: i32,
                            state: &EditorState) -> (u32, u32) {
    let scale = self.scale(state) as i32;
    let col = (x - self.left) / scale;
    let row = (y - self.top) / scale;
    let (width, height) = state.image().size();
    (std::cmp::max(0, std::cmp::min(col, width as i32)) as u32,
     std::cmp::max(0, std::cmp::min(row, height as i32)) as u32)
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
      let (width, height) = image.size();
      let from_color = image[(col, row)];
      if from_color == to_color { return false; }
      image[(col, row)] = to_color;
      let mut stack: Vec<(u32, u32)> = vec![(col, row)];
      while let Some((col, row)) = stack.pop() {
        let mut next: Vec<(u32, u32)> = vec![];
        if col > 0 { next.push((col - 1, row)); }
        if col < width - 1 { next.push((col + 1, row)); }
        if row > 0 { next.push((col, row - 1)); }
        if row < height - 1 { next.push((col, row + 1)); }
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
    let scale = self.scale(state);
    renderer.set_draw_color(Color::RGB(255, 255, 255));
    renderer.draw_rect(expand(self.rect(state), 2)).unwrap();
    render_image(renderer, state.image(), self.left, self.top, scale);
    if let Some((ref selected, x, y)) = state.selection {
      let left = self.left + x * (scale as i32);
      let top = self.top + y * (scale as i32);
      render_image(renderer, selected, left, top, scale);
      renderer.set_draw_color(Color::RGB(255, 191, 255));
      renderer.draw_rect(Rect::new(left, top, selected.width() * scale,
                                   selected.height() * scale)).unwrap();
    } else if let Some(rect) = self.dragged_rect(state) {
      renderer.set_draw_color(Color::RGB(255, 255, 191));
      renderer.draw_rect(Rect::new(self.left + rect.x() * (scale as i32),
                                   self.top + rect.y() * (scale as i32),
                                   rect.width() * scale,
                                   rect.height() * scale)).unwrap();
    }
  }

  fn handle_event(&mut self, event: &Event, state: &mut EditorState) -> bool {
    match event {
      &Event::KeyDown{keycode: Some(Keycode::Escape), ..} => {
        return state.try_unselect_with_undo();
      },
      &Event::MouseButtonDown{mouse_btn: Mouse::Left, x, y, ..} => {
        if self.rect(state).contains((x, y)) {
          match state.tool {
            Tool::Eyedropper => {
              return self.try_eyedrop(x, y, state);
            },
            Tool::PaintBucket => {
              state.push_change();
              return self.try_flood_fill(x, y, state);
            },
            Tool::Pencil => {
              state.push_change();
              return self.try_paint(x, y, state);
            },
            Tool::Select => {
              let rect = if let Some((ref selected, x, y)) = state.selection {
                Some(Rect::new(x, y, selected.width(), selected.height()))
              } else { None };
              if let Some(rect) = rect {
                let scale = self.scale(state);
                if !Rect::new(self.left + rect.x() * (scale as i32),
                              self.top + rect.y() * (scale as i32),
                              rect.width() * scale,
                              rect.height() * scale).contains((x, y)) {
                  state.try_unselect_with_undo();
                } else {
                  state.push_selection_move();
                }
              }
              self.drag_from_to = Some(CanvasDrag{
                from_selection: if let Some(r) = rect { (r.x(), r.y()) }
                                else { (0, 0) },
                from_pixel: (x, y),
                to_pixel: (x, y),
              });
              return true;
            },
          }
        } else {
          self.drag_from_to = None;
        }
      },
      &Event::MouseButtonUp{mouse_btn: Mouse::Left, ..} => {
        match state.tool {
          Tool::Select => {
            if state.selection.is_none() {
              if let Some(rect) = self.dragged_rect(state) {
                state.select_with_undo(&rect);
                self.drag_from_to = None;
                return true;
              }
            }
          },
          _ => {}
        }
        self.drag_from_to = None;
      },
      &Event::MouseMotion{x, y, mousestate, ..} => {
        if mousestate.left() {
          match state.tool {
            Tool::Pencil => {
              return self.try_paint(x, y, state);
            },
            Tool::Select => {
              let scale = self.scale(state) as i32;
              if let Some(ref mut drag) = self.drag_from_to {
                drag.to_pixel = (x, y);
                if let Some((_, ref mut sx, ref mut sy)) = state.selection {
                  let (fsx, fsy) = drag.from_selection;
                  let (fpx, fpy) = drag.from_pixel;
                  *sx = fsx + (x - fpx) / scale;
                  *sy = fsy + (y - fpy) / scale;
                }
                return true;
              }
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

fn image_to_sdl_surface(image: &Image) -> Surface {
  let mut surface = Surface::new(image.width(), image.height(),
                                 PixelFormatEnum::RGBA8888).unwrap();
  for row in 0..image.height() {
    for col in 0..image.width() {
      surface.fill_rect(Some(Rect::new(col as i32, row as i32, 1, 1)),
                        hex_pixel_to_sdl_color(image[(col, row)])).unwrap();
    }
  }
  surface
}

fn image_to_sdl_texture(renderer: &Renderer, image: &Image) -> Texture {
  renderer.create_texture_from_surface(&image_to_sdl_surface(image)).unwrap()
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

  let tool_icons = load_from_file(&"data/tool_icons.ahi".to_string()).unwrap();

  let mut state = EditorState::new(filepath, images);
  let mut elements: Vec<Box<GuiElement<EditorState>>> = vec![
    Box::new(UnsavedIndicator::new(462, 2)),
    // Color palette:
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
    // Toolbox:
    Box::new(ToolPicker::new( 4, 296, Tool::Pencil,      Keycode::P,
                             image_to_sdl_texture(&renderer, &tool_icons[0]))),
    Box::new(ToolPicker::new(28, 296, Tool::PaintBucket, Keycode::K,
                             image_to_sdl_texture(&renderer, &tool_icons[1]))),
    Box::new(ToolPicker::new(52, 296, Tool::Eyedropper,  Keycode::Y,
                             image_to_sdl_texture(&renderer, &tool_icons[2]))),
    Box::new(ToolPicker::new(76, 296, Tool::Select,      Keycode::S,
                             image_to_sdl_texture(&renderer, &tool_icons[3]))),
    // Canvases:
    Box::new(Canvas::new(8, 32, 256)),
    Box::new(Canvas::new(300, 32, 64)),
    // Images scrollbar:
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
          Keycode::A => {
            state.select_all_with_undo();
            needs_redraw = true;
          },
          Keycode::C => {
            state.copy_selection();
          },
          Keycode::H => {
            if kmod.intersects(keyboard::LSHIFTMOD | keyboard::RSHIFTMOD) {
              state.flip_image_horz();
              needs_redraw = true;
            }
          },
          Keycode::N => {
            state.add_new_image();
            needs_redraw = true;
          },
          Keycode::S => {
            state.save_to_file().unwrap();
            needs_redraw = true;
          },
          Keycode::V => {
            if kmod.intersects(keyboard::LSHIFTMOD | keyboard::RSHIFTMOD) {
              state.flip_image_vert();
            } else { state.paste_selection(); }
            needs_redraw = true;
          },
          Keycode::X => {
            state.cut_selection();
            needs_redraw = true;
          },
          Keycode::Z => {
            if kmod.intersects(keyboard::LSHIFTMOD | keyboard::RSHIFTMOD) {
              if state.redo() { needs_redraw = true; }
            } else if state.undo() { needs_redraw = true; }
          },
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
