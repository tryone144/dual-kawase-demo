// This file is part auf the dual-kawase-demo package.
//
// (c) 2019 Bernd Busse
//
// For the full copyright and license information, please view the README.md file
// that was distributed with this source code.
//

use sdl2::render::{Texture, TextureCreator};
use sdl2::ttf::Font;
use sdl2::ttf::Hinting;
use sdl2::ttf::Sdl2TtfContext;

use crate::blur::BlurContext;
use crate::renderer_gl::{SDLQuad, TextureQuad};

const INFO_ITERATIONS: &str = "Down-/Upsample Iterations";
const INFO_OFFSET: &str = "Blur Offset";
const INFO_CPU: &str = "CPU Time";
const INFO_GPU: &str = "GPU Time";

pub struct InfoOverlay<'r> {
    font: Font<'r, 'r>,
    quad_iterations: SDLQuad<'r>,
    quad_offset: SDLQuad<'r>,
    quad_cpu: SDLQuad<'r>,
    quad_gpu: SDLQuad<'r>,
}

impl<'r> InfoOverlay<'r> {
    pub fn new<T>(ttf_ctx: &'r Sdl2TtfContext,
                  texture_creator: &'r TextureCreator<T>,
                  blur_ctx: &BlurContext,
                  vp_size: (u32, u32))
                  -> Self {
        // Init text rendering
        let mut font = ttf_ctx.load_font("./assets/UbuntuMono-R.ttf", 16)
                              .expect("Cannot open font");
        font.set_hinting(Hinting::Normal);

        // Create overlay TextureQuads
        let quad_iterations = {
            let overlay_tex =
                render_to_texture(texture_creator,
                                  &font,
                                  &format!("{}: {}", INFO_ITERATIONS, blur_ctx.iterations()));
            SDLQuad::from_texture(overlay_tex, 20, 20, vp_size)
        };
        let quad_offset = {
            let overlay_tex =
                render_to_texture(texture_creator,
                                  &font,
                                  &format!("{}: {:.02}", INFO_OFFSET, blur_ctx.offset()));
            SDLQuad::from_texture(overlay_tex,
                                  20,
                                  quad_iterations.height() as i32 + 20,
                                  vp_size)
        };
        let quad_gpu = {
            let overlay_tex =
                render_to_texture(texture_creator,
                                  &font,
                                  &format!("{}: {:6.03}ms", INFO_GPU, blur_ctx.time_gpu()));
            let tex_top = vp_size.1 as i32 - overlay_tex.query().height as i32;
            SDLQuad::from_texture(overlay_tex, 20, tex_top - 20, vp_size)
        };
        let quad_cpu = {
            let overlay_tex =
                render_to_texture(texture_creator,
                                  &font,
                                  &format!("{}: {:6.03}ms", INFO_CPU, blur_ctx.time_cpu()));
            let tex_top = quad_gpu.pos().1 - overlay_tex.query().height as i32;
            SDLQuad::from_texture(overlay_tex, 20, tex_top - 20, vp_size)
        };

        Self { font,
               quad_iterations,
               quad_offset,
               quad_cpu,
               quad_gpu }
    }

    pub fn resize(&mut self, vp_size: (u32, u32)) {
        self.quad_iterations.update_vp(vp_size);
        self.quad_offset.update_vp(vp_size);

        let gpu_top = vp_size.1 as i32 - self.quad_gpu.height() as i32;
        self.quad_gpu.update_pos(20, gpu_top - 20, vp_size);
        let cpu_top = self.quad_gpu.pos().1 - self.quad_cpu.height() as i32;
        self.quad_cpu.update_pos(20, cpu_top, vp_size);
    }

    pub fn update<T>(&mut self,
                     texture_creator: &'r TextureCreator<T>,
                     blur_ctx: &BlurContext,
                     vp_size: (u32, u32)) {
        // Update overlay textures
        let overlay_tex1 =
            render_to_texture(texture_creator,
                              &self.font,
                              &format!("{}: {}", INFO_ITERATIONS, blur_ctx.iterations()));
        self.quad_iterations.update_texture(overlay_tex1, vp_size);

        let overlay_tex2 =
            render_to_texture(texture_creator,
                              &self.font,
                              &format!("{}: {:.02}", INFO_OFFSET, blur_ctx.offset()));
        self.quad_offset.update_texture(overlay_tex2, vp_size);

        let overlay_tex4 =
            render_to_texture(texture_creator,
                              &self.font,
                              &format!("{}: {:6.03}ms", INFO_GPU, blur_ctx.time_gpu()));
        self.quad_gpu.update_texture(overlay_tex4, vp_size);

        let overlay_tex3 =
            render_to_texture(texture_creator,
                              &self.font,
                              &format!("{}: {:6.03}ms", INFO_CPU, blur_ctx.time_cpu()));
        self.quad_cpu.update_texture(overlay_tex3, vp_size);
    }

    pub fn draw(&mut self, blend: bool) {
        self.quad_iterations.draw(blend);
        self.quad_offset.draw(blend);
        self.quad_cpu.draw(blend);
        self.quad_gpu.draw(blend);
    }
}

#[inline]
fn render_to_texture<'r, T: 'r>(creator: &'r TextureCreator<T>,
                                font: &Font,
                                message: &str)
                                -> Texture<'r> {
    let text_surf = font.render(message)
                        .blended((255, 255, 255, 255))
                        .expect("Cannot render text to surface");
    creator.create_texture_from_surface(text_surf)
           .expect("Cannot convert surface to texture")
}
