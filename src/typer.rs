use crate::glyphface::GlyphFace;

const CANVAS_WIDTH: u16 = 200;

#[derive(Clone, Debug, Copy)]
pub struct ArrayCoord {
    pub x: u16,
    pub y: u16,
}

impl ArrayCoord {
    pub fn to_index(&self) -> u16 {
        (self.y * CANVAS_WIDTH + self.x).clamp(0, CANVAS_WIDTH*CANVAS_WIDTH - 1)
    }
    pub fn new(x: u16, y: u16) -> ArrayCoord {
        ArrayCoord {
            x, y
        }
    }
    pub fn set(&mut self, x: u16, y: u16) {
        self.x = x;
        self.y = y;
    }
}

pub struct Typer {
    pub typemode: bool,
    pub started: bool,
    pub current_start: ArrayCoord,
    pub head: ArrayCoord,
    gwidth: u16,
    gheight: u16,
    g: GlyphFace
}

impl Typer {
    pub fn new() -> Typer {
        Typer {
            typemode: false,
            started: false,
            current_start: ArrayCoord::new(0,0),
            head: ArrayCoord::new(0,0),
            gwidth: 16,
            gheight: 16,
            g: GlyphFace::new(0)
        }
    }
    pub fn place_head_and_start(&mut self, x: u16, y: u16) {
        self.head.set(x , y);
        self.current_start.set(x , y);
        self.started = true;
    }
    pub fn type_letter(&mut self, canvas: &mut Vec<u8>, charcode: u8, guitex: &Vec<u8>) {

        if charcode == 10 /*newline*/ {
            self.head.y += self.gheight;
            self.head.x = self.current_start.x;
        } else {
            if self.head.x >= CANVAS_WIDTH - self.gwidth {
                self.type_letter(canvas, 10, guitex);
            }
            let letter_buffer = self.letter_buffer(charcode, guitex);
            let start_index = self.head.to_index();

            for i in 0..self.gheight {
                let row_start_index = start_index + i * CANVAS_WIDTH;
                let letter_start_index = i * self.gwidth;
                for k in 0..self.gwidth {
                    let current_row_index = (row_start_index + k).clamp(0, CANVAS_WIDTH*CANVAS_WIDTH - 1);
                    let current_letter_row_index = letter_start_index + k;

                    canvas[current_row_index as usize] = letter_buffer[current_letter_row_index as usize];
                }
            }

            self.head.x += self.gwidth;
        }
    }
    pub fn backspace(&mut self, canvas: &mut Vec<u8>) {
            if self.head.x <= self.current_start.x {
                if self.head.y >= self.gheight && self.head.y > self.current_start.y {
                    self.head.y -= self.gheight;
                }
                let remainder = self.current_start.x % self.gwidth;

                self.head.x = (CANVAS_WIDTH - ((self.gwidth + (self.gwidth/2)) - remainder)); //- xadjust;
            } else {
                self.head.x -= self.gwidth;
            }

            
            let start_index = self.head.to_index();

            for i in 0..self.gheight {
                let row_start_index = start_index + i * CANVAS_WIDTH;
                for k in 0..self.gwidth {
                    let current_row_index = (row_start_index + k).clamp(0, CANVAS_WIDTH*CANVAS_WIDTH - 1);

                    canvas[current_row_index as usize] = 127;
                }
            }
    }
    fn letter_buffer(&mut self, charcode: u8, guitex: &Vec<u8>) -> Vec<u8> {
        let mut vec = Vec::new();

        self.g.set_char(charcode);
        

        let x_start = (self.g.tlx * 544.0) as u16;
        let y_start = (self.g.tly * 544.0) as u16;

        for i in 0..self.gheight {
            let row_start = x_start + (y_start + i) * 544;

            for k in 0..self.gwidth {
                let index = row_start + k;
                vec.push(guitex[index as usize]);
            }
        }
        vec
    }
}