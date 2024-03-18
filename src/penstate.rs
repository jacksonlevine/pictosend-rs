use crate::fixtures::Fixture;

pub struct OffsetSpot {
    pub x: i8,
    pub y: i8
}

pub struct PenState {
    pub pentype: PenType
}

pub enum PenType {
    HugePen,
    FatPen,
    ThinPen,
    TinyPen
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
            PenType::TinyPen => &[
                OffsetSpot{x: 0, y: 0},
            ],
            PenType::HugePen => & [
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

                
                OffsetSpot{x: -3, y: -2},
                OffsetSpot{x: -3, y: -1},
                OffsetSpot{x: -3, y: 0},
                OffsetSpot{x: -3, y: 1},
                OffsetSpot{x: -3, y: 2},

                OffsetSpot{x: 3, y: -2},
                OffsetSpot{x: 3, y: -1},
                OffsetSpot{x: 3, y: 0},
                OffsetSpot{x: 3, y: 1},
                OffsetSpot{x: 3, y: 2},

                OffsetSpot{x: -2, y: 3},
                OffsetSpot{x: -1, y: 3},
                OffsetSpot{x: 0, y: 3},
                OffsetSpot{x: 1, y: 3},
                OffsetSpot{x: 2, y: 3},

                OffsetSpot{x: -2, y: -3},
                OffsetSpot{x: -1, y: -3},
                OffsetSpot{x: 0, y: -3},
                OffsetSpot{x: 1, y: -3},
                OffsetSpot{x: 2, y: -3},

                OffsetSpot{x: -2, y: -2},
                OffsetSpot{x: 2, y: -2},
                OffsetSpot{x: 2, y: 2},
                OffsetSpot{x: -2, y: 2},
            ]
        }
    }

    pub fn next(&self) -> PenType {
        match self {
            PenType::HugePen => PenType::TinyPen,
            PenType::FatPen => PenType::HugePen,
            PenType::ThinPen => PenType::FatPen,
            PenType::TinyPen => PenType::ThinPen
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