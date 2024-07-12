use std::mem::transmute;

use bytemuck::PodCastError;
use naga::{
    front::spv,
    valid::{Capabilities, ValidationFlags, Validator},
    Binding, ScalarKind, VectorSize,
};
use wasmer::ValueType;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BufferUsages, CommandEncoder, MultisampleState, PrimitiveState, RenderPipelineDescriptor,
    ShaderModule, ShaderModuleDescriptor, ShaderSource, SurfaceTexture, TextureView, VertexFormat,
};

use crate::{app::System, display::MaybeGraphics};

#[derive(Clone, Copy, Debug, ValueType)]
#[repr(C)]
pub struct BufferSlice {
    pub ptr: u32,
    pub size: u32,
    pub item_size: u32,
}

pub struct Buffer {
    pub buffer: wgpu::Buffer,
    pub item_size: usize,
    pub usage: BufferUsages,
}

#[derive(Clone, Copy, Debug, ValueType)]
#[repr(C)]
pub struct ExternPipelineInfo {
    pub attributes: Span,
    pub fragment: ExternPipelineShaderInfo,
    pub vertex: ExternPipelineShaderInfo,
}

#[derive(Clone, Copy, Debug, ValueType)]
#[repr(C)]
pub struct ExternPipelineShaderInfo {
    pub entry_point: Span,
    pub shader: u32,
}

#[derive(Clone, Debug)]
pub struct PipelineInfo {
    pub attributes: Vec<VertexAttribute>,
    pub fragment: PipelineShaderInfo,
    pub vertex: PipelineShaderInfo,
}

#[derive(Clone, Debug)]
pub struct PipelineShaderInfo {
    pub entry_point: String,
    pub shader: u32,
}

pub struct RenderFrame {
    pub encoder: CommandEncoder,
    pub frame: SurfaceTexture,
    pub pass: Option<wgpu::RenderPass<'static>>,
    pub pipelined: bool,
    pub view: TextureView,
}

pub struct Shader {
    compiled: ShaderModule,
    info: naga::valid::ModuleInfo,
    module: naga::Module,
}

#[derive(Clone, Copy, Debug, ValueType)]
#[repr(C)]
pub struct Span {
    pub ptr: u32,
    pub len: u32,
}

#[derive(Clone, Copy, Debug, ValueType)]
#[repr(C)]
pub struct VertexAttribute {
    pub format: u32,
    pub buffer_index: u32,
}

struct VertexAttributesInfo {
    attributes: Vec<wgpu::VertexAttribute>,
    stride: u64,
}

pub fn buffered_ensure<'a>(system: &'a mut System) {
    let MaybeGraphics::Graphics(gfx) = &mut system.display.graphics else {
        return;
    };
    pass_ensure(system);
    let Some(frame) = system.frame.as_mut() else {
        return;
    };
    let Some(pass) = &mut frame.pass else {
        return;
    };
    let pass = unsafe { transmute::<_, &mut wgpu::RenderPass<'a>>(pass) };
    let index = system
        .buffers
        .iter()
        .find(|it| it.usage == BufferUsages::INDEX)
        .unwrap();
    let vertex = system
        .buffers
        .iter()
        .find(|it| it.usage == BufferUsages::VERTEX)
        .unwrap();
    pass.set_index_buffer(index.buffer.slice(..), wgpu::IndexFormat::Uint16);
    pass.set_vertex_buffer(0, vertex.buffer.slice(..));
}

pub fn create_buffer(
    system: &mut System,
    contents: &[u8],
    typ: u32,
    _usage: u32,
    item_size: usize,
) {
    let MaybeGraphics::Graphics(gfx) = &mut system.display.graphics else {
        panic!();
    };
    let usage = match typ {
        0 => BufferUsages::VERTEX,
        1 => BufferUsages::INDEX,
        _ => panic!(),
    };
    let buffer = gfx.device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents,
        usage,
    });
    system.buffers.push(Buffer {
        buffer,
        item_size,
        usage,
    });
}

pub fn create_pipeline(system: &mut System, info: PipelineInfo) {
    // let shader = &system.shaders[info.vertex.shader as usize];
}

pub fn end_pass(system: &mut System) {
    let Some(frame) = system.frame.as_mut() else {
        return;
    };
    if let Some(pass) = frame.pass.take() {
        drop(pass);
    }
}

pub fn pass_ensure(system: &mut System) {
    let MaybeGraphics::Graphics(gfx) = &mut system.display.graphics else {
        return;
    };
    if system.frame.is_none() {
        let frame = gfx.surface.get_current_texture().unwrap();
        let view = frame.texture.create_view(&Default::default());
        let encoder = gfx.device.create_command_encoder(&Default::default());
        system.frame = Some(RenderFrame {
            encoder,
            frame,
            pass: None,
            pipelined: false,
            view,
        });
    }
    let Some(frame) = system.frame.as_mut() else {
        panic!()
    };
    if frame.pass.is_some() {
        return;
    }
    let view = &frame.view;
    let encoder = &mut frame.encoder;
    let pass = unsafe { &mut *(encoder as *mut CommandEncoder) }.begin_render_pass(
        &wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: unsafe { &*(view as *const TextureView) },
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        },
    );
    frame.pass = Some(unsafe { transmute(pass) });
}

fn pipeline_ensure(system: &mut System) {
    if !system.pipelines.is_empty() {
        return;
    }
    let MaybeGraphics::Graphics(gfx) = &mut system.display.graphics else {
        return;
    };
    let Some(shader) = system.shaders.get(0) else {
        return;
    };
    let device = &gfx.device;
    // gfx.surface.get_default_config(gfx., width, height)
    // let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
    //     label: None,
    //     entries: &[BindGroupLayoutEntry {
    //         binding: 0,
    //         visibility: ShaderStages::VERTEX_FRAGMENT,
    //         ty: wgpu::BindingType::Buffer {
    //             ty: wgpu::BufferBindingType::Uniform,
    //             has_dynamic_offset: false,
    //             min_binding_size: None, // wgpu::BufferSize::new(64),
    //         },
    //         count: None,
    //     }],
    // });
    // let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
    //     label: None,
    //     bind_group_layouts: &[&bind_group_layout],
    //     push_constant_ranges: &[],
    // });
    let vertex_entry_point = "vs_main";
    let attr_info = vertex_attributes_build(shader, vertex_entry_point);
    let vertex_attr_layout = wgpu::VertexBufferLayout {
        array_stride: attr_info.stride,
        // TODO Which vertex and which instance?
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &attr_info.attributes,
    };
    let surface_formats = gfx.surface.get_capabilities(&gfx.adapter).formats;
    let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
        label: None,
        // layout: Some(&pipeline_layout),
        layout: None,
        vertex: wgpu::VertexState {
            module: &shader.compiled,
            entry_point: vertex_entry_point,
            compilation_options: Default::default(),
            buffers: &[vertex_attr_layout],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader.compiled,
            entry_point: "fs_main",
            compilation_options: Default::default(),
            targets: &[Some(surface_formats[0].into())],
        }),
        primitive: PrimitiveState {
            cull_mode: Some(wgpu::Face::Back),
            ..Default::default()
        },
        depth_stencil: None,
        multisample: MultisampleState::default(),
        multiview: None,
    });
    system.pipelines.push(pipeline);
}

pub fn pipelined_ensure<'a>(system: &'a mut System) {
    pipeline_ensure(system);
    pass_ensure(system);
    let Some(frame) = system.frame.as_mut() else {
        return;
    };
    if frame.pipelined {
        return;
    }
    let Some(pass) = &mut frame.pass else {
        return;
    };
    let pass = unsafe { transmute::<_, &mut wgpu::RenderPass<'a>>(pass) };
    pass.set_pipeline(&system.pipelines[0]);
}

pub fn shader_create(system: &mut System, bytes: &[u8]) -> Shader {
    let MaybeGraphics::Graphics(gfx) = &mut system.display.graphics else {
        panic!();
    };
    let module = spv::parse_u8_slice(bytes, &Default::default()).unwrap();
    let mut validator = Validator::new(ValidationFlags::all(), Capabilities::empty());
    let info = validator
        .validate(&module)
        .expect("Shader validation failed");
    let mut spirv_buffer = Vec::<u32>::new();
    let spirv: &[u32] = bytemuck::try_cast_slice(bytes).unwrap_or_else(|err| match err {
        PodCastError::AlignmentMismatch => {
            // Copy into an aligned buffer if not already aligned.
            for chunk in bytes.chunks_exact(4) {
                let word = u32::from_le_bytes(chunk.try_into().unwrap());
                spirv_buffer.push(word);
            }
            &spirv_buffer
        }
        _ => panic!(),
    });
    let compiled = gfx.device.create_shader_module(ShaderModuleDescriptor {
        label: None,
        source: ShaderSource::SpirV(std::borrow::Cow::Borrowed(spirv)),
    });
    Shader {
        compiled,
        info,
        module,
    }
}

pub fn uniforms_apply(system: &mut System, bytes: &[u8]) {
    pipelined_ensure(system);
}

fn vertex_attributes_build(shader: &Shader, entry_point: &str) -> VertexAttributesInfo {
    let entry = shader
        .module
        .entry_points
        .iter()
        .find(|it| it.name == entry_point)
        .unwrap();
    let types = &shader.module.types;
    let mut offset = 0;
    let attributes: Vec<_> = entry
        .function
        .arguments
        .iter()
        .filter_map(|arg| {
            let Some(Binding::Location { location, .. }) = arg.binding else {
                return None;
            };
            let format = match &types[arg.ty].inner {
                naga::TypeInner::Scalar(naga::Scalar { kind, width }) => match (kind, width) {
                    (ScalarKind::Sint, 4) => VertexFormat::Sint32,
                    (ScalarKind::Uint, 4) => VertexFormat::Uint32,
                    (ScalarKind::Float, 4) => VertexFormat::Float32,
                    _ => todo!(),
                },
                naga::TypeInner::Vector {
                    size,
                    scalar: naga::Scalar { kind, width },
                } => match (kind, width, size) {
                    (ScalarKind::Float, 4, VectorSize::Bi) => VertexFormat::Float32x2,
                    (ScalarKind::Float, 4, VectorSize::Tri) => VertexFormat::Float32x3,
                    (ScalarKind::Float, 4, VectorSize::Quad) => VertexFormat::Float32x4,
                    _ => todo!(),
                },
                naga::TypeInner::Matrix { .. } => todo!(),
                naga::TypeInner::Atomic(_) => todo!(),
                naga::TypeInner::Pointer { .. } => todo!(),
                naga::TypeInner::ValuePointer { .. } => todo!(),
                naga::TypeInner::Array { .. } => todo!(),
                naga::TypeInner::Struct { .. } => todo!(),
                naga::TypeInner::Image { .. } => todo!(),
                naga::TypeInner::Sampler { .. } => todo!(),
                naga::TypeInner::AccelerationStructure => todo!(),
                naga::TypeInner::RayQuery => todo!(),
                naga::TypeInner::BindingArray { .. } => todo!(),
            };
            let attr = wgpu::VertexAttribute {
                format,
                offset,
                shader_location: location,
            };
            offset += format.size();
            Some(attr)
        })
        .collect();
    VertexAttributesInfo {
        attributes,
        // TODO Padding/alignment above and here.
        stride: offset,
    }
}
