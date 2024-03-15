use glfw::{Action, Context, Key};

mod glsetup;
use glsetup::GlSetup;

mod penstate;
use penstate::{PenState, PenType};

struct MousePos {
    x: i16, 
    y: i16,
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
        let m_x_dist = (mouse.x as f32 / width as f32).clamp(0.0, 1.0);
        let m_y_dist = (mouse.y as f32 / height as f32).clamp(0.0, 1.0);

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
        self.x = xpos as i16;
        self.y = ypos as i16;
    }
}

fn main() {
    
    let mut mouse = MousePos::new();
    let mut penstate = PenState::new(PenType::ThinPen);

    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();

    let (mut window, events) = glfw.create_window(800, 600, "Hello this is window", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    let mut width = 800;
    let mut height = 600;

    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);
    window.set_mouse_button_polling(true);
    window.make_current();

    let mut gl_setup = GlSetup::new(&mut window);
    let mut draw_pixels = TextureData::new(200, 200);

    while !window.should_close() {
        glfw.poll_events();
        mouse.update_pos(&mut window);

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl_setup.update_texture(&draw_pixels.data);
            gl_setup.draw();
        }
        
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true)
                },
                glfw::WindowEvent::MouseButton(mousebutton, action, _) => {
                    if let Action::Press | Action::Release = action {
                        mouse.clicked = action == Action::Press;
                    }
                    mouse.button = mousebutton;
                },
                glfw::WindowEvent::FramebufferSize(wid, hei) => {
                    width = wid;
                    height = hei;
                    println!("New vp: {width}, {height}");
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
                    draw_pixels.draw(&mouse, &penstate, width, height, 254);
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
