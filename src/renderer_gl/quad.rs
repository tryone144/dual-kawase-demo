// This file is part auf the dual-kawase-demo package.
//
// (c) 2019 Bernd Busse
//
// For the full copyright and license information, please view the README.md file
// that was distributed with this source code.
//

use std::ops::{Deref, DerefMut};

use gl::types::{GLint, GLuint, GLvoid};

use super::{ArrayBuffer, ElementArrayBuffer, VertexArray};

pub struct Quad {
    width: u32,
    height: u32,
    x: i32,
    y: i32,
    flip_horiz: bool,
    vertices: Vec<f32>,
    indices: Vec<u32>,
    vbo: ArrayBuffer,
    ebo: ElementArrayBuffer,
    vao: VertexArray,
}

impl Quad {
    pub fn new(x: i32,
               y: i32,
               width: u32,
               height: u32,
               vp_size: (u32, u32),
               center: bool,
               flip_horiz: bool)
               -> Self {
        // Create vertex and index arrays
        let (vertices, indices) = if center {
            crate::utils::centered_quad_keep_aspect(width as f32,
                                                    height as f32,
                                                    vp_size.0 as f32,
                                                    vp_size.1 as f32,
                                                    flip_horiz)
        } else {
            crate::utils::quad_at_pos(x,
                                      y,
                                      width,
                                      height,
                                      vp_size.0 as f32,
                                      vp_size.1 as f32,
                                      flip_horiz)
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
            gl::VertexAttribPointer(0,
                                    2,
                                    gl::FLOAT,
                                    gl::FALSE,
                                    (4 * std::mem::size_of::<f32>()) as GLint,
                                    std::ptr::null());
            gl::VertexAttribPointer(1,
                                    2,
                                    gl::FLOAT,
                                    gl::FALSE,
                                    (4 * std::mem::size_of::<f32>()) as GLint,
                                    (2 * std::mem::size_of::<f32>()) as *const GLvoid);
        }
        vbo.unbind();
        vao.unbind();

        Self { width,
               height,
               x,
               y,
               flip_horiz,
               vertices,
               indices,
               vbo,
               ebo,
               vao }
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
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

    pub fn update_vp(&mut self, vp_size: (u32, u32)) {
        let (vertices, indices) = crate::utils::quad_at_pos(self.x,
                                                            self.y,
                                                            self.width,
                                                            self.height,
                                                            vp_size.0 as f32,
                                                            vp_size.1 as f32,
                                                            self.flip_horiz);

        self.vertices = vertices;
        self.indices = indices;
        self.refresh_buffers();
    }

    pub fn fit_center(&mut self, vp_size: (u32, u32)) {
        let (vertices, indices) = crate::utils::centered_quad_keep_aspect(self.width as f32,
                                                                          self.height as f32,
                                                                          vp_size.0 as f32,
                                                                          vp_size.1 as f32,
                                                                          self.flip_horiz);

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

            gl::DrawElements(gl::TRIANGLES,
                             self.indices.len() as i32,
                             gl::UNSIGNED_INT,
                             self.indices.as_ptr() as *const GLvoid);

            gl::DisableVertexAttribArray(0);
            gl::DisableVertexAttribArray(1);
        }
        self.vao.unbind();

        unsafe {
            gl::Disable(gl::BLEND);
        }
    }
}

pub trait TextureQuad<T> {
    fn from_texture(tex: T, x: i32, y: i32, vp_size: (u32, u32)) -> Self;
    fn texture(&self) -> &T;
    fn update_texture(&mut self, tex: T, vp_size: (u32, u32));
    fn draw(&mut self, blend: bool);
}

pub struct GLQuad {
    texture: GLuint,
    quad: Quad,
}

impl GLQuad {
    pub fn new_with_texture(x: i32, y: i32, width: u32, height: u32, vp_size: (u32, u32)) -> Self {
        let texture = super::create_texture_bgra(width, height, None);
        let quad = Quad::new(x, y, width as u32, height as u32, vp_size, false, false);

        Self { texture, quad }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        super::resize_texture_bgra(self.texture, width, height, None);
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
        let (width, height) = super::get_texture_size(tex);
        let quad = Quad::new(x, y, width, height, vp_size, false, true);
        Self { texture: tex, quad }
    }

    fn texture(&self) -> &GLuint {
        &self.texture
    }

    fn update_texture(&mut self, tex: GLuint, vp_size: (u32, u32)) {
        self.texture = tex;
        let (width, height) = super::get_texture_size(self.texture);

        self.quad.resize(width, height);
        self.update_vp(vp_size);
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
