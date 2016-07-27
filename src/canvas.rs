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

use ahi::Image;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::{Point, Rect};
use sdl2::render::{Renderer, Texture};
use sdl2::surface::Surface;

// ========================================================================= //

pub struct Canvas<'a> {
    clip_rect: Option<Rect>,
    prev_clip_rect: Option<Rect>,
    renderer: &'a mut Renderer<'static>,
}

impl<'a> Canvas<'a> {
    pub fn from_renderer(renderer: &'a mut Renderer<'static>) -> Canvas<'a> {
        Canvas {
            clip_rect: None,
            prev_clip_rect: None,
            renderer: renderer,
        }
    }

    pub fn size(&self) -> (u32, u32) {
        if let Some(rect) = self.clip_rect {
            (rect.width(), rect.height())
        } else {
            self.renderer.logical_size()
        }
    }

    pub fn rect(&self) -> Rect {
        let (width, height) = self.size();
        Rect::new(0, 0, width, height)
    }

    pub fn new_sprite(&self, image: &Image) -> Sprite {
        let width = image.width();
        let height = image.height();
        let mut data = image.rgba_data();
        let format = if cfg!(target_endian = "big") {
            PixelFormatEnum::RGBA8888
        } else {
            PixelFormatEnum::ABGR8888
        };
        let surface = Surface::from_data(&mut data,
                                         width,
                                         height,
                                         width * 4,
                                         format)
                          .unwrap();
        Sprite {
            width: width,
            height: height,
            texture: self.renderer
                         .create_texture_from_surface(&surface)
                         .unwrap(),
        }
    }

    pub fn draw_sprite(&mut self, sprite: &Sprite, topleft: Point) {
        let (x, y) = match self.clip_rect {
            Some(rect) => (rect.x(), rect.y()),
            None => (0, 0),
        };
        self.renderer.copy(&sprite.texture,
                           None,
                           Some(Rect::new(x + topleft.x(),
                                          y + topleft.y(),
                                          sprite.width(),
                                          sprite.height())));
    }

    pub fn clear(&mut self, color: (u8, u8, u8, u8)) {
        let (r, g, b, a) = color;
        self.renderer.set_draw_color(Color::RGBA(r, g, b, a));
        if let Some(rect) = self.clip_rect {
            self.renderer.fill_rect(rect).unwrap();
        } else {
            self.renderer.clear();
        }
    }

    pub fn present(&mut self) {
        self.renderer.present();
    }

    pub fn draw_pixel(&mut self, color: (u8, u8, u8, u8), point: Point) {
        self.fill_rect(color, Rect::new(point.x(), point.y(), 1, 1));
    }

    pub fn draw_rect(&mut self, color: (u8, u8, u8, u8), rect: Rect) {
        let (r, g, b, a) = color;
        self.renderer.set_draw_color(Color::RGBA(r, g, b, a));
        let subrect = self.subrect(rect);
        self.renderer.draw_rect(subrect).unwrap();
    }

    pub fn fill_rect(&mut self, color: (u8, u8, u8, u8), rect: Rect) {
        let (r, g, b, a) = color;
        self.renderer.set_draw_color(Color::RGBA(r, g, b, a));
        let subrect = self.subrect(rect);
        self.renderer.fill_rect(subrect).unwrap();
    }

    pub fn subcanvas(&mut self, rect: Rect) -> Canvas {
        let new_clip_rect = Some(self.subrect(rect));
        self.renderer.set_clip_rect(new_clip_rect);
        Canvas {
            clip_rect: new_clip_rect,
            prev_clip_rect: self.clip_rect,
            renderer: self.renderer,
        }
    }

    fn subrect(&self, mut child: Rect) -> Rect {
        if let Some(parent) = self.clip_rect {
            child.offset(parent.x(), parent.y());
            if let Some(intersection) = parent.intersection(child) {
                intersection
            } else {
                child.resize(0, 0);
                child
            }
        } else {
            child
        }
    }
}

impl<'a> Drop for Canvas<'a> {
    fn drop(&mut self) {
        self.renderer.set_clip_rect(self.prev_clip_rect);
    }
}

// ========================================================================= //

pub struct Sprite {
    width: u32,
    height: u32,
    texture: Texture,
}

impl Sprite {
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}

// ========================================================================= //
