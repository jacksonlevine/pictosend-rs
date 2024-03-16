pub struct Fixture {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub tooltip: String,
    pub texX: i8,
    pub texY: i8
}

impl Fixture {
    pub fn new(x: f32, y: f32, width: f32, height: f32, tooltip: String, texX: i8, texY: i8) -> Fixture {
        Fixture {
            x, y, width, height, tooltip, texX, texY
        }
    }
}

pub struct Fixtures {
    pub fixtures: Vec<Fixture>,
    pub dirty: bool,
    pub vbo: gl::types::GLuint
}

impl Fixtures {
    pub fn rebuild_geometry(&self) -> Vec<f32> {
        let mut data = Vec::new();
        for (index, fix) in self.fixtures.iter().enumerate() {
            data.extend_from_slice(&[
                fix.x,           fix.y,            0.0, 0.0, index as f32 + 1.0,
                fix.x,           fix.y+fix.height, 0.0, 1.0, index as f32 + 1.0,
                fix.x+fix.width, fix.y+fix.height, 1.0, 1.0, index as f32 + 1.0,

                fix.x+fix.width, fix.y+fix.height, 1.0, 1.0, index as f32 + 1.0,
                fix.x+fix.width, fix.y,            1.0, 0.0, index as f32 + 1.0,
                fix.x,           fix.y,            0.0, 0.0, index as f32 + 1.0,
            ]);
        }
        data
    }
}