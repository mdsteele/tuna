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
// - Windowed instead of fullscreen
// - Finish lasso tool
// - Limited region redraws
// - Zoom/scroll

mod canvas;
mod element;
mod event;
mod paint;
mod state;
mod util;
mod view;

use self::canvas::{Canvas, Resources};
use self::element::GuiElement;
use self::event::Event;
use self::state::EditorState;
use self::view::EditorView;
use sdl2::rect::Point;
use sdl2::render::Canvas as SdlCanvas;
use sdl2::video::Window;
use std::time::Instant;

//===========================================================================//

const FRAME_DELAY_MILLIS: u32 = 100;

fn render_screen<E: GuiElement<EditorState, ()>>(
    renderer: &mut SdlCanvas<Window>,
    resources: &Resources,
    state: &EditorState,
    gui: &E,
) {
    gui.draw(state, resources, &mut Canvas::from_renderer(renderer));
    renderer.present();
}

//===========================================================================//

fn main() {
    let mut state = {
        let args: Vec<String> = std::env::args().collect();
        let (filepath, collection) = if args.len() >= 2 {
            let filepath = &args[1];
            (filepath.clone(), util::load_ahi_from_file(filepath).unwrap())
        } else {
            ("./out.ahi".to_string(), ahi::Collection::new())
        };
        EditorState::new(filepath, collection)
    };

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let editor_width = EditorView::WIDTH;
    let editor_height = EditorView::HEIGHT;
    let sdl_window = video_subsystem
        .window("AHI Editor", 2 * editor_width, 2 * editor_height)
        .position_centered()
        .build()
        .unwrap();
    let mut renderer = sdl_window.into_canvas().build().unwrap();
    renderer.set_logical_size(editor_width, editor_height).unwrap();
    renderer.set_blend_mode(sdl2::render::BlendMode::Blend);
    let texture_creator = renderer.texture_creator();
    let resources = Resources::new(&texture_creator);

    let mut gui = EditorView::new(Point::new(0, 0));
    render_screen(&mut renderer, &resources, &state, &gui);

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut last_clock_tick = Instant::now();
    loop {
        let now = Instant::now();
        let elapsed_millis = now
            .duration_since(last_clock_tick)
            .as_millis()
            .min(u32::MAX as u128) as u32;
        let opt_sdl_event = if elapsed_millis >= FRAME_DELAY_MILLIS {
            None
        } else {
            event_pump.wait_event_timeout(FRAME_DELAY_MILLIS - elapsed_millis)
        };
        let event = match opt_sdl_event {
            None => {
                last_clock_tick = now;
                Event::ClockTick
            }
            Some(sdl_event) => match Event::from_sdl2(&sdl_event) {
                Some(event) => event,
                None => continue,
            },
        };
        let action = match event {
            Event::Quit => return,
            event => gui.on_event(&event, &mut state),
        };
        if action.should_redraw() {
            render_screen(&mut renderer, &resources, &state, &gui);
        }
    }
}

//===========================================================================//
