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
    swap_tex: (GLuint, GLuint),
    copy_program: Program,
    down_program: Program,
    up_program: Program,
}

impl BlurContext {
    pub fn new(vp_size: (u32, u32)) -> Self {
        // init framebuffer for background rendering
        let mut fbo: GLuint = 0;
        unsafe {
            gl::GenFramebuffers(1, &mut fbo);
        }

        // init temporary swap textures
        let tex1 = crate::renderer_gl::create_texture(vp_size.0, vp_size.1);
        let tex2 = crate::renderer_gl::create_texture(vp_size.0, vp_size.1);

        // init shader and program
        let vert_shader = VertexShader::from_source(include_str!("shaders/tex_quad.vert"))
            .expect("Cannot compile copy vertex shader");
        let frag_shader =
            FragmentShader::from_source(include_str!("shaders/tex_quad.frag"))
                .expect("Cannot compile copy fragment shader");
        let copy_program = Program::from_shaders(&[vert_shader.into(), frag_shader.into()], None)
            .expect("Cannot link copy program");

        let down_vert_shader = VertexShader::from_source(include_str!("shaders/dual_kawase_down.vert"))
           .expect("Cannot compile downsample vertex shader");
        let down_frag_shader =
           FragmentShader::from_source(include_str!("shaders/dual_kawase_down.frag"))
               .expect("Cannot compile downsample fragment shader");
        let down_program = Program::from_shaders(&[down_vert_shader.into(), down_frag_shader.into()],
                                                 Some(&["iteration", "halfpixel", "offset"]))
            .expect("Cannot link downsample program");

        let up_vert_shader = VertexShader::from_source(include_str!("shaders/dual_kawase_up.vert"))
           .expect("Cannot compile upsample vertex shader");
        let up_frag_shader =
           FragmentShader::from_source(include_str!("shaders/dual_kawase_up.frag"))
               .expect("Cannot compile upsample fragment shader");
        let up_program = Program::from_shaders(&[up_vert_shader.into(), up_frag_shader.into()],
                                               Some(&["iteration", "halfpixel", "offset", "opacity"]))
            .expect("Cannot link upsample program");

        Self { iterations: 0,
               offset: 0,
               fbo,
               swap_tex: (tex1, tex2),
               copy_program,
               down_program,
               up_program }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        crate::renderer_gl::resize_texture(self.swap_tex.0, width, height);
        crate::renderer_gl::resize_texture(self.swap_tex.1, width, height);
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

    fn bind_fbo(&self, tgt_tex: GLuint) -> Result<(), GLuint> {
        unsafe {
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, self.fbo);
            gl::FramebufferTexture2D(gl::DRAW_FRAMEBUFFER,
                                     gl::COLOR_ATTACHMENT0,
                                     gl::TEXTURE_2D,
                                     tgt_tex,
                                     0);
            gl::DrawBuffer(gl::COLOR_ATTACHMENT0);
        }

        // check attachment status
        let status = unsafe { gl::CheckFramebufferStatus(gl::DRAW_FRAMEBUFFER) };
        if status != gl::FRAMEBUFFER_COMPLETE {
            return Err(status);
        }

        Ok(())
    }

    fn unbind_fbo(&self) {
        unsafe {
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, 0);
            gl::DrawBuffer(gl::BACK);
        }
    }

    fn copy(&mut self, rect: &mut Quad, src: &mut Texture, tgt: GLuint) {
        self.bind_fbo(tgt).expect("Failed to activate framebuffer");
        self.copy_program.activate();

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
            src.gl_bind_texture();
        }
        rect.draw(false);
        unsafe {
            src.gl_unbind_texture();
        }

        self.copy_program.unbind();
        self.unbind_fbo();
    }

    pub fn blur(&mut self, src_tex: &mut Texture, tgt_tex: &GLQuad) {
        let src_width = src_tex.query().width;
        let src_height = src_tex.query().height;

        let vp = Viewport::from_window(tgt_tex.width(), tgt_tex.height());
        let mut quad = Quad::new(0, 0, src_width, src_height, vp.size(), true, true);

        vp.activate();

        if self.iterations() == 0 {
            self.copy(&mut quad, src_tex, *tgt_tex.texture());
        } else {
            // Downsample
            let mut tgt_width = src_width as f32 / 2.0;
            let mut tgt_height = src_height as f32 / 2.0;

            self.down_program.activate();
            self.down_program
                .set_uniform_1i("iteration", 1)
                .expect("Cannot set downsample uniform");
            self.down_program
                .set_uniform_2f("halfpixel", (0.5 / tgt_width, 0.5 / tgt_height))
                .expect("Cannot set downsample uniform");
            self.down_program
                .set_uniform_1f("offset", self.offset() as f32)
                .expect("Cannot set downsample uniform");

            self.bind_fbo(self.swap_tex.0)
                .expect("Failed to activate framebuffer");

            unsafe {
                gl::Clear(gl::COLOR_BUFFER_BIT);
                src_tex.gl_bind_texture();
            }
            quad.draw(false);
            unsafe {
                src_tex.gl_unbind_texture();
            }

            self.down_program.unbind();
            self.unbind_fbo();

            // Upsample
            tgt_width = tgt_tex.width() as f32;
            tgt_height = tgt_tex.height() as f32;

            self.up_program.activate();
            self.up_program
                .set_uniform_1i("iteration", 1)
                .expect("Cannot set upsample uniform");
            self.up_program
                .set_uniform_2f("halfpixel", (0.5 / tgt_width, 0.5 / tgt_height))
                .expect("Cannot set upsample uniform");
            self.up_program
                .set_uniform_1f("offset", self.offset() as f32)
                .expect("Cannot set upsample uniform");
            self.up_program
                .set_uniform_1f("opacity", 1.0)
                .expect("Cannot set upsample uniform");

            self.bind_fbo(*tgt_tex.texture())
                .expect("Failed to activate framebuffer");

            unsafe {
                gl::Clear(gl::COLOR_BUFFER_BIT);
                gl::BindTexture(gl::TEXTURE_2D, self.swap_tex.0);
            }
            quad.draw(false);
            unsafe {
                gl::BindTexture(gl::TEXTURE_2D, 0);
            }

            self.up_program.unbind();
            self.unbind_fbo();
        }
    }
}
