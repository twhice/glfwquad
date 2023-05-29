use super::*;

pub(crate) fn get_uniform_location(program: GLuint, name: &str) -> Option<i32> {
    let cname = CString::new(name).unwrap_or_else(|e| panic!("{}", e));
    let location = unsafe { glGetUniformLocation(program, cname.as_ptr()) };

    if location == -1 {
        return None;
    }

    Some(location)
}

#[derive(Clone, Copy, Debug)]
pub enum UniformType {
    /// One 32-bit wide float (equivalent to `f32`)
    Float1,
    /// Two 32-bit wide floats (equivalent to `[f32; 2]`)
    Float2,
    /// Three 32-bit wide floats (equivalent to `[f32; 3]`)
    Float3,
    /// Four 32-bit wide floats (equivalent to `[f32; 4]`)
    Float4,
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

impl UniformType {
    /// Byte size for a given UniformType
    pub fn size(&self) -> usize {
        match self {
            UniformType::Float1 => 4,
            UniformType::Float2 => 8,
            UniformType::Float3 => 12,
            UniformType::Float4 => 16,
            UniformType::Int1 => 4,
            UniformType::Int2 => 8,
            UniformType::Int3 => 12,
            UniformType::Int4 => 16,
            UniformType::Mat4 => 64,
        }
    }
}

#[derive(Clone)]
pub struct UniformDesc {
    pub(crate) name: String,
    pub(crate) uniform_type: UniformType,
    pub(crate) array_count: usize,
}

#[derive(Clone)]
pub struct UniformBlockLayout {
    pub uniforms: Vec<UniformDesc>,
}

impl UniformDesc {
    pub fn new(name: &str, uniform_type: UniformType) -> UniformDesc {
        UniformDesc {
            name: name.to_string(),
            uniform_type,
            array_count: 1,
        }
    }

    pub fn array(self, array_count: usize) -> UniformDesc {
        UniformDesc {
            array_count,
            ..self
        }
    }
}
