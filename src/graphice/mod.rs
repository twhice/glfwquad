use std::{ffi::CString, mem};

pub mod blend;
pub mod buffer;
pub mod cache;
pub mod elspsed_query;
pub mod features;
pub mod gl;
pub mod pass;
pub mod pipeline;
pub mod shader;
pub mod stencil;
mod transmute;
pub mod uniform;

use blend::*;
use buffer::*;
use cache::*;
use features::*;
use gl::*;
use pass::*;
use pipeline::*;
use shader::*;
use stencil::*;
use uniform::*;

use std::{error::Error, fmt::Display};
mod texture;
pub use texture::{FilterMode, Texture, TextureAccess, TextureFormat, TextureParams, TextureWrap};

pub type ColorMask = (bool, bool, bool, bool);
pub const MAX_VERTEX_ATTRIBUTES: usize = 16;
pub const MAX_SHADERSTAGE_IMAGES: usize = 12;

pub struct GraphicsContext {
    shaders: Vec<ShaderInternal>,
    pipelines: Vec<PipelineInternal>,
    passes: Vec<RenderPassInternal>,
    default_framebuffer: GLuint,
    cache: GlCache,

    pub(crate) features: Features,
    pub(crate) window: Option<*mut glfw::Window>,
}

impl GraphicsContext {
    pub fn new(is_gles2: bool) -> GraphicsContext {
        unsafe {
            let mut default_framebuffer: GLuint = 0;
            glGetIntegerv(
                GL_FRAMEBUFFER_BINDING,
                &mut default_framebuffer as *mut _ as *mut _,
            );
            let mut vao = 0;

            glGenVertexArrays(1, &mut vao as *mut _);
            glBindVertexArray(vao);
            GraphicsContext {
                default_framebuffer,
                shaders: vec![],
                pipelines: vec![],
                passes: vec![],
                features: Features::from_gles2(is_gles2),
                cache: GlCache {
                    stored_index_buffer: 0,
                    stored_index_type: None,
                    stored_vertex_buffer: 0,
                    index_buffer: 0,
                    index_type: None,
                    vertex_buffer: 0,
                    cur_pipeline: None,
                    color_blend: None,
                    alpha_blend: None,
                    stencil: None,
                    color_write: (true, true, true, true),
                    cull_face: CullFace::Nothing,
                    stored_texture: 0,
                    textures: [0; MAX_SHADERSTAGE_IMAGES],
                    attributes: [None; MAX_VERTEX_ATTRIBUTES],
                },
                window: None,
            }
        }
    }

    pub fn features(&self) -> &Features {
        &self.features
    }
}

impl GraphicsContext {
    pub fn set_cull_face(&mut self, cull_face: CullFace) -> &mut Self {
        if self.cache.cull_face == cull_face {
            return self;
        }

        match cull_face {
            CullFace::Nothing => unsafe {
                glDisable(GL_CULL_FACE);
            },
            CullFace::Front => unsafe {
                glEnable(GL_CULL_FACE);
                glCullFace(GL_FRONT);
            },
            CullFace::Back => unsafe {
                glEnable(GL_CULL_FACE);
                glCullFace(GL_BACK);
            },
        }
        self.cache.cull_face = cull_face;
        self
    }

    pub fn set_color_write(&mut self, color_write: ColorMask) -> &mut Self {
        if self.cache.color_write == color_write {
            return self;
        }
        let (r, g, b, a) = color_write;
        unsafe { glColorMask(r as _, g as _, b as _, a as _) }
        self.cache.color_write = color_write;
        self
    }

    pub fn set_blend(
        &mut self,
        color_blend: Option<BlendState>,
        alpha_blend: Option<BlendState>,
    ) -> &mut Self {
        if color_blend.is_none() && alpha_blend.is_some() {
            panic!("AlphaBlend without ColorBlend");
        }
        if self.cache.color_blend == color_blend && self.cache.alpha_blend == alpha_blend {
            return self;
        }

        unsafe {
            if let Some(color_blend) = color_blend {
                if self.cache.color_blend.is_none() {
                    glEnable(GL_BLEND);
                }

                let BlendState {
                    equation: eq_rgb,
                    sfactor: src_rgb,
                    dfactor: dst_rgb,
                } = color_blend;

                if let Some(BlendState {
                    equation: eq_alpha,
                    sfactor: src_alpha,
                    dfactor: dst_alpha,
                }) = alpha_blend
                {
                    glBlendFuncSeparate(
                        src_rgb.into(),
                        dst_rgb.into(),
                        src_alpha.into(),
                        dst_alpha.into(),
                    );
                    glBlendEquationSeparate(eq_rgb.into(), eq_alpha.into());
                } else {
                    glBlendFunc(src_rgb.into(), dst_rgb.into());
                    glBlendEquationSeparate(eq_rgb.into(), eq_rgb.into());
                }
            } else if self.cache.color_blend.is_some() {
                glDisable(GL_BLEND);
            }
        }

        self.cache.color_blend = color_blend;
        self.cache.alpha_blend = alpha_blend;
        self
    }

    pub fn set_stencil(&mut self, stencil_test: Option<StencilState>) -> &mut Self {
        if self.cache.stencil == stencil_test {
            return self;
        }
        unsafe {
            if let Some(stencil) = stencil_test {
                if self.cache.stencil.is_none() {
                    glEnable(GL_STENCIL_TEST);
                }

                let front = &stencil.front;
                glStencilOpSeparate(
                    GL_FRONT,
                    front.fail_op.into(),
                    front.depth_fail_op.into(),
                    front.pass_op.into(),
                );
                glStencilFuncSeparate(
                    GL_FRONT,
                    front.test_func.into(),
                    front.test_ref,
                    front.test_mask,
                );
                glStencilMaskSeparate(GL_FRONT, front.write_mask);

                let back = &stencil.back;
                glStencilOpSeparate(
                    GL_BACK,
                    back.fail_op.into(),
                    back.depth_fail_op.into(),
                    back.pass_op.into(),
                );
                glStencilFuncSeparate(
                    GL_BACK,
                    back.test_func.into(),
                    back.test_ref.into(),
                    back.test_mask,
                );
                glStencilMaskSeparate(GL_BACK, back.write_mask);
            } else if self.cache.stencil.is_some() {
                glDisable(GL_STENCIL_TEST);
            }
        }

        self.cache.stencil = stencil_test;
        self
    }

    /// Set a new viewport rectangle.
    /// Should be applied after begin_pass.
    pub fn apply_viewport(&mut self, x: i32, y: i32, w: i32, h: i32) -> &mut Self {
        unsafe {
            glViewport(x, y, w, h);
        }
        self
    }

    /// Set a new scissor rectangle.
    /// Should be applied after begin_pass.
    pub fn apply_scissor_rect(&mut self, x: i32, y: i32, w: i32, h: i32) -> &mut Self {
        unsafe {
            glScissor(x, y, w, h);
        }
        self
    }

    pub fn apply_bindings(&mut self, bindings: &Bindings) -> &mut Self {
        let pip = &self.pipelines[self.cache.cur_pipeline.unwrap().0];
        let shader = &self.shaders[pip.shader.0];

        for (n, shader_image) in shader.images.iter().enumerate() {
            let bindings_image = bindings
                .images
                .get(n)
                .unwrap_or_else(|| panic!("Image count in bindings and shader did not match!"));
            if let Some(gl_loc) = shader_image.gl_loc {
                unsafe {
                    self.cache.bind_texture(n, bindings_image.texture);
                    glUniform1i(gl_loc, n as i32);
                }
            }
        }

        self.cache.bind_buffer(
            GL_ELEMENT_ARRAY_BUFFER,
            bindings.index_buffer.gl_buf,
            bindings.index_buffer.index_type,
        );

        let pip = &self.pipelines[self.cache.cur_pipeline.unwrap().0];

        for attr_index in 0..MAX_VERTEX_ATTRIBUTES {
            let cached_attr = &mut self.cache.attributes[attr_index];

            let pip_attribute = pip.layout.get(attr_index).copied();

            if let Some(Some(attribute)) = pip_attribute {
                let vb = bindings.vertex_buffers[attribute.buffer_index];

                if cached_attr.map_or(true, |cached_attr| {
                    attribute != cached_attr.attribute || cached_attr.gl_vbuf != vb.gl_buf
                }) {
                    self.cache
                        .bind_buffer(GL_ARRAY_BUFFER, vb.gl_buf, vb.index_type);

                    unsafe {
                        glVertexAttribPointer(
                            attr_index as GLuint,
                            attribute.size,
                            attribute.type_,
                            GL_FALSE as u8,
                            attribute.stride,
                            attribute.offset as *mut _,
                        );
                        if self.features.instancing {
                            glVertexAttribDivisor(attr_index as GLuint, attribute.divisor as u32);
                        }
                        glEnableVertexAttribArray(attr_index as GLuint);
                    };

                    let cached_attr = &mut self.cache.attributes[attr_index];
                    *cached_attr = Some(CachedAttribute {
                        attribute,
                        gl_vbuf: vb.gl_buf,
                    });
                }
            } else {
                if cached_attr.is_some() {
                    unsafe {
                        glDisableVertexAttribArray(attr_index as GLuint);
                    }
                    *cached_attr = None;
                }
            }
        }
        self
    }

    pub fn apply_uniforms<U>(&mut self, uniforms: &U) -> &mut Self {
        self.apply_uniforms_from_bytes(uniforms as *const _ as *const u8, std::mem::size_of::<U>());
        self
    }

    #[doc(hidden)]
    /// Apply uniforms data from array of bytes with very special layout.
    /// Hidden because `apply_uniforms` is the recommended and safer way to work with uniforms.
    pub fn apply_uniforms_from_bytes(&mut self, uniform_ptr: *const u8, size: usize) -> &mut Self {
        let pip = &self.pipelines[self.cache.cur_pipeline.unwrap().0];
        let shader = &self.shaders[pip.shader.0];

        let mut offset = 0;

        for (_, uniform) in shader.uniforms.iter().enumerate() {
            use UniformType::*;

            assert!(
                offset <= size - uniform.uniform_type.size() / 4,
                "Uniforms struct does not match shader uniforms layout"
            );

            unsafe {
                let data = (uniform_ptr as *const f32).offset(offset as isize);
                let data_int = (uniform_ptr as *const i32).offset(offset as isize);

                if let Some(gl_loc) = uniform.gl_loc {
                    match uniform.uniform_type {
                        Float1 => {
                            glUniform1fv(gl_loc, uniform.array_count, data);
                        }
                        Float2 => {
                            glUniform2fv(gl_loc, uniform.array_count, data);
                        }
                        Float3 => {
                            glUniform3fv(gl_loc, uniform.array_count, data);
                        }
                        Float4 => {
                            glUniform4fv(gl_loc, uniform.array_count, data);
                        }
                        Int1 => {
                            glUniform1iv(gl_loc, uniform.array_count, data_int);
                        }
                        Int2 => {
                            glUniform2iv(gl_loc, uniform.array_count, data_int);
                        }
                        Int3 => {
                            glUniform3iv(gl_loc, uniform.array_count, data_int);
                        }
                        Int4 => {
                            glUniform4iv(gl_loc, uniform.array_count, data_int);
                        }
                        Mat4 => {
                            glUniformMatrix4fv(gl_loc, uniform.array_count, 0, data);
                        }
                    }
                }
            }
            offset += uniform.uniform_type.size() / 4 * uniform.array_count as usize;
        }
        self
    }

    #[inline]
    pub fn clear(&self, clear: Clear) {
        clear.apply()
    }

    /// Draw elements using currently applied bindings and pipeline.
    ///
    /// + `base_element` specifies starting offset in `index_buffer`.
    /// + `num_elements` specifies length of the slice of `index_buffer` to draw.
    /// + `num_instances` specifies how many instances should be rendered.
    ///
    /// NOTE: num_instances > 1 might be not supported by the GPU (gl2.1 and gles2).
    /// `features.instancing` check is required.
    pub fn draw(&self, base_element: i32, num_elements: i32, num_instances: i32) -> &Self {
        assert!(
            self.cache.cur_pipeline.is_some(),
            "Drawing without any binded pipeline"
        );

        if !self.features.instancing && num_instances != 1 {
            eprintln!("Instanced rendering is not supported by the GPU");
            eprintln!("Ignoring this draw call");
            return self;
        }

        let pip = &self.pipelines[self.cache.cur_pipeline.unwrap().0];
        let primitive_type = pip.params.primitive_type.into();
        let index_type = self.cache.index_type.expect("Unset index buffer type");

        unsafe {
            if self.features.instancing {
                glDrawElementsInstanced(
                    primitive_type,
                    num_elements,
                    index_type.into(),
                    (index_type.size() as i32 * base_element) as *mut _,
                    num_instances,
                );
            } else {
                glDrawElements(
                    primitive_type,
                    num_elements,
                    index_type.into(),
                    (index_type.size() as i32 * base_element) as *mut _,
                );
            }
        }
        self
    }
}

impl GraphicsContext {
    pub fn window(&self) -> &glfw::Window {
        unsafe { &*self.window.unwrap() }
    }

    pub fn window_mut(&mut self) -> &mut glfw::Window {
        unsafe { &mut *self.window.unwrap() }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Clear {
    color: Option<(f32, f32, f32, f32)>,
    depth: Option<f32>,
    stencil: Option<i32>,
}

impl Clear {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn color(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.color = Some((r, g, b, a));
        self
    }

    #[inline]
    pub fn depth(mut self, depth: f32) -> Self {
        self.depth = Some(depth);
        self
    }

    #[inline]
    pub fn stencil(mut self, stencil: i32) -> Self {
        self.stencil = Some(stencil);
        self
    }

    #[inline]
    pub fn apply(self) {
        let Self {
            color,
            depth,
            stencil,
        } = self;
        let mut bits = 0;
        if let Some((r, g, b, a)) = color {
            bits |= GL_COLOR_BUFFER_BIT;
            unsafe {
                glClearColor(r, g, b, a);
            }
        }

        if let Some(v) = depth {
            bits |= GL_DEPTH_BUFFER_BIT;
            unsafe {
                glClearDepthf(v);
            }
        }

        if let Some(v) = stencil {
            bits |= GL_STENCIL_BUFFER_BIT;
            unsafe {
                glClearStencil(v);
            }
        }

        if bits != 0 {
            unsafe {
                glClear(bits);
            }
        }
    }
}
