#![allow(unused_variables, unused_imports)]

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

extern crate piston_window;
extern crate vecmath;
extern crate assert;
extern crate rand;

use self::piston_window::*;

use std::collections::{HashMap};

mod graphics;
use graphics::*;
mod game;
use game::*;

use self::graphics::FontCache;
use self::piston_window::math::*;

pub fn inside(rect: [f64;4], p: Vec2d) -> bool {
    let p = sub(p, [rect[0],rect[1]]);
    p[0]>=0.0 && p[1]>=0.0 && p[0]<=rect[2] && p[1]<=rect[3]
}

#[derive(Clone,Debug)]
pub enum Graphics {
    Rectangle(f64,f64),
    Color([f32;4], Box<Graphics>),
    Translate(Vec2d, Box<Graphics>),
    Scale(f64, Box<Graphics>),
    Text(u32, String),
    Group(Vec<Graphics>),
    Click(Action, Box<Graphics>),
}

impl Graphics {
    pub fn color(self, col: [f32;4]) -> Graphics {
        Graphics::Color(col, Box::new(self))
    }
    pub fn translate(self, v: Vec2d) -> Graphics {
        Graphics::Translate(v, Box::new(self))
    }
    pub fn scale(self, s: f64) -> Graphics {
        Graphics::Scale(s, Box::new(self))
    }
    pub fn click(self, ac: Action) -> Graphics {
        Graphics::Click(ac, Box::new(self))
    }
}

    /*
    pub fn bbox(&self) -> [f64;4] {
        use Graphics::*;
        match self {
            &Rectangle(w,h) => [0.0, 0.0, w,h],
            &Translate(v, ref gr)  => {
                let mut r = gr.bbox();
                r[0] -= v[0];
                r[1] -= v[1];
                r
            }
            &Scale(s, ref gr) => {
                let mut r = gr.bbox();
                for i in 0..3 {
                    r[i] *= s;
                }
                r
            }
            &Color(_, ref gr) | &Click(_, ref gr) => {
                gr.bbox()
            }
            &Text(size, ref txt) => {
            }
        }
    }
    */

#[derive(Clone, Debug)]
enum Prim<'a> {
    PrimColor([f32;4]),
    PrimTransform(Matrix2d),
    PrimDraw(&'a [Graphics]),
    PrimDrawS(&'a Graphics),
    PrimClick(Action),
}

use Prim::*;

fn main() {
    use self::Tile::*;
    use self::Card::*;
    let mut map = test_map();
    let cards = vec![((0,1), Card::Farm)];

    let mut window: PistonWindow =
        WindowSettings::new("Ludum dare 38!", [512; 2])
            .exit_on_esc(true)
            .samples(8)
            .vsync(true)
            .build().unwrap();

    let factory = window.factory.clone();

    let mut font = FontCache::new(factory, "assets/NotoSans-Regular.ttf");

    let mut zoom = 1.6;
    let mut middle_pressed = false;
    let mut left_pressed = false;
    let mut shift = [0.0, 0.0];
    let mut mouse_pos = [-1000000.0, -1000000.0];

    let mut hover_action = None;
    let mut deck = Deck::new();
    let mut state = State::Def;

    while let Some(e) = window.next() {
        let out = window.output_color.clone();

        e.mouse_scroll(|_, y| {
            zoom = clamp(0.6, zoom+y*0.2, 5.0);
        });

        e.cursor(|b| {
            middle_pressed = false;
            left_pressed = false;
        });

        e.press(|btn| {
            match btn {
                Button::Mouse(MouseButton::Middle) => {
                    middle_pressed = true;
                }
                Button::Mouse(MouseButton::Left) => left_pressed = true,
                _   => {}
            }
        });

        e.release(|btn| {
            match btn {
                Button::Mouse(MouseButton::Right) => { state = State::Def; }
                Button::Mouse(MouseButton::Middle) => { middle_pressed = false; }
                Button::Mouse(MouseButton::Left) => {
                    left_pressed = false;
                    if let Some(h) = hover_action.take() {
                        println!("{:?}", h);
                        match h {
                            Action::Deck(c, i) => { state = State::PlaceCard(c,i); }
                            Action::Field(p) => { 
                                if let State::PlaceCard(c,i) = state.clone() {
                                    state = State::Def; 
                                    map.place_card(p, c.clone());
                                    deck.remove_card(i);
                                }
                            }
                        }
                    }
                }
                _   => {}
            }
        });

        let tile_size = 100.0;

        e.mouse_relative(|x,y| {
            if middle_pressed {
                shift = math::add(shift, [x,y]);
            }
        });

        e.mouse_cursor(|x,y| {
            mouse_pos = [x, y];
        });

        window.draw_2d(&e, |c, mut g| {
            let last_hover_action = hover_action.take();

            clear([0.5, 0.5, 0.5, 1.0], g);


            let field = {
                map.build_graphics(&state)
                    .translate(shift)
                    .scale(zoom)
            };

            let ui = {
                let v = c.get_view_size();

                let r = Graphics::Rectangle(v[0],200.0)
                    .color([0.3,0.3,0.3,1.0]);
                let cards = deck.draw(v[0], &state).scale(2.0);
                Graphics::Group(vec![r, cards])
                    .translate([0.0, v[1]-200.0])
            };

            let mut graphics = vec![field, ui];
            if let &State::PlaceCard(ref c,i) = &state {
                graphics.push(c.draw().translate(mouse_pos));
            } 
            let graphics = Graphics::Group(graphics);

            let mut stack = vec![PrimDrawS(&graphics)];
            let singleton = |gr| PrimDrawS(gr);
            let mut trans = identity();
            let mut color = [0.0, 0.0, 0.0, 1.0];
            let mut hovered = false;

            while let Some(e) = stack.pop() {
                use Graphics::*;
                match e {
                    PrimColor(c)        => { color = c; }
                    PrimTransform(t)    => { trans = t; }
                    PrimClick(c)    => {
                        if hovered { 
                            hover_action = Some(c);
                        }
                    }
                    PrimDrawS(s0)   => {
                        match s0 {
                            &Rectangle(w,h) => {
                                rectangle(color, [0.0, 0.0, w, h], 
                                          multiply(c.transform,trans), g);

                                let ti = vecmath::mat2x3_inv(trans);
                                let p = transform_pos(ti, mouse_pos);
                                if inside([0.0,0.0,w,h], p) {
                                    hovered = true;
                                }
                            }
                            &Color(col, ref gr) => {
                                stack.push(PrimColor(col));
                                color = col;
                                stack.push(singleton(gr));
                            }
                            &Translate(t, ref gr) => {
                                stack.push(PrimTransform(trans));
                                trans = trans.trans(t[0],t[1]);
                                stack.push(singleton(gr));
                            }
                            &Scale(t, ref gr) => {
                                stack.push(PrimTransform(trans));
                                trans = trans.scale(t,t);
                                stack.push(singleton(gr));
                            }
                            &Text(size,ref txt) => {
                                let s = (get_scale(trans)[1]*size as f64).ceil();
                                let sf = size as f64 / s;
                                text([0.0,0.0,0.0,1.0], s as u32, txt, &mut font, 
                                     multiply(c.transform,
                                              trans.scale(sf, sf)), g);
                            }
                            &Group(ref children) => {
                                stack.push(PrimDraw(children));
                            }
                            &Click(ref ac, ref gr) => {
                                stack.push(PrimClick(ac.clone()));
                                stack.push(singleton(gr));
                                hovered = false;
                            }
                        }
                    }
                    PrimDraw(gra) => {
                        if let Some((s0,s1)) = gra.split_first() {
                            stack.push(PrimDraw(s1));
                            stack.push(PrimDrawS(s0));
                        }
                    }
                }
            }

            if hover_action != last_hover_action {
                left_pressed = false;
            }
        });
    }

    //loop 
    {
        let nec_pops:u32 = cards.iter()
            .map(|e| &e.1)
            .map(|e| 
            match e {
                &Farm   => 100,
                &Lumber => 100,
            })
            .sum();

        let pops = map.pops();

        if pops==0 {
            println!("Game over!");
            return;
        }

        let effectivity = 
            if nec_pops<pops {
                nec_pops as f64/pops as f64
            } else {
                1.0
            };
    }
}
