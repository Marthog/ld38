#![allow(unused_variables, unused_imports)]

extern crate piston_window;

use self::piston_window::*;

mod font_cache;
use self::font_cache::FontCache;

const TILE_WIDTH: f32 = 100.0;
const TILE_HEIGHT: f32 = 100.0;
const CARD_WIDTH: f32 = 100.0; 
const CARD_HEIGHT: f32 = 20.0;

pub fn window() {
    let mut window: PistonWindow =
        WindowSettings::new("Hello World!", [512; 2])
            .exit_on_esc(true)
            .samples(8)
            .vsync(true)
            .build().unwrap();

    let factory = window.factory.clone();

    let mut font = FontCache::new(factory, "assets/NotoSans-Regular.ttf");

    while let Some(e) = window.next() {
        let out = window.output_color.clone();
        window.draw_2d(&e, |c, mut g| {
            clear([0.5, 0.5, 0.5, 1.0], g);

            rectangle([1.0, 0.0, 0.0, 1.0], // red
                      [0.0, 0.0, 100.0, 100.0], // rectangle
                      c.transform, g);

            rectangle([0.0, 0.0, 1.0, 1.0], // blue
                      [50.0, 50.0, 100.0, 100.0], // rectangle
                      c.transform, g);

            text([0.0,0.0,0.0,1.0],
                 32,
                 "Hello, world!",
                 &mut font,
                 c.transform.trans(0.0,32.0),
                 g);
        });
    }
}
