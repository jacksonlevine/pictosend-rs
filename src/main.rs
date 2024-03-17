use glfw::{Action, Context, Key};

mod glsetup;
use glsetup::GlSetup;

mod penstate;
use history::ChatHistory;
use penstate::{PenState, PenType};

mod fixtures;
use fixtures::{Fixture, Fixtures};

mod textureface;

mod network;
use network::Connection;

use std::net::Shutdown;
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};
use bincode::serialized_size;
use std::io::{Read, Write};

mod history;
struct MousePos {
    x: i32, 
    y: i32,
    clicked: bool,
    button: glfw::MouseButton
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct TextureData {
    name: [u8; 24],
    data: Vec<u8>,
    request_history: bool,
    request_history_length: bool,
    history_length: i32,
    confirm_history: bool
}

impl TextureData {
    fn new(myname: &String) -> Self {
        let bytes = myname.as_bytes();
        let mut fixed_size_text = [0u8; 24];
        fixed_size_text[..bytes.len()].copy_from_slice(bytes);

        TextureData {
            name: fixed_size_text,
            data: [0; 200 * 200].to_vec(),
            request_history: false,
            request_history_length: false,
            history_length: 0,
            confirm_history: false
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

    let myname = String::from("Test name");
    let mut gotHistoryLength = false;
    let mut gotHistory = false;
    
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
    window.set_scroll_polling(true);
    window.make_current();

    let mut gl_setup = GlSetup::new();
    let draw_pixels = Arc::new(Mutex::new(TextureData::new(&myname)));
    let mut fixtures = Fixtures::new().unwrap();

    let serialized_size = serialized_size(&(*(draw_pixels.lock().unwrap()))).unwrap();
    println!("Serialized size of TextureData: {} bytes", serialized_size);

    let history = Arc::new(Mutex::new(ChatHistory::new()));

    let connection = Arc::new(Mutex::new(Connection::new()));

    let test_func = Box::new(|| {
        println!("Test!");
    });


    let send_func: Box<dyn Fn()> = {
        let connection = Arc::clone(&connection);
        let draw_pixels = Arc::clone(&draw_pixels);
        Box::new(move || {
            let mut connection = connection.lock().unwrap();
            let mut draw_pixels = draw_pixels.lock().unwrap();
            connection.send(&draw_pixels);
            (*draw_pixels).data.fill(0);
            println!("Sending");
        })
    };

    let jump_to_present_func: Box<dyn Fn()> = {
        let history = Arc::clone(&history);
        Box::new(move || {
            let mut history = history.lock().unwrap();
            history.scroll_offset = 0.0;
        })
    };

    fixtures.set_fixtures(vec![
        Fixture {x:-1.0, y: -1.0, width: 0.2, height: 0.1, tooltip: String::from("Clear Drawing"), texx: 6, texy: 0, func: test_func.clone()},
        Fixture {x:-0.8, y: -1.0, width: 0.2, height: 0.1, tooltip: String::from("Brightnesss Down"), texx: 5, texy: 0, func: test_func.clone()},
        Fixture {x:-0.6, y: -1.0, width: 0.2, height: 0.1, tooltip: String::from("Brightnesss Up"), texx: 4, texy: 0, func: test_func.clone()},
        Fixture {x:-0.4, y: -1.0, width: 0.2, height: 0.1, tooltip: String::from("Toggle Camera"), texx: 3, texy: 0, func: test_func.clone()},
        Fixture {x:-0.2, y: -1.0, width: 0.2, height: 0.1, tooltip: String::from("Send Drawing"), texx: 1, texy: 0, func: send_func},
        Fixture {x:0.8, y: 0.0, width: 0.2, height: 0.1, tooltip: String::from("Scroll To Present"), texx: 7, texy: 0, func: jump_to_present_func},
    ]);


    while !window.should_close() {
        glfw.poll_events();
        mouse.update_pos(&mut window);

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            history.lock().unwrap().draw(width, height, gl_setup.scroll_shader, &myname);
            gl_setup.update_texture(&draw_pixels.lock().unwrap().data);
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

        if !gotHistoryLength || !gotHistory {

            let mut locked_conn = connection.lock().unwrap();

            let mut cloned_stream = {
                let locked_stream = locked_conn.stream.lock().unwrap();
                locked_stream.try_clone().unwrap() // Clone the TcpStream
            };
            drop(locked_conn); // Explicitly drop the lock

            cloned_stream.set_nonblocking(false).unwrap();
            
            let mut locked_conn = connection.lock().unwrap();
            (*locked_conn).request_history_length(&myname, &mut cloned_stream);
            drop(locked_conn);

            let mut buffer = [0; crate::network::PACKET_SIZE];

            match (cloned_stream).read_exact(&mut buffer) {
                Ok(_) => {
                    let received_data: TextureData = bincode::deserialize(&buffer).unwrap();
                    let history_size = received_data.history_length;
                    gotHistoryLength = true;

                    let mut locked_conn = connection.lock().unwrap();
                    (*locked_conn).request_history(&myname, &mut cloned_stream);
                    drop(locked_conn);

                    let mut history_buffer = vec![0; history_size as usize];

                    match (cloned_stream).read_exact(&mut history_buffer) {
                        Ok(_) => {
                            let history_vec: Vec<TextureData> = bincode::deserialize(&history_buffer).unwrap();
                            gotHistory = true;
                            let mut his = history.lock().unwrap();
                            his.history = history_vec;
                            his.dirty = true;
                            
                            let mut locked_conn = connection.lock().unwrap();
                            (*locked_conn).confirm_history(&myname, &mut cloned_stream);
                            drop(locked_conn);
                        }
                        Err(e) => {
                            println!("Failed to read from server: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("Failed to read from server: {}", e);
                }
            }

            (cloned_stream).set_nonblocking(true).unwrap();


        } else {
            connection.lock().unwrap().receive(&history);
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
                        if fixtures.clicked_on_id != 0.0 {
                            (fixtures.fixtures[(fixtures.clicked_on_id - 1.0) as usize].func)();
                        }
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
                },
                glfw::WindowEvent::Scroll(_, yoff) => {
                    if yoff > 0.0 {
                        history.lock().unwrap().scroll_offset -= 0.075;
                    }
                    if yoff < 0.0 {
                        if history.lock().unwrap().scroll_offset < 0.0 {
                            history.lock().unwrap().scroll_offset += 0.075;
                        }
                    }
                }
                _ => {}
            }
        }

        if mouse.clicked {
            match mouse.button {
                glfw::MouseButtonLeft => {
                    if fixtures.moused_over_id == 0.0 {
                        draw_pixels.lock().unwrap().draw(&mouse, &penstate, width, height, 254);
                    }
                },
                glfw::MouseButtonRight => {
                    draw_pixels.lock().unwrap().draw(&mouse, &penstate, width, height, 0);
                },
                _ => ()
            }
        }

        window.swap_buffers();
    }
    connection.lock().unwrap().stream.lock().unwrap().shutdown(Shutdown::Both).unwrap();
}
