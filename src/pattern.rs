use bitvec::prelude::*;

pub struct Pattern {
    store: BitVec<Lsb0, usize>,
    size: (usize, usize),
}

impl Pattern {
    pub fn from_plaintext<'a>(lines: impl Iterator<Item = &'a str>) -> Result<Self, &'static str> {
        let mut store: BitVec<Lsb0, usize> = BitVec::new();
        let mut width = None;

        for line in lines
            .filter(|&s| !s.starts_with('!'))
            .filter(|&s| !s.is_empty())
        {
            let line = line.trim();

            match (width, line.len()) {
                (None, x) => width = Some(x),
                (Some(x), y) if x == y => {}
                _ => return Err("not all lines are the same length!"),
            }

            store.extend(line.chars().map(|c| c == 'O'));
        }

        let length = store.len();

        match width {
            Some(x) => Ok(Pattern {
                store,
                size: (x, length / x),
            }),
            None => Err("no pattern lines found"),
        }
    }

    pub fn place(
        &self,
        world: &mut BitSlice<Lsb0, usize>,
        world_size: (usize, usize),
        position: (usize, usize),
    ) -> Result<(), &'static str> {
        if world_size.0 * world_size.1 > world.len() {
            return Err("world not big enough for given size");
        }

        if (position.0 + self.size.0) > world_size.0 || (position.1 + self.size.1) > world_size.1 {
            return Err(
                "place: not enough space in world to place this pattern with given position",
            );
        }

        for row in 0..self.size.1 {
            let beginning = position.0 + (world_size.0 * (position.1 + row));
            let end = beginning + self.size.0;

            let target = &mut world[beginning..end];
            let origin = &self.store[(row * self.size.0)..((row + 1) * self.size.0)];
            target.copy_from_bitslice(origin);
        }

        Ok(())
    }

    pub fn calc_midpoint_placement(
        &self,
        world_size: (usize, usize),
    ) -> Result<(usize, usize), &'static str> {
        if self.size.0 > world_size.0 || self.size.1 > world_size.1 {
            return Err(
                "calc: not enough space in world to place this pattern with given position",
            );
        }

        Ok((
            (world_size.0 - self.size.0) / 2,
            (world_size.1 - self.size.1) / 2,
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn plaintext_pattern() {
        let pattern = "!Name: Glider\n\
                               !\n\
                               .O.\n\
                               ..O\n\
                               OOO";
        let pattern = Pattern::from_plaintext(pattern.lines()).unwrap();

        assert_eq!(pattern.size, (3, 3));
        assert_eq!(pattern.store, bits![0, 1, 0, 0, 0, 1, 1, 1, 1]);
    }
}
