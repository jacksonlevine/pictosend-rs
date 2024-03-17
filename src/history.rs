use serde::{Deserialize, Serialize};
use regex::Regex;
use crate::{glyphface::GlyphFace, TextureData};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatHistory {
    pub history: Vec<TextureData>,
    pub vbo: gl::types::GLuint,
    pub vao: gl::types::GLuint,
    pub display_data: Vec<f32>,
    pub dirty: bool,
    pub scroll_offset: f32,
    pub texture: gl::types::GLuint,
    pub name_starts: Vec<f32>,
    pub name_geometry: Vec<f32>,
    pub name_vbo: gl::types::GLuint,
    pub name_dirty: bool
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
            texture,
            name_starts: Vec::new(),
            name_geometry: Vec::new(),
            name_vbo: 0,
            name_dirty: false
        }
    }

    fn bind_scroll_geometry(&self, vbo: gl::types::GLuint, upload: bool, shader: gl::types::GLuint, data: &Vec<f32>) {
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            if upload  {
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (data.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                    data.as_ptr() as *const gl::types::GLvoid,
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
                self.name_dirty = true;
                self.display_data.clear();
                self.name_starts.clear();
                gl::DeleteBuffers(1, &self.vbo);
                gl::GenBuffers(1, &mut self.vbo);

                let wid = 250.0 / windowwidth as f32;
                let hei = 500.0 / windowheight as f32;

                let space = 550.0 / windowheight as f32;

                let bytes = myname.as_bytes();
                let mut fixed_size_text = [0u8; 24];
                fixed_size_text[..bytes.len()].copy_from_slice(bytes);
                
                for i in 0..self.history.len() {
                    for v in 0..6 {
                        let vstart = v*4;
                        self.display_data.extend_from_slice(&[
                            (if self.history[i].name == fixed_size_text { 0.3 } else { -0.3 } ) + QUAD_VERTICES[vstart+0] * wid,
                            (QUAD_VERTICES[vstart+1] * hei) + 0.8 + ((self.history.len() - 1 - i) as f32 * space),
                            QUAD_VERTICES[vstart+2], 
                            QUAD_VERTICES[vstart+3],
                        ]);
                        if v == 0 {
                            self.name_starts.extend_from_slice(&[
                                (if self.history[i].name == fixed_size_text { 0.3 } else { -0.3 } ) + QUAD_VERTICES[vstart+0] * wid,
                                (QUAD_VERTICES[vstart+1] * hei) + 0.8 + ((self.history.len() - 1 - i) as f32 * space),
                            ]);
                        }
                    }
                }
                self.dirty = false;
                self.bind_scroll_geometry(self.vbo, true, shader, &self.display_data);
            } else {
                self.bind_scroll_geometry(self.vbo, false, shader, &self.display_data);
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

    pub fn draw_names(&mut self, windowwidth: i32, windowheight: i32, shader: gl::types::GLuint, texture: gl::types::GLuint) {
        let gwidth: f32 = 32.0/windowwidth as f32;
        let gheight: f32 = 32.0/windowheight as f32;

        static mut vbo: gl::types::GLuint = 0;
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::UseProgram(shader);
        }
        if self.name_dirty {
            self.name_geometry.clear();
            unsafe {
                gl::DeleteBuffers(1, &vbo);
                gl::GenBuffers(1, &mut vbo);
            }
            for i in 0..self.history.len() {
                let name = self.history[i].name;

                let mut namestring = String::from_utf8(name.to_vec()).unwrap();

                // Create a regex that matches only printable ASCII characters
                let re = Regex::new(r"[ -~]").unwrap();
                // Filter the string to only include characters that match the regex
                namestring = namestring.chars().filter(|c| re.is_match(&c.to_string())).collect();

                let letters_count = namestring.chars().count();

                let nbs = i * 2;
                let namex = self.name_starts[nbs+0];
                let namey = self.name_starts[nbs+1] - gheight;

                let mut g = GlyphFace::new(0);
                for l in 0..letters_count {
                    g.set_char(name[l]);
                    self.name_geometry.extend_from_slice(&[
                        l as f32 * gwidth + namex,          namey,            g.blx,g.bly,
                        l as f32 * gwidth + namex,          namey + gheight,  g.tlx,g.tly,
                        l as f32 * gwidth + namex + gwidth, namey + gheight,  g.trx, g.tr_y,

                        l as f32 * gwidth + namex + gwidth, namey + gheight,  g.trx, g.tr_y,
                        l as f32 * gwidth + namex + gwidth, namey,            g.brx, g.bry,
                        l as f32 * gwidth + namex,          namey,            g.blx,g.bly,
                    ]);
                }
            }
            unsafe {
                self.bind_scroll_geometry(vbo, true, shader, &self.name_geometry);
            }
            self.name_dirty = false;
        } else {
            unsafe {
                self.bind_scroll_geometry(vbo, false, shader, &self.name_geometry);
            }
        }

        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::DrawArrays(gl::TRIANGLES, 0, (self.name_geometry.len()/4) as i32);
        }
            
    }
}