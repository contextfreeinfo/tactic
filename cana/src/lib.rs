use lz4_flex::frame::FrameDecoder;
use naga::{
    back::glsl::{self, WriterFlags},
    front::spv::{self, Options},
    proc::{BoundsCheckPolicies, BoundsCheckPolicy},
    valid::{Capabilities, ModuleInfo, ValidationFlags, Validator},
    Module, ShaderStage,
};
use std::io::Read;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

// TODO Rename this for js access?
#[wasm_bindgen(js_name = "lz4Decompress")]
pub fn lz4_decompress(source: &[u8]) -> Vec<u8> {
    let mut dest = vec![0u8; 0];
    FrameDecoder::new(source).read_to_end(&mut dest).unwrap();
    dest
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    alert(&format!("Hello, {}!", name));
}

#[wasm_bindgen]
pub struct Shader {
    module: Module,
    info: ModuleInfo,
}

impl Shader {
    pub fn new(bytes: &[u8]) -> Shader {
        let module = spv::parse_u8_slice(bytes, &Options::default()).unwrap();
        let mut validator = Validator::new(ValidationFlags::all(), Capabilities::empty());
        let info = validator.validate(&module).unwrap();
        Shader { module, info }
    }
}

#[wasm_bindgen(js_name = "shaderNew")]
pub fn shader_new(bytes: &[u8]) -> Shader {
    Shader::new(bytes)
}

#[no_mangle]
pub fn translate_to_glsl(
    module: &Module,
    info: &ModuleInfo,
    shader_stage: ShaderStage,
    entry_point: String,
) -> String {
    let options = glsl::Options {
        // TODO Bind to specific locations if miniquad could support explicit locations?
        // binding_map: todo!(),
        version: glsl::Version::Embedded {
            version: 300,
            is_webgl: true,
        },
        writer_flags: WriterFlags::empty(),
        ..glsl::Options::default()
    };
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
        BoundsCheckPolicies {
            // Be safe by default until I know better.
            index: BoundsCheckPolicy::Restrict,
            buffer: BoundsCheckPolicy::Restrict,
            image_load: BoundsCheckPolicy::Restrict,
            image_store: BoundsCheckPolicy::Restrict,
            binding_array: BoundsCheckPolicy::Restrict,
        },
    )
    .unwrap();
    writer.write().unwrap();
    buffer
}