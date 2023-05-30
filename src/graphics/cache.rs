use super::*;

#[derive(Default, Copy, Clone)]
pub(crate) struct CachedAttribute {
    pub(crate) attribute: VertexAttributeInternal,
    pub(crate) gl_vbuf: GLuint,
}

pub(crate) struct GlCache {
    pub(crate) stored_index_buffer: GLuint,
    pub(crate) stored_index_type: Option<IndexType>,
    pub(crate) stored_vertex_buffer: GLuint,
    pub(crate) stored_texture: GLuint,
    pub(crate) index_buffer: GLuint,
    pub(crate) index_type: Option<IndexType>,
    pub(crate) vertex_buffer: GLuint,
    pub(crate) textures: [GLuint; MAX_SHADERSTAGE_IMAGES],
    pub(crate) cur_pipeline: Option<Pipeline>,
    pub(crate) color_blend: Option<BlendState>,
    pub(crate) alpha_blend: Option<BlendState>,
    pub(crate) stencil: Option<StencilState>,
    pub(crate) color_write: ColorMask,
    pub(crate) cull_face: CullFace,
    pub(crate) attributes: [Option<CachedAttribute>; MAX_VERTEX_ATTRIBUTES],
}

impl GlCache {
    pub(crate) fn bind_buffer(
        &mut self,
        target: GLenum,
        buffer: GLuint,
        index_type: Option<IndexType>,
    ) {
        if target == GL_ARRAY_BUFFER {
            if self.vertex_buffer != buffer {
                self.vertex_buffer = buffer;
                unsafe {
                    glBindBuffer(target, buffer);
                }
            }
        } else {
            if self.index_buffer != buffer {
                self.index_buffer = buffer;
                unsafe {
                    glBindBuffer(target, buffer);
                }
            }
            self.index_type = index_type;
        }
    }

    pub(crate) fn store_buffer_binding(&mut self, target: GLenum) {
        if target == GL_ARRAY_BUFFER {
            self.stored_vertex_buffer = self.vertex_buffer;
        } else {
            self.stored_index_buffer = self.index_buffer;
            self.stored_index_type = self.index_type;
        }
    }

    pub(crate) fn restore_buffer_binding(&mut self, target: GLenum) {
        if target == GL_ARRAY_BUFFER {
            if self.stored_vertex_buffer != 0 {
                self.bind_buffer(target, self.stored_vertex_buffer, None);
                self.stored_vertex_buffer = 0;
            }
        } else {
            if self.stored_index_buffer != 0 {
                self.bind_buffer(target, self.stored_index_buffer, self.stored_index_type);
                self.stored_index_buffer = 0;
            }
        }
    }

    pub(crate) fn bind_texture(&mut self, slot_index: usize, texture: GLuint) {
        unsafe {
            glActiveTexture(GL_TEXTURE0 + slot_index as GLuint);
            if self.textures[slot_index] != texture {
                glBindTexture(GL_TEXTURE_2D, texture);
                self.textures[slot_index] = texture;
            }
        }
    }

    pub(crate) fn store_texture_binding(&mut self, slot_index: usize) {
        self.stored_texture = self.textures[slot_index];
    }

    pub(crate) fn restore_texture_binding(&mut self, slot_index: usize) {
        self.bind_texture(slot_index, self.stored_texture);
    }

    pub(crate) fn clear_buffer_bindings(&mut self) {
        self.bind_buffer(GL_ARRAY_BUFFER, 0, None);
        self.vertex_buffer = 0;

        self.bind_buffer(GL_ELEMENT_ARRAY_BUFFER, 0, None);
        self.index_buffer = 0;
    }

    pub(crate) fn clear_texture_bindings(&mut self) {
        for ix in 0..MAX_SHADERSTAGE_IMAGES {
            if self.textures[ix] != 0 {
                self.bind_texture(ix, 0);
                self.textures[ix] = 0;
            }
        }
    }
}
