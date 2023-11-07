extern crate gl;
extern crate glutin;
extern crate libc;

use std::io::Error;

use gb::{Button, GameBoy};

use std::ffi::CString;
use std::mem;
use std::str;

use gl::types::*;
use glutin::event::{ElementState, VirtualKeyCode};

#[inline]
pub fn load_our_game_rom() -> Result<Vec<u8>, Error> {
    use std::{fs::File, io::Read};
    let mut rom = Vec::new();
    let file = File::open("./rom/game.gb");
    file.and_then(|mut f| f.read_to_end(&mut rom))?;
    Ok(rom)
}

pub struct Glcx {
    #[allow(unused)]
    tex: GLuint,
    #[allow(unused)]
    program: GLuint,
    #[allow(unused)]
    frag: GLuint,
    #[allow(unused)]
    vert: GLuint,
    #[allow(unused)]
    ebo: GLuint,
    #[allow(unused)]
    vbo: GLuint,
    #[allow(unused)]
    vao: GLuint,
}

fn main() -> Result<(), Error> {
    let rom_data = load_our_game_rom()?;

    let gameboy = GameBoy::new(rom_data);

    let event_loop: glutin::event_loop::EventLoop<()> =
        glutin::event_loop::EventLoop::with_user_event();
    let inner_size = glutin::dpi::LogicalSize {
        width: gameboy.width(),
        height: gameboy.height(),
    };
    let window_builder = glutin::window::WindowBuilder::new()
        .with_title("GameBoy")
        .with_inner_size(inner_size)
        .with_resizable(true);
    let gl_window = glutin::ContextBuilder::new()
        .build_windowed(window_builder, &event_loop)
        .unwrap();
    let gl_window = unsafe { gl_window.make_current().unwrap() };

    gl::load_with(|s| gl_window.get_proc_address(s) as *const _);

    let cx = Glcx::new();
    event_loop.run(move |event, _, control_flow| {
        let window = gl_window.window();
        match event {
            glutin::event::Event::WindowEvent {
                window_id: _,
                event: wevent,
            } => match wevent {
                glutin::event::WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(virt_keycode) = input.virtual_keycode {
                        let button = match virt_keycode {
                            VirtualKeyCode::A => Button::A,
                            VirtualKeyCode::B => Button::B,
                            VirtualKeyCode::Z => Button::Select,
                            VirtualKeyCode::X => Button::Start,
                            VirtualKeyCode::Left => Button::Left,
                            VirtualKeyCode::Right => Button::Right,
                            VirtualKeyCode::Down => Button::Down,
                            VirtualKeyCode::Up => Button::Up,

                            _ => {
                                *control_flow = glutin::event_loop::ControlFlow::Poll;
                                return;
                            }
                        };
                        match input.state {
                            ElementState::Pressed => gameboy.keydown(button),
                            ElementState::Released => gameboy.keyup(button),
                        }
                    }

                    *control_flow = glutin::event_loop::ControlFlow::Poll
                }
                glutin::event::WindowEvent::Resized(glutin::dpi::PhysicalSize {
                    width: _,
                    height: _,
                }) => *control_flow = glutin::event_loop::ControlFlow::Poll,
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit
                }
                _ => *control_flow = glutin::event_loop::ControlFlow::Poll,
            },
            glutin::event::Event::MainEventsCleared => window.request_redraw(),
            glutin::event::Event::RedrawRequested(_) => {
                gameboy.frame();
                cx.draw(&gameboy);
                gl_window.swap_buffers().unwrap();

                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            _ => {
                let next_frame_time =
                    std::time::Instant::now() + std::time::Duration::from_millis(5);
                *control_flow =
                    glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);
            }
        }
    })
}

const VERTEX: &str = r"#version 150 core
in vec2 position;
in vec3 color;
in vec2 texcoord;
out vec3 Color;
out vec2 Texcoord;
void main() {
   Color = color;
   Texcoord = texcoord;
   gl_Position = vec4(position, 0.0, 1.0);
}
";

const FRAGMENT: &str = r"#version 150 core
in vec3 Color;
in vec2 Texcoord;
out vec4 outColor;
uniform sampler2D tex;
void main() {
   outColor = texture(tex, Texcoord);
}
";

impl Glcx {
    pub fn new() -> Glcx {
        unsafe {
            let mut vao = 0;
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            let mut vbo = 0;
            gl::GenBuffers(1, &mut vbo);

            const VERTICES: &[f32] = &[
                //  Position   Color             Texcoords
                -1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, // Top-left
                1.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, // Top-right
                1.0, -1.0, 0.0, 0.0, 1.0, 1.0, 1.0, // Bottom-right
                -1.0, -1.0, 1.0, 1.0, 1.0, 0.0, 1.0, // Bottom-left
            ];
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (VERTICES.len() * 4) as libc::ssize_t,
                VERTICES.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            let mut ebo = 0;
            gl::GenBuffers(1, &mut ebo);

            const ELEMENTS: &[GLuint] = &[0, 1, 2, 2, 3, 0];

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                std::mem::size_of_val(ELEMENTS) as libc::ssize_t,
                ELEMENTS.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            let vert = gl::CreateShader(gl::VERTEX_SHADER);
            let src = CString::new(VERTEX).unwrap();
            gl::ShaderSource(vert, 1, &src.as_ptr(), std::ptr::null::<i32>());
            gl::CompileShader(vert);

            // Create and compile the fragment shader
            let frag = gl::CreateShader(gl::FRAGMENT_SHADER);
            let src = CString::new(FRAGMENT).unwrap();
            gl::ShaderSource(frag, 1, &src.as_ptr(), std::ptr::null::<i32>());
            gl::CompileShader(frag);

            let program = gl::CreateProgram();
            gl::AttachShader(program, vert);
            gl::AttachShader(program, frag);
            let buf = CString::new("outColor").unwrap();
            gl::BindFragDataLocation(program, 0, buf.as_ptr());
            gl::LinkProgram(program);
            assert_eq!(gl::GetError(), 0);
            gl::UseProgram(program);

            let buf = CString::new("position").unwrap();
            let pos_attrib = gl::GetAttribLocation(program, buf.as_ptr());
            gl::EnableVertexAttribArray(pos_attrib as u32);
            gl::VertexAttribPointer(
                pos_attrib as u32,
                2,
                gl::FLOAT,
                gl::FALSE,
                (7 * mem::size_of::<GLfloat>()) as i32,
                std::ptr::null(),
            );

            let buf = CString::new("color").unwrap();
            let col_attrib = gl::GetAttribLocation(program, buf.as_ptr());
            gl::EnableVertexAttribArray(col_attrib as u32);
            gl::VertexAttribPointer(
                col_attrib as u32,
                3,
                gl::FLOAT,
                gl::FALSE,
                (7 * mem::size_of::<GLfloat>()) as i32,
                (2 * mem::size_of::<GLfloat>()) as *const _,
            );

            let buf = CString::new("texcoord").unwrap();
            let tex_attrib = gl::GetAttribLocation(program, buf.as_ptr());
            gl::EnableVertexAttribArray(tex_attrib as u32);
            gl::VertexAttribPointer(
                tex_attrib as u32,
                2,
                gl::FLOAT,
                gl::FALSE,
                (7 * mem::size_of::<GLfloat>()) as i32,
                (5 * mem::size_of::<GLfloat>()) as *const _,
            );

            // Load textures
            let mut tex = 0;
            gl::GenTextures(1, &mut tex);

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, tex);
            let buf = CString::new("tex").unwrap();
            gl::Uniform1i(gl::GetUniformLocation(program, buf.as_ptr()), 0);

            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_S,
                gl::CLAMP_TO_EDGE as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_T,
                gl::CLAMP_TO_EDGE as i32,
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

            Glcx {
                tex,
                program,
                frag,
                vert,
                ebo,
                vbo,
                vao,
            }
        }
    }

    pub fn draw(&self, gb: &GameBoy) {
        unsafe {
            gl::ClearColor(0.0, 0.0, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGB as i32,
                gb.width() as i32,
                gb.height() as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                gb.data().as_ptr() as *const _,
            );
            assert_eq!(gl::GetError(), 0);

            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());
        }
    }
}
