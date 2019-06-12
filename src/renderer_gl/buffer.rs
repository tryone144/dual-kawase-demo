// This file is part auf the dual-kawase-demo package.
//
// (c) 2019 Bernd Busse
//
// For the full copyright and license information, please view the README.md file
// that was distributed with this source code.
//

use gl::types::{GLenum, GLsizeiptr, GLuint, GLvoid};

pub trait BufferType {
    const BUFFER_TYPE: GLuint;
}

pub struct Buffer<B: BufferType> {
    vbo: GLuint,
    _marker: std::marker::PhantomData<B>,
}

impl<B: BufferType> Buffer<B> {
    pub fn new() -> Self {
        let mut vbo: GLuint = 0;
        unsafe {
            gl::GenBuffers(1, &mut vbo);
        }

        Self { vbo,
               _marker: std::marker::PhantomData }
    }

    pub fn set_data<T>(&mut self, data: &[T], kind: GLenum) {
        unsafe {
            gl::BufferData(B::BUFFER_TYPE,
                           (data.len() * std::mem::size_of::<T>()) as GLsizeiptr,
                           data.as_ptr() as *const GLvoid,
                           kind);
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindBuffer(B::BUFFER_TYPE, self.vbo);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindBuffer(B::BUFFER_TYPE, 0);
        }
    }
}

impl<B: BufferType> Drop for Buffer<B> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.vbo);
        }
    }
}

pub struct BufferTypeArray;
impl BufferType for BufferTypeArray {
    const BUFFER_TYPE: GLuint = gl::ARRAY_BUFFER;
}

pub struct BufferTypeElementArray;
impl BufferType for BufferTypeElementArray {
    const BUFFER_TYPE: GLuint = gl::ELEMENT_ARRAY_BUFFER;
}

pub type ArrayBuffer = Buffer<BufferTypeArray>;
pub type ElementArrayBuffer = Buffer<BufferTypeElementArray>;

pub struct VertexArray {
    vao: GLuint,
}

impl VertexArray {
    pub fn new() -> Self {
        let mut vao: GLuint = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
        }

        Self { vao }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.vao);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindVertexArray(0);
        }
    }
}

impl Drop for VertexArray {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}
