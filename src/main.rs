// This file is part auf the dual-kawase-demo package.
//
// (c) 2019 Bernd Busse
//
// For the full copyright and license information, please view the README.md file
// that was distributed with this source code.
//

use std::path::Path;

use sdl2::event::{Event, WindowEvent};
use sdl2::gfx::framerate::FPSManager;
use sdl2::image::{InitFlag, LoadSurface};
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::surface::Surface;
use sdl2::ttf::Hinting;

mod blur;
mod renderer_gl;
mod utils;

use blur::BlurContext;
use renderer_gl::{FragmentShader, GLQuad, Program, SDLQuad, TextureQuad, VertexShader, Viewport};

const WINDOW_TITLE: &str = "Dual-Filter Kawase Blur — Demo";
const WIN_WIDTH: u32 = 1280;
const WIN_HEIGHT: u32 = 720;

const INFO_1: &str = "Down-/Upsample Iterations";
const INFO_2: &str = "Blur Offset";
const INFO_3: &str = "CPU Time";
const INFO_4: &str = "GPU Time";

fn run(image_file: &Path) {
    // Init SDL2 with subsystems
    let sdl = sdl2::init().expect("Cannot initialize SDL2");
    let video_subsystem = sdl.video().expect("Cannot initialize video subsystem");
    let _image_ctx = sdl2::image::init(InitFlag::PNG | InitFlag::JPG);
    let ttf = sdl2::ttf::init().expect("Cannot initialize ttf subsystem");

    let mut fps_manager = FPSManager::new();
    fps_manager.set_framerate(60)
               .expect("Cannot set target framerate");

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(3, 3);

    // Create window
    let window = video_subsystem.window(WINDOW_TITLE, WIN_WIDTH, WIN_HEIGHT)
                                .resizable()
                                .opengl()
                                .build()
                                .expect("Cannot create OpenGL window");
    let mut viewport = Viewport::from_window(WIN_WIDTH, WIN_HEIGHT);

    let _gl_context = window.gl_create_context().expect("Cannot load GL context");
    gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);

    // Load main window canvas
    let canvas = window.into_canvas()
                       .build()
                       .expect("Cannot get window canvas");
    let texture_creator = canvas.texture_creator();

    // Init text rendering
    let mut font = ttf.load_font("./assets/UbuntuMono-R.ttf", 16)
                      .expect("Cannot open font");
    font.set_hinting(Hinting::Normal);

    // Load image as texture
    let image_surface = Surface::from_file(image_file).expect("Cannot load base image");
    let mut base_texture = renderer_gl::scaled_texture_from_surface(&texture_creator,
                                                                    &image_surface,
                                                                    WIN_WIDTH,
                                                                    WIN_HEIGHT);

    // Init full-screen image display
    let mut background_img = GLQuad::new_with_texture(0,
                                                      0,
                                                      base_texture.query().width,
                                                      base_texture.query().height,
                                                      viewport.size());
    background_img.fit_center(viewport.size());

    // Init blur context
    let mut ctx = BlurContext::new((background_img.width(), background_img.height()));

    // Init overlay text
    let mut overlay_iterations = {
        let overlay_tex =
            renderer_gl::render_to_texture(&texture_creator,
                                           &font,
                                           &format!("{}: {}", INFO_1, ctx.iterations()));
        SDLQuad::from_texture(overlay_tex, 20, 20, viewport.size())
    };
    let mut overlay_offset = {
        let overlay_tex =
            renderer_gl::render_to_texture(&texture_creator,
                                           &font,
                                           &format!("{}: {:.02}", INFO_2, ctx.offset()));
        SDLQuad::from_texture(overlay_tex,
                              20,
                              overlay_iterations.height() as i32 + 20,
                              viewport.size())
    };
    let mut overlay_gpu = {
        let overlay_tex =
            renderer_gl::render_to_texture(&texture_creator,
                                           &font,
                                           &format!("{}: {:6.03}ms", INFO_4, ctx.time_gpu()));
        let tex_top = viewport.height() as i32 - overlay_tex.query().height as i32;
        SDLQuad::from_texture(overlay_tex, 20, tex_top - 20, viewport.size())
    };
    let mut overlay_cpu = {
        let overlay_tex =
            renderer_gl::render_to_texture(&texture_creator,
                                           &font,
                                           &format!("{}: {:6.03}ms", INFO_3, ctx.time_cpu()));
        let tex_top = overlay_gpu.pos().1 - overlay_tex.query().height as i32;
        SDLQuad::from_texture(overlay_tex, 20, tex_top - 20, viewport.size())
    };

    // Init main shader and program
    let vert_shader = VertexShader::from_source(include_str!("shaders/tex_quad.vert"))
        .expect("Cannot compile vertex shader");
    let frag_shader = FragmentShader::from_source(include_str!("shaders/tex_quad.frag"))
        .expect("Cannot compile fragment shader");

    let main_program = Program::from_shaders(&[vert_shader.into(), frag_shader.into()], None)
        .expect("Cannot link main program");

    // Init GL state
    unsafe {
        gl::ClearColor(0.2, 0.2, 0.3, 1.0);
        gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
    }

    viewport.activate();

    // Main loop
    let mut ev_pump = sdl.event_pump().unwrap();
    let mut redraw = false;
    'mainloop: loop {
        // Handle all queued events
        for event in ev_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'mainloop,
                Event::Window { win_event: WindowEvent::Resized(w, h),
                                .. } => {
                    // Update viewport
                    viewport.update_size(w as u32, h as u32);
                    viewport.activate();

                    // Update image texture
                    base_texture = renderer_gl::scaled_texture_from_surface(&texture_creator,
                                                                            &image_surface,
                                                                            viewport.size().0,
                                                                            viewport.size().1);
                    renderer_gl::set_texture_params(&mut base_texture);
                    let base_w = base_texture.query().width;
                    let base_h = base_texture.query().height;
                    background_img.resize(base_w, base_h);
                    ctx.resize(base_w, base_h);

                    // Update vertex positions
                    background_img.fit_center(viewport.size());
                    overlay_iterations.update_vp(viewport.size());
                    overlay_offset.update_vp(viewport.size());

                    let gpu_top = viewport.height() as i32 - overlay_gpu.height() as i32;
                    overlay_gpu.update_pos(20, gpu_top - 20, viewport.size());
                    let cpu_top = overlay_gpu.pos().1 - overlay_cpu.height() as i32;
                    overlay_cpu.update_pos(20, cpu_top, viewport.size());

                    // Redraw blur
                    redraw = true;
                },
                Event::KeyDown { keycode: Some(Keycode::Escape),
                                 .. }
                | Event::KeyDown { keycode: Some(Keycode::Q),
                                 .. } => {
                    break 'mainloop;
                },
                Event::KeyDown { keycode: Some(Keycode::Left),
                                 .. } => {
                    if ctx.iterations() > 0 {
                        ctx.set_iterations(ctx.iterations() - 1);
                        redraw = true;
                    }
                },
                Event::KeyDown { keycode: Some(Keycode::Right),
                                 .. } => {
                    let scale = 1 << (ctx.iterations() + 1);
                    let base = base_texture.query();
                    if base.width / scale > 10 || base.height / scale > 10 {
                        ctx.set_iterations(ctx.iterations() + 1);
                        redraw = true;
                    }
                },
                Event::KeyDown { keycode: Some(Keycode::Up),
                                 .. } => {
                    if ctx.offset() < 25.0 {
                        ctx.set_offset(ctx.offset() + 0.25);
                        redraw = true;
                    } else {
                        ctx.set_offset(25.0);
                        redraw = true;
                    }
                },
                Event::KeyDown { keycode: Some(Keycode::Down),
                                 .. } => {
                    if ctx.offset() > 0.0 {
                        ctx.set_offset(ctx.offset() - 0.25);
                    } else {
                        ctx.set_offset(0.0);
                    }
                    redraw = true;
                },
                Event::KeyDown { scancode: Some(Scancode::S),
                                 .. } => {
                    let mut count = 0;
                    let mut fname;
                    loop {
                        count += 1;
                        fname = format!("blurresult_{}.png", count);
                        if !Path::new(&fname).exists() {
                            break;
                        }
                    }
                    let path = Path::new(&fname);

                    println!("Save image to {:?}", path);
                    renderer_gl::save_texture_to_png(*background_img.texture(), path);
                },
                Event::KeyDown { scancode: Some(Scancode::R),
                                 .. } => {
                    if ctx.offset() > 0.0 || ctx.iterations() != 0 {
                        ctx.set_offset(0.0);
                        ctx.set_iterations(0);
                        redraw = true;
                    }
                },
                Event::KeyDown { keycode: Some(Keycode::Return),
                                 .. }
                | Event::KeyDown { keycode: Some(Keycode::Space),
                                 .. } => {
                    // Force a redraw
                    redraw = true;
                },
                _ => (),
            }
        }

        if redraw {
            redraw = false;
            // Redraw blur texture
            ctx.blur(&mut base_texture, &background_img);

            // Update overlay textures
            let overlay_tex1 =
                renderer_gl::render_to_texture(&texture_creator,
                                               &font,
                                               &format!("{}: {}", INFO_1, ctx.iterations()));
            overlay_iterations.update_texture(overlay_tex1, viewport.size());
            let overlay_tex2 =
                renderer_gl::render_to_texture(&texture_creator,
                                               &font,
                                               &format!("{}: {:.02}", INFO_2, ctx.offset()));
            overlay_offset.update_texture(overlay_tex2, viewport.size());
            let overlay_tex3 =
                renderer_gl::render_to_texture(&texture_creator,
                                               &font,
                                               &format!("{}: {:6.03}ms", INFO_3, ctx.time_cpu()));
            overlay_cpu.update_texture(overlay_tex3, viewport.size());
            let overlay_tex4 =
                renderer_gl::render_to_texture(&texture_creator,
                                               &font,
                                               &format!("{}: {:6.03}ms", INFO_4, ctx.time_gpu()));
            overlay_gpu.update_texture(overlay_tex4, viewport.size());
        }

        // Draw window contents here
        viewport.activate();
        unsafe {
            gl::DrawBuffer(gl::BACK);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        main_program.activate();

        // Draw background texture
        background_img.draw(true);

        // Draw overlay text
        overlay_iterations.draw(true);
        overlay_offset.draw(true);
        overlay_gpu.draw(true);
        overlay_cpu.draw(true);

        main_program.unbind();

        // Display rendered scene
        canvas.window().gl_swap_window();

        fps_manager.delay();
    }
}

fn main() {
    let args: Vec<_> = std::env::args().collect();

    if args.len() != 2 {
        println!("Usage: cargo run /path/to/image.(png|jpg)");
        std::process::exit(1);
    }
    let image_file = Path::new(&args[1]);

    // Init graphics and run main loop
    run(image_file);
}
