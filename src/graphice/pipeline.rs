use super::*;

#[derive(Clone, Debug)]
pub struct BufferLayout {
    pub stride: i32,
    pub step_func: VertexStep,
    pub step_rate: i32,
}

#[derive(Clone, Debug)]
pub struct PipelineLayout {
    pub buffers: &'static [BufferLayout],
    pub attributes: &'static [VertexAttribute],
}

impl Default for BufferLayout {
    fn default() -> BufferLayout {
        BufferLayout {
            stride: 0,
            step_func: VertexStep::PerVertex,
            step_rate: 1,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PrimitiveType {
    Triangles,
    TriangleStrip,
    Lines,
    LineStrip,
}

/// A pixel-wise comparison function.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Comparison {
    Never,
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
    Equal,
    NotEqual,
    Always,
}

/// Specify whether front- or back-facing polygons can be culled.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CullFace {
    Nothing,
    Front,
    Back,
}

/// Define front- and back-facing polygons.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FrontFaceOrder {
    Clockwise,
    CounterClockwise,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct PipelineConf {
    pub cull_face: CullFace,
    pub front_face_order: FrontFaceOrder,
    pub depth_test: Comparison,
    pub depth_write: bool,
    pub depth_write_offset: Option<(f32, f32)>,
    /// Color (RGB) blend function. If None - blending will be disabled for this pipeline.
    /// Usual use case to get alpha-blending:
    ///```
    ///# use miniquad::{PipelineParams, BlendState, BlendValue, BlendFactor, Equation};
    ///PipelineParams {
    ///    color_blend: Some(BlendState::new(
    ///        Equation::Add,
    ///        BlendFactor::Value(BlendValue::SourceAlpha),
    ///        BlendFactor::OneMinusValue(BlendValue::SourceAlpha))
    ///    ),
    ///    ..Default::default()
    ///};
    ///```
    pub color_blend: Option<BlendState>,
    /// Alpha blend function. If None - alpha will be blended with same equation than RGB colors.
    /// One of possible separate alpha channel blend settings is to avoid blending with WebGl background.
    /// On webgl canvas's resulting alpha channel will be used to blend the whole canvas background.
    /// To avoid modifying only alpha channel, but keep usual transparency:
    ///```
    ///# use miniquad::{PipelineParams, BlendState, BlendValue, BlendFactor, Equation};
    ///PipelineParams {
    ///    color_blend: Some(BlendState::new(
    ///        Equation::Add,
    ///        BlendFactor::Value(BlendValue::SourceAlpha),
    ///        BlendFactor::OneMinusValue(BlendValue::SourceAlpha))
    ///    ),
    ///    alpha_blend: Some(BlendState::new(
    ///        Equation::Add,
    ///        BlendFactor::Zero,
    ///        BlendFactor::One)
    ///    ),
    ///    ..Default::default()
    ///};
    ///```
    /// The same results may be achieved with ColorMask(true, true, true, false)
    pub alpha_blend: Option<BlendState>,
    pub stencil_test: Option<StencilState>,
    pub color_write: ColorMask,
    pub primitive_type: PrimitiveType,
}

impl Default for PipelineConf {
    fn default() -> PipelineConf {
        PipelineConf {
            cull_face: CullFace::Nothing,
            front_face_order: FrontFaceOrder::CounterClockwise,
            depth_test: Comparison::Always, // no depth test,
            depth_write: false,             // no depth write,
            depth_write_offset: None,
            color_blend: None,
            alpha_blend: None,
            stencil_test: None,
            color_write: (true, true, true, true),
            primitive_type: PrimitiveType::Triangles,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Pipeline(pub(crate) usize);

impl Pipeline {
    pub fn new(
        ctx: &mut GraphicsContext,
        buffer_layout: &[BufferLayout],
        attributes: &[VertexAttribute],
        shader: Shader,
    ) -> Pipeline {
        Self::with_params(ctx, buffer_layout, attributes, shader, Default::default())
    }

    pub fn with_params(
        ctx: &mut GraphicsContext,
        buffer_layout: &[BufferLayout],
        attributes: &[VertexAttribute],
        shader: Shader,
        params: PipelineConf,
    ) -> Pipeline {
        #[derive(Clone, Copy, Default)]
        struct BufferCacheData {
            stride: i32,
            offset: i64,
        }

        let mut buffer_cache: Vec<BufferCacheData> =
            vec![BufferCacheData::default(); buffer_layout.len()];

        for VertexAttribute {
            format,
            buffer_index,
            ..
        } in attributes
        {
            let layout = buffer_layout.get(*buffer_index).unwrap_or_else(|| panic!());
            let mut cache = buffer_cache
                .get_mut(*buffer_index)
                .unwrap_or_else(|| panic!());

            if layout.stride == 0 {
                cache.stride += format.byte_len();
            } else {
                cache.stride = layout.stride;
            }
            // WebGL 1 limitation
            assert!(cache.stride <= 255);
        }

        let program = ctx.shaders[shader.0].program;

        let attributes_len = attributes
            .iter()
            .map(|layout| match layout.format {
                VertexFormat::Mat4 => 4,
                _ => 1,
            })
            .sum();

        let mut vertex_layout: Vec<Option<VertexAttributeInternal>> = vec![None; attributes_len];

        for VertexAttribute {
            name,
            format,
            buffer_index,
        } in attributes
        {
            let mut buffer_data = &mut buffer_cache
                .get_mut(*buffer_index)
                .unwrap_or_else(|| panic!());
            let layout = buffer_layout.get(*buffer_index).unwrap_or_else(|| panic!());

            let cname = CString::new(*name).unwrap_or_else(|e| panic!("{}", e));
            let attr_loc = unsafe { glGetAttribLocation(program, cname.as_ptr() as *const _) };
            let attr_loc = if attr_loc == -1 { None } else { Some(attr_loc) };
            let divisor = if layout.step_func == VertexStep::PerVertex {
                0
            } else {
                layout.step_rate
            };

            let mut attributes_count: usize = 1;
            let mut format = *format;

            if format == VertexFormat::Mat4 {
                format = VertexFormat::Float4;
                attributes_count = 4;
            }
            for i in 0..attributes_count {
                if let Some(attr_loc) = attr_loc {
                    let attr_loc = attr_loc as GLuint + i as GLuint;

                    let attr = VertexAttributeInternal {
                        attr_loc,
                        size: format.size(),
                        type_: format.type_(),
                        offset: buffer_data.offset,
                        stride: buffer_data.stride,
                        buffer_index: *buffer_index,
                        divisor,
                    };

                    assert!(
                        attr_loc < vertex_layout.len() as u32,
                        "attribute: {} outside of allocated attributes array len: {}",
                        name,
                        vertex_layout.len()
                    );
                    vertex_layout[attr_loc as usize] = Some(attr);
                }
                buffer_data.offset += format.byte_len() as i64
            }
        }

        let pipeline = PipelineInternal {
            layout: vertex_layout,
            shader,
            params,
        };

        ctx.pipelines.push(pipeline);
        Pipeline(ctx.pipelines.len() - 1)
    }

    pub fn set_blend(&self, ctx: &mut GraphicsContext, color_blend: Option<BlendState>) {
        let mut pipeline = &mut ctx.pipelines[self.0];
        pipeline.params.color_blend = color_blend;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub(crate) struct VertexAttributeInternal {
    pub(crate) attr_loc: GLuint,
    pub(crate) size: i32,
    pub(crate) type_: GLuint,
    pub(crate) offset: i64,
    pub(crate) stride: i32,
    pub(crate) buffer_index: usize,
    pub(crate) divisor: i32,
}

pub(crate) struct PipelineInternal {
    pub(crate) layout: Vec<Option<VertexAttributeInternal>>,
    pub(crate) shader: Shader,
    pub(crate) params: PipelineConf,
}

impl GraphicsContext {
    pub fn apply_pipeline(&mut self, pipeline: &Pipeline) {
        self.cache.cur_pipeline = Some(*pipeline);

        {
            let pipeline = &self.pipelines[pipeline.0];
            let shader = &mut self.shaders[pipeline.shader.0];
            unsafe {
                glUseProgram(shader.program);
            }

            unsafe {
                glEnable(GL_SCISSOR_TEST);
            }

            if pipeline.params.depth_write {
                unsafe {
                    glEnable(GL_DEPTH_TEST);
                    glDepthFunc(pipeline.params.depth_test.into())
                }
            } else {
                unsafe {
                    glDisable(GL_DEPTH_TEST);
                }
            }

            match pipeline.params.front_face_order {
                FrontFaceOrder::Clockwise => unsafe {
                    glFrontFace(GL_CW);
                },
                FrontFaceOrder::CounterClockwise => unsafe {
                    glFrontFace(GL_CCW);
                },
            }
        }

        self.set_cull_face(self.pipelines[pipeline.0].params.cull_face);
        self.set_blend(
            self.pipelines[pipeline.0].params.color_blend,
            self.pipelines[pipeline.0].params.alpha_blend,
        );

        self.set_stencil(self.pipelines[pipeline.0].params.stencil_test);
        self.set_color_write(self.pipelines[pipeline.0].params.color_write);
    }
}
