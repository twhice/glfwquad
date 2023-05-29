use super::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum VertexFormat {
    /// One 32-bit wide float (equivalent to `f32`)
    Float1,
    /// Two 32-bit wide floats (equivalent to `[f32; 2]`)
    Float2,
    /// Three 32-bit wide floats (equivalent to `[f32; 3]`)
    Float3,
    /// Four 32-bit wide floats (equivalent to `[f32; 4]`)
    Float4,
    /// One unsigned 8-bit integer (equivalent to `u8`)
    Byte1,
    /// Two unsigned 8-bit integers (equivalent to `[u8; 2]`)
    Byte2,
    /// Three unsigned 8-bit integers (equivalent to `[u8; 3]`)
    Byte3,
    /// Four unsigned 8-bit integers (equivalent to `[u8; 4]`)
    Byte4,
    /// One unsigned 16-bit integer (equivalent to `u16`)
    Short1,
    /// Two unsigned 16-bit integers (equivalent to `[u16; 2]`)
    Short2,
    /// Tree unsigned 16-bit integers (equivalent to `[u16; 3]`)
    Short3,
    /// Four unsigned 16-bit integers (equivalent to `[u16; 4]`)
    Short4,
    /// One unsigned 32-bit integers (equivalent to `[u32; 1]`)
    Int1,
    /// Two unsigned 32-bit integers (equivalent to `[u32; 2]`)
    Int2,
    /// Three unsigned 32-bit integers (equivalent to `[u32; 3]`)
    Int3,
    /// Four unsigned 32-bit integers (equivalent to `[u32; 4]`)
    Int4,
    /// Four by four matrix of 32-bit floats
    Mat4,
}

impl VertexFormat {
    pub fn size(&self) -> i32 {
        match self {
            VertexFormat::Float1 => 1,
            VertexFormat::Float2 => 2,
            VertexFormat::Float3 => 3,
            VertexFormat::Float4 => 4,
            VertexFormat::Byte1 => 1,
            VertexFormat::Byte2 => 2,
            VertexFormat::Byte3 => 3,
            VertexFormat::Byte4 => 4,
            VertexFormat::Short1 => 1,
            VertexFormat::Short2 => 2,
            VertexFormat::Short3 => 3,
            VertexFormat::Short4 => 4,
            VertexFormat::Int1 => 1,
            VertexFormat::Int2 => 2,
            VertexFormat::Int3 => 3,
            VertexFormat::Int4 => 4,
            VertexFormat::Mat4 => 16,
        }
    }

    pub fn byte_len(&self) -> i32 {
        match self {
            VertexFormat::Float1 => 1 * 4,
            VertexFormat::Float2 => 2 * 4,
            VertexFormat::Float3 => 3 * 4,
            VertexFormat::Float4 => 4 * 4,
            VertexFormat::Byte1 => 1,
            VertexFormat::Byte2 => 2,
            VertexFormat::Byte3 => 3,
            VertexFormat::Byte4 => 4,
            VertexFormat::Short1 => 1 * 2,
            VertexFormat::Short2 => 2 * 2,
            VertexFormat::Short3 => 3 * 2,
            VertexFormat::Short4 => 4 * 2,
            VertexFormat::Int1 => 1 * 4,
            VertexFormat::Int2 => 2 * 4,
            VertexFormat::Int3 => 3 * 4,
            VertexFormat::Int4 => 4 * 4,
            VertexFormat::Mat4 => 16 * 4,
        }
    }

    pub(crate) fn type_(&self) -> GLuint {
        match self {
            VertexFormat::Float1 => GL_FLOAT,
            VertexFormat::Float2 => GL_FLOAT,
            VertexFormat::Float3 => GL_FLOAT,
            VertexFormat::Float4 => GL_FLOAT,
            VertexFormat::Byte1 => GL_UNSIGNED_BYTE,
            VertexFormat::Byte2 => GL_UNSIGNED_BYTE,
            VertexFormat::Byte3 => GL_UNSIGNED_BYTE,
            VertexFormat::Byte4 => GL_UNSIGNED_BYTE,
            VertexFormat::Short1 => GL_UNSIGNED_SHORT,
            VertexFormat::Short2 => GL_UNSIGNED_SHORT,
            VertexFormat::Short3 => GL_UNSIGNED_SHORT,
            VertexFormat::Short4 => GL_UNSIGNED_SHORT,
            VertexFormat::Int1 => GL_UNSIGNED_INT,
            VertexFormat::Int2 => GL_UNSIGNED_INT,
            VertexFormat::Int3 => GL_UNSIGNED_INT,
            VertexFormat::Int4 => GL_UNSIGNED_INT,
            VertexFormat::Mat4 => GL_FLOAT,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum VertexStep {
    PerVertex,
    PerInstance,
}

impl Default for VertexStep {
    fn default() -> VertexStep {
        VertexStep::PerVertex
    }
}

#[derive(Clone, Debug)]
pub struct VertexAttribute {
    pub name: &'static str,
    pub format: VertexFormat,
    pub buffer_index: usize,
}

impl VertexAttribute {
    pub const fn new(name: &'static str, format: VertexFormat) -> VertexAttribute {
        Self::with_buffer(name, format, 0)
    }

    pub const fn with_buffer(
        name: &'static str,
        format: VertexFormat,
        buffer_index: usize,
    ) -> VertexAttribute {
        VertexAttribute {
            name,
            format,
            buffer_index,
        }
    }
}
