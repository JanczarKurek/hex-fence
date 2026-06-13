use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AxialCoord {
    pub q: i32,
    pub r: i32,
}

impl AxialCoord {
    pub const fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    pub fn is_inside_board(self, board_radius: i32) -> bool {
        let s = -self.q - self.r;
        self.q.abs() <= board_radius && self.r.abs() <= board_radius && s.abs() <= board_radius
    }

    pub fn neighbors(self) -> [Self; 6] {
        [
            Self::new(self.q + 1, self.r),
            Self::new(self.q + 1, self.r - 1),
            Self::new(self.q, self.r - 1),
            Self::new(self.q - 1, self.r),
            Self::new(self.q - 1, self.r + 1),
            Self::new(self.q, self.r + 1),
        ]
    }

    pub fn neighbor_in_direction(self, direction: usize) -> Self {
        self.neighbors()[direction % 6]
    }

    pub fn direction_to(self, other: Self) -> Option<usize> {
        self.neighbors()
            .iter()
            .position(|neighbor| *neighbor == other)
    }

    pub fn is_on_side(self, side: usize, board_radius: i32) -> bool {
        match side % 6 {
            0 => self.q == board_radius,
            1 => self.r == board_radius,
            2 => self.q + self.r == -board_radius,
            3 => self.q == -board_radius,
            4 => self.r == -board_radius,
            _ => self.q + self.r == board_radius,
        }
    }
}

pub fn side_midpoint(side: usize, board_radius: i32) -> AxialCoord {
    let mid = board_radius / 2;
    match side % 6 {
        0 => AxialCoord::new(board_radius, -mid),
        1 => AxialCoord::new(-mid, board_radius),
        2 => AxialCoord::new(-mid, -mid),
        3 => AxialCoord::new(-board_radius, mid),
        4 => AxialCoord::new(mid, -board_radius),
        _ => AxialCoord::new(mid, mid),
    }
}

/// Number of on-board cells for a hexagon of the given radius: `3R^2 + 3R + 1`.
pub fn cell_count(board_radius: i32) -> usize {
    let r = board_radius as i64;
    (3 * r * r + 3 * r + 1) as usize
}

/// All on-board cells in a deterministic order (sorted by `(r, q)`).
///
/// This ordering is the canonical cell index used by the neural-network encoding,
/// so it must stay stable across the Rust self-play and Python training sides.
pub fn board_cells(board_radius: i32) -> Vec<AxialCoord> {
    let mut cells = Vec::with_capacity(cell_count(board_radius));
    for r in -board_radius..=board_radius {
        for q in -board_radius..=board_radius {
            let coord = AxialCoord::new(q, r);
            if coord.is_inside_board(board_radius) {
                cells.push(coord);
            }
        }
    }
    cells
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_count_matches_enumeration() {
        for radius in 1..=6 {
            assert_eq!(cell_count(radius), board_cells(radius).len());
        }
    }

    #[test]
    fn known_cell_counts() {
        assert_eq!(cell_count(3), 37);
        assert_eq!(cell_count(4), 61);
    }
}
