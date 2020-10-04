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

use super::namebox::ImageNameBox;
use super::palette::ColorPalette;
use super::scrollbar::ImagesScrollbar;
use super::textbox::ModalTextBox;
use super::tiles::TileView;
use super::toolbox::Toolbox;
use super::unsaved::UnsavedIndicator;
use crate::canvas::{Canvas, Resources};
use crate::element::{Action, AggregateElement, GuiElement, SubrectElement};
use crate::event::{Event, Keycode, COMMAND, SHIFT};
use crate::paint::ImageCanvas;
use crate::state::EditorState;
use sdl2::rect::{Point, Rect};

//===========================================================================//

pub struct EditorView {
    element: SubrectElement<AggregateElement<EditorState>>,
}

impl EditorView {
    pub const WIDTH: u32 = 480;
    pub const HEIGHT: u32 = 320;

    pub fn new(offset: Point) -> EditorView {
        let elements: Vec<Box<dyn GuiElement<EditorState>>> = vec![
            Box::new(ModalTextBox::new(2, 296)),
            Box::new(ColorPalette::new(10, 136)),
            Box::new(Toolbox::new(4, 10)),
            Box::new(ImagesScrollbar::new(436, 11)),
            Box::new(ImageCanvas::new(60, 16, 256)),
            Box::new(ImageCanvas::new(326, 16, 64)),
            Box::new(TileView::new(326, 96, 96, 96)),
            Box::new(ImageNameBox::new(326, 230)),
            Box::new(UnsavedIndicator::new(326, 256)),
        ];
        EditorView {
            element: SubrectElement::new(
                AggregateElement::new(elements),
                Rect::new(
                    offset.x(),
                    offset.y(),
                    EditorView::WIDTH,
                    EditorView::HEIGHT,
                ),
            ),
        }
    }
}

impl GuiElement<EditorState> for EditorView {
    fn draw(
        &self,
        state: &EditorState,
        resources: &Resources,
        canvas: &mut Canvas,
    ) {
        canvas.clear((64, 64, 64, 255));
        let rect = canvas.rect();
        canvas.draw_rect((127, 127, 127, 127), rect);
        self.element.draw(state, resources, canvas);
    }

    fn handle_event(
        &mut self,
        event: &Event,
        state: &mut EditorState,
    ) -> Action {
        match event {
            &Event::KeyDown(Keycode::Backspace, kmod) if kmod == COMMAND => {
                Action::redraw_if(state.mutation().delete_image()).and_stop()
            }
            &Event::KeyDown(Keycode::A, kmod) if kmod == COMMAND => {
                state.mutation().select_all();
                Action::redraw().and_stop()
            }
            &Event::KeyDown(Keycode::B, kmod) if kmod == COMMAND | SHIFT => {
                Action::redraw_if(state.begin_set_metrics()).and_stop()
            }
            &Event::KeyDown(Keycode::C, kmod) if kmod == COMMAND => {
                state.mutation().copy_selection();
                Action::ignore().and_stop()
            }
            &Event::KeyDown(Keycode::G, kmod) if kmod == COMMAND => {
                Action::redraw_if(state.begin_goto()).and_stop()
            }
            &Event::KeyDown(Keycode::H, kmod) if kmod == COMMAND | SHIFT => {
                state.mutation().flip_selection_horz();
                Action::redraw().and_stop()
            }
            &Event::KeyDown(Keycode::L, kmod) if kmod == COMMAND | SHIFT => {
                state.mutation().rotate_selection_counterclockwise();
                Action::redraw().and_stop()
            }
            &Event::KeyDown(Keycode::N, kmod) if kmod == COMMAND => {
                Action::redraw_if(state.begin_new_image()).and_stop()
            }
            &Event::KeyDown(Keycode::O, kmod) if kmod == COMMAND => {
                Action::redraw_if(state.begin_load_file()).and_stop()
            }
            &Event::KeyDown(Keycode::R, kmod) if kmod == COMMAND => {
                Action::redraw_if(state.begin_resize()).and_stop()
            }
            &Event::KeyDown(Keycode::R, kmod) if kmod == COMMAND | SHIFT => {
                state.mutation().rotate_selection_clockwise();
                Action::redraw().and_stop()
            }
            &Event::KeyDown(Keycode::S, kmod) if kmod == COMMAND => {
                state.save_to_file().unwrap();
                Action::redraw().and_stop()
            }
            &Event::KeyDown(Keycode::S, kmod) if kmod == COMMAND | SHIFT => {
                Action::redraw_if(state.begin_save_as()).and_stop()
            }
            &Event::KeyDown(Keycode::T, kmod) if kmod == COMMAND | SHIFT => {
                Action::redraw_if(state.begin_set_test_sentence()).and_stop()
            }
            &Event::KeyDown(Keycode::V, kmod) if kmod == COMMAND => {
                state.mutation().paste_selection();
                Action::redraw().and_stop()
            }
            &Event::KeyDown(Keycode::V, kmod) if kmod == COMMAND | SHIFT => {
                state.mutation().flip_selection_vert();
                Action::redraw().and_stop()
            }
            &Event::KeyDown(Keycode::X, kmod) if kmod == COMMAND => {
                state.mutation().cut_selection();
                Action::redraw().and_stop()
            }
            &Event::KeyDown(Keycode::Z, kmod) if kmod == COMMAND => {
                Action::redraw_if(state.undo()).and_stop()
            }
            &Event::KeyDown(Keycode::Z, kmod) if kmod == COMMAND | SHIFT => {
                Action::redraw_if(state.redo()).and_stop()
            }
            &Event::KeyDown(Keycode::Num2, kmod)
                if kmod == COMMAND | SHIFT =>
            {
                state.mutation().scale_selection_2x();
                Action::redraw().and_stop()
            }
            _ => self.element.handle_event(event, state),
        }
    }
}

//===========================================================================//
