// This file is part auf the dual-kawase-demo package.
//
// (c) 2019 Bernd Busse
//
// For the full copyright and license information, please view the README.md file
// that was distributed with this source code.
//

use std::collections::HashMap;
use std::ffi::{CStr, CString};

use gl::types::{GLchar, GLenum, GLint, GLuint};

pub struct Program {
    id: GLuint,
    uniform_map: HashMap<String, GLint>,
}

impl Program {
    pub fn from_shaders(shaders: &[GlShader], uniforms: Option<&[&str]>) -> Result<Self, String> {
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

        // store uniform locations
        let mut uniform_map: HashMap<String, GLint> =
            HashMap::with_capacity(uniforms.map_or(0, |u| u.len()));

        if let Some(uniform_iter) = uniforms {
            for uniform in uniform_iter {
                let c_name = CString::new(*uniform).unwrap();
                let loc =
                    unsafe { gl::GetUniformLocation(program_id, c_name.as_ptr() as *const GLchar) };
                if loc < 0 {
                    return Err(format!("Cannot find location of uniform '{}'", *uniform));
                }
                uniform_map.insert(String::from(*uniform), loc);
            }
        }

        // detach supplied shaders to allow them to be freed
        for shader in shaders {
            unsafe {
                gl::DetachShader(program_id, shader.id());
            }
        }

        Ok(Self { id: program_id,
                  uniform_map })
    }

    pub fn _id(&self) -> GLuint {
        self.id
    }

    pub fn activate(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::UseProgram(0);
        }
    }

    pub fn set_uniform_1i(&self, name: &str, value: i32) -> Result<(), String> {
        let loc = self.uniform_map
                      .get(name)
                      .ok_or_else(|| format!("Uniform location '{}' not found", name))?;
        unsafe {
            gl::Uniform1i(*loc, value);
        }
        Ok(())
    }

    pub fn set_uniform_1f(&self, name: &str, value: f32) -> Result<(), String> {
        let loc = self.uniform_map
                      .get(name)
                      .ok_or_else(|| format!("Uniform location '{}' not found", name))?;
        unsafe {
            gl::Uniform1f(*loc, value);
        }
        Ok(())
    }

    pub fn set_uniform_2f(&self, name: &str, values: (f32, f32)) -> Result<(), String> {
        let loc = self.uniform_map
                      .get(name)
                      .ok_or_else(|| format!("Uniform location '{}' not found", name))?;
        unsafe {
            gl::Uniform2f(*loc, values.0, values.1);
        }
        Ok(())
    }
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
        unsafe {
            gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
        }

        let error = super::new_cstring_with_len(len as usize);
        unsafe {
            gl::GetShaderInfoLog(id, len, std::ptr::null_mut(), error.as_ptr() as *mut GLchar);
        }

        return Err(error.to_string_lossy().into_owned());
    }

    Ok(id)
}
