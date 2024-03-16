static ONE_PIXEL: f32 = 0.0034722222222222222222;
static TEXTURE_WIDTH: f32 = 0.0555555555555555555555;
static ONE_OVER_16: f32 = 0.0625;

pub struct TextureFace {
    pub tlx: f32,
    pub tly: f32, 
    pub blx: f32,
    pub bly: f32,
    pub brx: f32,
    pub bry: f32,
    pub trx: f32,
    pub tr_y: f32
}

impl TextureFace {
    pub fn new(x: i8, y: i8) -> TextureFace {
        TextureFace {
            tlx: 0.0 + ONE_PIXEL + (ONE_OVER_16 * x as f32),
            tly: 1.0 - (y as f32 * ONE_OVER_16) - TEXTURE_WIDTH - ONE_PIXEL,
            blx: 0.0 + ONE_PIXEL + (ONE_OVER_16 * x as f32),
            bly: 1.0 - (y as f32 * ONE_OVER_16) - ONE_PIXEL,
            brx: 0.0 + ONE_PIXEL + (ONE_OVER_16 * x as f32) + TEXTURE_WIDTH,
            bry: 1.0 - (y as f32 * ONE_OVER_16) - ONE_PIXEL,
            trx: 0.0 + ONE_PIXEL + (ONE_OVER_16 * x as f32) + TEXTURE_WIDTH,
            tr_y: 1.0 - (y as f32 * ONE_OVER_16) - TEXTURE_WIDTH - ONE_PIXEL
        }
    }
}