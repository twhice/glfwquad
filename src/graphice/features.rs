pub struct Features {
    pub instancing: bool,
}

impl Features {
    pub fn from_gles2(is_gles2: bool) -> Self {
        Features {
            instancing: !is_gles2,
        }
    }
}
