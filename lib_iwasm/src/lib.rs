use std::slice::from_raw_parts;

use instance::Instance;
use wasmer::{
    CompileError, Engine, EngineBuilder, ExportError, Features, InstantiationError,
    MemoryAccessError, MemoryError, RuntimeError,
};
use wasmer_compiler_singlepass::Singlepass;
use wasmer_wasix::WasiRuntimeError;

#[derive(Debug)]
pub struct Runtime {
    runtime: tokio::runtime::Runtime,
    engine: Engine,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("compile: {0}")]
    Compile(#[from] CompileError),
    #[error("build tokio runtime: {0}")]
    BuildRuntime(#[source] std::io::Error),
    #[error("wasi runtime: {0}")]
    WasiRuntime(#[from] Box<WasiRuntimeError>),
    #[error("memory")]
    Memory(#[from] MemoryError),
    #[error("instantiation: {0}")]
    Instantiation(#[from] Box<InstantiationError>),
    #[error("export: {0}")]
    Export(#[from] ExportError),
    #[error("runtime: {0}")]
    Runtime(#[from] RuntimeError),
    #[error("memory access: {0}")]
    MemoryAccess(#[from] MemoryAccessError),
}

mod aggregate_result;
mod buffer;
mod instance;
mod prepare_data_result;

impl Runtime {
    fn new() -> Result<Self, std::io::Error> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;
        let engine = EngineBuilder::new(Singlepass::new())
            .set_features(Some(Features::new()))
            .engine();
        Ok(Runtime {
            runtime,
            engine: engine.into(),
        })
    }

    fn create_instance(&self, wasm_bytes: &[u8]) -> Result<Instance, Error> {
        Instance::new(self.engine.clone(), wasm_bytes)
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct Buffer {
    pub ptr: *mut u8,
    pub len: usize,
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            std::alloc::dealloc(
                self.ptr,
                std::alloc::Layout::from_size_align(self.len, 1).unwrap(),
            );
        }
    }
}

impl From<Vec<u8>> for Buffer {
    fn from(mut data: Vec<u8>) -> Self {
        let len = data.len();
        let ptr = data.as_mut_ptr();
        std::mem::forget(data);
        Buffer { ptr, len }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct RuntimeResult {
    pub runtime: Option<Box<Runtime>>,
    pub error: Option<Box<Buffer>>,
}

#[no_mangle]
pub extern "C" fn runtime_result_destroy(result: Box<RuntimeResult>) {
    drop(result);
}

#[no_mangle]
pub extern "C" fn runtime_create() -> Box<RuntimeResult> {
    match Runtime::new() {
        Ok(runtime) => Box::new(RuntimeResult {
            runtime: Some(Box::new(runtime)),
            error: None,
        }),
        Err(error) => Box::new(RuntimeResult {
            runtime: None,
            error: Some(Box::new(error.to_string().into_bytes().into())),
        }),
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Slice {
    pub ptr: *mut u8,
    pub len: usize,
}

#[no_mangle]
pub extern "C" fn slice_to_buffer(slice: Slice) -> Box<Buffer> {
    let data = unsafe { from_raw_parts(slice.ptr, slice.len) };
    Box::new(data.to_vec().into())
}

#[no_mangle]
pub extern "C" fn buffer_destroy(buffer: Box<Buffer>) {
    drop(buffer);
}

#[no_mangle]
pub extern "C" fn buffer_slice(buffer: &Buffer) -> Slice {
    Slice {
        ptr: buffer.ptr,
        len: buffer.len,
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct InstanceResult {
    pub instance: Option<Box<Instance>>,
    pub error: Option<Box<Buffer>>,
}

#[no_mangle]
pub extern "C" fn instance_result_destroy(result: Box<InstanceResult>) {
    drop(result);
}

#[no_mangle]
pub extern "C" fn runtime_create_instance(
    runtime: &Runtime,
    wasm_bytes: Slice,
) -> Box<InstanceResult> {
    let _guard = runtime.runtime.enter();

    let wasm_bytes = unsafe { from_raw_parts(wasm_bytes.ptr, wasm_bytes.len) };
    match runtime.create_instance(wasm_bytes) {
        Ok(instance) => Box::new(InstanceResult {
            instance: Some(Box::new(instance)),
            error: None,
        }),
        Err(error) => Box::new(InstanceResult {
            instance: None,
            error: Some(Box::new(error.to_string().into_bytes().into())),
        }),
    }
}

#[test]
fn test() {
    let runtime = Runtime::new().unwrap();
    let wasm_bytes =
        include_bytes!("../../processor/target/wasm32-wasmer-wasi/release/processor.wasm");
    let _guard = runtime.runtime.enter();
    let instance = runtime.create_instance(wasm_bytes).unwrap();
    let config = "IntelliX";
    let request = "https://api.coingecko.com/api/v3/coins/bitcoin/history?date=24-11-2024";
    let data = instance
        .prepare_data(config.as_bytes(), request.as_bytes())
        .unwrap()
        .unwrap();
    let request = "first";
    let aggregation = instance
        .aggregate(config.as_bytes(), &[&data], request.as_bytes())
        .unwrap()
        .unwrap();
    assert_eq!(
        aggregation.data,
        serde_json::from_str::<serde_json::Value>(include_str!("../../expected.json")).unwrap()
            ["market_data"]["current_price"]["usd"]
            .as_f64()
            .unwrap()
            .to_le_bytes()
            .to_vec()
    );
}
