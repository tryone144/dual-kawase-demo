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

mod buffer;
mod quad;
mod shader;
mod surface;
mod viewport;

pub use self::buffer::{ArrayBuffer, ElementArrayBuffer, VertexArray};
pub use self::quad::{GLQuad, Quad, TextureQuad};
pub use self::shader::{FragmentShader, GlShader, Program, VertexShader};
pub use self::surface::ImgSurface;
pub use self::viewport::Viewport;

enum Alignment {
    RED,
    BGRA,
}

impl Alignment {
    pub fn value(&self) -> i32 {
        match self {
            Alignment::RED => 1,
            Alignment::BGRA => 4,
        }
    }
}

pub fn create_texture_bgra(width: u32, height: u32, data: Option<Vec<u8>>) -> GLuint {
    create_texture(width, height, data, Alignment::BGRA)
}

pub fn create_texture_red(width: u32, height: u32, data: Option<Vec<u8>>) -> GLuint {
    create_texture(width, height, data, Alignment::RED)
}

fn create_texture(width: u32, height: u32, data: Option<Vec<u8>>, align: Alignment) -> GLuint {
    let mut texture: GLuint = 0;
    unsafe {
        gl::GenTextures(1, &mut texture);
    }

    resize_texture(texture, width, height, data, align);

    texture
}

pub fn resize_texture_bgra(tex: GLuint, width: u32, height: u32, data: Option<Vec<u8>>) {
    resize_texture(tex, width, height, data, Alignment::BGRA)
}

pub fn resize_texture_red(tex: GLuint, width: u32, height: u32, data: Option<Vec<u8>>) {
    resize_texture(tex, width, height, data, Alignment::RED)
}

fn resize_texture(tex: GLuint, width: u32, height: u32, data: Option<Vec<u8>>, align: Alignment) {
    let raw_data = match data {
        Some(vec) => vec.as_ptr() as *const GLvoid,
        None => std::ptr::null(),
    };

    let components = match align {
        Alignment::RED => gl::RED,
        Alignment::BGRA => gl::RGBA8,
    };
    let format = match align {
        Alignment::RED => gl::RED,
        Alignment::BGRA => gl::BGRA,
    };

    unsafe {
        gl::PixelStorei(gl::UNPACK_ALIGNMENT, align.value());
        gl::BindTexture(gl::TEXTURE_2D, tex);
        gl::TexImage2D(gl::TEXTURE_2D,
                       0,
                       components as i32,
                       width as i32,
                       height as i32,
                       0,
                       format,
                       gl::UNSIGNED_BYTE,
                       raw_data);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAX_LEVEL, 0);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
        gl::BindTexture(gl::TEXTURE_2D, 0);
    }
}

pub fn get_texture_size(tex: GLuint) -> (u32, u32) {
    let mut width: GLint = 0;
    let mut height: GLint = 0;
    unsafe {
        gl::BindTexture(gl::TEXTURE_2D, tex);
        gl::GetTexLevelParameteriv(gl::TEXTURE_2D, 0, gl::TEXTURE_WIDTH, &mut width);
        gl::GetTexLevelParameteriv(gl::TEXTURE_2D, 0, gl::TEXTURE_HEIGHT, &mut height);
        gl::BindTexture(gl::TEXTURE_2D, 0);
    }

    (width as u32, height as u32)
}

pub fn save_texture_to_png(tex: GLuint, filename: &Path) {
    // get texture size
    let (width, height) = get_texture_size(tex);

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
