#![allow(unused_variables, unused_imports)]

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

extern crate piston_window;

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

    /// Get all the places, where you can put a card.
    pub fn card_options(&self) -> Vec<(Coord,Card)> {
        let mut places = Vec::new();
        use self::Tile::*;
        use self::Card::*;
        let w = self.width;

        for y in 0..self.height {
            for x in 0..self.width {
                let coord = (x,y);
                let mut add = |t| { places.push((coord,t)); };

                match self.tiles[(y*w+x) as usize] {
                    Forrest => { if !self.cards.contains_key(&coord) { add(Lumber) } }
                    Farmland => { if !self.cards.contains_key(&coord) { add(Farm) } }
                    Coal | Iron | Mountain | City(_) => {}
                }
            }
        }

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

    let mut zoom = 2.0;
    let mut right_pressed = false;
    let mut shift = [0.0, 0.0];

    while let Some(e) = window.next() {
        let out = window.output_color.clone();

        e.mouse_scroll(|_, y| {
            zoom = clamp(0.24, zoom+y*0.1, 4.0);
        });

        e.cursor(|b| {
            right_pressed = false;
        });

        e.press(|btn| {
            match btn {
                Button::Mouse(MouseButton::Right) => {
                    right_pressed = true;
                }
                _   => {}
            }
        });

        e.release(|btn| {
            match btn {
                Button::Mouse(MouseButton::Right) => right_pressed = false,
                _   => {}
            }
        });

        e.mouse_relative(|x,y| {
            if right_pressed {
                shift = math::add(shift, [x,y]);
            }
        });

        let tilesize = 100.0 * zoom;

        window.draw_2d(&e, |mut c, mut g| {
            clear([0.5, 0.5, 0.5, 1.0], g);

            for y in 0..map.height {
                for x in 0..map.width {

                    let ref tile = map.tiles[(y*map.width+x) as usize];

                    let trans = c.transform
                        .trans(x as f64*tilesize,y as f64*tilesize)
                        .trans(shift[0], shift[1]);

                    rectangle(tile.color(),
                        [0.0, 0.0, tilesize, tilesize],
                        trans, g);

                    text([0.0,0.0,0.0,1.0],
                         (12.0*zoom) as u32,
                         tile.text(),
                         &mut font,
                         trans.trans(10.0*zoom,10.0*zoom),
                         g);
                }
            }


            let v = c.get_view_size();
            rectangle([0.3,0.3,0.3,1.0],
                      [0.0, v[1]-200.0, v[0], 200.0],
                      c.transform, g);
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
    #[should_panic]
    fn card_placement_fail() {
        let mut map = test_map();
        map.place_card((0,0),Lumber);
        map.place_card((0,0),Lumber);
    }
}
