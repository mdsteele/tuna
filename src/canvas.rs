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

use crate::util;
use ahi;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::{Point, Rect};
use sdl2::render::Canvas as SdlCanvas;
use sdl2::render::{Texture, TextureCreator};
use sdl2::surface::Surface;
use sdl2::video::{Window, WindowContext};
use std::collections::HashMap;

//===========================================================================//

pub struct Canvas<'a> {
    clip_rect: Option<Rect>,
    prev_clip_rect: Option<Rect>,
    renderer: &'a mut SdlCanvas<Window>,
}

impl<'a> Canvas<'a> {
    pub fn from_renderer(renderer: &'a mut SdlCanvas<Window>) -> Canvas<'a> {
        Canvas { clip_rect: None, prev_clip_rect: None, renderer }
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

    pub fn draw_sprite(&mut self, sprite: &Sprite, topleft: Point) {
        let (x, y) = match self.clip_rect {
            Some(rect) => (rect.x(), rect.y()),
            None => (0, 0),
        };
        self.renderer
            .copy(
                &sprite.texture,
                None,
                Some(Rect::new(
                    x + topleft.x(),
                    y + topleft.y(),
                    sprite.width(),
                    sprite.height(),
                )),
            )
            .unwrap();
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

    pub fn draw_image(
        &mut self,
        image: &ahi::Image,
        palette: &ahi::Palette,
        left: i32,
        top: i32,
        scale: u32,
    ) {
        for row in 0..image.height() {
            for col in 0..image.width() {
                let pixel = image[(col, row)];
                let (r, g, b, a) = palette[pixel];
                if a > 0 {
                    self.fill_rect(
                        (r, g, b, a),
                        Rect::new(
                            left + (scale * col) as i32,
                            top + (scale * row) as i32,
                            scale,
                            scale,
                        ),
                    );
                }
            }
        }
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
        if a > 0 {
            self.renderer.set_draw_color(Color::RGBA(r, g, b, a));
            let subrect = self.subrect(rect);
            self.renderer.fill_rect(subrect).unwrap();
        }
    }

    pub fn draw_string(
        &mut self,
        font: &Font,
        mut left: i32,
        top: i32,
        string: &str,
    ) {
        for chr in string.chars() {
            let glyph = font.glyph(chr);
            left -= glyph.left_edge;
            self.draw_sprite(&glyph.sprite, Point::new(left, top));
            left += glyph.right_edge;
        }
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

//===========================================================================//

pub struct Sprite<'a> {
    width: u32,
    height: u32,
    texture: Texture<'a>,
}

impl<'a> Sprite<'a> {
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}

//===========================================================================//

struct Glyph<'a> {
    sprite: Sprite<'a>,
    left_edge: i32,
    right_edge: i32,
}

pub struct Font<'a> {
    glyphs: HashMap<char, Glyph<'a>>,
    default_glyph: Glyph<'a>,
    _baseline: i32,
}

impl<'a> Font<'a> {
    pub fn text_width(&self, text: &str) -> i32 {
        let mut width = 0;
        for chr in text.chars() {
            let glyph = self.glyph(chr);
            width += glyph.right_edge - glyph.left_edge;
        }
        width
    }

    fn glyph(&self, chr: char) -> &Glyph {
        self.glyphs.get(&chr).unwrap_or(&self.default_glyph)
    }
}

//===========================================================================//

pub struct Resources<'a> {
    arrows: Vec<Sprite<'a>>,
    font: Font<'a>,
    tool_icons: Vec<Sprite<'a>>,
    unsaved_icon: Sprite<'a>,
}

impl<'a> Resources<'a> {
    pub fn new(creator: &'a TextureCreator<WindowContext>) -> Resources<'a> {
        Resources {
            arrows: load_sprites_from_file(creator, "data/arrows.ahi"),
            font: load_font_from_file(creator, "data/medfont.ahf"),
            tool_icons: load_sprites_from_file(creator, "data/tool_icons.ahi"),
            unsaved_icon: load_sprite_from_file(creator, "data/unsaved.ahi"),
        }
    }

    pub fn arrow_down(&self) -> &Sprite {
        &self.arrows[1]
    }

    pub fn arrow_up(&self) -> &Sprite {
        &self.arrows[0]
    }

    pub fn font(&self) -> &Font {
        &self.font
    }

    pub fn tool_icon(&self, icon: ToolIcon) -> &Sprite {
        &self.tool_icons[icon as usize]
    }

    pub fn unsaved_icon(&self) -> &Sprite {
        &self.unsaved_icon
    }
}

//===========================================================================//

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum ToolIcon {
    Pencil = 0,
    PaintBucket,
    Eyedropper,
    Select,
    Line,
    Checkerboard,
    Oval,
    Rectangle,
    PaletteSwap,
    PaletteReplace,
    Watercolor,
    Lasso,
    MirrorNone,
    MirrorHorz,
    MirrorVert,
    MirrorBoth,
    MirrorRot2,
    MirrorRot4,
    ArrowLeft,
    ArrowRight,
}

//===========================================================================//

fn load_glyph<'a>(
    creator: &'a TextureCreator<WindowContext>,
    glyph: &ahi::Glyph,
) -> Glyph<'a> {
    Glyph {
        sprite: load_sprite_from_image(creator, glyph.image()),
        left_edge: glyph.left_edge(),
        right_edge: glyph.right_edge(),
    }
}

fn load_font_from_file<'a>(
    creator: &'a TextureCreator<WindowContext>,
    path: &str,
) -> Font<'a> {
    let font = util::load_ahf_from_file(&path.to_string()).unwrap();
    let mut glyphs = HashMap::new();
    for chr in font.chars() {
        glyphs.insert(chr, load_glyph(creator, &font[chr]));
    }
    Font {
        glyphs,
        default_glyph: load_glyph(creator, font.default_glyph()),
        _baseline: font.baseline(),
    }
}

fn load_sprites_from_file<'a>(
    creator: &'a TextureCreator<WindowContext>,
    path: &str,
) -> Vec<Sprite<'a>> {
    let collection = util::load_ahi_from_file(&path.to_string()).unwrap();
    collection
        .images
        .iter()
        .map(|image| load_sprite_from_image(creator, image))
        .collect()
}

fn load_sprite_from_file<'a>(
    creator: &'a TextureCreator<WindowContext>,
    path: &str,
) -> Sprite<'a> {
    let collection = util::load_ahi_from_file(&path.to_string()).unwrap();
    load_sprite_from_image(creator, &collection.images[0])
}

fn load_sprite_from_image<'a>(
    creator: &'a TextureCreator<WindowContext>,
    image: &ahi::Image,
) -> Sprite<'a> {
    let width = image.width();
    let height = image.height();
    let mut data = image.rgba_data(ahi::Palette::default());
    let format = if cfg!(target_endian = "big") {
        PixelFormatEnum::RGBA8888
    } else {
        PixelFormatEnum::ABGR8888
    };
    let surface =
        Surface::from_data(&mut data, width, height, width * 4, format)
            .unwrap();
    Sprite {
        width,
        height,
        texture: creator.create_texture_from_surface(&surface).unwrap(),
    }
}

//===========================================================================//
