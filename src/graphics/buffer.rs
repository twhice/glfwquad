use super::*;
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BufferType {
    VertexBuffer,
    IndexBuffer,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BufferUsage {
    Immutable,
    Dynamic,
    Stream,
}

fn gl_buffer_target(buffer_type: &BufferType) -> GLenum {
    match buffer_type {
        BufferType::VertexBuffer => GL_ARRAY_BUFFER,
        BufferType::IndexBuffer => GL_ELEMENT_ARRAY_BUFFER,
    }
}

fn gl_usage(usage: &BufferUsage) -> GLenum {
    match usage {
        BufferUsage::Immutable => GL_STATIC_DRAW,
        BufferUsage::Dynamic => GL_DYNAMIC_DRAW,
        BufferUsage::Stream => GL_STREAM_DRAW,
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum IndexType {
    Byte,
    Short,
    Int,
}

impl IndexType {
    pub fn for_type<T>() -> IndexType {
        match std::mem::size_of::<T>() {
            1 => IndexType::Byte,
            2 => IndexType::Short,
            4 => IndexType::Int,
            _ => panic!("Unsupported index buffer index type"),
        }
    }

    pub fn size(self) -> u8 {
        match self {
            IndexType::Byte => 1,
            IndexType::Short => 2,
            IndexType::Int => 4,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Buffer {
    pub(crate) gl_buf: GLuint,
    pub(crate) buffer_type: BufferType,
    pub(crate) size: usize,
    pub(crate) index_type: Option<IndexType>,
}

impl Buffer {
    /// Create an immutable buffer resource object.
    /// ```ignore
    /// #[repr(C)]
    /// struct Vertex {
    ///     pos: Vec2,
    ///     uv: Vec2,
    /// }
    /// let vertices: [Vertex; 4] = [
    ///     Vertex { pos : Vec2 { x: -0.5, y: -0.5 }, uv: Vec2 { x: 0., y: 0. } },
    ///     Vertex { pos : Vec2 { x:  0.5, y: -0.5 }, uv: Vec2 { x: 1., y: 0. } },
    ///     Vertex { pos : Vec2 { x:  0.5, y:  0.5 }, uv: Vec2 { x: 1., y: 1. } },
    ///     Vertex { pos : Vec2 { x: -0.5, y:  0.5 }, uv: Vec2 { x: 0., y: 1. } },
    /// ];
    /// let buffer = Buffer::immutable(ctx, BufferType::VertexBuffer, &vertices);
    /// ```
    pub fn immutable<T>(ctx: &mut GraphicsContext, buffer_type: BufferType, data: &[T]) -> Buffer {
        let index_type = if buffer_type == BufferType::IndexBuffer {
            Some(IndexType::for_type::<T>())
        } else {
            None
        };

        let gl_target = gl_buffer_target(&buffer_type);
        let gl_usage = gl_usage(&BufferUsage::Immutable);
        let size = mem::size_of_val(data);
        let mut gl_buf: u32 = 0;

        unsafe {
            glGenBuffers(1, &mut gl_buf as *mut _);
            ctx.cache.store_buffer_binding(gl_target);
            ctx.cache.bind_buffer(gl_target, gl_buf, index_type);
            glBufferData(gl_target, size as _, std::ptr::null() as *const _, gl_usage);
            glBufferSubData(gl_target, 0, size as _, data.as_ptr() as *const _);
            ctx.cache.restore_buffer_binding(gl_target);
        }

        Buffer {
            gl_buf,
            buffer_type,
            size,
            index_type,
        }
    }

    pub fn stream(ctx: &mut GraphicsContext, buffer_type: BufferType, size: usize) -> Buffer {
        let index_type = if buffer_type == BufferType::IndexBuffer {
            Some(IndexType::Short)
        } else {
            None
        };

        let gl_target = gl_buffer_target(&buffer_type);
        let gl_usage = gl_usage(&BufferUsage::Stream);
        let mut gl_buf: u32 = 0;

        unsafe {
            glGenBuffers(1, &mut gl_buf as *mut _);
            ctx.cache.store_buffer_binding(gl_target);
            ctx.cache.bind_buffer(gl_target, gl_buf, None);
            glBufferData(gl_target, size as _, std::ptr::null() as *const _, gl_usage);
            ctx.cache.restore_buffer_binding(gl_target);
        }

        Buffer {
            gl_buf,
            buffer_type,
            size,
            index_type,
        }
    }

    pub fn index_stream(ctx: &mut GraphicsContext, index_type: IndexType, size: usize) -> Buffer {
        let gl_target = gl_buffer_target(&BufferType::IndexBuffer);
        let gl_usage = gl_usage(&BufferUsage::Stream);
        let mut gl_buf: u32 = 0;

        unsafe {
            glGenBuffers(1, &mut gl_buf as *mut _);
            ctx.cache.store_buffer_binding(gl_target);
            ctx.cache.bind_buffer(gl_target, gl_buf, None);
            glBufferData(gl_target, size as _, std::ptr::null() as *const _, gl_usage);
            ctx.cache.restore_buffer_binding(gl_target);
        }

        Buffer {
            gl_buf,
            buffer_type: BufferType::IndexBuffer,
            size,
            index_type: Some(index_type),
        }
    }

    pub fn update<T>(&self, ctx: &mut GraphicsContext, data: &[T]) {
        if self.buffer_type == BufferType::IndexBuffer {
            assert!(self.index_type.is_some());
            assert!(self.index_type.unwrap() == IndexType::for_type::<T>());
        };

        let size = mem::size_of_val(data);

        assert!(size <= self.size);

        let gl_target = gl_buffer_target(&self.buffer_type);
        ctx.cache.store_buffer_binding(gl_target);
        ctx.cache
            .bind_buffer(gl_target, self.gl_buf, self.index_type);
        unsafe { glBufferSubData(gl_target, 0, size as _, data.as_ptr() as *const _) };
        ctx.cache.restore_buffer_binding(gl_target);
    }

    /// Size of buffer in bytes
    pub fn size(&self) -> usize {
        self.size
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe { glDeleteBuffers(1, &self.gl_buf as *const _) }
    }
}

/// Geometry bindings
#[derive(Clone, Debug)]
pub struct Bindings {
    /// Vertex buffers. Data contained in the buffer must match layout
    /// specified in the `Pipeline`.
    ///
    /// Most commonly vertex buffer will contain `(x,y,z,w)` coordinates of the
    /// vertex in 3d space, as well as `(u,v)` coordinates that map the vertex
    /// to some position in the corresponding `Texture`.
    pub vertex_buffers: Vec<Buffer>,
    /// Index buffer which instructs the GPU in which order to draw vertices
    /// from a vertex buffer, with each subsequent 3 indices forming a
    /// triangle.
    pub index_buffer: Buffer,
    /// Textures to be used with when drawing the geometry in the fragment
    /// shader.
    pub images: Vec<Texture>,
}
