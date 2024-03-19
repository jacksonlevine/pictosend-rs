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
use network::*;

use std::net::{Shutdown, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use serde::{Serialize, Deserialize};
use bincode::serialized_size;
use std::io::{Read, Write};
use std::time::{SystemTime, UNIX_EPOCH, Duration, Instant};
use crate::typer::Typer;
use crate::winflash::flash_window;
use std::sync::atomic::{AtomicBool, Ordering};

use lerp::Lerp;

mod history;
mod glyphface;
use std::io;

mod typer;

mod winflash;
struct MousePos {
    x: i32, 
    y: i32,
    lastx: i32,
    lasty: i32,
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
    confirm_history: bool,
    timestamp: u128
}

impl TextureData {
    fn new(myname: &String) -> Self {
        let bytes = myname.as_bytes();
        let mut fixed_size_text = [0u8; 24];
        fixed_size_text[..bytes.len()].copy_from_slice(bytes);
        let now = SystemTime::now();

        TextureData {
            name: fixed_size_text,
            data: [127; 200 * 200].to_vec(),
            request_history: false,
            request_history_length: false,
            history_length: 0,
            confirm_history: false,
            timestamp: now.duration_since(UNIX_EPOCH).unwrap().as_millis()
        }
    }

    fn draw(&mut self, mouse: &MousePos, pen: &PenState, width: i32, height: i32, value: u8) {
        let adjusted_m_y = (mouse.y - (height / 2)).max(0);

        let max = self.data.len() - 1;

        let lastadjusted_m_y = (mouse.lasty - (height / 2)).max(0);
 
        for i in 0..15 {

            let t: f32 = i as f32 / 15.0;

            let mut m_x_dist = (mouse.x as f32 / width as f32).clamp(0.0, 1.0);
            let mut m_y_dist = (adjusted_m_y as f32 / (height / 2) as f32).clamp(0.0, 1.0);

            let lastm_x_dist = (mouse.lastx as f32 / width as f32).clamp(0.0, 1.0);
            let lastm_y_dist = (lastadjusted_m_y as f32 / (height / 2) as f32).clamp(0.0, 1.0);

            m_x_dist = lastm_x_dist.lerp(m_x_dist, t);
            m_y_dist = lastm_y_dist.lerp(m_y_dist, t);

            let d_x = (m_x_dist * 200.0) as i32;
            let d_y = (m_y_dist * 200.0) as i32;

            let d_center = (d_y * 200 + d_x).clamp(0, max as i32);

            for o in pen.pentype.get_spots() {
                let d_index = d_center + o.x as i32 + (o.y as i32 * 200);
                self.data[d_index.clamp(0, max as i32) as usize] = value;
            }
        }
            

    }
}

impl MousePos {
    pub fn new() -> MousePos {
        MousePos {
            x: 0, y: 0, lastx: 0, lasty: 0, clicked: false, button: glfw::MouseButtonLeft
        }
    }
    fn update_pos(&mut self, window: &mut glfw::Window) {
        let (xpos, ypos) = window.get_cursor_pos();
        self.lastx = self.x;
        self.lasty = self.y;
        self.x = xpos as i32;
        self.y = ypos as i32;
    }
}

struct CameraStuff {
    camera: Option<escapi::Device>,
    camera_mode: bool,
    brightness: i16
}

impl CameraStuff {
    pub fn new() -> CameraStuff {
        CameraStuff {
            camera: None,
            camera_mode: false,
            brightness: 0
        }
    }

    pub fn toggle(&mut self) {
        if !self.camera_mode {
            self.camera = Some(escapi::init(0, 200, 200, 15).expect("Failed to initialize camera"));
            self.camera_mode = true;
        } else {
            self.camera = None;
            self.camera_mode = false;
        }
    }
}

struct FixtureSwap {
    tooltip: String,
    newtexx: i8,
    newtexy: i8
}

fn increase_contrast(image: &[u8], factor: f32) -> Vec<u8> {
    let min_value = *image.iter().min().unwrap() as f32;
    let max_value = *image.iter().max().unwrap() as f32;

    image
        .iter()
        .map(|&pixel| {
            let pixel = pixel as f32;
            let stretched = 255.0 * (pixel - min_value) / (max_value - min_value);
            let enhanced = 255.0 / (1.0 + ((255.0 - stretched) / stretched).powf(factor));
            enhanced as u8
        })
        .collect()
}


fn glfw_mouse_pos_to_canvas_pos(mouse: &MousePos, window: &glfw::Window) -> (u16, u16) {
    let (width, height) = window.get_size();

    let adjusted_m_y = (mouse.y - (height / 2)).max(0);

    let mut m_x_dist = (mouse.x as f32 / width as f32).clamp(0.0, 1.0);
    let mut m_y_dist = (adjusted_m_y as f32 / (height / 2) as f32).clamp(0.0, 1.0);

    let d_x = (m_x_dist * 200.0) as u16;
    let d_y = (m_y_dist * 200.0) as u16;

    return (d_x, d_y);
}


fn main() {
    let mut previous_time = Instant::now();
    let mut delta_time: f32 = 0.0;
    
    let mut cam_timer: f32 = 0.0;

    let cam = Arc::new(Mutex::new(CameraStuff::new()));

    let mut myname = String::new();

    println!("What would you like your name to be?");
    loop {
        io::stdin()
                .read_line(&mut myname)
                .expect("Failed to read line");

        if myname.len() > 24 {
            println!("Name cannot be longer than 24 characters. Please type a shorter name.");
        } else if myname.len() < 2 {
            println!("Name cannot be shorter than 2 characters. Please type a longer name.");
        } else {
            break;
        }
    }

    let mut serverip = String::new();
    println!("Please type the server IP in the format address:port");
    loop {
        io::stdin()
                .read_line(&mut serverip)
                .expect("Failed to read line");
        println!("Trying to connect to {serverip}");
        serverip = serverip.trim().to_string();
        break;
    }

    let mut gotHistoryLength = false;
    let mut gotHistory = false;

    let should_close = Arc::new(AtomicBool::new(false));
    
    let mut mouse = MousePos::new();
    let penstate = Arc::new(Mutex::new(PenState::new(PenType::ThinPen)));

    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();

    let (mut window, events) = glfw.create_window(400, 800, "PictoSend RS", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    let window_handle = window.get_win32_window() as winapi::shared::windef::HWND;
    
    gl::load_with(|s| window.get_proc_address(s) as *const _);

    let mut width = 400;
    let mut height = 800;

    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);
    window.set_mouse_button_polling(true);
    window.set_scroll_polling(true);
    window.set_focus_polling(true);
    window.set_char_polling(true);
    window.make_current();

    let mut gl_setup = GlSetup::new();
    let draw_pixels = Arc::new(Mutex::new(TextureData::new(&myname)));
    let cam_pixels: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(vec![0u8; 200*200]));
    let text_pixels: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(vec![127u8; 200*200]));
    let mut fixtures = Arc::new(Mutex::new(Fixtures::new().unwrap()));
    let mut typer = Arc::new(Mutex::new(Typer::new()));

    let mut fixtureswapqueue: Arc<Mutex<Vec<FixtureSwap>>> = Arc::new(Mutex::new(Vec::new()));

    //let serialized_size = serialized_size(&(*(draw_pixels.lock().unwrap()))).unwrap();
    //println!("Serialized size of TextureData: {} bytes", serialized_size);

    let history = Arc::new(Mutex::new(ChatHistory::new()));


    let recv_connection = TcpStream::connect(serverip).unwrap();
    let send_connection = recv_connection.try_clone().unwrap();

    let rconnection = Arc::new(Mutex::new(recv_connection));
    let sconnection = Arc::new(Mutex::new(send_connection));

    let send_func: Box<dyn Fn()> = {
        let connection = Arc::clone(&sconnection);


        let draw_pixels = Arc::clone(&draw_pixels);
        let cam_pixels = Arc::clone(&cam_pixels);
        let text_pixels = Arc::clone(&text_pixels);
        Box::new(move || {
            let mut draw_pixels = draw_pixels.lock().unwrap();
            let cam_pixels = cam_pixels.lock().unwrap();
            let mut text_pixels = text_pixels.lock().unwrap();

            let now = SystemTime::now();
            draw_pixels.timestamp = now.duration_since(UNIX_EPOCH).unwrap().as_millis();
            for i in 0..200*200 {
                if draw_pixels.data[i] == 127 as u8 {
                    draw_pixels.data[i] = cam_pixels[i];
                }
                if text_pixels[i] != 127 as u8 {
                    draw_pixels.data[i] = text_pixels[i];
                }
            }
            send(&draw_pixels, &connection);
            (*draw_pixels).data.fill(127);
            (*text_pixels).fill(127);
            println!("Sending");
        })
    };

    let clear_func: Box<dyn Fn()> = {
        let draw_pixels = Arc::clone(&draw_pixels);
        let text_pixels = Arc::clone(&text_pixels);
        Box::new(move || {
            let mut draw_pixels = draw_pixels.lock().unwrap();
            let mut text_pixels = text_pixels.lock().unwrap();
            (*draw_pixels).data.fill(127);
            (*text_pixels).fill(127);
        })
    };

    let jump_to_present_func: Box<dyn Fn()> = {
        let history = Arc::clone(&history);
        Box::new(move || {
            let mut history = history.lock().unwrap();
            history.scroll_offset = 0.0;
        })
    };

    let cam_func: Box<dyn Fn()> = {
        let cam = Arc::clone(&cam);
        let cam_pixels = Arc::clone(&cam_pixels);
        let fixswaps = Arc::clone(&fixtureswapqueue);
            
        Box::new(move || {
            let mut cam = cam.lock().unwrap();
            let mut cam_pixels = cam_pixels.lock().unwrap();
            let mut fixswaps = fixswaps.lock().unwrap();
            cam.toggle();

            let (newx, newy) = match cam.camera_mode {
                true => {
                    (14, 0)
                },
                false => {
                    (*cam_pixels).fill(0);
                    (3, 0)
                }
            };
            fixswaps.push(FixtureSwap{
                tooltip: String::from("Toggle Camera"), 
                newtexx: newx, 
                newtexy: newy});
        })
    };

    let brightdown_func: Box<dyn Fn()> = {
        let cam = Arc::clone(&cam);
        Box::new(move || {
            let mut cam = cam.lock().unwrap();
            cam.brightness -= 20;
        })
    };

    let brightup_func: Box<dyn Fn()> = {
        let cam = Arc::clone(&cam);
        Box::new(move || {
            let mut cam = cam.lock().unwrap();
            cam.brightness += 20;
        })
    };

    let swap_pens_func: Box<dyn Fn()> = {
        let pens = Arc::clone(&penstate);
        let fixswaps = Arc::clone(&fixtureswapqueue);
        Box::new(move || {
            let mut pens = pens.lock().unwrap();
            let mut fixswaps = fixswaps.lock().unwrap();
            pens.pentype = pens.pentype.next();
            let (newx, newy) = match pens.pentype {
                PenType::ThinPen => {
                    (8, 0)
                },
                PenType::FatPen => {
                    (9, 0)
                },
                PenType::HugePen => {
                    (10, 0)
                },
                PenType::TinyPen => {
                    (11, 0)
                }
            };
            fixswaps.push(FixtureSwap{
                tooltip: String::from("Swap Pen"), 
                newtexx: newx, 
                newtexy: newy});
        })
    };

    let toggle_text_func: Box<dyn Fn()> = {
        let typer = Arc::clone(&typer);
        let fixswaps = Arc::clone(&fixtureswapqueue);
        Box::new(move || {
            let mut fixswaps = fixswaps.lock().unwrap();
            let mut typer = typer.lock().unwrap();
            typer.typemode = !typer.typemode;
            let (newx, newy) = match typer.typemode {
                true => {
                    (13, 0)
                },
                false => {
                    typer.started = false;
                    (12, 0)
                }
            };
            fixswaps.push(FixtureSwap{
                tooltip: String::from("Text Mode"), 
                newtexx: newx, 
                newtexy: newy});
            drop(typer);
        })
    };

    fixtures.lock().unwrap().set_fixtures(vec![
        Fixture {x:-1.0, y: -1.0, width: 0.2, height: 0.1, tooltip: String::from("Clear Drawing"), texx: 6, texy: 0, func: clear_func},
        Fixture {x:-0.8, y: -1.0, width: 0.2, height: 0.1, tooltip: String::from("Brightnesss Down"), texx: 5, texy: 0, func: brightdown_func},
        Fixture {x:-0.6, y: -1.0, width: 0.2, height: 0.1, tooltip: String::from("Brightnesss Up"), texx: 4, texy: 0, func: brightup_func},
        Fixture {x:-0.4, y: -1.0, width: 0.2, height: 0.1, tooltip: String::from("Toggle Camera"), texx: 3, texy: 0, func: cam_func},
        Fixture {x:-0.2, y: -1.0, width: 0.2, height: 0.1, tooltip: String::from("Send Drawing"), texx: 1, texy: 0, func: send_func},
        Fixture {x:0.0, y: -1.0, width: 0.2, height: 0.1, tooltip: String::from("Text Mode"), texx: 12, texy: 0, func: toggle_text_func},
        Fixture {x:0.8, y: 0.0, width: 0.2, height: 0.1, tooltip: String::from("Scroll To Present"), texx: 7, texy: 0, func: jump_to_present_func},
        Fixture {x:0.8, y: -1.0, width: 0.2, height: 0.1, tooltip: String::from("Swap Pen"), texx: 8, texy: 0, func: swap_pens_func}
    ]);

    
    let mut recv_jh: Option<JoinHandle<()>> = None;


    while !window.should_close() {
        glfw.poll_events();
        mouse.update_pos(&mut window);

        let current_time = Instant::now();
        delta_time = current_time.duration_since(previous_time).as_secs_f32();
        previous_time = current_time;

        let mut lock_fixtures = fixtures.lock().unwrap();

        for fixswap in &(*(fixtureswapqueue.lock().unwrap())) {
            lock_fixtures.change_tex_coords(String::from(&fixswap.tooltip), fixswap.newtexx, fixswap.newtexy);
        }
        (fixtureswapqueue.lock().unwrap()).clear();

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            let was_dirty = history.lock().unwrap().draw(width, height, gl_setup.scroll_shader, &myname);
            if was_dirty {
                flash_window(window_handle);
            }
            history.lock().unwrap().draw_names(width, height, gl_setup.scroll_shader, lock_fixtures.texture);
            gl_setup.update_texture(&draw_pixels.lock().unwrap().data);
            gl_setup.update_cam_texture(&cam_pixels.lock().unwrap());
            gl_setup.update_text_texture(&text_pixels.lock().unwrap());
            gl_setup.draw();
            lock_fixtures.draw(gl_setup.menu_shader);
            lock_fixtures.draw_tooltip(&window, gl_setup.menu_shader);
        }
        lock_fixtures.get_moused_over(&mouse, width, height);
        unsafe {
            let moe_location = gl::GetUniformLocation(gl_setup.menu_shader, b"mousedOverElement\0".as_ptr() as *const i8);
            gl::Uniform1f(moe_location, lock_fixtures.moused_over_id);
            let coe_location = gl::GetUniformLocation(gl_setup.menu_shader, b"clickedOnElement\0".as_ptr() as *const i8);
            gl::Uniform1f(coe_location, lock_fixtures.clicked_on_id); 
        }
        drop(lock_fixtures);

        let lock_cam = cam.lock().unwrap();
        if lock_cam.camera_mode {
            if cam_timer > 0.25 {
                let frame = lock_cam.camera.as_ref().unwrap().capture().expect("Failed to cap frame");

                let extracted_data: Vec<u8> = frame
                    .chunks(4) // Group the data into chunks of 4 bytes (RGBA)
                    .map(|chunk| (255 as i32 - (chunk[0] as i32 + lock_cam.brightness as i32) as i32).clamp(0, 255) as u8) // Take the first byte from each chunk
                    .collect();

                let contrast_frame = increase_contrast(&extracted_data, 5.0);

                cam_pixels.lock().unwrap().clone_from_slice(&contrast_frame);
                
                cam_timer = 0.0;
            } else {
                cam_timer += delta_time;
            }
        }
        drop(lock_cam);
            
        if !gotHistoryLength || !gotHistory {

            let mut locked_conn = rconnection.lock().unwrap();

            request_history_length(&myname, &mut locked_conn);


            let mut buffer = [0; crate::network::PACKET_SIZE];

            match (locked_conn).read_exact(&mut buffer) {
                Ok(_) => {
                    let received_data: TextureData = bincode::deserialize(&buffer).unwrap();
                    let history_size = received_data.history_length;
                    gotHistoryLength = true;


                    request_history(&myname, &mut locked_conn);


                    let mut history_buffer = vec![0; history_size as usize];

                    match (locked_conn).read_exact(&mut history_buffer) {
                        Ok(_) => {
                            let history_vec: Vec<TextureData> = bincode::deserialize(&history_buffer).unwrap();
                            gotHistory = true;
                            let mut his = history.lock().unwrap();
                            his.history = history_vec;
                            his.dirty = true;
                            
                            confirm_history(&myname, &mut locked_conn);
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
            drop(locked_conn);
            //(cloned_stream).set_nonblocking(true).unwrap();

            let connection_clone = Arc::clone(&rconnection);
            let history_clone = Arc::clone(&history);
            let should_close_clone = Arc::clone(&should_close);

            recv_jh = Some(std::thread::spawn(move || {
                receive(&history_clone, &connection_clone, &should_close_clone);
            }));
        }


        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true)
                },
                glfw::WindowEvent::Key(Key::Backspace, _, Action::Press, _) => {
                    let mut typerlock = typer.lock().unwrap();

                    if typerlock.started {
                        typerlock.backspace(&mut text_pixels.lock().unwrap());
                    }

                    drop(typerlock);
                },
                glfw::WindowEvent::Key(Key::Enter, _, Action::Press, _) => {
                    let mut typerlock = typer.lock().unwrap();

                    if typerlock.started {
                        typerlock.type_letter(&mut text_pixels.lock().unwrap(), 10, &fixtures.lock().unwrap().guitexpixels);
                    }

                    drop(typerlock);
                },
                glfw::WindowEvent::Char(code) => {
                    
                    let mut typerlock = typer.lock().unwrap();
                    if typerlock.started {
                        typerlock.type_letter(&mut text_pixels.lock().unwrap(), code as u8, &fixtures.lock().unwrap().guitexpixels);
                    }
                    drop(typerlock);
                },
                glfw::WindowEvent::MouseButton(mousebutton, action, _) => {
                    mouse.clicked = action == Action::Press;
                    mouse.button = mousebutton;
                    let mut lock_fixtures = fixtures.lock().unwrap();
                    if action == Action::Release {
                        if lock_fixtures.clicked_on_id != 0.0 {
                            (lock_fixtures.fixtures[(lock_fixtures.clicked_on_id - 1.0) as usize].func)();
                        }
                        lock_fixtures.clicked_on_id = 0.0;
                    } else if action == Action::Press {
                        if lock_fixtures.moused_over_id != 0.0 {
                            lock_fixtures.clicked_on_id = lock_fixtures.moused_over_id;
                        } else {
                            let mut typerlock = typer.lock().unwrap();
                            if typerlock.typemode {
                                let (x, y) = glfw_mouse_pos_to_canvas_pos(&mouse, &window);
                                typerlock.place_head_and_start(x, y);
                            }
                            drop(typerlock);
                        }
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
                },
                glfw::WindowEvent::Focus(foc) => {
                    if foc {
                        history.lock().unwrap().dirty = true;
                    }
                }
                _ => {}
            }
        }

        if mouse.clicked {
            if !typer.lock().unwrap().typemode {
                match mouse.button {
                    glfw::MouseButtonLeft => {
                        let mut lock_fixtures = fixtures.lock().unwrap();
                        if lock_fixtures.moused_over_id == 0.0 {
                            draw_pixels.lock().unwrap().draw(&mouse, &penstate.lock().unwrap(), width, height, 254);
                        }
                    },
                    glfw::MouseButtonRight => {
                        draw_pixels.lock().unwrap().draw(&mouse, &penstate.lock().unwrap(), width, height, 0);
                    },
                    _ => ()
                }
            }
        }
        

        window.swap_buffers();
    }
    should_close.store(true, Ordering::Relaxed);
    recv_jh.unwrap().join().unwrap();

    rconnection.lock().unwrap().shutdown(Shutdown::Both).unwrap();
    sconnection.lock().unwrap().shutdown(Shutdown::Both).unwrap();
}
