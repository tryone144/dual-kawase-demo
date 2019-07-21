// This file is part auf the dual-kawase-demo package.
//
// (c) 2019 Bernd Busse
//
// For the full copyright and license information, please view the README.md file
// that was distributed with this source code.
//

use cpu_time::ProcessTime;
use gl::types::{GLint, GLuint};

use crate::renderer_gl::{FragmentShader, GLQuad, ImgSurface, Program, Quad, TextureQuad,
                         VertexShader, Viewport};

pub const MAX_ITERATIONS: usize = 8;

pub struct Framebuffer {
    fbo: GLuint,
    tex: GLuint,
    size: (u32, u32),
}

impl Framebuffer {
    pub fn from_fbo(fbo: GLuint) -> Self {
        Self { fbo,
               tex: 0,
               size: (0, 0) }
    }

    pub fn attach_texture(&mut self, tex: GLuint) -> Result<(), GLuint> {
        self.tex = tex;

        unsafe {
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, self.fbo);
            gl::FramebufferTexture2D(gl::DRAW_FRAMEBUFFER,
                                     gl::COLOR_ATTACHMENT0,
                                     gl::TEXTURE_2D,
                                     self.tex,
                                     0);
        }

        // check attachment status
        let status = unsafe { gl::CheckFramebufferStatus(gl::DRAW_FRAMEBUFFER) };
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        self.size = crate::renderer_gl::get_texture_size(self.tex);

        if status != gl::FRAMEBUFFER_COMPLETE {
            return Err(status);
        }

        Ok(())
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        crate::renderer_gl::resize_texture(self.tex, width, height, None);
        self.size = (width, height);
    }

    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    pub fn bind_fbo(&self) {
        unsafe {
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, self.fbo);
            gl::DrawBuffer(gl::COLOR_ATTACHMENT0);
        }
    }

    pub fn unbind_fbo(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl::DrawBuffer(gl::BACK);
        }
    }

    pub fn bind_tex(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.tex);
        }
    }

    pub fn unbind_tex(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }
}

pub struct BlurContext {
    iterations: u32,
    offset: f32,
    framebuffers: Vec<Framebuffer>,
    copy_program: Program,
    down_program: Program,
    up_program: Program,
    time_cpu: u128,
    time_gpu: u64,
}

impl BlurContext {
    pub fn new(vp_size: (u32, u32)) -> Self {
        // init framebuffers for background rendering
        let mut fbos: Vec<GLuint> = Vec::with_capacity(MAX_ITERATIONS + 1);
        unsafe {
            gl::GenFramebuffers(MAX_ITERATIONS as i32 + 1, fbos.as_mut_ptr() as *mut GLuint);
            fbos.set_len(MAX_ITERATIONS + 1);
        }

        let mut framebuffers = Vec::with_capacity(MAX_ITERATIONS);
        framebuffers.push(Framebuffer::from_fbo(fbos[0]));

        // init framebuffers with target textures
        for (i, fbo) in fbos.iter().enumerate().skip(1) {
            let mut fb = Framebuffer::from_fbo(*fbo);
            let tex = crate::renderer_gl::create_texture(vp_size.0 / (1 << i),
                                                         vp_size.1 / (1 << i),
                                                         None);

            fb.attach_texture(tex)
              .expect("Failed to attach texture to framebuffer");
            framebuffers.push(fb);
        }

        // init shader and program
        let vert_shader = VertexShader::from_source(include_str!("shaders/tex_quad.vert"))
            .expect("Cannot compile copy vertex shader");
        let frag_shader = FragmentShader::from_source(include_str!("shaders/tex_quad.frag"))
            .expect("Cannot compile copy fragment shader");
        let copy_program = Program::from_shaders(&[vert_shader.into(), frag_shader.into()], None)
            .expect("Cannot link copy program");

        let down_vert_shader =
            VertexShader::from_source(include_str!("shaders/dual_kawase_down.vert"))
                .expect("Cannot compile downsample vertex shader");
        let down_frag_shader =
            FragmentShader::from_source(include_str!("shaders/dual_kawase_down.frag"))
                .expect("Cannot compile downsample fragment shader");
        let down_program = Program::from_shaders(
            &[down_vert_shader.into(), down_frag_shader.into()],
            Some(&["iteration", "halfpixel", "offset"]),
        )
        .expect("Cannot link downsample program");

        let up_vert_shader = VertexShader::from_source(include_str!("shaders/dual_kawase_up.vert"))
            .expect("Cannot compile upsample vertex shader");
        let up_frag_shader =
            FragmentShader::from_source(include_str!("shaders/dual_kawase_up.frag"))
                .expect("Cannot compile upsample fragment shader");
        let up_program = Program::from_shaders(
            &[up_vert_shader.into(), up_frag_shader.into()],
            Some(&["iteration", "halfpixel", "offset", "opacity"]),
        )
        .expect("Cannot link upsample program");

        Self { iterations: 0,
               offset: 0.0,
               framebuffers,
               copy_program,
               down_program,
               up_program,
               time_cpu: 0,
               time_gpu: 0 }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        // resize target textures
        for (i, fb) in self.framebuffers.iter_mut().enumerate().skip(1) {
            fb.resize(width / (1 << i), height / (1 << i));
        }
    }

    pub fn iterations(&self) -> u32 {
        self.iterations
    }

    pub fn set_iterations(&mut self, iterations: u32) {
        self.iterations = iterations;
    }

    pub fn inc_iterations(&mut self, iter_delta: i32) {
        self.iterations = (self.iterations as i32 + iter_delta) as u32;
    }

    pub fn offset(&self) -> f32 {
        self.offset
    }

    pub fn set_offset(&mut self, offset: f32) {
        self.offset = offset;
    }

    pub fn inc_offset(&mut self, off_delta: f32) {
        self.offset += off_delta;
    }

    pub fn time_cpu(&self) -> f32 {
        (self.time_cpu as f64 / 1000f64).round() as f32 / 1000.0
    }

    pub fn time_gpu(&self) -> f32 {
        (self.time_gpu as f64 / 1000f64).round() as f32 / 1000.0
    }

    fn copy(&mut self, rect: &mut Quad, src: GLuint, tgt: GLuint) {
        let fb = &mut self.framebuffers[0];
        fb.attach_texture(tgt)
          .expect("Failed to attach target texture to framebuffer");
        fb.bind_fbo();
        self.copy_program.activate();

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::BindTexture(gl::TEXTURE_2D, src);
        }
        rect.draw(false);
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }

        self.copy_program.unbind();
        fb.unbind_fbo();
    }

    pub fn blur(&mut self, source_tex: &ImgSurface, target_quad: &GLQuad) {
        let (src_width, src_height) = source_tex.size();

        let vp = Viewport::from_window(target_quad.width(), target_quad.height());
        let mut quad = Quad::new(0, 0, src_width, src_height, vp.size(), true, true);

        vp.activate();

        // Start timer
        let mut gpu_time_query: GLuint = 0;
        unsafe {
            gl::GenQueries(1, &mut gpu_time_query);
            gl::BeginQuery(gl::TIME_ELAPSED, gpu_time_query);
        }

        let cpu_time_start = ProcessTime::now();

        if self.iterations() == 0 {
            self.copy(&mut quad, source_tex.texture(), *target_quad.texture());
        } else {
            // Attach target texture to framebuffer
            self.framebuffers[0].attach_texture(*target_quad.texture())
                                .expect("Failed to attach target texture to framebuffer");

            // Downsample
            self.down_program.activate();
            self.down_program
                .set_uniform_1f("offset", self.offset() as f32)
                .expect("Cannot set downsample uniform");

            for iteration in 0..MAX_ITERATIONS.min(self.iterations() as usize) {
                let (tgt_width, tgt_height) = self.framebuffers[iteration + 1].size();

                self.down_program
                    .set_uniform_1i("iteration", iteration as i32)
                    .expect("Cannot set downsample uniform");
                self.down_program
                    .set_uniform_2f("halfpixel",
                                    (0.5 / tgt_width as f32, 0.5 / tgt_height as f32))
                    .expect("Cannot set downsample uniform");

                self.framebuffers[iteration + 1].bind_fbo();

                if iteration == 0 {
                    // first iteration: copy from source
                    unsafe {
                        gl::BindTexture(gl::TEXTURE_2D, source_tex.texture());
                    }
                } else {
                    // copy from last iteration
                    self.framebuffers[iteration].bind_tex();
                }
                unsafe {
                    gl::Clear(gl::COLOR_BUFFER_BIT);
                }

                // draw texture to fbo
                quad.draw(false);
            }

            self.down_program.unbind();

            // Upsample
            self.up_program.activate();
            self.up_program
                .set_uniform_1f("offset", self.offset() as f32)
                .expect("Cannot set upsample uniform");
            self.up_program
                .set_uniform_1f("opacity", 1.0)
                .expect("Cannot set upsample uniform");

            for iteration in (0..MAX_ITERATIONS.min(self.iterations() as usize)).rev() {
                let (tgt_width, tgt_height) = self.framebuffers[iteration].size();

                self.up_program
                    .set_uniform_1i("iteration", iteration as i32)
                    .expect("Cannot set upsample uniform");
                self.up_program
                    .set_uniform_2f("halfpixel",
                                    (0.5 / tgt_width as f32, 0.5 / tgt_height as f32))
                    .expect("Cannot set upsample uniform");
                // self.up_program
                //    .set_uniform_1f("opacity", 1.0)
                //    .expect("Cannot set upsample uniform");

                self.framebuffers[iteration].bind_fbo();

                // draw texture to fbo
                unsafe {
                    gl::Clear(gl::COLOR_BUFFER_BIT);
                }
                self.framebuffers[iteration + 1].bind_tex();
                quad.draw(false);
            }

            self.framebuffers[0].unbind_fbo();
            self.framebuffers[0].unbind_tex();
            self.up_program.unbind();
        }

        // Stop timer
        self.time_cpu = cpu_time_start.elapsed().as_nanos();

        unsafe {
            gl::EndQuery(gl::TIME_ELAPSED);
        }

        let mut result_avail: GLint = 0;
        unsafe {
            while result_avail == 0 {
                gl::GetQueryObjectiv(gpu_time_query,
                                     gl::QUERY_RESULT_AVAILABLE,
                                     &mut result_avail);
            }
            gl::GetQueryObjectui64v(gpu_time_query, gl::QUERY_RESULT, &mut self.time_gpu);
            gl::DeleteQueries(1, &gpu_time_query);
        }
    }
}
