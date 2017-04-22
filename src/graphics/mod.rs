#![allow(unused_variables, unused_imports)]

extern crate piston_window;
extern crate gfx_text;

use self::piston_window::*;

mod font_cache;

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
    let mut text = gfx_text::new(factory).build().unwrap();

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

            // Add some text 10 pixels down and right from the top left screen corner.
            text.add(
                "The quick brown fox jumps over the lazy dog",  // Text to add
                [10, 10],                                       // Position
                [0.0, 0.0, 0.16, 1.0],                        // Text color
            );

            // Draw text.
            text.draw(&mut g.encoder, &out);
        });
    }
}
