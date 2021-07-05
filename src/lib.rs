#![allow(dead_code)]

use bitvec::prelude::*;

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
            let neighbor_indices =
                moore_neighborhood_wrapping((index % size.0, index / size.0), size);

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
            let neighbor_indices =
                moore_neighborhood_wrapping((index % size.0, index / size.0), size);

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

impl ConwaysLife {
    pub fn simulate_with_logic(status: bool, alive_neighbors: usize) -> bool {
        if status {
            (2..=3).contains(&alive_neighbors)
        } else {
            alive_neighbors == 3
        }
    }
}

fn moore_neighborhood_wrapping(cell: (usize, usize), size: (usize, usize)) -> [(usize, usize); 8] {
    if cell.0 != 0 && cell.1 != 0 {
        [
            (cell.0 - 1, cell.1 - 1),
            (cell.0, cell.1 - 1),
            ((cell.0 + 1) % size.0, cell.1 - 1),
            ((cell.0 + 1) % size.0, cell.1),
            ((cell.0 + 1) % size.0, (cell.1 + 1) % size.1),
            (cell.0, (cell.1 + 1) % size.1),
            (cell.0 - 1, (cell.1 + 1) % size.1),
            (cell.0 - 1, cell.1),
        ]
    } else {
        let early_column = size.0 - 1;
        let early_row = size.1 - 1;
        [
            (early_column, early_row),
            (cell.0, early_row),
            ((cell.0 + 1) % size.0, early_row),
            ((cell.0 + 1) % size.0, cell.1),
            ((cell.0 + 1) % size.0, (cell.1 + 1) % size.1),
            (cell.0, (cell.1 + 1) % size.1),
            (early_column, (cell.1 + 1) % size.1),
            (early_column, cell.1),
        ]
    }
}

#[cfg(test)]
mod test {
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn neighborhood_simple() {
        let size = (3, 3);
        let cell = (1, 1); // the cell in the middle

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
        let cell = (0, 0);

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
    fn conways_life_compiles() {
        let _life = LifeLike::new("B3/S23").unwrap();
    }

    proptest! {
        #[test]
        fn pt_lifelike_compiles_valid_strings(rule in "B[0-8]{0,8}/S[0-8]{0,8}") {
            prop_assert!(LifeLike::new(&rule).is_ok())
        }
    }

    proptest! {
        #[test]
        fn pt_lifelike_matches_logic(neighbors in 0usize..9) {
            let life = LifeLike::new("B3/S23").unwrap();
            let alive = ConwaysLife::simulate_with_logic(true, neighbors) == life.simulate(true, neighbors);
            let dead = ConwaysLife::simulate_with_logic(false, neighbors) == life.simulate(false, neighbors);
            prop_assert!(alive && dead)
        }
    }

    proptest! {
        #[test]
        fn pt_lifelike_index_reversible(neighbors in 0usize..9) {
            prop_assert_eq!((true, neighbors & 0b1111), LifeLike::decode_index(LifeLike::encode_index(true, neighbors)));
            prop_assert_eq!((false, neighbors & 0b1111), LifeLike::decode_index(LifeLike::encode_index(false, neighbors)));
        }
    }
}
