use image::GenericImageView;
use crate::textureface::TextureFace;

pub struct Fixture {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub tooltip: String,
    pub texx: i8,
    pub texy: i8
}

impl Fixture {
    pub fn new(x: f32, y: f32, width: f32, height: f32, tooltip: String, texx: i8, texy: i8) -> Fixture {
        Fixture {
            x, y, width, height, tooltip, texx, texy
        }
    }
}

pub struct Fixtures {
    pub fixtures: Vec<Fixture>,
    pub dirty: bool,
    pub vbo: gl::types::GLuint,
    pub vao: gl::types::GLuint,
    pub data: Vec<f32>,
    pub texture: gl::types::GLuint
}

impl Fixtures {
    pub fn new() -> Result<Fixtures, String> {
        let mut vao: gl::types::GLuint = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
        }
        let mut texture: gl::types::GLuint = 0;
        let img = match image::open("assets/gui.png") {
            Ok(img) => img,
            Err(e) => return Err(format!("Failed to load texture: {}", e))
        };
        let (width, height) = img.dimensions();
        unsafe {
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D, texture);
    
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                width as i32,
                height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                img.to_rgba8().as_flat_samples().as_slice().as_ptr() as *const gl::types::GLvoid,
            );
    
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
    
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
        Ok(
            Fixtures {
                fixtures: Vec::new(),
                dirty: true,
                vbo: 0,
                vao,
                data: Vec::new(),
                texture
            }
        )
    }

    pub fn set_fixtures(&mut self, fixs: Vec<Fixture>) {
        self.fixtures = fixs;
        self.dirty = true;
    }

    fn rebuild_geometry(&mut self) {
        let mut data: Vec<f32> = Vec::new();
        for (index, fix) in self.fixtures.iter().enumerate() {
            let t = TextureFace::new(fix.texx, fix.texy);
            data.extend_from_slice(&[
                fix.x,           fix.y,            t.blx, t.bly,  index as f32 + 1.0,
                fix.x,           fix.y+fix.height, t.tlx, t.tly,  index as f32 + 1.0,
                fix.x+fix.width, fix.y+fix.height, t.trx, t.tr_y, index as f32 + 1.0,

                fix.x+fix.width, fix.y+fix.height, t.trx, t.tr_y, index as f32 + 1.0,
                fix.x+fix.width, fix.y,            t.brx, t.bry,  index as f32 + 1.0,
                fix.x,           fix.y,            t.blx, t.bly,  index as f32 + 1.0,
            ]);
        }
        self.data = data;
    }

    fn bind_geometry(&self, upload: bool, shader: gl::types::GLuint) {

        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            if upload {
                gl::BufferData(
                    gl::ARRAY_BUFFER, 
                    (self.data.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr, 
                    self.data.as_ptr() as *const gl::types::GLvoid,
                    gl::STATIC_DRAW);
            }
            let pos_attrib = gl::GetAttribLocation(
                shader, 
                b"pos\0".as_ptr() as *const i8) as gl::types::GLuint;
            gl::EnableVertexAttribArray(
                pos_attrib);
            gl::VertexAttribPointer(
                pos_attrib, 
                2,
                gl::FLOAT,
                gl::FALSE,
                (5 * std::mem::size_of::<f32>()) as gl::types::GLint,
                std::ptr::null());
        
            let tex_attrib = gl::GetAttribLocation(
                shader, 
                b"texcoord\0".as_ptr() as *const i8) as gl::types::GLuint;
            gl::EnableVertexAttribArray(
                tex_attrib);
            gl::VertexAttribPointer(tex_attrib,
                 2, 
                 gl::FLOAT, 
                 gl::FALSE, 
                 (5 * std::mem::size_of::<f32>()) as gl::types::GLint, 
                 (2 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid);
        
            let element_id_attrib = gl::GetAttribLocation(
                shader, 
                b"elementid\0".as_ptr() as *const i8) as gl::types::GLuint;

            gl::EnableVertexAttribArray(element_id_attrib);
            gl::VertexAttribPointer(
                element_id_attrib, 
                1, 
                gl::FLOAT, 
                gl::FALSE, 
                (5 * std::mem::size_of::<f32>()) as gl::types::GLint, 
                (4 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid);
        }
    }

    pub fn draw(&mut self, shader: gl::types::GLuint) {
        unsafe {
            gl::BindVertexArray(self.vao);
            if self.dirty {
                
                    gl::DeleteBuffers(1, &self.vbo);
                    gl::GenBuffers(1, &mut self.vbo);
                    self.rebuild_geometry();
                    self.bind_geometry(true, shader);
                self.dirty = false;
            } else {
                self.bind_geometry(false, shader);
            }
            gl::BindTexture(gl::TEXTURE_2D, self.texture);
            gl::UseProgram(shader);
            gl::DrawArrays(gl::TRIANGLES, 0, (self.fixtures.len() * 6) as i32);
        }
    }
}