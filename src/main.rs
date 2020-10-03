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
// - Limited region redraws

extern crate ahi;
extern crate num_integer;
extern crate sdl2;

mod canvas;
mod editor;
mod element;
mod event;
mod namebox;
mod paint;
mod palette;
mod scrollbar;
mod state;
mod textbox;
mod tiles;
mod toolbox;
mod unsaved;
mod util;

use self::canvas::{Font, Sprite, Window};
use self::editor::EditorView;
use self::element::GuiElement;
use self::event::Event;
use self::state::EditorState;
use sdl2::rect::Point;
use std::rc::Rc;

//===========================================================================//

const FRAME_DELAY_MILLIS: u32 = 100;

fn render_screen<E: GuiElement<EditorState>>(
    window: &mut Window,
    state: &EditorState,
    gui: &E,
) {
    gui.draw(state, &mut window.canvas());
    window.present();
}

fn load_font(window: &Window, path: &str) -> Font {
    let ahf = util::load_ahf_from_file(&path.to_string()).unwrap();
    window.new_font(&ahf)
}

fn load_sprite(window: &Window, path: &str) -> Sprite {
    let images = util::load_ahi_from_file(&path.to_string()).unwrap();
    window.new_sprite(&images[0])
}

fn load_sprites(window: &Window, path: &str) -> Vec<Sprite> {
    let images = util::load_ahi_from_file(&path.to_string()).unwrap();
    images.iter().map(|image| window.new_sprite(image)).collect()
}

fn window_size(
    (ideal_width, ideal_height): (u32, u32),
    aspect_ratio: f64,
) -> ((u32, u32), Point) {
    let ideal_ratio = (ideal_width as f64) / (ideal_height as f64);
    if aspect_ratio > ideal_ratio {
        let actual_width =
            (aspect_ratio * (ideal_height as f64)).round() as u32;
        (
            (actual_width, ideal_height),
            Point::new(((actual_width - ideal_width) / 2) as i32, 0),
        )
    } else {
        let actual_height =
            ((ideal_width as f64) / aspect_ratio).round() as u32;
        (
            (ideal_width, actual_height),
            Point::new(0, ((actual_height - ideal_height) / 2) as i32),
        )
    }
}

//===========================================================================//

fn main() {
    let mut state = {
        let args: Vec<String> = std::env::args().collect();
        let (filepath, images) = if args.len() >= 2 {
            let filepath = &args[1];
            (filepath.clone(), util::load_ahi_from_file(filepath).unwrap())
        } else {
            ("./out.ahi".to_string(), vec![])
        };
        EditorState::new(filepath, images)
    };

    let sdl_context = sdl2::init().unwrap();
    let event_subsystem = sdl_context.event().unwrap();
    let timer_subsystem = sdl_context.timer().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let ideal_width = EditorView::WIDTH;
    let ideal_height = EditorView::HEIGHT;
    let sdl_window = video_subsystem
        .window("AHI Editor", ideal_width, ideal_height)
        .position_centered()
        .fullscreen_desktop()
        .build()
        .unwrap();
    let (native_width, native_height) = sdl_window.size();
    let aspect_ratio: f64 = (native_width as f64) / (native_height as f64);
    let ((actual_width, actual_height), gui_offset) =
        window_size((ideal_width, ideal_height), aspect_ratio);
    let mut renderer = sdl_window.into_canvas().build().unwrap();
    renderer.set_logical_size(actual_width, actual_height).unwrap();
    let mut window = Window::from_renderer(&mut renderer);

    let tool_icons: Vec<Sprite> = load_sprites(&window, "data/tool_icons.ahi");
    let arrows: Vec<Sprite> = load_sprites(&window, "data/arrows.ahi");
    let unsaved_icon = load_sprite(&window, "data/unsaved.ahi");
    let font: Rc<Font> = Rc::new(load_font(&window, "data/medfont.ahf"));

    let mut gui =
        EditorView::new(tool_icons, arrows, unsaved_icon, font, gui_offset);

    render_screen(&mut window, &state, &gui);

    Event::register_clock_ticks(&event_subsystem);
    let _timer = timer_subsystem.add_timer(
        FRAME_DELAY_MILLIS,
        Box::new(|| {
            Event::push_clock_tick(&event_subsystem);
            FRAME_DELAY_MILLIS
        }),
    );

    let mut event_pump = sdl_context.event_pump().unwrap();
    loop {
        let event = match Event::from_sdl2(&event_pump.wait_event()) {
            Some(event) => event,
            None => continue,
        };
        let action = match event {
            Event::Quit => return,
            event => gui.handle_event(&event, &mut state),
        };
        if action.should_redraw() {
            render_screen(&mut window, &state, &gui);
        }
    }
}

//===========================================================================//
