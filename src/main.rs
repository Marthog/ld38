#![allow(unused_variables, unused_imports)]

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

extern crate piston_window;
extern crate vecmath;
extern crate assert;

use self::piston_window::*;

use std::collections::{HashMap};

mod graphics;
use self::graphics::FontCache;
use self::piston_window::math::*;

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug)]
pub enum Card {
    Farm,
    Lumber,
}
type Coord = (u32,u32);

#[derive(Clone, Debug)]
pub enum Tile {
    Forrest,
    Farmland,
    Mountain,
    Coal,
    Iron,
    City(u32),
}

impl Tile {
    pub fn color(&self) -> [f32;4] {
        use self::Tile::*;
        let s = match self {
            &Forrest    => [0.2, 0.8, 0.4],
            &Farmland   => [0.4, 1.0, 0.4],
            &Mountain   => [0.4, 0.4, 0.4],
            &Coal       => [0.2, 0.2, 0.2],
            &Iron       => [0.8, 0.2, 0.2],
            &City(_)    => [0.8, 0.6, 0.6],
        };
        [s[0],s[1],s[2],1.0]
    }

    pub fn text(&self) -> &'static str {
        use self::Tile::*;
        match self {
            &Forrest    => "Forrest",
            &Farmland   => "Farmland",
            &Mountain   => "Mountain",
            &Coal       => "Coal",
            &Iron       => "Mountain",
            &City(_)    => "City",
        }
    }

}

#[derive(Clone, Debug, Default)]
pub struct Map {
    pub tiles: Vec<Tile>,
    pub width: u32,
    pub height: u32,
    pub cards: HashMap<Coord,Card>,
}

impl Map {
    pub fn new(width: u32, height: u32, tiles: Vec<Tile>) -> Map {
        assert_eq!(tiles.len() as u32, width*height);
        Map {
            width: width,
            height: height,
            tiles: tiles,
            cards: HashMap::new(),
        }
    }

    /// Total population.
    pub fn pops(&self) -> u32 {
        use self::Tile::*;
        self.tiles.iter().map(|e| 
            match e {
                &City(p)    => p,
                _           => 0,
            }
        ).sum()
    }

    /// Neccessary population.
    pub fn nec_pops(&self) -> u32 {
        use self::Tile::*;
        use self::Card::*;

        let workers:u32 = self.cards.iter()
            .map(|e| e.1)
            .map(|e| 
            match e {
                &Farm   => 100,
                &Lumber => 100,
            })
            .sum();
        let admin = self.pops() / 10;
        workers+admin
    }

    pub fn build_graphics(&self) -> Graphics {
        use Graphics::*;
        let mut group = Vec::new();
        self.each(|x,y,tile| {
            let tile_size = 100.0;
            let bg = Rectangle(tile_size,tile_size)
                .color(tile.color())
                .click(y*self.width+x);

            let txt = Text(12, tile.text().to_string())
                .translate([10.0, 10.0]);

            let r = Group(vec![bg,txt])
                .translate([x as f64*tile_size,y as f64*tile_size]);

            group.push(r);
        });
        Group(group)
    }

    pub fn each<F>(&self, mut f: F) -> ()
    where F: FnMut(u32, u32, &Tile) -> ()
    {
        for y in 0..self.height {
            for x in 0..self.width {
                f(x, y, &self.tiles[(y*self.width + x) as usize]);
            }
        }
    }

    pub fn each_mut<F>(&mut self, mut f: F) -> ()
    where F: FnMut(u32, u32, &mut Tile) -> ()
    {
        for y in 0..self.height {
            for x in 0..self.width {
                f(x, y, &mut self.tiles[(y*self.width + x) as usize]);
            }
        }
    }

    /// Get all the places, where you can put a card.
    pub fn card_options(&self) -> Vec<(Coord,Card)> {
        let mut places = Vec::new();
        use self::Tile::*;
        use self::Card::*;
        let w = self.width;

        self.each(|x,y,tile| {
            let coord = (x,y);
            let mut add = |t| { places.push((coord,t)); };

            match *tile {
                Forrest => { if !self.cards.contains_key(&coord) { add(Lumber) } }
                Farmland => { if !self.cards.contains_key(&coord) { add(Farm) } }
                Coal | Iron | Mountain | City(_) => {}
            }
        });

        places
    }

    /// Place a card on the map.
    pub fn place_card(&mut self, coord: Coord, card: Card) {
        let c = (coord, card.clone());
        debug_assert!(self.card_options().iter().any(|x| x==&c));

        use std::collections::hash_map::Entry::*;
        let v = self.cards.insert(coord, card);
        assert!(v.is_none());
    }
}

fn clamp<T: PartialOrd>(min: T, val: T, max: T) -> T {
    if val<min {
        min
    } else if val>max {
        max
    } else {
        val
    }
}

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
    Click(u32, Box<Graphics>),
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
    pub fn click(self, id: u32) -> Graphics {
        Graphics::Click(id, Box::new(self))
    }
}

#[derive(Clone, Debug)]
enum Prim<'a> {
    PrimColor([f32;4]),
    PrimTransform(Matrix2d),
    PrimDraw(&'a [Graphics]),
    PrimDrawS(&'a Graphics),
    PrimClick(u32),
}

use Prim::*;

fn main() {
    use self::Tile::*;
    use self::Card::*;
    let map = test_map();
    let cards = vec![((0,1), Card::Farm)];

    let mut window: PistonWindow =
        WindowSettings::new("Ludum dare 38!", [512; 2])
            .exit_on_esc(true)
            .samples(8)
            .vsync(true)
            .build().unwrap();

    let factory = window.factory.clone();

    let mut font = FontCache::new(factory, "assets/NotoSans-Regular.ttf");

    let mut zoom = 1.0;
    let mut right_pressed = false;
    let mut left_pressed = false;
    let mut shift = [0.0, 0.0];
    let mut mouse_pos = [-1000000.0, -1000000.0];

    while let Some(e) = window.next() {
        let out = window.output_color.clone();

        e.mouse_scroll(|_, y| {
            zoom = clamp(0.24, zoom+y*0.1, 4.0);
        });

        e.cursor(|b| {
            right_pressed = false;
            left_pressed = false;
        });

        e.press(|btn| {
            match btn {
                Button::Mouse(MouseButton::Right) => {
                    right_pressed = true;
                }
                Button::Mouse(MouseButton::Left) => left_pressed = true,
                _   => {}
            }
        });

        e.release(|btn| {
            match btn {
                Button::Mouse(MouseButton::Right) => right_pressed = false,
                Button::Mouse(MouseButton::Left) => {
                    left_pressed = false;
                }
                _   => {}
            }
        });

        let tile_size = 100.0;


        e.mouse_relative(|x,y| {
            if right_pressed {
                shift = math::add(shift, [x,y]);
            }
        });

        e.mouse_cursor(|x,y| {
            mouse_pos = [x, y];
        });

        let tilesize = 100.0 * zoom;

        window.draw_2d(&e, |c, mut g| {
            clear([0.5, 0.5, 0.5, 1.0], g);

            let field = {
                map.build_graphics()
                    .translate(shift)
                    .scale(zoom)
            };

            let ui = {
                let v = c.get_view_size();

                Graphics::Rectangle(v[0],200.0)
                    .translate([0.0, v[1]-200.0])
                    .color([0.3,0.3,0.3,1.0])
            };

            let graphics = Graphics::Group(vec![field, ui]);

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
                    PrimClick(c)    => { if hovered { println!("{}", c); } }
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
                            &Click(id, ref gr) => {
                                stack.push(PrimClick(id));
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

fn test_map() -> Map {
    use self::Tile::*;
    Map::new(2,3,
        vec![Forrest,Mountain,
                Farmland,City(1000),
                Farmland,Coal])
}


#[cfg(test)]
mod tests {
    use super::*;
    use self::Tile::*;
    use self::Card::*;

    #[test]
    fn test_test_map() {
        test_map();
    }

    #[test]
    fn map_pops() {
        assert_eq!(test_map().pops(), 1000);
    }

    #[test]
    fn card_options() {
        let map = test_map();
        assert_eq!(map.card_options(), vec![
                ((0,0),Lumber),
                ((0,1),Farm),
                ((0,2),Farm),
        ])
    }

    #[test]
    fn card_placement() {
        let mut map = test_map();
        map.place_card((0,0),Lumber);
    }

    #[test]
    fn test_clamp() {
        assert_eq!(clamp(0.0, 10.0, 100.0), 10.0);
        assert_eq!(clamp(0.0, 100.0, 10.0), 10.0);
        assert_eq!(clamp(10.0, 0.0, 100.0), 10.0);
    }


    #[test]
    #[should_panic]
    fn card_placement_fail() {
        let mut map = test_map();
        map.place_card((0,0),Lumber);
        map.place_card((0,0),Lumber);
    }
}
