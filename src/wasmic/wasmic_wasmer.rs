#![allow(non_snake_case)]

use miniquad::EventHandler;
use wasmer::{
    imports, Function, FunctionEnv, FunctionEnvMut, Instance, MemoryView, Module, Store, Value,
    ValueType, WasmPtr, WasmRef,
};

use crate::platform::{Platform, WindowState};

use super::help::{
    apply_bindings, apply_pipeline, apply_uniforms, begin_pass, commit_frame, draw, end_pass,
    new_buffer, new_pipeline, new_rendering_context, new_shader, Bindings, BufferSlice,
    ExternBindings, ExternPipelineInfo, PipelineInfo, PipelineShaderInfo, Span,
};

pub struct App {
    env: FunctionEnv<Platform>,
    listen: Function,
    store: Store,
}

impl App {
    pub fn new(store: Store, instance: Instance, env: FunctionEnv<Platform>) -> Self {
        let listen = instance.exports.get_function("listen").unwrap().clone();
        Self { env, listen, store }
    }
}

impl<'a> EventHandler for App {
    fn update(&mut self) {}

    fn draw(&mut self) {
        self.listen.call(&mut self.store, &[Value::I32(0)]).unwrap();
    }

    fn mouse_motion_event(&mut self, x: f32, y: f32) {
        let platform = self.env.as_mut(&mut self.store);
        platform.window_state.pointer = [x, y];
    }

    fn resize_event(&mut self, width: f32, height: f32) {
        let platform = self.env.as_mut(&mut self.store);
        platform.window_state.size = [width, height];
    }
}

pub fn wasmish(wasm: &[u8]) -> App {
    let mut store = Store::default();
    let module = Module::new(&store, wasm).unwrap();
    let env = FunctionEnv::new(&mut store, Platform::new(0));
    let import_object = imports! {
        "env" => {
            "taca_RenderingContext_applyBindings" => Function::new_typed_with_env(&mut store, &env, taca_RenderingContext_applyBindings),
            "taca_RenderingContext_applyPipeline" => Function::new_typed_with_env(&mut store, &env, taca_RenderingContext_applyPipeline),
            "taca_RenderingContext_applyUniforms" => Function::new_typed_with_env(&mut store, &env, taca_RenderingContext_applyUniforms),
            "taca_RenderingContext_beginPass" => Function::new_typed_with_env(&mut store, &env, taca_RenderingContext_beginPass),
            "taca_RenderingContext_commitFrame" => Function::new_typed_with_env(&mut store, &env, taca_RenderingContext_commitFrame),
            "taca_RenderingContext_draw" => Function::new_typed_with_env(&mut store, &env, taca_RenderingContext_draw),
            "taca_RenderingContext_endPass" => Function::new_typed_with_env(&mut store, &env, taca_RenderingContext_endPass),
            "taca_RenderingContext_newBuffer" => Function::new_typed_with_env(&mut store, &env, taca_RenderingContext_newBuffer),
            "taca_RenderingContext_newPipeline" => Function::new_typed_with_env(&mut store, &env, taca_RenderingContext_newPipeline),
            "taca_RenderingContext_newShader" => Function::new_typed_with_env(&mut store, &env, taca_RenderingContext_newShader),
            "taca_Window_get" => Function::new_typed_with_env(&mut store, &env, taca_Window_get),
            "taca_Window_newRenderingContext" => Function::new_typed_with_env(&mut store, &env, taca_Window_newRenderingContext),
            "taca_Window_print" => Function::new_typed_with_env(&mut store, &env, taca_Window_print),
            "taca_Window_state" => Function::new_typed_with_env(&mut store, &env, taca_Window_state),
        }
    };
    let instance = Instance::new(&mut store, &module, &import_object).unwrap();
    let platform = env.as_mut(&mut store);
    platform.init_state();
    platform.memory = Some(instance.exports.get_memory("memory").unwrap().clone());
    // Start up.
    let start = instance.exports.get_function("_start").unwrap();
    start.call(&mut store, &[]).unwrap();
    App::new(store, instance, env)
}

pub fn print(text: &str) {
    println!("{text}");
}

fn read_span<T>(view: &MemoryView, span: Span) -> Vec<T>
where
    T: Copy + ValueType,
{
    WasmPtr::<T>::new(span.ptr)
        .slice(&view, span.len)
        .unwrap()
        .read_to_vec()
        .unwrap()
}

fn read_string(view: &MemoryView, span: Span) -> String {
    WasmPtr::<u8>::new(span.ptr)
        .read_utf8_string(&view, span.len)
        .unwrap()
}

fn taca_RenderingContext_applyBindings(
    mut env: FunctionEnvMut<Platform>,
    context: u32,
    bindings: u32,
) {
    let (platform, store) = env.data_and_store_mut();
    let view = platform.memory.as_ref().unwrap().view(&store);
    let bindings = WasmPtr::<ExternBindings>::new(bindings)
        .read(&view)
        .unwrap();
    let bindings = Bindings {
        vertex_buffers: read_span(&view, bindings.vertex_buffers),
        index_buffer: bindings.index_buffer,
    };
    apply_bindings(platform, context, bindings);
}

fn taca_RenderingContext_applyPipeline(
    mut env: FunctionEnvMut<Platform>,
    context: u32,
    pipeline: u32,
) {
    let platform = env.data_mut();
    apply_pipeline(platform, context, pipeline)
}

fn taca_RenderingContext_applyUniforms(
    mut env: FunctionEnvMut<Platform>,
    context: u32,
    bytes: u32,
) {
    let (platform, store) = env.data_and_store_mut();
    let view = platform.memory.as_ref().unwrap().view(&store);
    let uniforms = WasmPtr::<Span>::new(bytes).read(&view).unwrap();
    let uniforms = read_span::<u8>(&view, uniforms);
    apply_uniforms(platform, context, &uniforms);
}

fn taca_RenderingContext_beginPass(mut env: FunctionEnvMut<Platform>, context: u32) {
    let platform = env.data_mut();
    begin_pass(platform, context)
}

fn taca_RenderingContext_commitFrame(mut env: FunctionEnvMut<Platform>, context: u32) {
    let platform = env.data_mut();
    commit_frame(platform, context)
}

fn taca_RenderingContext_draw(
    mut env: FunctionEnvMut<Platform>,
    context: u32,
    item_begin: i32,
    item_count: i32,
    instance_count: i32,
) {
    let platform = env.data_mut();
    draw(platform, context, item_begin, item_count, instance_count);
}

fn taca_RenderingContext_endPass(mut env: FunctionEnvMut<Platform>, context: u32) {
    let platform = env.data_mut();
    end_pass(platform, context)
}

fn taca_RenderingContext_newBuffer(
    mut env: FunctionEnvMut<Platform>,
    context: u32,
    typ: u32,
    usage: u32,
    slice: u32,
) -> u32 {
    let (platform, store) = env.data_and_store_mut();
    let view = platform.memory.as_ref().unwrap().view(&store);
    let slice = WasmPtr::<BufferSlice>::new(slice).read(&view).unwrap();
    let buffer = view
        .copy_range_to_vec(slice.ptr as u64..(slice.ptr + slice.size) as u64)
        .unwrap();
    new_buffer(
        platform,
        context,
        typ,
        usage,
        &buffer,
        slice.item_size as usize,
    )
}

fn taca_RenderingContext_newPipeline(
    mut env: FunctionEnvMut<Platform>,
    context: u32,
    info: u32,
) -> u32 {
    let (platform, store) = env.data_and_store_mut();
    let view = platform.memory.as_ref().unwrap().view(&store);
    let info = WasmPtr::<ExternPipelineInfo>::new(info)
        .read(&view)
        .unwrap();
    let attributes = read_span(&view, info.attributes);
    let info = PipelineInfo {
        attributes,
        fragment: PipelineShaderInfo {
            entry_point: read_string(&view, info.fragment.entry_point),
            shader: info.fragment.shader,
        },
        vertex: PipelineShaderInfo {
            entry_point: read_string(&view, info.vertex.entry_point),
            shader: info.vertex.shader,
        },
    };
    new_pipeline(platform, context, info)
}

fn taca_RenderingContext_newShader(
    mut env: FunctionEnvMut<Platform>,
    _context: u32,
    bytes: u32,
) -> u32 {
    let (platform, store) = env.data_and_store_mut();
    let view = platform.memory.as_ref().unwrap().view(&store);
    let bytes = WasmPtr::<Span>::new(bytes).read(&view).unwrap();
    let bytes = read_span(&view, bytes);
    new_shader(platform, &bytes)
}

fn taca_Window_get(mut _env: FunctionEnvMut<Platform>) -> u32 {
    1
}

fn taca_Window_newRenderingContext(mut env: FunctionEnvMut<Platform>, _window: u32) -> u32 {
    new_rendering_context(env.data_mut())
}

fn taca_Window_print(mut env: FunctionEnvMut<Platform>, _window: u32, text: u32) {
    let (platform, store) = env.data_and_store_mut();
    let view = platform.memory.as_ref().unwrap().view(&store);
    let text = WasmPtr::<Span>::new(text).read(&view).unwrap();
    let text = read_string(&view, text);
    print(&text);
}

fn taca_Window_state(mut env: FunctionEnvMut<Platform>, result: u32, _window: u32) {
    let (platform, store) = env.data_and_store_mut();
    let view = platform.memory.as_ref().unwrap().view(&store);
    WasmRef::<WindowState>::new(&view, result as u64)
        .write(platform.window_state)
        .unwrap();
}