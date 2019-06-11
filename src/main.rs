// This file is part auf the dual-kawase-demo package.
//
// (c) 2019 Bernd Busse
//
// For the full copyright and license information, please view the README.md file
// that was distributed with this source code.
//

use gl::types::{GLint, GLvoid};
use sdl2::event::Event;

mod renderer_gl;

use renderer_gl::{
    ArrayBuffer, ElementArrayBuffer, FragmentShader, Program, VertexArray, VertexShader,
};

const WINDOW_TITLE: &str = "Dual-Filter Kawase Blur — Demo";
const WIN_WIDTH: u32 = 1280;
const WIN_HEIGHT: u32 = 720;

fn main() {
    println!("Hello, world!");

    // Init SDL2 with video subsystem
    let sdl = sdl2::init().expect("Cannot initialize SDL2");
    let video_subsystem = sdl.video().expect("Cannot initialize video subsystem");

    let gl_attr = video_subsystem.gl_attr();

    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(3, 3);

    // Create window
    let window = video_subsystem
        .window(WINDOW_TITLE, WIN_WIDTH, WIN_HEIGHT)
        .opengl()
        .build()
        .expect("Cannot create OpenGL window");

    let _gl_context = window.gl_create_context().expect("Cannot load GL context");
    let _gl =
        gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);

    // Init shader and program
    let vert_shader = VertexShader::from_source(include_str!("shaders/triangle.vert"))
        .expect("Cannot compile vertex shader");
    let frag_shader = FragmentShader::from_source(include_str!("shaders/triangle.frag"))
        .expect("Cannot compile fragment shader");

    let main_program = Program::from_shaders(&[vert_shader.into(), frag_shader.into()])
        .expect("Cannot link main program");

    // Init GL state
    unsafe {
        //gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
        gl::Viewport(0, 0, WIN_WIDTH as i32, WIN_HEIGHT as i32);
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
    }

    // Init full-screen square
    let vertices: Vec<f32> = vec![
        -1.0, -1.0, 0.0, 1.0, -1.0, 0.0, 1.0, 1.0, 0.0, -1.0, 1.0, 0.0,
    ];
    let indices: Vec<u32> = vec![0, 1, 3, 1, 2, 3];

    let mut vbo = ArrayBuffer::new();
    vbo.bind();
    vbo.set_data(&vertices, gl::STATIC_DRAW);
    vbo.unbind();

    let mut ebo = ElementArrayBuffer::new();
    ebo.bind();
    ebo.set_data(&indices, gl::STATIC_DRAW);
    ebo.unbind();

    let vao = VertexArray::new();
    vao.bind();
    vbo.bind();
    unsafe {
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            (3 * std::mem::size_of::<f32>()) as GLint,
            std::ptr::null(),
        );
    }
    vbo.unbind();
    vao.unbind();

    // Main loop
    let mut ev_pump = sdl.event_pump().unwrap();
    'main: loop {
        // Handle all queued events
        for event in ev_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'main,
                _ => (),
            }
        }

        // Draw window contents here
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        // Use main program
        main_program.activate();
        vao.bind();
        unsafe {
            gl::DrawElements(
                gl::TRIANGLES,
                6,
                gl::UNSIGNED_INT,
                indices.as_ptr() as *const GLvoid,
            );
        }

        // Display rendered scene
        window.gl_swap_window();

        std::thread::sleep(std::time::Duration::new(0, 1e9 as u32 / 60));
    }
}
