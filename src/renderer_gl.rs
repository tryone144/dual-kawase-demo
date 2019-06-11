// This file is part auf the dual-kawase-demo package.
//
// (c) 2019 Bernd Busse
//
// For the full copyright and license information, please view the README.md file
// that was distributed with this source code.
//

use std::ffi::CString;

use gl::types::{GLint, GLvoid};
use sdl2::render::Texture;

mod buffer;
mod shader;
mod viewport;

pub use self::buffer::{ArrayBuffer, ElementArrayBuffer, VertexArray};
pub use self::shader::{FragmentShader, GlShader, Program, VertexShader};
pub use self::viewport::Viewport;

pub struct TextureQuad<'a> {
    texture: Texture<'a>,
    x: i32,
    y: i32,
    vbo: ArrayBuffer,
    ebo: ElementArrayBuffer,
    vao: VertexArray,
    vertices: Vec<f32>,
    indices: Vec<u32>,
}

impl<'a> TextureQuad<'a> {
    pub fn from_texture(base: Texture<'a>, vp_size: (u32, u32)) -> Self {
        // Create vertex and index arrays
        let (vertices, indices) = quad_at_pos(
            0,
            0,
            base.query().width,
            base.query().height,
            vp_size.0 as f32,
            vp_size.0 as f32,
        );

        // init vertex buffer object
        let mut vbo = ArrayBuffer::new();
        vbo.bind();
        vbo.set_data(&vertices, gl::STATIC_DRAW);
        vbo.unbind();

        // init element buffer object
        let mut ebo = ElementArrayBuffer::new();
        ebo.bind();
        ebo.set_data(&indices, gl::STATIC_DRAW);
        ebo.unbind();

        // init vertex array object
        let vao = VertexArray::new();
        vao.bind();
        vbo.bind();
        unsafe {
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0,
                2,
                gl::FLOAT,
                gl::FALSE,
                (4 * std::mem::size_of::<f32>()) as GLint,
                std::ptr::null(),
            );
            gl::DisableVertexAttribArray(0);

            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                (4 * std::mem::size_of::<f32>()) as GLint,
                (2 * std::mem::size_of::<f32>()) as *const GLvoid,
            );
            gl::DisableVertexAttribArray(1);
        }
        vbo.unbind();
        vao.unbind();

        Self {
            texture: base,
            x: 0,
            y: 0,
            vertices,
            indices,
            vbo,
            ebo,
            vao,
        }
    }

    fn refresh_buffers(&mut self) {
        self.vbo.bind();
        self.vbo.set_data(&self.vertices, gl::STATIC_DRAW);
        self.vbo.unbind();

        self.ebo.bind();
        self.ebo.set_data(&self.indices, gl::STATIC_DRAW);
        self.ebo.unbind();
    }

    pub fn update_texture(&mut self, texture: Texture<'a>) {
        self.texture = texture;
    }

    pub fn update_vp(&mut self, vp_size: (u32, u32)) {
        let (vertices, indices) = quad_at_pos(
            self.x,
            self.y,
            self.texture.query().width,
            self.texture.query().height,
            vp_size.0 as f32,
            vp_size.1 as f32,
        );

        self.vertices = vertices;
        self.indices = indices;
        self.refresh_buffers();
    }

    pub fn fit_center(&mut self, vp_size: (u32, u32)) {
        let (vertices, indices) = centered_quad_keep_aspect(
            self.texture.query().width as f32,
            self.texture.query().height as f32,
            vp_size.0 as f32,
            vp_size.1 as f32,
        );

        self.vertices = vertices;
        self.indices = indices;
        self.refresh_buffers();
    }

    pub fn draw(&mut self, blend: bool) {
        if blend {
            unsafe {
                gl::Enable(gl::BLEND);
                gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            }
        }

        self.vao.bind();
        unsafe {
            self.texture.gl_bind_texture();

            gl::EnableVertexAttribArray(0);
            gl::EnableVertexAttribArray(1);

            gl::DrawElements(
                gl::TRIANGLES,
                self.indices.len() as i32,
                gl::UNSIGNED_INT,
                self.indices.as_ptr() as *const GLvoid,
            );

            gl::DisableVertexAttribArray(0);
            gl::DisableVertexAttribArray(1);

            self.texture.gl_unbind_texture();
        }
        self.vao.unbind();
    }
}

#[inline]
pub fn centered_quad_keep_aspect(
    width: f32,
    height: f32,
    win_w: f32,
    win_h: f32,
) -> (Vec<f32>, Vec<u32>) {
    let vertices: Vec<f32> = vec![
        -(width / win_w),
        -(height / win_h),
        0.0,
        1.0,
        (width / win_w),
        -(height / win_h),
        1.0,
        1.0,
        (width / win_w),
        (height / win_h),
        1.0,
        0.0,
        -(width / win_w),
        (height / win_h),
        0.0,
        0.0,
    ];
    let indices: Vec<u32> = vec![0, 1, 3, 1, 2, 3];

    (vertices, indices)
}

#[inline]
pub fn quad_at_pos(x: i32, y: i32, w: u32, h: u32, win_w: f32, win_h: f32) -> (Vec<f32>, Vec<u32>) {
    let left = x as f32;
    let right = (x + w as i32) as f32;
    let top = win_h - (y + h as i32) as f32;
    let bottom = win_h - y as f32;

    let vertices: Vec<f32> = vec![
        (2.0 * left / win_w) - 1.0,
        (2.0 * bottom / win_h) - 1.0,
        0.0,
        0.0,
        (2.0 * right / win_w) - 1.0,
        (2.0 * bottom / win_h) - 1.0,
        1.0,
        0.0,
        (2.0 * right / win_w) - 1.0,
        (2.0 * top / win_h) - 1.0,
        1.0,
        1.0,
        (2.0 * left / win_w) - 1.0,
        (2.0 * top / win_h) - 1.0,
        0.0,
        1.0,
    ];
    let indices: Vec<u32> = vec![0, 1, 3, 1, 2, 3];

    (vertices, indices)
}

fn new_cstring_with_len(len: usize) -> CString {
    // allocate sufficiently sized buffer
    let mut buffer: Vec<u8> = Vec::with_capacity(len + 1);
    // fill with spaces
    buffer.extend([b' '].into_iter().cycle().take(len));
    // convert to CString
    unsafe { CString::from_vec_unchecked(buffer) }
}
