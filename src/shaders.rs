use anyhow::Result;
use naga::{
    back::glsl::{self, WriterFlags},
    front::spv::{self, Options},
    proc::BoundsCheckPolicies,
    valid::{Capabilities, ValidationFlags, Validator},
    ShaderStage,
};

#[derive(Debug)]
pub struct GlslShaders {
    pub vertex: String,
    pub fragment: String,
}

pub fn shaders() -> Result<GlslShaders> {
    let source = include_bytes!("shader.spv");
    // println!("{source}");
    let module = spv::parse_u8_slice(source, &Options::default())?;
    let mut validator = Validator::new(ValidationFlags::all(), Capabilities::empty());
    let info = validator.validate(&module)?;
    let vertex = translate_to_glsl(&module, &info, ShaderStage::Vertex)?;
    let fragment = translate_to_glsl(&module, &info, ShaderStage::Fragment)?;
    Ok(GlslShaders { vertex, fragment })
}

fn translate_to_glsl(
    module: &naga::Module,
    info: &naga::valid::ModuleInfo,
    shader_stage: naga::ShaderStage,
) -> Result<String> {
    let options = glsl::Options {
        version: glsl::Version::Embedded {
            version: 300,
            is_webgl: true,
        },
        writer_flags: WriterFlags::empty(),
        ..glsl::Options::default()
    };
    let entry_point = match shader_stage {
        ShaderStage::Vertex => "vs_main",
        ShaderStage::Fragment => "fs_main",
        ShaderStage::Compute => todo!(),
    }
    .into();
    let pipeline_options = glsl::PipelineOptions {
        shader_stage,
        entry_point,
        multiview: None,
    };
    let mut buffer = String::new();
    let mut writer = glsl::Writer::new(
        &mut buffer,
        module,
        info,
        &options,
        &pipeline_options,
        // TODO What bounds checks???
        BoundsCheckPolicies::default(),
    )?;
    writer.write()?;
    // println!("{buffer}");
    Ok(buffer)
}
