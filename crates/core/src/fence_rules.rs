use crate::hex::AxialCoord;
use crate::state::EdgeKey;

/// A fence shape. The discriminant order here is load-bearing: it is the `shape`
/// axis of the neural-network policy index map and must stay in sync between the
/// Rust self-play side and the Python training side. See [`FenceShape::to_index`].
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum FenceShape {
    S,
    SMirrored,
    C,
    Y,
}

impl FenceShape {
    /// All shapes in canonical index order.
    pub const ALL: [FenceShape; 4] = [
        FenceShape::S,
        FenceShape::SMirrored,
        FenceShape::C,
        FenceShape::Y,
    ];

    pub fn next(self) -> Self {
        match self {
            Self::S => Self::SMirrored,
            Self::SMirrored => Self::C,
            Self::C => Self::Y,
            Self::Y => Self::S,
        }
    }

    /// Canonical index in `0..4` used by the policy head's fence block.
    pub fn to_index(self) -> usize {
        match self {
            Self::S => 0,
            Self::SMirrored => 1,
            Self::C => 2,
            Self::Y => 3,
        }
    }

    pub fn from_index(index: usize) -> Option<Self> {
        Self::ALL.get(index).copied()
    }
}

pub fn fence_edges(anchor: AxialCoord, shape: FenceShape, orientation: usize) -> [EdgeKey; 3] {
    let o = orientation % 6;
    let n0 = anchor.neighbor_in_direction(o);
    let n1 = anchor.neighbor_in_direction((o + 1) % 6);

    match shape {
        FenceShape::C => [
            EdgeKey::from_cells(anchor, n0),
            EdgeKey::from_cells(anchor, n1),
            EdgeKey::from_cells(anchor, anchor.neighbor_in_direction((o + 2) % 6)),
        ],
        // Three fence segments sharing one common hex-grid vertex.
        FenceShape::Y => [
            EdgeKey::from_cells(anchor, n0),
            EdgeKey::from_cells(anchor, n1),
            EdgeKey::from_cells(n0, n1),
        ],
        // Connected zig-zag path of three segments.
        FenceShape::S => {
            // Chain: (anchor-n0) -> (anchor-n1) -> (n1-next)
            // where each neighboring pair shares a fence endpoint.
            let next = n1.neighbor_in_direction((o + 3) % 6);
            [
                EdgeKey::from_cells(anchor, n0),
                EdgeKey::from_cells(anchor, n1),
                EdgeKey::from_cells(n1, next),
            ]
        }
        // Mirrored connected zig-zag path of three segments.
        FenceShape::SMirrored => {
            // Chain: (anchor-n0) -> (anchor-n1) -> (n0-next)
            // where each neighboring pair shares a fence endpoint.
            let next = n0.neighbor_in_direction((o + 4) % 6);
            [
                EdgeKey::from_cells(anchor, n0),
                EdgeKey::from_cells(anchor, n1),
                EdgeKey::from_cells(n0, next),
            ]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shape_index_roundtrips() {
        for (index, shape) in FenceShape::ALL.into_iter().enumerate() {
            assert_eq!(shape.to_index(), index);
            assert_eq!(FenceShape::from_index(index), Some(shape));
        }
        assert_eq!(FenceShape::from_index(4), None);
    }
}
