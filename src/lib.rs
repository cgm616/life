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

pub trait Algorithm<T> {
    fn next_state(current: Cell, context: &[T], size: (usize, usize)) -> T;
}

pub struct ConwaysLife;

impl Algorithm<bool> for ConwaysLife {
    fn next_state(current: Cell, context: &[bool], size: (usize, usize)) -> bool {
        let neighbor_indices = moore_neighborhood_wrapping(current, size);

        let alive = neighbor_indices
            .iter()
            .map(|(x, y)| context[x + (size.0 * y)])
            .filter(|&i| i)
            .count();

        if context[current.x + (size.0 * current.y)] {
            if alive < 2 {
                false
            } else if alive > 3 {
                false
            } else {
                true
            }
        } else {
            if alive == 3 {
                true
            } else {
                false
            }
        }
    }
}

impl ConwaysLife {}

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

fn moore_neighborhood_wrapping(cell: Cell, size: (usize, usize)) -> [(usize, usize); 8] {
    [
        (
            cell.x.checked_sub(1).unwrap_or(size.0 - 1),
            cell.y.checked_sub(1).unwrap_or(size.1 - 1),
        ),
        (cell.x, cell.y.checked_sub(1).unwrap_or(size.1 - 1)),
        (
            (cell.x + 1) % size.0,
            cell.y.checked_sub(1).unwrap_or(size.1 - 1),
        ),
        ((cell.x + 1) % size.0, cell.y),
        ((cell.x + 1) % size.0, (cell.y + 1) % size.1),
        (cell.x, (cell.y + 1) % size.1),
        (
            cell.x.checked_sub(1).unwrap_or(size.0 - 1),
            (cell.y + 1) % size.1,
        ),
        (cell.x.checked_sub(1).unwrap_or(size.0 - 1), cell.y),
    ]
}

#[cfg(test)]
mod test {
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
}
