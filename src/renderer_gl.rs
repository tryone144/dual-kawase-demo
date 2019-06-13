// This file is part auf the dual-kawase-demo package.
//
// (c) 2019 Bernd Busse
//
// For the full copyright and license information, please view the README.md file
// that was distributed with this source code.
//

use std::ffi::CString;

use gl::types::GLuint;
use sdl2::render::Texture;

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

#[inline]
pub fn centered_quad_keep_aspect(width: f32,
                                 height: f32,
                                 win_w: f32,
                                 win_h: f32,
                                 flip_h: bool)
                                 -> (Vec<f32>, Vec<u32>) {
    let vertices: Vec<f32> = vec![-(width / win_w),
                                  -(height / win_h),
                                  0.0,
                                  if flip_h { 0.0 } else { 1.0 },
                                  (width / win_w),
                                  -(height / win_h),
                                  1.0,
                                  if flip_h { 0.0 } else { 1.0 },
                                  (width / win_w),
                                  (height / win_h),
                                  1.0,
                                  if flip_h { 1.0 } else { 0.0 },
                                  -(width / win_w),
                                  (height / win_h),
                                  0.0,
                                  if flip_h { 1.0 } else { 0.0 }];
    let indices: Vec<u32> = vec![0, 1, 3, 1, 2, 3];

    (vertices, indices)
}

#[inline]
pub fn quad_at_pos(x: i32,
                   y: i32,
                   w: u32,
                   h: u32,
                   win_w: f32,
                   win_h: f32,
                   flip_h: bool)
                   -> (Vec<f32>, Vec<u32>) {
    let left = x as f32;
    let right = (x + w as i32) as f32;
    let top = win_h - y as f32;
    let bottom = win_h - (y + h as i32) as f32;

    let vertices: Vec<f32> = vec![(2.0 * left / win_w) - 1.0,
                                  (2.0 * bottom / win_h) - 1.0,
                                  0.0,
                                  if flip_h { 1.0 } else { 0.0 },
                                  (2.0 * right / win_w) - 1.0,
                                  (2.0 * bottom / win_h) - 1.0,
                                  1.0,
                                  if flip_h { 1.0 } else { 0.0 },
                                  (2.0 * right / win_w) - 1.0,
                                  (2.0 * top / win_h) - 1.0,
                                  1.0,
                                  if flip_h { 0.0 } else { 1.0 },
                                  (2.0 * left / win_w) - 1.0,
                                  (2.0 * top / win_h) - 1.0,
                                  0.0,
                                  if flip_h { 0.0 } else { 1.0 }];
    let indices: Vec<u32> = vec![0, 1, 3, 1, 2, 3];

    (vertices, indices)
}

fn new_cstring_with_len(len: usize) -> CString {
    // allocate sufficiently sized buffer
    let mut buffer: Vec<u8> = Vec::with_capacity(len + 1);
    // fill with spaces
    buffer.extend([b' '].iter().cycle().take(len));
    // convert to CString
    unsafe { CString::from_vec_unchecked(buffer) }
}
