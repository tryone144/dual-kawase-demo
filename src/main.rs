// This file is part auf the dual-kawase-demo package.
//
// (c) 2019 Bernd Busse
//
// For the full copyright and license information, please view the README.md file
// that was distributed with this source code.
//

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

use sdl2::event::{Event, WindowEvent};
use sdl2::gfx::framerate::FPSManager;
use sdl2::keyboard::{Keycode, Mod, Scancode};
use sdl2::video::FullscreenType;

mod blur;
mod overlay;
mod renderer_gl;
mod utils;

use blur::BlurContext;
use overlay::InfoOverlay;
use renderer_gl::{FragmentShader, GLQuad, ImgSurface, Program, TextureQuad, VertexShader, Viewport};

const WINDOW_TITLE: &str = "Dual-Filter Kawase Blur — Demo";
const WIN_WIDTH: u32 = 1280;
const WIN_HEIGHT: u32 = 720;

fn run(image_file: &Path) {
    println!("Load base image '{}' ...", image_file.display());
    let base_image = image::open(image_file).expect("Cannot load base image");

    // Init SDL2 with subsystems
    let sdl = sdl2::init().expect("Cannot initialize SDL2");
    let video_subsystem = sdl.video().expect("Cannot initialize video subsystem");

    let mut fps_manager = FPSManager::new();
    fps_manager.set_framerate(60)
               .expect("Cannot set target framerate");

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(3, 3);
    gl_attr.set_double_buffer(true);
    gl_attr.set_context_flags().forward_compatible().set();

    // Create window
    let mut window = video_subsystem.window(WINDOW_TITLE, WIN_WIDTH, WIN_HEIGHT)
                                    .resizable()
                                    .opengl()
                                    .build()
                                    .expect("Cannot create OpenGL window");
    let mut viewport = Viewport::from_window(WIN_WIDTH, WIN_HEIGHT);

    // TODO: Make use of transform matrix and don't rely on recalulating quad vertices

    let _gl_context = window.gl_create_context().expect("Cannot load GL context");
    gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);
    println!("GL version/profile: {}.{} / {:?}",
             gl_attr.context_version().0,
             gl_attr.context_version().1,
             gl_attr.context_profile());

    // Load image as texture
    let base_surface = Arc::new(Mutex::new(ImgSurface::new_from_image(&base_image,
                                                                      viewport.width(),
                                                                      viewport.height())));

    // Init full-screen image display
    let mut background_img = {
        let base = base_surface.lock().unwrap();
        let mut quad = GLQuad::new_with_texture(0, 0, base.width(), base.height(), viewport.size());
        quad.fit_center(viewport.size());

        quad
    };

    // Init blur context
    let mut blur_ctx = BlurContext::new(background_img.size());

    // Init overlay text
    let mut overlay = InfoOverlay::new(&blur_ctx, &viewport);

    // Init main shader and program
    let vert_shader = VertexShader::from_source(include_str!("shaders/tex_quad.vert"))
        .expect("Cannot compile vertex shader");
    let frag_shader = FragmentShader::from_source(include_str!("shaders/tex_quad.frag"))
        .expect("Cannot compile fragment shader");

    let mut main_program =
        Program::from_shaders(&[vert_shader.into(), frag_shader.into()],
                              Some(&["transform"])).expect("Cannot link main program");

    main_program.activate();
    main_program.set_uniform_mat4f("transform", &utils::matrix4f_identity())
                .expect("Cannot set 'transform' in main program");
    main_program.unbind();

    // Init GL state
    unsafe {
        gl::ClearColor(0.2, 0.2, 0.3, 1.0);
        gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
        gl::Enable(gl::CULL_FACE);
        gl::CullFace(gl::BACK);
    }

    viewport.activate();

    // redraw mutex lock
    let redraw = Arc::new(Mutex::new(true));
    macro_rules! sync_redraw {
        ($ref:ident | $synced:block) => {{
            let mut $ref = redraw.lock().unwrap();
            $synced
        }};
        ($synced:block) => {
            sync_redraw!(_redraw_lock | $synced)
        };
    }
    macro_rules! try_sync_redraw {
        ($ref:ident | $synced:block) => {{
            let mut redraw_lock = redraw.try_lock();
            if let Ok(ref mut $ref) = redraw_lock {
                $synced
            }
        }};
    }

    // Main loop
    println!("Init done. Start main loop ...");
    let mut ev_pump = sdl.event_pump().unwrap();
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

                    // Update overlay
                    overlay.resize(&viewport);

                    // Resize base image
                    let (new_w, new_h) = viewport.size();
                    let surf_ref = base_surface.clone();
                    let redraw_ref = redraw.clone();

                    thread::spawn(move || {
                        // Block further redraw events
                        let mut update_lock = redraw_ref.lock().unwrap();

                        // XXX Just rescale image and emit redraw event. Texture update is done in
                        //     main thread
                        let mut surf = surf_ref.lock().unwrap();
                        surf.resize_image(new_w, new_h);

                        // Sent redraw blur event
                        *update_lock = true;
                    });
                },
                Event::KeyDown { keycode: Some(Keycode::Escape),
                                 .. }
                | Event::KeyDown { keycode: Some(Keycode::Q),
                                 keymod: Mod::NOMOD,
                                 .. } => {
                    break 'mainloop;
                },
                Event::KeyDown { scancode: Some(Scancode::F),
                                 keymod: Mod::NOMOD,
                                 .. }
                | Event::KeyDown { keycode: Some(Keycode::F11),
                                 keymod: Mod::NOMOD,
                                 .. } => {
                    match window.fullscreen_state() {
                        FullscreenType::Off => {
                            window.set_fullscreen(FullscreenType::Desktop)
                                  .unwrap_or_else(|err| {
                                      eprintln!("Cannot enter fullscreen mode: {}", err)
                                  });
                        },
                        FullscreenType::True | FullscreenType::Desktop => {
                            window.set_fullscreen(FullscreenType::Off)
                                  .unwrap_or_else(|err| {
                                      eprintln!("Cannot leave fullscreen mode: {}", err)
                                  });
                        },
                    };
                },
                Event::KeyDown { keycode: Some(Keycode::Left),
                                 .. } => {
                    sync_redraw!(
                                 redraw_ref | {
                        if blur_ctx.iterations() > 0 {
                            blur_ctx.inc_iterations(-1);
                            *redraw_ref = true;
                        }
                    }
                    );
                },
                Event::KeyDown { keycode: Some(Keycode::Right),
                                 .. } => {
                    sync_redraw!(
                                 redraw_ref | {
                        let scale = 1 << (blur_ctx.iterations() + 1);
                        let surf = base_surface.lock().unwrap();
                        if (surf.width() / scale > 10 || surf.height() / scale > 10)
                           && blur_ctx.iterations() < blur::MAX_ITERATIONS as u32
                        {
                            blur_ctx.inc_iterations(1);
                            *redraw_ref = true;
                        }
                    }
                    );
                },
                Event::KeyDown { keycode: Some(Keycode::Up),
                                 keymod,
                                 .. } if (keymod & Mod::RSHIFTMOD) | (keymod & Mod::LSHIFTMOD) != Mod::NOMOD =>
                {
                    sync_redraw!(
                                 redraw_ref | {
                        if blur_ctx.offset() <= 24.0 {
                            blur_ctx.inc_offset(1.0);
                            *redraw_ref = true;
                        } else {
                            blur_ctx.set_offset(25.0);
                            *redraw_ref = true;
                        }
                    }
                    );
                }
                Event::KeyDown { keycode: Some(Keycode::Up),
                                 .. } => {
                    sync_redraw!(
                                 redraw_ref | {
                        if blur_ctx.offset() <= 24.75 {
                            blur_ctx.inc_offset(0.25);
                            *redraw_ref = true;
                        } else {
                            blur_ctx.set_offset(25.0);
                            *redraw_ref = true;
                        }
                    }
                    );
                },
                Event::KeyDown { keycode: Some(Keycode::Down),
                                 keymod,
                                 .. } if (keymod & Mod::RSHIFTMOD) | (keymod & Mod::LSHIFTMOD) != Mod::NOMOD =>
                {
                    sync_redraw!(
                                 redraw_ref | {
                        if blur_ctx.offset() >= 1.0 {
                            blur_ctx.inc_offset(-1.0);
                        } else {
                            blur_ctx.set_offset(0.0);
                        }
                        *redraw_ref = true;
                    }
                    );
                }
                Event::KeyDown { keycode: Some(Keycode::Down),
                                 .. } => {
                    sync_redraw!(
                                 redraw_ref | {
                        if blur_ctx.offset() >= 0.25 {
                            blur_ctx.inc_offset(-0.25);
                        } else {
                            blur_ctx.set_offset(0.0);
                        }
                        *redraw_ref = true;
                    }
                    );
                },
                Event::KeyDown { scancode: Some(Scancode::S),
                                 keymod,
                                 repeat: false,
                                 .. } if (keymod & Mod::RCTRLMOD) | (keymod & Mod::LCTRLMOD) != Mod::NOMOD =>
                {
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

                    println!("Save image to '{}' ...", path.display());
                    renderer_gl::save_texture_to_png(*background_img.texture(), path);
                }
                Event::KeyDown { keycode: Some(Keycode::Num1),
                                 keymod: Mod::NOMOD,
                                 repeat: false,
                                 .. } => {
                    sync_redraw!(
                                 redraw_ref | {
                        blur_ctx.set_iterations(1);
                        blur_ctx.set_offset(1.5);
                        *redraw_ref = true;
                    }
                    );
                },
                Event::KeyDown { keycode: Some(Keycode::Num2),
                                 keymod: Mod::NOMOD,
                                 repeat: false,
                                 .. } => {
                    sync_redraw!(
                                 redraw_ref | {
                        blur_ctx.set_iterations(1);
                        blur_ctx.set_offset(2.0);
                        *redraw_ref = true;
                    }
                    );
                },
                Event::KeyDown { keycode: Some(Keycode::Num3),
                                 keymod: Mod::NOMOD,
                                 repeat: false,
                                 .. } => {
                    sync_redraw!(
                                 redraw_ref | {
                        blur_ctx.set_iterations(2);
                        blur_ctx.set_offset(2.5);
                        *redraw_ref = true;
                    }
                    );
                },
                Event::KeyDown { keycode: Some(Keycode::Num4),
                                 keymod: Mod::NOMOD,
                                 repeat: false,
                                 .. } => {
                    sync_redraw!(
                                 redraw_ref | {
                        blur_ctx.set_iterations(2);
                        blur_ctx.set_offset(3.0);
                        *redraw_ref = true;
                    }
                    );
                },
                Event::KeyDown { keycode: Some(Keycode::Num5),
                                 keymod: Mod::NOMOD,
                                 repeat: false,
                                 .. } => {
                    sync_redraw!(
                                 redraw_ref | {
                        blur_ctx.set_iterations(3);
                        blur_ctx.set_offset(2.75);
                        *redraw_ref = true;
                    }
                    );
                },
                Event::KeyDown { keycode: Some(Keycode::Num6),
                                 keymod: Mod::NOMOD,
                                 repeat: false,
                                 .. } => {
                    sync_redraw!(
                                 redraw_ref | {
                        blur_ctx.set_iterations(3);
                        blur_ctx.set_offset(3.5);
                        *redraw_ref = true;
                    }
                    );
                },
                Event::KeyDown { keycode: Some(Keycode::Num7),
                                 keymod: Mod::NOMOD,
                                 repeat: false,
                                 .. } => {
                    sync_redraw!(
                                 redraw_ref | {
                        blur_ctx.set_iterations(3);
                        blur_ctx.set_offset(4.25);
                        *redraw_ref = true;
                    }
                    );
                },
                Event::KeyDown { keycode: Some(Keycode::Num8),
                                 keymod: Mod::NOMOD,
                                 repeat: false,
                                 .. } => {
                    sync_redraw!(
                                 redraw_ref | {
                        blur_ctx.set_iterations(3);
                        blur_ctx.set_offset(5.0);
                        *redraw_ref = true;
                    }
                    );
                },
                Event::KeyDown { keycode: Some(Keycode::Num9),
                                 keymod: Mod::NOMOD,
                                 repeat: false,
                                 .. } => {
                    sync_redraw!(
                                 redraw_ref | {
                        blur_ctx.set_iterations(4);
                        blur_ctx.set_offset(3.75);
                        *redraw_ref = true;
                    }
                    );
                },
                Event::KeyDown { keycode: Some(Keycode::Num0),
                                 keymod: Mod::NOMOD,
                                 repeat: false,
                                 .. }
                | Event::KeyDown { scancode: Some(Scancode::R),
                                 keymod: Mod::NOMOD,
                                 repeat: false,
                                 .. } => {
                    sync_redraw!(
                                 redraw_ref | {
                        if blur_ctx.offset() > 0.0 || blur_ctx.iterations() != 0 {
                            blur_ctx.set_offset(0.0);
                            blur_ctx.set_iterations(0);
                            *redraw_ref = true;
                        }
                    }
                    );
                },
                Event::KeyDown { scancode: Some(Scancode::R),
                                 keymod,
                                 repeat: false,
                                 .. } if (keymod & Mod::RCTRLMOD) | (keymod & Mod::LCTRLMOD) != Mod::NOMOD =>
                {
                    // Force a redraw
                    *redraw.lock().unwrap() = true;
                }
                Event::KeyDown { keycode: Some(Keycode::Return),
                                 .. }
                | Event::KeyDown { keycode: Some(Keycode::Space),
                                 .. } => {
                    // Force a redraw
                    *redraw.lock().unwrap() = true;
                },
                _ => (),
            }
        }

        // check for redraw events
        try_sync_redraw!(
                         redraw_ref | {
            if **redraw_ref {
                **redraw_ref = false;

                // Redraw blur texture
                let mut surf = base_surface.lock().unwrap();
                surf.refresh_texture();

                // Update vertex positions
                background_img.resize(surf.width(), surf.height());
                background_img.fit_center(viewport.size());

                blur_ctx.resize(surf.width(), surf.height());
                blur_ctx.blur(&surf, &background_img);

                // Update overlay
                overlay.update(&blur_ctx);

                println!("Blurred ({}x{}) texture with {{offset: {}, iterations: {:.02}}}",
                         surf.width(),
                         surf.height(),
                         blur_ctx.offset(),
                         blur_ctx.iterations());
                println!("   => Time CPU: {:6.03}ms, GPU: {:6.03}ms",
                         blur_ctx.time_cpu(),
                         blur_ctx.time_gpu());
            }
        }
        );

        // Draw window contents here
        viewport.activate();
        unsafe {
            gl::DrawBuffer(gl::BACK);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        // sync_redraw!({
        // Draw background texture
        main_program.activate();
        background_img.draw(true);
        main_program.unbind();
        // Draw overlay text
        overlay.draw(true);
        //});

        // Display rendered scene
        window.gl_swap_window();

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
