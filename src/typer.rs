use std::thread::current;

use crate::glyphface::GlyphFace;

const CANVAS_WIDTH: i32 = 200;

#[derive(Clone, Debug, Copy)]
pub struct ArrayCoord {
    pub x: i32,
    pub y: i32,
}

impl ArrayCoord {
    pub fn to_index(&self) -> i32 {
        self.y * CANVAS_WIDTH + self.x
    }
    pub fn new(x: i32, y: i32) -> ArrayCoord {
        ArrayCoord {
            x, y
        }
    }
    pub fn set(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }
}

pub struct Typer {
    pub typemode: bool,
    pub current_start: ArrayCoord,
    pub head: ArrayCoord,
    gwidth: i32,
    gheight: i32,
    g: GlyphFace
}

impl Typer {
    pub fn new() -> Typer {
        Typer {
            typemode: false,
            current_start: ArrayCoord::new(0,0),
            head: ArrayCoord::new(0,0),
            gwidth: 16,
            gheight: 16,
            g: GlyphFace::new(0)
        }
    }
    pub fn place_head_and_start(&mut self, x: i32, y: i32) {
        self.head.set(x, y);
        self.current_start.set(x, y);
    }
    pub fn type_letter(&mut self, canvas: &mut Vec<u8>, charcode: u8, guitex: &Vec<u8>) {

        if charcode == 10 /*newline*/ {
            self.head.y += self.gheight;
            self.head.x = self.current_start.x;
        } else {
            let letter_buffer = self.letter_buffer(charcode, guitex);
            let start_index = self.head.to_index();

            for i in 0..self.gheight {
                let row_start_index = start_index + i * CANVAS_WIDTH;
                let letter_start_index = i * self.gwidth;
                for k in 0..self.gwidth {
                    let current_row_index = row_start_index + k;
                    let current_letter_row_index = letter_start_index + k;

                    canvas[current_row_index as usize] = letter_buffer[current_letter_row_index as usize];
                }
            }

            self.head.x += self.gwidth;
        }
    }
    fn letter_buffer(&mut self, charcode: u8, guitex: &Vec<u8>) -> Vec<u8> {
        let mut vec = Vec::new();

        self.g.set_char(charcode);
        

        let x_start = (self.g.tlx * 544.0) as i32;
        let y_start = (self.g.tly * 544.0) as i32;

        for i in 0..self.gheight {
            let row_start = x_start + y_start * 544;

            for k in 0..self.gwidth {
                let index = row_start + k;
                vec.push(guitex[index as usize]);
            }
        }
        vec
    }
}