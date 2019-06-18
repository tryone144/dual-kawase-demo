// This file is part auf the dual-kawase-demo package.
//
// (c) 2019 Bernd Busse
//
// For the full copyright and license information, please view the README.md file
// that was distributed with this source code.
//

use std::path::Path;
use std::thread;

use gl::types::{GLint, GLuint, GLvoid};
use sdl2::render::{Texture, TextureCreator};
use sdl2::surface::Surface;
use sdl2::ttf::Font;

mod buffer;
mod quad;
mod shader;
mod viewport;

pub use self::buffer::{ArrayBuffer, ElementArrayBuffer, VertexArray};
pub use self::quad::{GLQuad, Quad, SDLQuad, TextureQuad};
pub use self::shader::{FragmentShader, GlShader, Program, VertexShader};
pub use self::viewport::Viewport;

pub fn create_texture(width: u32, height: u32) -> GLuint {
    let mut texture: GLuint = 0;
    unsafe {
        gl::GenTextures(1, &mut texture);
        gl::BindTexture(gl::TEXTURE_2D, texture);
        gl::TexImage2D(gl::TEXTURE_2D,
                       0,
                       gl::RGBA8 as i32,
                       width as i32,
                       height as i32,
                       0,
                       gl::BGRA,
                       gl::UNSIGNED_BYTE,
                       std::ptr::null());
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAX_LEVEL, 0);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
        gl::BindTexture(gl::TEXTURE_2D, 0);
    }

    texture
}

pub fn resize_texture(tex: GLuint, width: u32, height: u32) {
    unsafe {
        gl::BindTexture(gl::TEXTURE_2D, tex);
        gl::TexImage2D(gl::TEXTURE_2D,
                       0,
                       gl::RGBA8 as i32,
                       width as i32,
                       height as i32,
                       0,
                       gl::BGRA,
                       gl::UNSIGNED_BYTE,
                       std::ptr::null());
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAX_LEVEL, 0);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
        gl::BindTexture(gl::TEXTURE_2D, 0);
    }
}

pub fn set_texture_params<'r>(tex: &mut Texture<'r>) {
    unsafe {
        tex.gl_bind_texture();
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAX_LEVEL, 0);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
        tex.gl_unbind_texture();
    }
}

pub fn scaled_texture_from_surface<'a, T: 'a>(creator: &'a TextureCreator<T>,
                                              base: &Surface,
                                              width: u32,
                                              height: u32)
                                              -> Texture<'a> {
    let (scaled_width, scaled_height) =
        crate::utils::scale_keep_aspect(base.width(), base.height(), width, height);
    let mut scaled_surface =
        Surface::new(scaled_width, scaled_height, creator.default_pixel_format())
            .expect("Cannot create temporary surface");

    base.blit_scaled(None, &mut scaled_surface, None)
        .expect("Cannot scale base image");

    creator.create_texture_from_surface(scaled_surface)
           .expect("Cannot convert image to texture")
}

#[inline]
pub fn render_to_texture<'r, T: 'r>(creator: &'r TextureCreator<T>,
                                    font: &Font,
                                    message: &str)
                                    -> Texture<'r> {
    let text_surf = font.render(message)
                        .blended((255, 255, 255, 255))
                        .expect("Cannot render text to surface");
    creator.create_texture_from_surface(text_surf)
           .expect("Cannot convert surface to texture")
}

pub fn save_texture_to_png(tex: GLuint, filename: &Path) {
    // get texture size
    let mut width: GLint = 0;
    let mut height: GLint = 0;
    unsafe {
        gl::BindTexture(gl::TEXTURE_2D, tex);
        gl::GetTexLevelParameteriv(gl::TEXTURE_2D, 0, gl::TEXTURE_WIDTH, &mut width);
        gl::GetTexLevelParameteriv(gl::TEXTURE_2D, 0, gl::TEXTURE_HEIGHT, &mut height);
    }

    // get texture pixels
    let size: usize = width as usize * height as usize * 4;
    let mut pixel_buf: Vec<u8> = Vec::with_capacity(size);
    pixel_buf.extend([0u8].iter().cycle().take(size));
    unsafe {
        gl::BindTexture(gl::TEXTURE_2D, tex);
        gl::GetTexImage(gl::TEXTURE_2D,
                        0,
                        gl::RGBA,
                        gl::UNSIGNED_BYTE,
                        pixel_buf.as_ptr() as *mut GLvoid);
        gl::BindTexture(gl::TEXTURE_2D, 0);
    }

    // save pixels to file
    let fname = filename.to_owned();
    thread::spawn(move || {
        match image::save_buffer(fname,
                                 &pixel_buf,
                                 width as u32,
                                 height as u32,
                                 image::RGBA(8)).map_err(|e| e.to_string())
        {
            Ok(_) => println!("Save complete"),
            Err(msg) => eprintln!("Cannot save blurred image: {}", msg),
        }
    });
}
