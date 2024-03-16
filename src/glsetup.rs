use gl;
use std::fs::File;
use std::io::Read;
use std::ptr;
use std::str;

pub struct GlSetup {
    pub draw_shader: gl::types::GLuint,
    pub menu_shader: gl::types::GLuint,
    pub scroll_shader: gl::types::GLuint,
    pub vao: gl::types::GLuint,
    pub vbo: gl::types::GLuint,
    pub texture: gl::types::GLuint
}

impl GlSetup {
    pub fn new(window: &mut glfw::Window) -> GlSetup {
        let vertex_shader = compile_shader("assets/vert.glsl", gl::VERTEX_SHADER);
        let fragment_shader = compile_shader("assets/frag.glsl", gl::FRAGMENT_SHADER);
        let draw_shader = link_shader_program(vertex_shader, fragment_shader);

        unsafe {
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);
        }

        let vertex_shader = compile_shader("assets/menuVert.glsl", gl::VERTEX_SHADER);
        let fragment_shader = compile_shader("assets/menuFrag.glsl", gl::FRAGMENT_SHADER);
        let menu_shader = link_shader_program(vertex_shader, fragment_shader);

        unsafe {
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);
        }

        let vertex_shader = compile_shader("assets/scrollVert.glsl", gl::VERTEX_SHADER);
        let fragment_shader = compile_shader("assets/frag.glsl", gl::FRAGMENT_SHADER);
        let scroll_shader = link_shader_program(vertex_shader, fragment_shader);

        unsafe {
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);
        }

        let mut vao: gl::types::GLuint = 0;
        let mut vbo: gl::types::GLuint = 0;
        let mut tex: gl::types::GLuint = 0;

        unsafe {
            gl::GenVertexArrays(1, &mut vao); 
            gl::GenBuffers(1, &mut vbo);

            gl::GenTextures(1, &mut tex);
            gl::BindTexture(gl::TEXTURE_2D, tex);
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RED as i32, 200, 200, 0, gl::RED, gl::UNSIGNED_BYTE, std::ptr::null());
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
        }

        let quad_vertices: [f32; 24] = [
            // positions    // texture coords
            -1.0,  -1.0,  0.0, 1.0,
            -1.0, 0.0,  0.0, 0.0,
            1.0, 0.0,  1.0, 0.0,

            1.0,  0.0,  1.0, 0.0,
            1.0, -1.0,  1.0, 1.0,
            -1.0,  -1.0,  0.0, 1.0
        ];

        unsafe {
            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (quad_vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                quad_vertices.as_ptr() as *const gl::types::GLvoid,
                gl::STATIC_DRAW
            );
        }

        unsafe {
            let pos_attrib = gl::GetAttribLocation(draw_shader, b"pos\0".as_ptr() as *const gl::types::GLchar);
            gl::EnableVertexAttribArray(pos_attrib as gl::types::GLuint);
            gl::VertexAttribPointer(
                pos_attrib as gl::types::GLuint,
                2,
                gl::FLOAT,
                gl::FALSE,
                (4 * std::mem::size_of::<f32>()) as gl::types::GLint,
                std::ptr::null()
            );
        
            let tex_attrib = gl::GetAttribLocation(draw_shader, b"texcoord\0".as_ptr() as *const gl::types::GLchar);
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

        GlSetup {
            draw_shader,
            menu_shader,
            scroll_shader,
            vao,
            vbo,
            texture: tex,
        }
    }

    pub fn draw(&mut self) {
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::BindTexture(gl::TEXTURE_2D, self.texture);
            gl::UseProgram(self.draw_shader);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
        }
    }

    pub fn update_texture(&mut self, data: &[u8]) {
        unsafe {
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
                data.as_ptr() as *const gl::types::GLvoid
            );
        }
    }
}

fn compile_shader(path: &str, shader_type: gl::types::GLenum) -> gl::types::GLuint {
    let mut file = File::open(path).unwrap();
    let mut shader_source = String::new();
    file.read_to_string(&mut shader_source).unwrap();
    let shader_source_c_str = std::ffi::CString::new(shader_source.as_bytes()).unwrap();

    let shader = unsafe { gl::CreateShader(shader_type) };

    unsafe {
        gl::ShaderSource(shader, 1, &shader_source_c_str.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        let mut success = gl::FALSE as gl::types::GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE as gl::types::GLint {
            let mut log = vec![0; 512];
            gl::GetShaderInfoLog(shader, 512, ptr::null_mut(), log.as_mut_ptr() as *mut gl::types::GLchar);
            println!("ERROR::SHADER::COMPILATION_FAILED\n{}", str::from_utf8(&log).unwrap());
        }
    }

    shader
}

fn link_shader_program(vertex_shader: gl::types::GLuint, fragment_shader: gl::types::GLuint) -> gl::types::GLuint {
    let program = unsafe { gl::CreateProgram() };
    unsafe {
        gl::AttachShader(program, vertex_shader);
        gl::AttachShader(program, fragment_shader);
        gl::LinkProgram(program);

        let mut success = gl::FALSE as gl::types::GLint;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
        if success != gl::TRUE as gl::types::GLint {
            let mut log = vec![0; 512];
            gl::GetProgramInfoLog(program, 512, ptr::null_mut(), log.as_mut_ptr() as *mut gl::types::GLchar);
            println!("ERROR::PROGRAM::LINKING_FAILED\n{}", str::from_utf8(&log).unwrap());
        }
    }

    program
}