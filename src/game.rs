#![allow(unused_variables, unused_imports)]
use std::collections::{HashMap};
use super::Graphics;
use ::Graphics::*;
use rand::{self, Rand, Rng};
use rand::distributions::{Range, Sample};
use std::mem;

#[derive(Clone,Debug, PartialEq, Eq)]
pub enum State {
    Def,
    PlaceCard(Card, u32),
}

#[derive(Clone,Debug, PartialEq, Eq)]
pub enum Action {
    Field((u32,u32)),
    Deck(Card, u32),
}

const CARD_WIDTH: f64 = 40.0;

#[repr(C)]
#[repr(u8)]
#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug)]
pub enum Card {
    Farm,
    Lumber,
}

impl Rand for Card {
    fn rand<R: Rng>(rng: &mut R) -> Self {
        let i: u8 = rand::distributions::Range::new(0,2)
            .sample(rng);
        unsafe { mem::transmute(i) }
    }
}

impl Card {
    pub fn color(&self) -> [f32;4] {
        use self::Card::*;
        let s = match self {
            &Farm    => [0.2, 0.8, 0.4],
            &Lumber  => [0.8, 0.6, 0.4],
        };
        [s[0],s[1],s[2],1.0]
    }

    pub fn title(&self) -> &'static str {
        use self::Card::*;
        match self {
            &Farm       => "Farm",
            &Lumber     => "Lumbermill",
        }
    }

    pub fn draw(&self) -> Graphics {
        let bg = Rectangle(CARD_WIDTH, 60.0)
            .color(self.color());

        let txt = Text(6, self.title().to_string())
            .translate([0.0, 8.0]);

        Group(vec![bg,txt])
    }
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

    pub fn matches(&self, coord: (u32,u32), card: &Card) -> bool {
        self.card_options().iter().any(|&(p,ref c)| coord==p && card==c)
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

    pub fn build_graphics(&self, state: &State) -> Graphics {
        let mut group = Vec::new();
        self.each(|x,y,tile| {
            let tile_size = 100.0;
            let mut bg = Rectangle(tile_size,tile_size)
                .color(tile.color());

            if let &State::PlaceCard(ref card,i) = state {
                if (self.matches((x,y), card)) {
                    bg = bg.click(Action::Field((x,y)));
                }
            }

            let txt = Text(12, tile.text().to_string())
                .translate([10.0, 10.0]);

            let mut gr = vec![bg,txt];
            if let Some(card) = self.cards.get(&(x,y)) {
                let c = card.draw()
                    .translate([20.0, 15.0]);
                    // .click(Action::Field((x,y)));

                gr.push(c);
            }

            let r = Group(gr)
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

pub fn clamp<T: PartialOrd>(min: T, val: T, max: T) -> T {
    if val<min {
        min
    } else if val>max {
        max
    } else {
        val
    }
}

pub fn test_map() -> Map {
    use self::Tile::*;
    use self::Card::*;

    let mut map = Map::new(2,3,
        vec![Forrest,Mountain,
                Farmland,City(1000),
                Farmland,Coal]);
    map.place_card((0,0),Lumber);
    map
}


pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    pub fn new() -> Deck {
        let mut d = Deck{cards: Vec::new()};
        d.fill();
        d
    }

    pub fn draw(&self, width: f64, state: &State) -> Graphics {
        let margin = 10.0;
        let dist = clamp(0.0, CARD_WIDTH+2.0*margin, 
                         (width-2.0*margin)/self.cards.len() as f64);
        let mut x = margin;
        let mut v = Vec::new();
        for i in 0..self.cards.len() {
            let c = &self.cards[i];
            let mut l = c.draw()
                .scale(1.2)
                .translate([x+margin, margin]);
            if let &State::Def = state {
                l = l.click(Action::Deck(c.clone(), i as u32));
            }
            x += dist;
            v.push(l);
        }
        Group(v)
    }

    pub fn fill(&mut self) {
        for i in 0..5 {
            self.cards.push(rand::random::<Card>());
        }
    }

    pub fn remove_card(&mut self, index: u32) {
        self.cards[index as usize] = rand::random::<Card>();
    }
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
                ((0,1),Farm),
                ((0,2),Farm),
        ])
    }

    #[test]
    fn card_placement() {
        let mut map = test_map();
        map.place_card((0,1),Farm);
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
    }
}
