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
use sdl2::rect::{Point, Rect};
use std::fs::File;
use std::io;
use std::rc::Rc;

mod canvas;
use self::canvas::{Canvas, Sprite};

mod element;
use self::element::{AggregateElement, GuiElement};

mod paint;
use self::paint::ImageCanvas;

mod palette;
use self::palette::ColorPalette;

mod scrollbar;
use self::scrollbar::ImagesScrollbar;

mod state;
use self::state::EditorState;

mod toolbox;
use self::toolbox::Toolbox;

mod unsaved;
use self::unsaved::UnsavedIndicator;

mod util;

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
        Rect::new(self.left, self.top, 472, 20)
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
            x += 14;
        }
    }
}

fn render_screen(canvas: &mut Canvas,
                 state: &EditorState,
                 gui: &AggregateElement<EditorState>) {
    canvas.clear((64, 64, 64, 255));
    gui.draw(state, canvas);
    canvas.present();
}

fn load_ahi_from_file(path: &String) -> io::Result<Vec<Image>> {
    let mut file = try!(File::open(path));
    Image::read_all(&mut file)
}

fn load_sprites(canvas: &Canvas, path: &str) -> Vec<Sprite> {
    let images = load_ahi_from_file(&path.to_string()).unwrap();
    images.iter().map(|image| canvas.new_sprite(image)).collect()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (filepath, images) = if args.len() >= 2 {
        let filepath = &args[1];
        (filepath.clone(), load_ahi_from_file(filepath).unwrap())
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

    let tool_icons: Vec<Sprite> = load_sprites(&canvas, "data/tool_icons.ahi");
    let arrows: Vec<Sprite> = load_sprites(&canvas, "data/arrows.ahi");
    let unsaved_sprite = {
        let images = load_ahi_from_file(&"data/unsaved.ahi".to_string())
                         .unwrap();
        canvas.new_sprite(&images[0])
    };
    let font: Rc<Vec<Sprite>> = Rc::new(load_sprites(&canvas,
                                                     "data/font.ahi"));

    let mut state = EditorState::new(filepath, images);
    let elements: Vec<Box<GuiElement<EditorState>>> = vec![
    Box::new(UnsavedIndicator::new(312, 256, unsaved_sprite)),
    Box::new(ColorPalette::new(4, 138)),
    Box::new(Toolbox::new(10, 10, tool_icons)),
    Box::new(FilePathTextBox::new(4, 296, font.clone())),
    Box::new(ImagesScrollbar::new(436, 11, arrows)),
    Box::new(ImageCanvas::new(48, 16, 256)),
    Box::new(ImageCanvas::new(314, 16, 64)),
  ];
    let mut gui = AggregateElement::new(elements);

    render_screen(&mut canvas, &state, &gui);

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
                if gui.handle_event(&event, &mut state) {
                    needs_redraw = true;
                }
            }
        }
        if needs_redraw {
            render_screen(&mut canvas, &state, &gui);
        }
    }
}

// ========================================================================= //
