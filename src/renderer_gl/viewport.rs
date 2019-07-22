// This file is part auf the dual-kawase-demo package.
//
// (c) 2019 Bernd Busse
//
// For the full copyright and license information, please view the README.md file
// that was distributed with this source code.
//

use crate::utils::Matrix4f;

pub struct Viewport {
    x: i32,
    y: i32,
    w: u32,
    h: u32,
}

impl Viewport {
    pub fn from_window(width: u32, height: u32) -> Self {
        Self { x: 0,
               y: 0,
               w: width,
               h: height }
    }

    pub fn width(&self) -> u32 {
        self.w
    }

    pub fn height(&self) -> u32 {
        self.h
    }

    pub fn size(&self) -> (u32, u32) {
        (self.w, self.h)
    }

    pub fn update_size(&mut self, width: u32, height: u32) {
        self.w = width;
        self.h = height;
    }

    pub fn transform(&self) -> Matrix4f {
        crate::utils::matrix4f_ortho(self.x as f32,
                                     (self.x + self.w as i32) as f32,
                                     self.y as f32,
                                     (self.y + self.h as i32) as f32,
                                     1.0,
                                     -1.0)
    }

    pub fn activate(&self) {
        unsafe {
            gl::Viewport(self.x, self.y, self.w as i32, self.h as i32);
        }
    }
}
