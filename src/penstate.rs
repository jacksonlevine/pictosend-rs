pub struct OffsetSpot {
    pub x: i8,
    pub y: i8
}

pub struct PenState {
    pub pentype: PenType
}

pub enum PenType {
    FatPen,
    ThinPen,
}

impl PenType {
    pub fn get_spots(&self) -> &'static [OffsetSpot] {
        match self {
            PenType::FatPen => &[
                OffsetSpot{x: 0, y: 0},

                OffsetSpot{x: 1, y: 0},
                OffsetSpot{x: -1, y: 0},
                OffsetSpot{x: 0, y: 1},
                OffsetSpot{x: 0, y: -1},

                OffsetSpot{x: 1, y: 1},
                OffsetSpot{x: -1, y: 1},
                OffsetSpot{x: 1, y: -1},
                OffsetSpot{x: -1, y: -1},

                OffsetSpot{x: -1, y: 2},
                OffsetSpot{x: 0, y: 2},
                OffsetSpot{x: 1, y: 2},

                OffsetSpot{x: -1, y: -2},
                OffsetSpot{x: 0, y: -2},
                OffsetSpot{x: 1, y: -2},

                OffsetSpot{x: 2, y: -1},
                OffsetSpot{x: 2, y: 0},
                OffsetSpot{x: 2, y: 1},

                OffsetSpot{x: -2, y: -1},
                OffsetSpot{x: -2, y: 0},
                OffsetSpot{x: -2, y: 1},
            ],
            PenType::ThinPen => &[
                OffsetSpot{x: 0, y: 0},

                OffsetSpot{x: 1, y: 0},
                OffsetSpot{x: -1, y: 0},
                OffsetSpot{x: 0, y: 1},
                OffsetSpot{x: 0, y: -1},

                OffsetSpot{x: 1, y: 1},
                OffsetSpot{x: -1, y: 1},
                OffsetSpot{x: 1, y: -1},
                OffsetSpot{x: -1, y: -1},
            ],
        }
    }
}

impl PenState {
    pub fn new(pentype: PenType) -> PenState {
        PenState {
            pentype
        }
    }
}