// This file is part auf the dual-kawase-demo package.
//
// (c) 2019 Bernd Busse
//
// For the full copyright and license information, please view the README.md file
// that was distributed with this source code.
//

use std::ffi::CString;
use std::ops::{Deref, DerefMut};

use gl::types::{GLint, GLuint, GLvoid};
use sdl2::render::Texture;

mod buffer;
mod shader;
mod viewport;

pub use self::buffer::{ArrayBuffer, ElementArrayBuffer, VertexArray};
pub use self::shader::{FragmentShader, GlShader, Program, VertexShader};
pub use self::viewport::Viewport;

pub struct Quad {
    width: u32,
    height: u32,
    x: i32,
    y: i32,
    vertices: Vec<f32>,
    indices: Vec<u32>,
    vbo: ArrayBuffer,
    ebo: ElementArrayBuffer,
    vao: VertexArray,
}

impl Quad {
    pub fn new(x: i32, y: i32, width: u32, height: u32, vp_size: (u32, u32), center: bool) -> Self {
        // Create vertex and index arrays
        let (vertices, indices) = if center {
            centered_quad_keep_aspect(
                width as f32,
                height as f32,
                vp_size.0 as f32,
                vp_size.1 as f32,
            )
        } else {
            quad_at_pos(x, y, width, height, vp_size.0 as f32, vp_size.1 as f32)
        };

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
            width,
            height,
            x,
            y,
            vertices,
            indices,
            vbo,
            ebo,
            vao,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    fn refresh_buffers(&mut self) {
        self.vbo.bind();
        self.vbo.set_data(&self.vertices, gl::STATIC_DRAW);
        self.vbo.unbind();

        self.ebo.bind();
        self.ebo.set_data(&self.indices, gl::STATIC_DRAW);
        self.ebo.unbind();
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        // FIXME: vertices are not updated
    }

    pub fn _update_pos(&mut self, x: i32, y: i32, vp_size: (u32, u32)) {
        self.x = x;
        self.y = y;
        self.update_vp(vp_size);
    }

    pub fn update_vp(&mut self, vp_size: (u32, u32)) {
        let (vertices, indices) = quad_at_pos(
            self.x,
            self.y,
            self.width,
            self.height,
            vp_size.0 as f32,
            vp_size.1 as f32,
        );

        self.vertices = vertices;
        self.indices = indices;
        self.refresh_buffers();
    }

    pub fn fit_center(&mut self, vp_size: (u32, u32)) {
        let (vertices, indices) = centered_quad_keep_aspect(
            self.width as f32,
            self.height as f32,
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
        }
        self.vao.unbind();
    }
}

pub trait TextureQuad<T> {
    fn from_texture(tex: T, x: i32, y: i32, vp_size: (u32, u32)) -> Self;
    fn texture(&self) -> &T;
    fn update_texture(&mut self, tex: T);
    fn draw(&mut self, blend: bool);
}

pub struct SDLQuad<'r> {
    texture: Texture<'r>,
    quad: Quad,
}

impl<'r> Deref for SDLQuad<'r> {
    type Target = Quad;

    fn deref(&self) -> &Self::Target {
        &self.quad
    }
}

impl<'r> DerefMut for SDLQuad<'r> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.quad
    }
}

impl<'r> TextureQuad<Texture<'r>> for SDLQuad<'r> {
    fn from_texture(tex: Texture<'r>, x: i32, y: i32, vp_size: (u32, u32)) -> Self {
        let quad = Quad::new(x, y, tex.query().width, tex.query().height, vp_size, false);
        Self { texture: tex, quad }
    }

    fn texture(&self) -> &Texture<'r> {
        &self.texture
    }

    fn update_texture(&mut self, tex: Texture<'r>) {
        self.texture = tex;
        self.quad
            .resize(self.texture.query().width, self.texture.query().height);
    }

    fn draw(&mut self, blend: bool) {
        unsafe {
            self.texture.gl_bind_texture();
        }
        self.quad.draw(blend);
        unsafe {
            self.texture.gl_unbind_texture();
        }
    }
}

pub struct GLQuad {
    texture: GLuint,
    quad: Quad,
}

impl GLQuad {
    pub fn with_texture(x: i32, y: i32, width: u32, height: u32, vp_size: (u32, u32)) -> Self {
        let mut texture: GLuint = 0;
        unsafe {
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA8 as i32,
                width as i32,
                height as i32,
                0,
                gl::BGRA,
                gl::UNSIGNED_BYTE,
                std::ptr::null(),
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAX_LEVEL, 0);
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }

        let quad = Quad::new(x, y, width as u32, height as u32, vp_size, false);
        Self { texture, quad }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.texture);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA8 as i32,
                width as i32,
                height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                std::ptr::null(),
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAX_LEVEL, 0);
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
        self.quad.resize(width, height);
    }
}

impl Deref for GLQuad {
    type Target = Quad;

    fn deref(&self) -> &Self::Target {
        &self.quad
    }
}

impl DerefMut for GLQuad {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.quad
    }
}

impl TextureQuad<GLuint> for GLQuad {
    fn from_texture(tex: GLuint, x: i32, y: i32, vp_size: (u32, u32)) -> Self {
        let mut width: GLint = 0;
        let mut height: GLint = 0;
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, tex);
            gl::GetTexParameteriv(gl::TEXTURE_2D, gl::TEXTURE_WIDTH, &mut width);
            gl::GetTexParameteriv(gl::TEXTURE_2D, gl::TEXTURE_HEIGHT, &mut height);
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }

        let quad = Quad::new(x, y, width as u32, height as u32, vp_size, false);
        Self { texture: tex, quad }
    }

    fn texture(&self) -> &GLuint {
        &self.texture
    }

    fn update_texture(&mut self, tex: GLuint) {
        self.texture = tex;
        let mut width: GLint = 0;
        let mut height: GLint = 0;
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.texture);
            gl::GetTexParameteriv(gl::TEXTURE_2D, gl::TEXTURE_WIDTH, &mut width);
            gl::GetTexParameteriv(gl::TEXTURE_2D, gl::TEXTURE_HEIGHT, &mut height);
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
        self.quad.resize(width as u32, height as u32);
    }

    fn draw(&mut self, blend: bool) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.texture);
        }
        self.quad.draw(blend);
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }
}

impl Drop for GLQuad {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.texture);
        }
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
        1.0,
    ];
    let indices: Vec<u32> = vec![0, 1, 3, 1, 2, 3];

    (vertices, indices)
}

#[inline]
pub fn quad_at_pos(x: i32, y: i32, w: u32, h: u32, win_w: f32, win_h: f32) -> (Vec<f32>, Vec<u32>) {
    let left = x as f32;
    let right = (x + w as i32) as f32;
    let top = win_h - y as f32;
    let bottom = win_h - (y + h as i32) as f32;

    let vertices: Vec<f32> = vec![
        (2.0 * left / win_w) - 1.0,
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
        0.0, // 1.0,
    ];
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
