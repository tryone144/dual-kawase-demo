// This file is part auf the dual-kawase-demo package.
//
// (c) 2019 Bernd Busse
//
// For the full copyright and license information, please view the README.md file
// that was distributed with this source code.
//

use sdl2::event::Event;


fn main() {
    println!("Hello, world!");

    // Init SDL2 with video subsystem
    let sdl = sdl2::init().expect("Cannot initialize SDL2");
    let video_subsystem = sdl.video().expect("Cannot initialize video subsystem");

    // Create window
    let window = video_subsystem.window("Demo", 900, 700)
        .opengl().build().expect("Cannot create OpenGL window");

    let _gl_context = window.gl_create_context().expect("Cannot load GL context");
    let _gl = gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);

    unsafe {
        gl::ClearColor(0.3, 0.3, 0.5, 1.0);
    }

    // Main loop
    let mut ev_pump = sdl.event_pump().unwrap();
    'main: loop {
        // Handle all queued events
        for event in ev_pump.poll_iter() {
            match event {
                Event::Quit{ .. } => break 'main,
                _ => (),
            }
        }

        // Draw window contents here
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        window.gl_swap_window();
    }
}
