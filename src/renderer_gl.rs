// This file is part auf the dual-kawase-demo package.
//
// (c) 2019 Bernd Busse
//
// For the full copyright and license information, please view the README.md file
// that was distributed with this source code.
//

use std::ffi::CString;

mod buffer;
mod quad;
mod shader;
mod viewport;

pub use self::buffer::{ArrayBuffer, ElementArrayBuffer, VertexArray};
pub use self::quad::{GLQuad, Quad, SDLQuad, TextureQuad};
pub use self::shader::{FragmentShader, GlShader, Program, VertexShader};
pub use self::viewport::Viewport;

#[inline]
pub fn centered_quad_keep_aspect(width: f32,
                                 height: f32,
                                 win_w: f32,
                                 win_h: f32)
                                 -> (Vec<f32>, Vec<u32>) {
    let vertices: Vec<f32> = vec![-(width / win_w),
                                  -(height / win_h),
                                  0.0,
                                  0.0,
                                  (width / win_w),
                                  -(height / win_h),
                                  1.0,
                                  0.0,
                                  (width / win_w),
                                  (height / win_h),
                                  1.0,
                                  1.0,
                                  -(width / win_w),
                                  (height / win_h),
                                  0.0,
                                  1.0,];
    let indices: Vec<u32> = vec![0, 1, 3, 1, 2, 3];

    (vertices, indices)
}

#[inline]
pub fn quad_at_pos(x: i32, y: i32, w: u32, h: u32, win_w: f32, win_h: f32) -> (Vec<f32>, Vec<u32>) {
    let left = x as f32;
    let right = (x + w as i32) as f32;
    let top = win_h - y as f32;
    let bottom = win_h - (y + h as i32) as f32;

    let vertices: Vec<f32> = vec![(2.0 * left / win_w) - 1.0,
                                  (2.0 * bottom / win_h) - 1.0,
                                  0.0,
                                  1.0, // 0.0,
                                  (2.0 * right / win_w) - 1.0,
                                  (2.0 * bottom / win_h) - 1.0,
                                  1.0,
                                  1.0, // 0.0,
                                  (2.0 * right / win_w) - 1.0,
                                  (2.0 * top / win_h) - 1.0,
                                  1.0,
                                  0.0, // 1.0,
                                  (2.0 * left / win_w) - 1.0,
                                  (2.0 * top / win_h) - 1.0,
                                  0.0,
                                  0.0, /* 1.0, */];
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
