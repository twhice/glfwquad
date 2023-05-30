use super::*;
pub enum PassAction {
    Nothing,
    Clear(Clear),
}

impl PassAction {
    pub fn clear_color(r: f32, g: f32, b: f32, a: f32) -> PassAction {
        PassAction::Clear(Clear {
            color: Some((r, g, b, a)),
            depth: Some(1.),
            stencil: None,
        })
    }
}

impl Default for PassAction {
    fn default() -> PassAction {
        PassAction::Clear(Clear {
            color: Some((0.0, 0.0, 0.0, 0.0)),
            depth: Some(1.),
            stencil: None,
        })
    }
}

pub(crate) struct RenderPassInternal {
    pub(crate) gl_fb: GLuint,
    pub(crate) texture: Texture,
    pub(crate) depth_texture: Option<Texture>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderPass(pub(crate) usize);

impl RenderPass {
    pub fn new(
        ctx: &mut GraphicsContext,
        color_img: Texture,
        depth_img: impl Into<Option<Texture>>,
    ) -> RenderPass {
        let mut gl_fb = 0;

        let depth_img = depth_img.into();

        unsafe {
            glGenFramebuffers(1, &mut gl_fb as *mut _);
            glBindFramebuffer(GL_FRAMEBUFFER, gl_fb);
            glFramebufferTexture2D(
                GL_FRAMEBUFFER,
                GL_COLOR_ATTACHMENT0,
                GL_TEXTURE_2D,
                color_img.texture,
                0,
            );
            if let Some(depth_img) = depth_img {
                glFramebufferTexture2D(
                    GL_FRAMEBUFFER,
                    GL_DEPTH_ATTACHMENT,
                    GL_TEXTURE_2D,
                    depth_img.texture,
                    0,
                );
            }
            glBindFramebuffer(GL_FRAMEBUFFER, ctx.default_framebuffer);
        }
        let pass = RenderPassInternal {
            gl_fb,
            texture: color_img,
            depth_texture: depth_img,
        };

        ctx.passes.push(pass);

        RenderPass(ctx.passes.len() - 1)
    }

    pub fn texture(&self, ctx: &mut GraphicsContext) -> Texture {
        let render_pass = &mut ctx.passes[self.0];

        render_pass.texture.clone()
    }

    pub fn delete(&self, ctx: &mut GraphicsContext) {
        let render_pass = &mut ctx.passes[self.0];

        unsafe { glDeleteFramebuffers(1, &mut render_pass.gl_fb as *mut _) }
    }
}

impl GraphicsContext {
    /// start rendering to the default frame buffer
    pub fn begin_default_pass(&mut self, action: PassAction) -> &mut Self {
        self.begin_pass(None, action);
        self
    }

    /// start rendering to an offscreen framebuffer
    pub fn begin_pass(
        &mut self,
        pass: impl Into<Option<RenderPass>>,
        action: PassAction,
    ) -> &mut Self {
        let (framebuffer, w, h) = match pass.into() {
            None => {
                let (screen_width, screen_height) = self.window().get_size();
                (
                    self.default_framebuffer,
                    screen_width as i32,
                    screen_height as i32,
                )
            }
            Some(pass) => {
                let pass = &self.passes[pass.0];
                (
                    pass.gl_fb,
                    pass.texture.width as i32,
                    pass.texture.height as i32,
                )
            }
        };
        unsafe {
            glBindFramebuffer(GL_FRAMEBUFFER, framebuffer);
            glViewport(0, 0, w, h);
            glScissor(0, 0, w, h);
        }
        match action {
            PassAction::Nothing => {}
            PassAction::Clear(clear) => {
                clear.apply();
            }
        }
        self
    }

    pub fn end_render_pass(&mut self) -> &mut Self {
        unsafe {
            glBindFramebuffer(GL_FRAMEBUFFER, self.default_framebuffer);
            self.cache.bind_buffer(GL_ARRAY_BUFFER, 0, None);
            self.cache.bind_buffer(GL_ELEMENT_ARRAY_BUFFER, 0, None);
        }
        self
    }

    pub fn commit_frame(&mut self) {
        self.cache.clear_buffer_bindings();
        self.cache.clear_texture_bindings();
    }
}
