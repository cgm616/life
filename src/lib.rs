use std::{
    convert::TryFrom,
    iter::FromIterator,
    ops::{Deref, DerefMut},
};

#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

use bitvec::prelude::*;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Cell {
    pub x: usize,
    pub y: usize,
}

impl From<(usize, usize)> for Cell {
    fn from(tuple: (usize, usize)) -> Cell {
        Cell {
            x: tuple.0,
            y: tuple.1,
        }
    }
}

pub trait Automata {
    fn update<O: BitOrder, T: BitStore>(
        &self,
        world: &BitSlice<O, T>,
        target: &mut BitSlice<O, T>,
        changes: &BitSlice<O, T>,
        size: (usize, usize),
    );
}

pub struct LifeLike {
    rules: Box<[bool; 18]>,
}

impl LifeLike {
    fn encode_index(status: bool, neighbors: usize) -> usize {
        let lowest = status as usize;
        lowest + ((neighbors & 0b1111) << 1)
    }

    fn decode_index(index: usize) -> (bool, usize) {
        (index & 1 != 0, (index & 0b11110) >> 1)
    }

    pub fn new(def: &str) -> Result<Self, &'static str> {
        if !def.is_ascii() {
            return Err("definition string must be ascii");
        }

        let (mut b, mut s) = def
            .split_once('/')
            .ok_or("invalid definition string format")?;

        b = b
            .strip_prefix(|c| c == 'b' || c == 'B')
            .ok_or("first part of def string must start with 'b'")?;
        s = s
            .strip_prefix(|c| c == 's' || c == 'S')
            .ok_or("second part of def string must start with 's'")?;

        let mut rules = [false; 18];

        for c in b.chars() {
            let neighbors = c
                .to_digit(10)
                .ok_or("could not parse numbers from definition string first part")?;
            rules[Self::encode_index(false, neighbors as usize)] = true;
        }

        for c in s.chars() {
            let neighbors = c
                .to_digit(10)
                .ok_or("could not parse numbers from definition string second part")?;
            rules[Self::encode_index(true, neighbors as usize)] = true;
        }

        Ok(LifeLike {
            rules: Box::new(rules),
        })
    }

    pub fn simulate(&self, status: bool, neighbors: usize) -> bool {
        self.rules[Self::encode_index(status, neighbors)]
    }
}

impl Automata for LifeLike {
    fn update<O: BitOrder, T: BitStore>(
        &self,
        world: &BitSlice<O, T>,
        target: &mut BitSlice<O, T>,
        changes: &BitSlice<O, T>,
        size: (usize, usize),
    ) {
        assert_eq!(world.len(), size.0 * size.1);

        let mut extras = Vec::new();

        for index in changes.iter_ones() {
            let neighbor_indices = moore_neighborhood_wrapping(
                Cell {
                    x: index % size.0,
                    y: index / size.0,
                },
                size,
            );

            let neighbors = neighbor_indices
                .iter()
                .map(|(x, y)| {
                    let combined = x + (y * size.0);
                    if !unsafe { *changes.get_unchecked(combined) } {
                        extras.push(combined);
                    }
                    unsafe { *world.get_unchecked(combined) }
                })
                .filter(|&i| i)
                .count();

            let status = *world.get(index).as_deref().unwrap();

            let next_status = self.simulate(status, neighbors);
            target.set(index, next_status);
        }

        extras.sort_unstable();
        extras.dedup();

        for &index in extras.iter() {
            let neighbor_indices = moore_neighborhood_wrapping(
                Cell {
                    x: index % size.0,
                    y: index / size.0,
                },
                size,
            );

            let neighbors = neighbor_indices
                .iter()
                .map(|(x, y)| {
                    let combined = x + (y * size.0);
                    unsafe { *world.get_unchecked(combined) }
                })
                .filter(|&i| i)
                .count();

            let status = *world.get(index).as_deref().unwrap();

            let next_status = self.simulate(status, neighbors);
            target.set(index, next_status);
        }
    }
}

pub struct ConwaysLife;

impl Automata for ConwaysLife {
    fn update<O: BitOrder, T: BitStore>(
        &self,
        world: &BitSlice<O, T>,
        target: &mut BitSlice<O, T>,
        changes: &BitSlice<O, T>,
        size: (usize, usize),
    ) {
        assert_eq!(world.len(), size.0 * size.1);

        let mut extras = Vec::new();

        for index in changes.iter_ones() {
            let neighbor_indices = moore_neighborhood_wrapping(
                Cell {
                    x: index % size.0,
                    y: index / size.0,
                },
                size,
            );

            let mut hood = 0u8;

            for (i, neighbor_index) in neighbor_indices.iter().enumerate() {
                let combined = neighbor_index.0 + (neighbor_index.1 * size.0);
                hood |= (unsafe { *world.get_unchecked(combined) } as u8) << i;
                if !unsafe { *changes.get_unchecked(combined) } {
                    extras.push(combined);
                }
            }

            let hood: MooreNeighborhood = hood.into();

            let own_status = *world.get(index).as_deref().unwrap();

            let next_status = ConwaysLife::simulate_with_lookup(own_status, hood);
            target.set(index, next_status);
        }

        extras.sort_unstable();
        extras.dedup();

        for &index in extras.iter() {
            let neighbor_indices = moore_neighborhood_wrapping(
                Cell {
                    x: index % size.0,
                    y: index / size.0,
                },
                size,
            );

            let mut hood = 0u8;

            for (i, neighbor_index) in neighbor_indices.iter().enumerate() {
                let combined = neighbor_index.0 + (neighbor_index.1 * size.0);
                hood |= (unsafe { *world.get_unchecked(combined) } as u8) << i;
            }

            let hood: MooreNeighborhood = hood.into();

            let own_status = *world.get(index).as_deref().unwrap();

            let next_status = ConwaysLife::simulate_with_lookup(own_status, hood);
            target.set(index, next_status);
        }
    }
}

impl ConwaysLife {
    const LOOKUP: BitArr!(for 512) = include!("lookup.rs");

    pub fn simulate_with_lookup(status: bool, hood: MooreNeighborhood) -> bool {
        let key = ((status as usize) << 8) | (*hood as usize);
        assert!(key < 512, "the key should never be bigger than 9 bits");
        unsafe { *Self::LOOKUP.get_unchecked(key) }
    }

    pub fn simulate_with_logic(status: bool, hood: MooreNeighborhood) -> bool {
        let alive_neighbors = hood.count_ones();

        if status {
            if alive_neighbors < 2 {
                false
            } else if alive_neighbors > 3 {
                false
            } else {
                true
            }
        } else {
            if alive_neighbors == 3 {
                true
            } else {
                false
            }
        }
    }
}

/*
pub struct Anneal;

impl Algorithm<bool> for Anneal {
    fn next_state(current: Cell, context: &[bool], size: (usize, usize)) -> bool {
        let neighbor_indices = moore_neighborhood_wrapping(current, size);

        let alive = neighbor_indices
            .iter()
            .map(|(x, y)| context[x + (size.0 * y)])
            .filter(|&i| i)
            .count();

        if context[current.x + (size.0 * current.y)] {
            if alive == 3 || alive > 4 {
                true
            } else {
                false
            }
        } else {
            if alive == 4 || alive > 5 {
                true
            } else {
                false
            }
        }
    }
}
*/

/// A compact representation of a Moore neighborhood of a given cell.
///
/// The underlying data is stored in a u8 according to the following scheme:
/// `0b76543210`
///    <==>
/// ```text
/// 0 | 1 | 2
/// 7 | _ | 3
/// 6 | 5 | 4
/// ```
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct MooreNeighborhood(u8);

impl Deref for MooreNeighborhood {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MooreNeighborhood {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub enum MooreNeighbor {
    TopLeft,
    TopMid,
    TopRight,
    MidRight,
    BottomRight,
    BottomMid,
    BottomLeft,
    MidLeft,
}

impl MooreNeighborhood {
    fn get_neighbor(&self, neighbor: MooreNeighbor) -> bool {
        (self.0 & neighbor.bit_mask()) != 0
    }
}

impl MooreNeighbor {
    fn bit_mask(&self) -> u8 {
        match self {
            Self::TopLeft => 1 << 0,
            Self::TopMid => 1 << 1,
            Self::TopRight => 1 << 2,
            Self::MidRight => 1 << 3,
            Self::BottomRight => 1 << 4,
            Self::BottomMid => 1 << 5,
            Self::BottomLeft => 1 << 6,
            Self::MidLeft => 1 << 7,
        }
    }
}

impl From<u8> for MooreNeighborhood {
    fn from(other: u8) -> Self {
        MooreNeighborhood(other)
    }
}

impl TryFrom<&[bool]> for MooreNeighborhood {
    type Error = ();

    fn try_from(slice: &[bool]) -> Result<Self, ()> {
        if slice.len() != 8 {
            return Err(());
        }

        Ok(MooreNeighborhood(
            slice
                .iter()
                .enumerate()
                .fold(0_u8, |acc, (index, value)| acc | ((*value as u8) << index)),
        ))
    }
}

impl Into<[bool; 8]> for MooreNeighborhood {
    fn into(self) -> [bool; 8] {
        [
            self.get_neighbor(MooreNeighbor::TopLeft),
            self.get_neighbor(MooreNeighbor::TopMid),
            self.get_neighbor(MooreNeighbor::TopRight),
            self.get_neighbor(MooreNeighbor::MidRight),
            self.get_neighbor(MooreNeighbor::BottomRight),
            self.get_neighbor(MooreNeighbor::BottomMid),
            self.get_neighbor(MooreNeighbor::BottomLeft),
            self.get_neighbor(MooreNeighbor::MidLeft),
        ]
    }
}

impl FromIterator<bool> for MooreNeighborhood {
    fn from_iter<T: IntoIterator<Item = bool>>(iter: T) -> Self {
        MooreNeighborhood(
            iter.into_iter()
                .take(8)
                .enumerate()
                .fold(0_u8, |acc, (index, value)| acc | ((value as u8) << index)),
        )
    }
}

fn moore_neighborhood_wrapping(cell: Cell, size: (usize, usize)) -> [(usize, usize); 8] {
    if cell.x != 0 && cell.y != 0 {
        [
            (cell.x - 1, cell.y - 1),
            (cell.x, cell.y - 1),
            ((cell.x + 1) % size.0, cell.y - 1),
            ((cell.x + 1) % size.0, cell.y),
            ((cell.x + 1) % size.0, (cell.y + 1) % size.1),
            (cell.x, (cell.y + 1) % size.1),
            (cell.x - 1, (cell.y + 1) % size.1),
            (cell.x - 1, cell.y),
        ]
    } else {
        let early_column = size.0 - 1;
        let early_row = size.1 - 1;
        [
            (early_column, early_row),
            (cell.x, early_row),
            ((cell.x + 1) % size.0, early_row),
            ((cell.x + 1) % size.0, cell.y),
            ((cell.x + 1) % size.0, (cell.y + 1) % size.1),
            (cell.x, (cell.y + 1) % size.1),
            (early_column, (cell.y + 1) % size.1),
            (early_column, cell.y),
        ]
    }
}

#[cfg(test)]
mod test {
    use std::convert::TryInto;

    use super::*;

    #[test]
    fn neighborhood_simple() {
        let size = (3, 3);
        let cell = Cell { x: 1, y: 1 }; // the cell in the middle

        assert_eq!(
            [
                (0, 0),
                (1, 0),
                (2, 0),
                (2, 1),
                (2, 2),
                (1, 2),
                (0, 2),
                (0, 1)
            ],
            moore_neighborhood_wrapping(cell, size)
        );
    }

    #[test]
    fn neighborhood_complex() {
        let size = (3, 3);
        let cell = Cell { x: 0, y: 0 };

        assert_eq!(
            [
                (2, 2),
                (0, 2),
                (1, 2),
                (1, 0),
                (1, 1),
                (0, 1),
                (2, 1),
                (2, 0)
            ],
            moore_neighborhood_wrapping(cell, size)
        );
    }

    #[test]
    fn packed_neighborhood() {
        let array = [true, true, false, false, false, false, true, false];
        let neighborhood: MooreNeighborhood = (&array[..]).try_into().unwrap();

        assert_eq!(0b01000011, neighborhood.0);

        assert!(neighborhood.get_neighbor(MooreNeighbor::TopLeft));
        assert!(neighborhood.get_neighbor(MooreNeighbor::TopMid));
        assert!(!neighborhood.get_neighbor(MooreNeighbor::TopRight));
        assert!(!neighborhood.get_neighbor(MooreNeighbor::MidRight));
        assert!(!neighborhood.get_neighbor(MooreNeighbor::BottomRight));
        assert!(!neighborhood.get_neighbor(MooreNeighbor::BottomMid));
        assert!(neighborhood.get_neighbor(MooreNeighbor::BottomLeft));
        assert!(!neighborhood.get_neighbor(MooreNeighbor::MidLeft));

        assert_eq!(
            array,
            <MooreNeighborhood as Into<[bool; 8]>>::into(neighborhood)
        );
    }

    /*
    #[test]
    fn simple_conways() {
        let size = (4, 4);

        let world = bits![0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 0, 0, 0];
        let target = bits![mut 0; 16];

        assert_eq!(world.len(), 16);
        assert_eq!(target.len(), 16);

        ConwaysLife::update(world, target, size);

        assert_eq!(target, world);
    }
    */

    #[test]
    fn conways_life_compiles() {
        let life = LifeLike::new("B3/S23").unwrap();
    }

    #[quickcheck]
    fn lifelike_matches_logic(hood: u8) -> bool {
        let life = LifeLike::new("B3/S23").unwrap();
        let hood: MooreNeighborhood = hood.into();
        let neighbors = hood.count_ones() as usize;
        let alive = ConwaysLife::simulate_with_logic(true, hood) == life.simulate(true, neighbors);
        let dead = ConwaysLife::simulate_with_logic(false, hood) == life.simulate(false, neighbors);
        alive && dead
    }

    #[quickcheck]
    fn lookup_matches_logic(hood: u8) -> bool {
        let hood: MooreNeighborhood = hood.into();
        let alive = ConwaysLife::simulate_with_logic(true, hood)
            == ConwaysLife::simulate_with_lookup(true, hood);
        let dead = ConwaysLife::simulate_with_logic(false, hood)
            == ConwaysLife::simulate_with_lookup(false, hood);
        alive && dead
    }

    #[quickcheck]
    fn lifelike_index_reversible(status: bool, neighbors: usize) -> bool {
        (status, neighbors & 0b1111)
            == LifeLike::decode_index(LifeLike::encode_index(status, neighbors))
    }
}
