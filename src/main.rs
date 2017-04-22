#![allow(unused_variables, unused_imports)]

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

extern crate piston_window;

use self::piston_window::*;

use std::collections::{HashMap};

mod graphics;
use self::graphics::FontCache;

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

#[derive(Clone, Debug, Default)]
pub struct Map {
    tiles: Vec<Tile>,
    width: u32,
    height: u32,
    cards: HashMap<Coord,Card>,
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

    while let Some(e) = window.next() {
        let out = window.output_color.clone();
        window.draw_2d(&e, |c, mut g| {
            clear([0.5, 0.5, 0.5, 1.0], g);

            text([0.0,0.0,0.0,1.0],
                 32,
                 "Hello, world!",
                 &mut font,
                 c.transform.trans(50.0,32.0),
                 g);

            rectangle([0.0, 0.0, 1.0, 1.0], // blue
                      [50.0, 50.0, 100.0, 100.0], // rectangle
                      c.transform, g);

            rectangle([1.0, 0.0, 0.0, 1.0], // red
                      [0.0, 0.0, 100.0, 100.0], // rectangle
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
        vec![Forrest,Forrest,
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
                ((0,0),Lumber), ((1,0),Lumber),
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
