extern crate freetype;
extern crate gfx_core;

use std::collections::HashMap;
use self::freetype::{Library, RenderMode,Vector,face};
use ::piston_window::*;
use ::piston_window::character::*;
use self::gfx_core::Resources;
use self::gfx_core::factory::Factory;

use std::fmt::Display;

struct Glyph<R:Resources> {
    bearing: [f64;2],
    size: [f64;2],
    tex: Texture<R>,
}

pub struct FontCache<R,F>
where R: Resources,
      F: Factory<R>,
{
    _lib: Library,
    chars: HashMap<(char,u32),Glyph<R>>,
    face: self::freetype::Face<'static>,
    factory: F
}

impl<R,F> CharacterCache for FontCache<R,F>
where R: Resources,
      F: Factory<R>,
{
    type Texture = Texture<R>;

    fn character<'a>(&'a mut self,
                     font_size: u32,
                     ch: char)
                     -> Character<'a, Self::Texture> 
    {
        let &mut FontCache{ref mut chars, ref mut face, ref mut factory, ..} = self;

        let glyph = chars.entry((ch,font_size))
            .or_insert_with(|| {
                // Load a character
                face.set_pixel_sizes(0, font_size).unwrap();
                face.load_char(ch as usize, face::RENDER).unwrap();

                // Get the glyph instance
                let glyph = face.glyph();
                let advance = glyph.advance();
                glyph.render_glyph(RenderMode::Lcd).unwrap();
                
                let bitmap = glyph.bitmap();
                let width = bitmap.width() as u32;
                let height = bitmap.rows() as u32;

                let settings = TextureSettings::new();
                    //.filter(Filter::Nearest);

                let tex = Texture::from_memory_alpha(factory, 
                                                     &bitmap.buffer(),
                                                     width,
                                                     height,
                                                     &settings).unwrap();

                Glyph {
                    size: [glyph.advance().x as f64 / 64.0, glyph.advance().y as f64 / 64.0],
                    bearing: [glyph.bitmap_left() as f64, glyph.bitmap_top() as f64],
                    tex: tex,
                }
            });

        Character {
            texture: &glyph.tex,
            offset: glyph.bearing,
            size: glyph.size,
        }
    }
}

impl<R,F> FontCache<R,F>
where R: Resources,
      F: Factory<R>,
{
    pub fn new(factory: F, file: &str) -> Self {
        let lib = Library::init().unwrap();
        let face = lib.new_face(file, 0).unwrap();

        FontCache{
            _lib: lib,
            chars: HashMap::new(),
            face: face,
            factory: factory,
        }
    }
}

