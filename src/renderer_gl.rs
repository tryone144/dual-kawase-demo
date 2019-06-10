// This file is part auf the dual-kawase-demo package.
//
// (c) 2019 Bernd Busse
//
// For the full copyright and license information, please view the README.md file
// that was distributed with this source code.
//

use gl::types::{GLchar, GLenum, GLint, GLuint};
use std::ffi::{CStr, CString};

pub trait GLid {
    fn id(&self) -> GLuint;
}

pub enum Shader {
    VertexShader(VertexShader),
    FragmentShader(FragmentShader),
}

impl From<VertexShader> for Shader {
    fn from(shader: VertexShader) -> Self {
        Shader::VertexShader(shader)
    }
}

impl From<FragmentShader> for Shader {
    fn from(shader: FragmentShader) -> Self {
        Shader::FragmentShader(shader)
    }
}

impl GLid for Shader {
    fn id(&self) -> GLuint {
        match self {
            Shader::VertexShader(shader) => shader.id(),
            Shader::FragmentShader(shader) => shader.id(),
        }
    }
}

pub struct VertexShader {
    id: GLuint,
}

impl VertexShader {
    pub fn from_source(source: &str) -> Result<Self, String> {
        // convert source into c-string
        let raw_source = CString::new(source).unwrap();

        // compile shader
        let id = shader_from_source(raw_source.as_c_str(), gl::VERTEX_SHADER)?;

        Ok(Self { id })
    }
}

impl GLid for VertexShader {
    fn id(&self) -> GLuint {
        self.id
    }
}

impl Drop for VertexShader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.id);
        }
    }
}

pub struct FragmentShader {
    id: GLuint,
}

impl FragmentShader {
    pub fn from_source(source: &str) -> Result<Self, String> {
        // convert source into c-string
        let raw_source = CString::new(source).unwrap();

        // compile shader
        let id = shader_from_source(raw_source.as_c_str(), gl::FRAGMENT_SHADER)?;

        Ok(Self { id })
    }
}

impl GLid for FragmentShader {
    fn id(&self) -> GLuint {
        self.id
    }
}

impl Drop for FragmentShader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.id);
        }
    }
}

pub struct Program {
    id: GLuint,
}

impl Program {
    pub fn from_shaders(shaders: &[Shader]) -> Result<Self, String> {
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
            let error = new_cstring_with_len(len as usize);

            unsafe {
                gl::GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut len);
                gl::GetProgramInfoLog(
                    program_id,
                    len,
                    std::ptr::null_mut(),
                    error.as_ptr() as *mut GLchar,
                );
            }

            return Err(error.to_string_lossy().into_owned());
        }

        Ok(Self { id: program_id })
    }

    pub fn id(&self) -> GLuint {
        self.id
    }

    pub fn activate(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

fn new_cstring_with_len(len: usize) -> CString {
    // allocate sufficiently sized buffer
    let mut buffer: Vec<u8> = Vec::with_capacity(len + 1);
    // fill with spaces
    buffer.extend([b' '].into_iter().cycle().take(len));
    // convert to CString
    unsafe { CString::from_vec_unchecked(buffer) }
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
        let error = new_cstring_with_len(len as usize);

        unsafe {
            gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
            gl::GetShaderInfoLog(id, len, std::ptr::null_mut(), error.as_ptr() as *mut GLchar);
        }

        return Err(error.to_string_lossy().into_owned());
    }

    Ok(id)
}
