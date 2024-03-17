use serde::{Deserialize, Serialize};
use bincode::serialized_size;

use crate::TextureData;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatHistory {
    pub history: Vec<TextureData>,
    pub vbo: gl::types::GLuint,
    pub vao: gl::types::GLuint,
    pub display_data: Vec<f32>,
    pub dirty: bool,
    pub scroll_offset: f32,
    pub texture: gl::types::GLuint
}

impl ChatHistory {
    pub fn new() -> ChatHistory {
        let mut vao: gl::types::GLuint = 0;
        let mut texture: gl::types::GLuint = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RED as i32, 200, 200, 0, gl::RED, gl::UNSIGNED_BYTE, std::ptr::null());

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
        }

        ChatHistory {
            history: Vec::new(),
            vbo: 0,
            vao,
            display_data: Vec::new(),
            dirty: true,
            scroll_offset: 0.0,
            texture
        }
    }

    fn bind_scroll_geometry(&self, upload: bool, shader: gl::types::GLuint) {
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            if upload  {
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (self.display_data.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                    self.display_data.as_ptr() as *const gl::types::GLvoid,
                    gl::STATIC_DRAW
                );
            }   
        }

        unsafe {
            let pos_attrib = gl::GetAttribLocation(shader, b"pos\0".as_ptr() as *const gl::types::GLchar);
            gl::EnableVertexAttribArray(pos_attrib as gl::types::GLuint);
            gl::VertexAttribPointer(
                pos_attrib as gl::types::GLuint,
                2,
                gl::FLOAT,
                gl::FALSE,
                (4 * std::mem::size_of::<f32>()) as gl::types::GLint,
                std::ptr::null()
            );
        
            let tex_attrib = gl::GetAttribLocation(shader, b"texcoord\0".as_ptr() as *const gl::types::GLchar);
            gl::EnableVertexAttribArray(tex_attrib as gl::types::GLuint);
            gl::VertexAttribPointer(
                tex_attrib as gl::types::GLuint,
                2,
                gl::FLOAT,
                gl::FALSE,
                (4 * std::mem::size_of::<f32>()) as gl::types::GLint,
                (2 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid
            );
        }
    }

    pub fn draw(&mut self, windowwidth: i32, windowheight: i32, shader: gl::types::GLuint, myname: &String) {
        static QUAD_VERTICES: [f32; 24] = [
            // positions    // texture coords
            -1.0,  -1.0,  0.0, 1.0,
            -1.0, 0.0,    0.0, 0.0,
            1.0, 0.0,     1.0, 0.0,

            1.0,  0.0,    1.0, 0.0,
            1.0, -1.0,    1.0, 1.0,
            -1.0,  -1.0,  0.0, 1.0
        ];

        unsafe {
            gl::BindVertexArray(self.vao);
            gl::UseProgram(shader);
            if self.dirty {
                self.display_data.clear();
                gl::DeleteBuffers(1, &self.vbo);
                gl::GenBuffers(1, &mut self.vbo);

                let wid = 250.0 / windowwidth as f32;
                let hei = 500.0 / windowheight as f32;

                let space = 550.0 / windowheight as f32;
                
                for i in 0..self.history.len() {
                    for v in 0..6 {
                        let vstart = v*4;
                        self.display_data.extend_from_slice(&[
                            QUAD_VERTICES[vstart+0] * wid,
                            (QUAD_VERTICES[vstart+1] * hei) + 0.8 + ((self.history.len() - 1 - i) as f32 * space),
                            QUAD_VERTICES[vstart+2], 
                            QUAD_VERTICES[vstart+3],
                        ]);
                    }
                }
                self.dirty = false;
                self.bind_scroll_geometry(true, shader);
            } else {
                self.bind_scroll_geometry(false, shader);
            }

            let scroll_location = gl::GetUniformLocation(shader, b"scroll\0".as_ptr() as *const i8);
            gl::Uniform1f(scroll_location, self.scroll_offset);

            for i in 0..self.history.len() {
                let bs = i * 6;
                gl::BindTexture(gl::TEXTURE_2D, self.texture);
                gl::TexSubImage2D(
                    gl::TEXTURE_2D,
                    0,
                    0,
                    0,
                    200,
                    200,
                    gl::RED,
                    gl::UNSIGNED_BYTE,
                    self.history[i].data.as_ptr() as *const gl::types::GLvoid
                );
                gl::DrawArrays(gl::TRIANGLES, bs as i32, 6);
            }
        }
    }
}