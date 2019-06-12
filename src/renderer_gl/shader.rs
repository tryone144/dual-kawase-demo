// This file is part auf the dual-kawase-demo package.
//
// (c) 2019 Bernd Busse
//
// For the full copyright and license information, please view the README.md file
// that was distributed with this source code.
//

use gl::types::{GLchar, GLenum, GLint, GLuint};
use std::ffi::{CStr, CString};

pub struct Program {
    id: GLuint,
}

impl Program {
    pub fn from_shaders(shaders: &[GlShader]) -> Result<Self, String> {
        // generate new program id
        let program_id = unsafe { gl::CreateProgram() };

        // attack supplied shaders
        for shader in shaders {
            unsafe {
                gl::AttachShader(program_id, shader.id());
            }
        }

        // link program
        unsafe {
            gl::LinkProgram(program_id);
        }

        // detach supplied shaders to allow them to be freed
        for shader in shaders {
            unsafe {
                gl::DetachShader(program_id, shader.id());
            }
        }

        // check error
        let mut success: GLint = 1;
        unsafe {
            gl::GetProgramiv(program_id, gl::LINK_STATUS, &mut success);
        }

        if success == 0 {
            // `gl::LinkProgram` failed
            let mut len: GLint = 0;
            let error = super::new_cstring_with_len(len as usize);

            unsafe {
                gl::GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut len);
                gl::GetProgramInfoLog(program_id,
                                      len,
                                      std::ptr::null_mut(),
                                      error.as_ptr() as *mut GLchar);
            }

            return Err(error.to_string_lossy().into_owned());
        }

        Ok(Self { id: program_id })
    }

    pub fn _id(&self) -> GLuint {
        self.id
    }

    pub fn activate(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    // pub fn set_uniform(&self, name: &str, value: (f32, f32)) {
    //    let uniform: GLint =
    //        unsafe { gl::GetUniformLocation(self.id(), name.as_ptr() as *const GLchar) };

    //    unsafe {
    //        gl::Uniform2f(uniform, value.0, value.1);
    //    }
    //}
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

pub trait ShaderType {
    const SHADER_TYPE: GLuint;
}

pub struct Shader<S: ShaderType> {
    id: GLuint,
    _marker: std::marker::PhantomData<S>,
}

impl<S: ShaderType> Shader<S> {
    pub fn from_source(source: &str) -> Result<Self, String> {
        // convert source into c-string
        let raw_source = CString::new(source).unwrap();

        // compile shader
        let id = shader_from_source(raw_source.as_c_str(), S::SHADER_TYPE)?;

        Ok(Self { id,
                  _marker: std::marker::PhantomData })
    }

    fn id(&self) -> GLuint {
        self.id
    }
}

impl<S: ShaderType> Drop for Shader<S> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.id);
        }
    }
}

pub struct ShaderTypeVertex;
impl ShaderType for ShaderTypeVertex {
    const SHADER_TYPE: GLuint = gl::VERTEX_SHADER;
}

pub struct ShaderTypeFragment;
impl ShaderType for ShaderTypeFragment {
    const SHADER_TYPE: GLuint = gl::FRAGMENT_SHADER;
}

pub type VertexShader = Shader<ShaderTypeVertex>;
pub type FragmentShader = Shader<ShaderTypeFragment>;

pub enum GlShader {
    Vertex(VertexShader),
    Fragment(FragmentShader),
}

impl GlShader {
    pub fn id(&self) -> GLuint {
        match self {
            GlShader::Vertex(shader) => shader.id(),
            GlShader::Fragment(shader) => shader.id(),
        }
    }
}

impl From<VertexShader> for GlShader {
    fn from(shader: VertexShader) -> Self {
        GlShader::Vertex(shader)
    }
}

impl From<FragmentShader> for GlShader {
    fn from(shader: FragmentShader) -> Self {
        GlShader::Fragment(shader)
    }
}

fn shader_from_source(source: &CStr, kind: GLenum) -> Result<GLuint, String> {
    // generate new shader id
    let id: GLuint = unsafe { gl::CreateShader(kind) };

    // compile shader
    unsafe {
        gl::ShaderSource(id, 1, &source.as_ptr(), std::ptr::null());
        gl::CompileShader(id);
    }

    // check error
    let mut success: GLint = 1;
    unsafe {
        gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
    }

    if success == 0 {
        // `gl::CompileShader` failed
        let mut len: GLint = 0;
        let error = super::new_cstring_with_len(len as usize);

        unsafe {
            gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
            gl::GetShaderInfoLog(id, len, std::ptr::null_mut(), error.as_ptr() as *mut GLchar);
        }

        return Err(error.to_string_lossy().into_owned());
    }

    Ok(id)
}
