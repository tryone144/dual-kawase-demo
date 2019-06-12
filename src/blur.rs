// This file is part auf the dual-kawase-demo package.
//
// (c) 2019 Bernd Busse
//
// For the full copyright and license information, please view the README.md file
// that was distributed with this source code.
//

use gl::types::GLuint;
use sdl2::render::Texture;

use crate::renderer_gl::{FragmentShader, GLQuad, Program, Quad, TextureQuad, VertexShader,
                         Viewport};

pub struct BlurContext {
    iterations: u32,
    offset: u32,
    fbo: GLuint,
    program: Program,
}

impl BlurContext {
    pub fn new() -> Self {
        // init framebuffer for background rendering
        let mut fbo: GLuint = 0;
        unsafe {
            gl::GenFramebuffers(1, &mut fbo);
        }

        // Init shader and program
        let vert_shader = VertexShader::from_source(include_str!("shaders/dual_kawase_blur.vert"))
            .expect("Cannot compile vertex shader");
        let frag_shader =
            FragmentShader::from_source(include_str!("shaders/dual_kawase_blur.frag"))
                .expect("Cannot compile fragment shader");

        let program = Program::from_shaders(&[vert_shader.into(), frag_shader.into()])
            .expect("Cannot link main program");

        Self { iterations: 0,
               offset: 0,
               fbo,
               program }
    }

    pub fn iterations(&self) -> u32 {
        self.iterations
    }

    pub fn set_iterations(&mut self, iterations: u32) {
        self.iterations = iterations;
    }

    pub fn offset(&self) -> u32 {
        self.offset
    }

    pub fn set_offset(&mut self, offset: u32) {
        self.offset = offset;
    }

    pub fn blur(&mut self, src_tex: &mut Texture, tgt_tex: &GLQuad) {
        let src_width = src_tex.query().width;
        let src_height = src_tex.query().height;

        let vp = Viewport::from_window(tgt_tex.width(), tgt_tex.height());
        let mut quad = Quad::new(0, 0, src_width, src_height, vp.size(), false);

        self.program.activate();
        unsafe {
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, self.fbo);
            gl::FramebufferTexture2D(gl::DRAW_FRAMEBUFFER,
                                     gl::COLOR_ATTACHMENT0,
                                     gl::TEXTURE_2D,
                                     *tgt_tex.texture(),
                                     0);
            // println!(
            //    "Status: {:?}",
            //    gl::CheckFramebufferStatus(gl::DRAW_FRAMEBUFFER) == gl::FRAMEBUFFER_COMPLETE
            //);
        }
        vp.activate();
        unsafe {
            gl::DrawBuffer(gl::COLOR_ATTACHMENT0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            src_tex.gl_bind_texture();
        }
        quad.draw(false);
        unsafe {
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, 0);
            gl::DrawBuffer(gl::BACK);
            src_tex.gl_unbind_texture();
        }
    }
}
