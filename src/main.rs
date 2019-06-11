// This file is part auf the dual-kawase-demo package.
//
// (c) 2019 Bernd Busse
//
// For the full copyright and license information, please view the README.md file
// that was distributed with this source code.
//

use std::path::Path;

use gl::types::{GLint, GLvoid};
use sdl2::event::{Event, WindowEvent};
use sdl2::image::{InitFlag, LoadSurface};
use sdl2::keyboard::Keycode;
use sdl2::render::{Texture, TextureCreator};
use sdl2::surface::Surface;

mod renderer_gl;

use renderer_gl::{
    ArrayBuffer, ElementArrayBuffer, FragmentShader, Program, VertexArray, VertexShader, Viewport,
};

const WINDOW_TITLE: &str = "Dual-Filter Kawase Blur — Demo";
const WIN_WIDTH: u32 = 1280;
const WIN_HEIGHT: u32 = 720;

#[inline]
fn scale_keep_aspect(base_w: u32, base_h: u32, width: u32, height: u32) -> (u32, u32) {
    let base_ratio: f32 = base_w as f32 / base_h as f32;
    let scale_ratio: f32 = width as f32 / height as f32;

    if scale_ratio < base_ratio {
        // dest is taller -> fit to width
        (width, (width as f32 / base_ratio) as u32)
    } else {
        // dest is wider -> fit to height
        ((height as f32 * base_ratio) as u32, height)
    }
}

fn scaled_texture_from_surface<'a, T: 'a>(
    creator: &'a TextureCreator<T>,
    base: &Surface,
    width: u32,
    height: u32,
) -> Texture<'a> {
    let (scaled_width, scaled_height) =
        scale_keep_aspect(base.width(), base.height(), width, height);
    let mut scaled_surface =
        Surface::new(scaled_width, scaled_height, creator.default_pixel_format())
            .expect("Cannot create temporary surface");

    base.blit_scaled(None, &mut scaled_surface, None)
        .expect("Cannot scale base image");

    creator
        .create_texture_from_surface(scaled_surface)
        .expect("Cannot convert image to texture")
}

#[inline]
fn quad_keep_aspect(win_w: f32, win_h: f32, base_w: f32, base_h: f32) -> (Vec<f32>, Vec<u32>) {
    let vertices: Vec<f32> = vec![
        -(base_w / win_w),
        -(base_h / win_h),
        0.0,
        1.0,
        (base_w / win_w),
        -(base_h / win_h),
        1.0,
        1.0,
        (base_w / win_w),
        (base_h / win_h),
        1.0,
        0.0,
        -(base_w / win_w),
        (base_h / win_h),
        0.0,
        0.0,
    ];
    let indices: Vec<u32> = vec![0, 1, 3, 1, 2, 3];

    (vertices, indices)
}

fn main() {
    let args: Vec<_> = std::env::args().collect();

    if args.len() != 2 {
        println!("Usage: cargo run /path/to/image.(png|jpg)");
        std::process::exit(1);
    }

    let image_file = Path::new(&args[1]);

    println!("Hello, world!");

    // Init SDL2 with video subsystem
    let sdl = sdl2::init().expect("Cannot initialize SDL2");
    let video_subsystem = sdl.video().expect("Cannot initialize video subsystem");
    let _image_ctx = sdl2::image::init(InitFlag::PNG | InitFlag::JPG);

    let gl_attr = video_subsystem.gl_attr();

    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(3, 3);

    // Create window
    let window = video_subsystem
        .window(WINDOW_TITLE, WIN_WIDTH, WIN_HEIGHT)
        .resizable()
        .opengl()
        .build()
        .expect("Cannot create OpenGL window");
    let mut viewport = Viewport::from_window(WIN_WIDTH, WIN_HEIGHT);

    {
        let _gl_context = window.gl_create_context().expect("Cannot load GL context");
        let _gl = gl::load_with(|s| {
            video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void
        });
    }

    // Load image as texture
    let canvas = window
        .into_canvas()
        .build()
        .expect("Cannot get window canvas");
    let texture_creator = canvas.texture_creator();

    let image_surface = Surface::from_file(image_file).expect("Cannot load base image");
    let mut base_texture =
        scaled_texture_from_surface(&texture_creator, &image_surface, WIN_WIDTH, WIN_HEIGHT);

    // Init shader and program
    let vert_shader = VertexShader::from_source(include_str!("shaders/tex_quad.vert"))
        .expect("Cannot compile vertex shader");
    let frag_shader = FragmentShader::from_source(include_str!("shaders/tex_quad.frag"))
        .expect("Cannot compile fragment shader");

    let main_program = Program::from_shaders(&[vert_shader.into(), frag_shader.into()])
        .expect("Cannot link main program");

    // Init GL state
    unsafe {
        //gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
    }

    // Init full-screen square
    let (mut vertices, indices): (Vec<f32>, Vec<u32>) = quad_keep_aspect(
        canvas.viewport().width() as f32,
        canvas.viewport().height() as f32,
        base_texture.query().width as f32,
        base_texture.query().height as f32,
    );

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
            2,
            gl::FLOAT,
            gl::FALSE,
            (4 * std::mem::size_of::<f32>()) as GLint,
            std::ptr::null(),
        );
        gl::DisableVertexAttribArray(0);

        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(
            1,
            2,
            gl::FLOAT,
            gl::FALSE,
            (4 * std::mem::size_of::<f32>()) as GLint,
            (2 * std::mem::size_of::<f32>()) as *const GLvoid,
        );
        gl::DisableVertexAttribArray(1);
    }
    vbo.unbind();
    vao.unbind();

    viewport.activate();

    // Main loop
    let mut ev_pump = sdl.event_pump().unwrap();
    'mainloop: loop {
        // Handle all queued events
        for event in ev_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'mainloop,
                Event::Window {
                    win_event: WindowEvent::Resized(w, h),
                    ..
                } => {
                    viewport.update_size(w as u32, h as u32);
                    viewport.activate();

                    // Update image texture
                    base_texture = scaled_texture_from_surface(
                        &texture_creator,
                        &image_surface,
                        w as u32,
                        h as u32,
                    );

                    // Update vertex positions
                    vertices = quad_keep_aspect(
                        w as f32,
                        h as f32,
                        base_texture.query().width as f32,
                        base_texture.query().height as f32,
                    )
                    .0;

                    vbo.bind();
                    vbo.set_data(&vertices, gl::STATIC_DRAW);
                    vbo.unbind();
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                }
                | Event::KeyDown {
                    keycode: Some(Keycode::Q),
                    ..
                } => {
                    break 'mainloop;
                }
                //Event::KeyDown { keycode: Some(Keycode::Left), .. }
                //| Event::KeyDown { scancode: Some(Scancode::A), .. } => {
                //},
                //Event::KeyDown { keycode: Some(Keycode::Right), .. }
                //| Event::KeyDown { scancode: Some(Scancode::D), .. } => {
                //},
                //Event::KeyDown { keycode: Some(Keycode::Up), .. }
                //| Event::KeyDown { scancode: Some(Scancode::W), .. } => {
                //}
                //Event::KeyDown { keycode: Some(Keycode::Down), .. }
                //| Event::KeyDown { scancode: Some(Scancode::S), .. } => {
                //}
                _ => (),
            }
        }

        // Draw window contents here
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        // Use main program
        unsafe {
            base_texture.gl_bind_texture();
        }
        main_program.activate();
        vao.bind();
        unsafe {
            gl::EnableVertexAttribArray(0);
            gl::EnableVertexAttribArray(1);
            gl::DrawElements(
                gl::TRIANGLES,
                6,
                gl::UNSIGNED_INT,
                indices.as_ptr() as *const GLvoid,
            );
            gl::DisableVertexAttribArray(0);
            gl::DisableVertexAttribArray(1);
        }
        unsafe {
            base_texture.gl_unbind_texture();
        }

        // Display rendered scene
        canvas.window().gl_swap_window();

        std::thread::sleep(std::time::Duration::new(0, 1e9 as u32 / 60));
    }
}
