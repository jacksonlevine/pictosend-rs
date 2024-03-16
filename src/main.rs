use glfw::{Action, Context, Key};

mod glsetup;
use glsetup::GlSetup;

mod penstate;
use penstate::{PenState, PenType};

mod fixtures;
use fixtures::{Fixture, Fixtures};

mod textureface;
struct MousePos {
    x: i32, 
    y: i32,
    clicked: bool,
    button: glfw::MouseButton
}

struct TextureData {
    data: Vec<u8>
}

impl TextureData {
    fn new(width: i32, height: i32) -> Self {
        TextureData {
            data: vec![0; (width * height) as usize]
        }
    }

    fn draw(&mut self, mouse: &MousePos, pen: &PenState, width: i32, height: i32, value: u8) {
        let adjusted_m_y = (mouse.y - (height / 2)).max(0);

        let m_x_dist = (mouse.x as f32 / width as f32).clamp(0.0, 1.0);
        let m_y_dist = (adjusted_m_y as f32 / (height / 2) as f32).clamp(0.0, 1.0);

        let d_x = (m_x_dist * 200.0) as i32;
        let d_y = (m_y_dist * 200.0) as i32;

        let max = self.data.len() - 1;

        let d_center = (d_y * 200 + d_x).clamp(0, max as i32);

        for o in pen.pentype.get_spots() {
            let d_index = d_center + o.x as i32 + (o.y as i32 * 200);
            self.data[d_index.clamp(0, max as i32) as usize] = value;
        }
    }
}

impl MousePos {
    pub fn new() -> MousePos {
        MousePos {
            x: 0, y: 0, clicked: false, button: glfw::MouseButtonLeft
        }
    }
    fn update_pos(&mut self, window: &mut glfw::Window) {
        let (xpos, ypos) = window.get_cursor_pos();
        self.x = xpos as i32;
        self.y = ypos as i32;
    }
}

fn main() {
    
    let mut mouse = MousePos::new();
    let penstate = PenState::new(PenType::ThinPen);

    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();

    let (mut window, events) = glfw.create_window(400, 800, "PictoSend RS", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");
    
    gl::load_with(|s| window.get_proc_address(s) as *const _);

    let mut width = 400;
    let mut height = 800;

    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);
    window.set_mouse_button_polling(true);
    window.make_current();

    let mut gl_setup = GlSetup::new();
    let mut draw_pixels = TextureData::new(200, 200);
    let mut fixtures = Fixtures::new().unwrap();

    fixtures.set_fixtures(vec![
        Fixture {x:-1.0, y: -1.0, width: 0.2, height: 0.1, tooltip: String::from("Clear Drawing"), texx: 6, texy: 0},
        Fixture {x:-0.8, y: -1.0, width: 0.2, height: 0.1, tooltip: String::from("Brightnesss Down"), texx: 5, texy: 0},
        Fixture {x:-0.6, y: -1.0, width: 0.2, height: 0.1, tooltip: String::from("Brightnesss Up"), texx: 4, texy: 0},
        Fixture {x:-0.4, y: -1.0, width: 0.2, height: 0.1, tooltip: String::from("Toggle Camera"), texx: 3, texy: 0},
        Fixture {x:-0.2, y: -1.0, width: 0.2, height: 0.1, tooltip: String::from("Send Drawing"), texx: 1, texy: 0},
        Fixture {x:0.8, y: 0.0, width: 0.2, height: 0.1, tooltip: String::from("Scroll To Present"), texx: 7, texy: 0},
    ]);

    while !window.should_close() {
        glfw.poll_events();
        mouse.update_pos(&mut window);

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl_setup.update_texture(&draw_pixels.data);
            gl_setup.draw();
            fixtures.draw(gl_setup.menu_shader);
        }
        fixtures.get_moused_over(&mouse, width, height);
        unsafe {
            let moe_location = gl::GetUniformLocation(gl_setup.menu_shader, b"mousedOverElement\0".as_ptr() as *const i8);
            gl::Uniform1f(moe_location, fixtures.moused_over_id);
            let coe_location = gl::GetUniformLocation(gl_setup.menu_shader, b"clickedOnElement\0".as_ptr() as *const i8);
            gl::Uniform1f(coe_location, fixtures.clicked_on_id); 
        }
        
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true)
                },
                glfw::WindowEvent::MouseButton(mousebutton, action, _) => {
                    mouse.clicked = action == Action::Press;
                    mouse.button = mousebutton;
                    if action == Action::Release {
                        fixtures.clicked_on_id = 0.0;
                    } else if action == Action::Press {
                        fixtures.clicked_on_id = fixtures.moused_over_id;
                    }
                },
                glfw::WindowEvent::FramebufferSize(wid, hei) => {
                    width = wid;
                    height = hei;
                    unsafe {
                        gl::Viewport(0, 0, wid, hei);
                    }
                }
                _ => {}
            }
        }

        if mouse.clicked {
            match mouse.button {
                glfw::MouseButtonLeft => {
                    if fixtures.moused_over_id == 0.0 {
                        draw_pixels.draw(&mouse, &penstate, width, height, 254);
                    }
                },
                glfw::MouseButtonRight => {
                    draw_pixels.draw(&mouse, &penstate, width, height, 0);
                },
                _ => ()
            }
        }

        window.swap_buffers();
    }
}
