// This file is part auf the dual-kawase-demo package.
//
// (c) 2019 Bernd Busse
//
// For the full copyright and license information, please view the README.md file
// that was distributed with this source code.
//

use std::ffi::CString;

#[inline]
pub fn scale_keep_aspect(base_w: u32, base_h: u32, width: u32, height: u32) -> (u32, u32) {
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
                                  if flip_h { 1.0 } else { 0.0 },];
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
                                  if flip_h { 0.0 } else { 1.0 },];
    let indices: Vec<u32> = vec![0, 1, 3, 1, 2, 3];

    (vertices, indices)
}

#[inline]
pub fn new_cstring_with_len(len: usize) -> CString {
    // allocate sufficiently sized buffer
    let mut buffer: Vec<u8> = Vec::with_capacity(len + 1);
    // fill with spaces
    buffer.extend([b' '].iter().cycle().take(len));
    // convert to CString
    unsafe { CString::from_vec_unchecked(buffer) }
}
