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
use sdl2::keyboard::Keycode;
use sdl2::render::{Texture, TextureCreator};
use sdl2::surface::Surface;
use sdl2::ttf::{Font, Hinting};

mod blur;
mod renderer_gl;

use renderer_gl::{FragmentShader, GLQuad, Program, SDLQuad, TextureQuad, VertexShader, Viewport};

const WINDOW_TITLE: &str = "Dual-Filter Kawase Blur — Demo";
const WIN_WIDTH: u32 = 1280;
const WIN_HEIGHT: u32 = 720;

const INFO_1: &str = "Down-/Upsample Iterations";
const INFO_2: &str = "Blur Offset";

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
fn render_to_texture<'r, T: 'r>(
    creator: &'r TextureCreator<T>,
    font: &Font,
    message: &str,
) -> Texture<'r> {
    let text_surf = font
        .render(message)
        .blended((255, 255, 255, 255))
        .expect("Cannot render text to surface");
    creator
        .create_texture_from_surface(text_surf)
        .expect("Cannot convert surface to texture")
}

fn run(image_file: &Path) {
    // Init SDL2 with subsystems
    let sdl = sdl2::init().expect("Cannot initialize SDL2");
    let video_subsystem = sdl.video().expect("Cannot initialize video subsystem");
    let _image_ctx = sdl2::image::init(InitFlag::PNG | InitFlag::JPG);
    let ttf = sdl2::ttf::init().expect("Cannot initialize ttf subsystem");

    let mut fps_manager = FPSManager::new();
    fps_manager
        .set_framerate(60)
        .expect("Cannot set target framerate");

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

    let _gl_context = window.gl_create_context().expect("Cannot load GL context");
    gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);

    // Load main window canvas
    let canvas = window
        .into_canvas()
        .build()
        .expect("Cannot get window canvas");
    let texture_creator = canvas.texture_creator();

    // Init text rendering
    let mut font = ttf
        .load_font("./assets/UbuntuMono-R.ttf", 16)
        .expect("Cannot open font");
    font.set_hinting(Hinting::Normal);

    // Load image as texture
    let image_surface = Surface::from_file(image_file).expect("Cannot load base image");
    let mut base_texture =
        scaled_texture_from_surface(&texture_creator, &image_surface, WIN_WIDTH, WIN_HEIGHT);

    // Init full-screen image display
    let mut background_img = GLQuad::new_with_texture(
        0,
        0,
        base_texture.query().width,
        base_texture.query().height,
        viewport.size(),
    );
    background_img.fit_center(viewport.size());

    // Init blur context
    let mut ctx = blur::BlurContext::new();

    // Init overlay text
    let overlay_tex1 = render_to_texture(
        &texture_creator,
        &font,
        &format!("{}: {}", INFO_1, ctx.iterations()),
    );
    let mut overlay_iterations = SDLQuad::from_texture(overlay_tex1, 20, 20, viewport.size());
    let overlay_tex2 = render_to_texture(
        &texture_creator,
        &font,
        &format!("{}: {}", INFO_2, ctx.offset()),
    );
    let mut overlay_offset = SDLQuad::from_texture(
        overlay_tex2,
        20,
        20 + overlay_iterations.height() as i32,
        viewport.size(),
    );

    // Init main shader and program
    let vert_shader = VertexShader::from_source(include_str!("shaders/tex_quad.vert"))
        .expect("Cannot compile vertex shader");
    let frag_shader = FragmentShader::from_source(include_str!("shaders/tex_quad.frag"))
        .expect("Cannot compile fragment shader");

    let main_program = Program::from_shaders(&[vert_shader.into(), frag_shader.into()])
        .expect("Cannot link main program");

    // Init GL state
    unsafe {
        gl::ClearColor(0.2, 0.2, 0.3, 1.0);
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
                Event::Window {
                    win_event: WindowEvent::Resized(w, h),
                    ..
                } => {
                    // Update viewport
                    viewport.update_size(w as u32, h as u32);
                    viewport.activate();

                    // Update image texture
                    base_texture = scaled_texture_from_surface(
                        &texture_creator,
                        &image_surface,
                        viewport.size().0,
                        viewport.size().1,
                    );
                    background_img.resize(base_texture.query().width, base_texture.query().height);

                    // Update vertex positions
                    background_img.fit_center(viewport.size());
                    overlay_iterations.update_vp(viewport.size());
                    overlay_offset.update_vp(viewport.size());

                    // Redraw blur
                    redraw = true;
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

        // Redraw blur texture
        if redraw {
            redraw = false;
            ctx.blur(&mut base_texture, &background_img);
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

    println!("Hello, world!");
    run(image_file);
}
