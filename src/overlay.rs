// This file is part auf the dual-kawase-demo package.
//
// (c) 2019 Bernd Busse
//
// For the full copyright and license information, please view the README.md file
// that was distributed with this source code.
//

use gl::types::{GLfloat, GLint, GLuint, GLvoid};
use glyph_brush::rusttype::{Point, Rect, Scale};
use glyph_brush::{BrushAction, BrushError, GlyphBrush, GlyphBrushBuilder, GlyphVertex,
                  HorizontalAlign, Layout, Section, VerticalAlign};

use crate::blur::BlurContext;
use crate::renderer_gl::{ArrayBuffer, FragmentShader, Program, VertexArray, VertexShader, Viewport};

const INFO_ITERATIONS: &str = "Down-/Upsample Iterations";
const INFO_OFFSET: &str = "Blur Offset";
const INFO_CPU: &str = "CPU Time";
const INFO_GPU: &str = "GPU Time";

type Vertex = [GLfloat; 13];

struct InfoSection {
    text: String,
    position: (f32, f32),
    layout: Layout<glyph_brush::BuiltInLineBreaker>,
}

pub struct InfoOverlay<'a> {
    brush: GlyphBrush<'a, Vertex>,
    glyph_tex: GLuint,
    max_tex_size: u32,
    vertex_count: usize,
    max_vertices: usize,
    program: Program,
    vbo: ArrayBuffer,
    vao: VertexArray,
    sec_defaults: Section<'a>,
    sec_params: InfoSection,
    sec_time: InfoSection,
}

impl<'a> InfoOverlay<'a> {
    pub fn new(blur_ctx: &BlurContext, vp: &Viewport) -> Self {
        // Init TTF glyph renderer
        let font_data: &[u8] = include_bytes!("../assets/UbuntuMono-R.ttf");
        let glyph_brush = GlyphBrushBuilder::using_font_bytes(font_data).build();

        // Create glyph cache texture
        let dimensions = glyph_brush.texture_dimensions();
        let glyph_tex = crate::renderer_gl::create_texture_red(dimensions.0, dimensions.1, None);

        let max_tex_size = {
            let mut value: GLint = 0;
            unsafe { gl::GetIntegerv(gl::MAX_TEXTURE_SIZE, &mut value) };
            value as u32
        };

        // Init GLSL shaders / program
        let vert_shader = VertexShader::from_source(include_str!("shaders/glyphs.vert"))
            .expect("Cannot compile glyph vertex shader");
        let frag_shader = FragmentShader::from_source(include_str!("shaders/glyphs.frag"))
            .expect("Cannot compile glyph fragment shader");
        let mut program =
            Program::from_shaders(&[vert_shader.into(), frag_shader.into()],
                                  Some(&["transform"])).expect("Cannot link glyph program");

        program.activate();
        program.set_uniform_mat4f("transform", &vp.transform())
               .expect("Cannot set 'transform' in glyph program");
        program.unbind();

        // init vertex buffer / array object
        let vbo = ArrayBuffer::new();
        let vao = VertexArray::new();
        vao.bind();
        vbo.bind();

        let mut offset = 0;
        for (attr, size) in (&[3, 2, 2, 2, 4]).iter().enumerate() {
            unsafe {
                gl::VertexAttribPointer(attr as u32,
                                        *size,
                                        gl::FLOAT,
                                        gl::FALSE,
                                        std::mem::size_of::<Vertex>() as GLint,
                                        offset as *const GLvoid);
                gl::VertexAttribDivisor(attr as u32, 1);
            }

            offset += size * std::mem::size_of::<f32>() as i32;
        }
        vbo.unbind();
        vao.unbind();

        // Set layout defaults
        let defaults = Section { scale: Scale::uniform(16.0),
                                 color: [1.0, 1.0, 1.0, 1.0],
                                 z: 0.1,
                                 ..Section::default() };

        // Pre-compute output sections
        let sec_params = InfoSection { text: format!("{}: {}\n{}: {:.02}",
                                                     INFO_ITERATIONS,
                                                     blur_ctx.iterations(),
                                                     INFO_OFFSET,
                                                     blur_ctx.offset()),
                                       position: (20.0, 20.0),
                                       layout:
                                           Layout::default_wrap().h_align(HorizontalAlign::Left)
                                                                 .v_align(VerticalAlign::Top) };
        let sec_time = InfoSection { text: format!("{}: {:6.03}ms\n{}: {:6.03}ms",
                                                   INFO_CPU,
                                                   blur_ctx.time_cpu(),
                                                   INFO_GPU,
                                                   blur_ctx.time_gpu()),
                                     position: (20.0, vp.height() as f32 - 20.0),
                                     layout:
                                         Layout::default_wrap().h_align(HorizontalAlign::Left)
                                                               .v_align(VerticalAlign::Bottom) };

        Self { brush: glyph_brush,
               glyph_tex,
               max_tex_size,
               vertex_count: 0,
               max_vertices: 0,
               program,
               vbo,
               vao,
               sec_defaults: defaults,
               sec_params,
               sec_time }
    }

    pub fn update(&mut self, blur_ctx: &BlurContext) {
        self.sec_params.text = format!("{}: {}\n{}: {:.02}",
                                       INFO_ITERATIONS,
                                       blur_ctx.iterations(),
                                       INFO_OFFSET,
                                       blur_ctx.offset());
        self.sec_time.text = format!("{}: {:6.03}ms\n{}: {:6.03}ms",
                                     INFO_CPU,
                                     blur_ctx.time_cpu(),
                                     INFO_GPU,
                                     blur_ctx.time_gpu());
    }

    pub fn resize(&mut self, vp: &Viewport) {
        self.sec_time.position.1 = vp.height() as f32 - 20.0;
        self.program.activate();
        self.program
            .set_uniform_mat4f("transform", &vp.transform())
            .expect("Cannot set 'transform' in glyph program");
        self.program.unbind();
    }

    pub fn draw(&mut self, blend: bool) {
        // Queue sections for drawing
        self.brush.queue(Section { text: &self.sec_params.text,
                                   screen_position: self.sec_params.position,
                                   layout: self.sec_params.layout,
                                   ..self.sec_defaults });
        self.brush.queue(Section { text: &self.sec_time.text,
                                   screen_position: self.sec_time.position,
                                   layout: self.sec_time.layout,
                                   ..self.sec_defaults });

        let tex = self.glyph_tex;
        // Update part of gpu texture with new glyph alpha values
        let update_texture = |rect: Rect<u32>, tex_data: &[u8]| unsafe {
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
            gl::BindTexture(gl::TEXTURE_2D, tex);
            gl::TexSubImage2D(gl::TEXTURE_2D,
                              0,
                              rect.min.x as i32,
                              rect.min.y as i32,
                              rect.width() as i32,
                              rect.height() as i32,
                              gl::RED,
                              gl::UNSIGNED_BYTE,
                              tex_data.as_ptr() as *const GLvoid);
            gl::BindTexture(gl::TEXTURE_2D, 0);
        };

        // Process queued sections
        let mut brush_action;
        loop {
            brush_action = self.brush.process_queued(update_texture, vertex_from_glyph);
            match brush_action {
                Ok(_) => break,
                Err(BrushError::TextureTooSmall { suggested, .. }) => {
                    let brush_size = self.brush.texture_dimensions();
                    let (new_width, new_height) =
                        if (suggested.0 > self.max_tex_size || suggested.1 > self.max_tex_size)
                           && (brush_size.0 < self.max_tex_size || brush_size.1 < self.max_tex_size)
                        {
                            (self.max_tex_size, self.max_tex_size)
                        } else {
                            suggested
                        };

                    // Resize texture to fit more glyphs
                    crate::renderer_gl::resize_texture_red(self.glyph_tex,
                                                           new_width,
                                                           new_height,
                                                           None);
                    self.brush.resize_texture(new_width, new_height);
                },
            }
        }

        // Update processed glyphs
        match brush_action.expect("Cannot process glyph draw query") {
            BrushAction::Draw(vertices) => {
                // Draw new vertices
                self.vertex_count = vertices.len();
                self.vbo.bind();
                if self.max_vertices < self.vertex_count {
                    self.vbo.set_data(&vertices, gl::DYNAMIC_DRAW);
                } else {
                    self.vbo.update_data(0, &vertices);
                }
                self.vbo.unbind();
                self.max_vertices = self.max_vertices.max(self.vertex_count);
            },
            BrushAction::ReDraw => {},
        }

        // Draw all glyphs
        if blend {
            unsafe {
                gl::Enable(gl::BLEND);
                gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            }
        }

        self.program.activate();
        self.vao.bind();
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.glyph_tex);
            for i in 0..5 {
                gl::EnableVertexAttribArray(i);
            }
            gl::DrawArraysInstanced(gl::TRIANGLE_STRIP, 0, 4, self.vertex_count as i32);
            for i in 0..5 {
                gl::DisableVertexAttribArray(i);
            }
        }
        self.vao.unbind();
        self.program.unbind();

        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, 0);
            gl::Disable(gl::BLEND);
        }
    }
}

#[rustfmt::skip]
#[inline]
fn vertex_from_glyph(GlyphVertex { mut tex_coords,
                                   pixel_coords,
                                   bounds,
                                   color,
                                   z, }: GlyphVertex)
                     -> Vertex {
    let gl_bounds = bounds;
    let mut gl_rect = Rect { min: Point { x: pixel_coords.min.x as f32,
                                          y: pixel_coords.min.y as f32 },
                             max: Point { x: pixel_coords.max.x as f32,
                                          y: pixel_coords.max.y as f32 } };

    // handle overlapping bounds, modify uv_rect to preserve texture aspect
    if gl_rect.max.x > gl_bounds.max.x {
        let old_width = gl_rect.width();
        gl_rect.max.x = gl_bounds.max.x;
        tex_coords.max.x = tex_coords.min.x + tex_coords.width() * gl_rect.width() / old_width;
    }
    if gl_rect.min.x < gl_bounds.min.x {
        let old_width = gl_rect.width();
        gl_rect.min.x = gl_bounds.min.x;
        tex_coords.min.x = tex_coords.max.x - tex_coords.width() * gl_rect.width() / old_width;
    }
    if gl_rect.max.y > gl_bounds.max.y {
        let old_height = gl_rect.height();
        gl_rect.max.y = gl_bounds.max.y;
        tex_coords.max.y = tex_coords.min.y + tex_coords.height() * gl_rect.height() / old_height;
    }
    if gl_rect.min.y < gl_bounds.min.y {
        let old_height = gl_rect.height();
        gl_rect.min.y = gl_bounds.min.y;
        tex_coords.min.y = tex_coords.max.y - tex_coords.height() * gl_rect.height() / old_height;
    }

    [gl_rect.min.x, gl_rect.max.y, z,
     gl_rect.max.x, gl_rect.min.y,
     tex_coords.min.x, tex_coords.max.y,
     tex_coords.max.x, tex_coords.min.y,
     color[0], color[1], color[2], color[3]]
}
