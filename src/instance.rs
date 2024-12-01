//! An `Instance` is a running WASM instance with its store and memory.

use std::{slice::from_raw_parts, sync::Arc};

use parking_lot::Mutex;
use wasmer::{Engine, Extern, Function, Memory, Module, Store};
use wasmer_wasix::{generate_import_object_from_env, WasiEnv, WasiFunctionEnv, WasiVersion};

use crate::{
    aggregate_result::convert_aggregate_result, buffer::Buffer,
    prepare_data_result::convert_prepare_data_result, Error, Slice,
};

#[derive(Debug, Clone)]
pub struct Instance {
    instance: wasmer::Instance,
    memory: Memory,
    store: Arc<Mutex<Store>>,
    prepare_data: Function,
    buffer_array_create: Function,
    buffer_array_set_buffer: Function,
    aggregate: Function,
    wasi_env: WasiFunctionEnv,
}

pub struct Aggregation {
    pub data: Vec<u8>,
    pub digest: Vec<u8>,
}

impl Instance {
    pub fn new(engine: Engine, wasm_bytes: &[u8]) -> Result<Self, Error> {
        let mut store = Store::new(engine);
        let module = Module::new(&store, wasm_bytes)?;

        let mut wasi_env = WasiEnv::builder("wasm-processor-prototype")
            .finalize(&mut store)
            .map_err(Box::new)?;
        let mut imports =
            generate_import_object_from_env(&mut store, &wasi_env.env, WasiVersion::Snapshot1);
        imports.extend(&generate_import_object_from_env(
            &mut store,
            &wasi_env.env,
            WasiVersion::Wasix32v1,
        ));
        let memory_import = module
            .imports()
            .memories()
            .next()
            .expect("WASI module must have a memory import");
        let memory = Memory::new(&mut store, *memory_import.ty())?;
        imports.define(
            memory_import.module(),
            memory_import.name(),
            Extern::Memory(memory.clone()),
        );
        let instance = wasmer::Instance::new(&mut store, &module, &imports).map_err(Box::new)?;

        wasi_env.initialize_with_memory(
            &mut store,
            instance.clone(),
            Some(memory.clone()),
            true,
        )?;

        let prepare_data = instance.exports.get_function("prepare_data")?.clone();
        let buffer_array_create = instance
            .exports
            .get_function("buffer_array_create")?
            .clone();
        let buffer_array_set_buffer = instance
            .exports
            .get_function("buffer_array_set_buffer")?
            .clone();
        let aggregate = instance.exports.get_function("aggregate")?.clone();

        Ok(Instance {
            instance,
            memory,
            store: Arc::new(Mutex::new(store)),
            prepare_data,
            buffer_array_create,
            buffer_array_set_buffer,
            aggregate,
            wasi_env,
        })
    }

    pub fn inner(&self) -> &wasmer::Instance {
        &self.instance
    }

    pub fn memory(&self) -> &Memory {
        &self.memory
    }

    pub fn store(&self) -> &Arc<Mutex<Store>> {
        &self.store
    }

    pub fn prepare_data(
        &self,
        config: &[u8],
        request: &[u8],
    ) -> Result<Result<Vec<u8>, String>, Error> {
        let config = Buffer::new_with_data(self, config)?;
        let request = Buffer::new_with_data(self, request)?;
        let result = self.prepare_data.call(
            &mut self.store.lock(),
            &[config.inner().clone(), request.inner().clone()],
        )?;
        convert_prepare_data_result(self, &result[0])
    }

    pub fn aggregate(
        &self,
        config: &[u8],
        data: &[&[u8]],
        request: &[u8],
    ) -> Result<Result<Aggregation, String>, Error> {
        let config = Buffer::new_with_data(self, config)?;
        let data_array = self
            .buffer_array_create
            .call(&mut self.store.lock(), &[(data.len() as i32).into()])?[0]
            .clone();
        for (index, data) in data.iter().enumerate() {
            let buffer = Buffer::new_with_data(self, data)?;
            self.buffer_array_set_buffer.call(
                &mut self.store.lock(),
                &[
                    data_array.clone(),
                    (index as i32).into(),
                    buffer.inner().clone(),
                ],
            )?;
        }
        let request = Buffer::new_with_data(self, request)?;
        let result = self.aggregate.call(
            &mut self.store.lock(),
            &[config.inner().clone(), data_array, request.inner().clone()],
        )?;
        convert_aggregate_result(self, &result[0])
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        self.wasi_env.on_exit(&mut self.store.lock(), None);
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct PrepareDataResult {
    pub data: crate::Buffer,
    pub is_error: bool,
}

#[no_mangle]
pub extern "C" fn instance_prepare_data(
    instance: &Instance,
    config: Slice,
    request: Slice,
) -> Box<PrepareDataResult> {
    let config = unsafe { from_raw_parts(config.ptr, config.len) };
    let request = unsafe { from_raw_parts(request.ptr, request.len) };
    Box::new(match instance.prepare_data(config, request) {
        Ok(Ok(data)) => PrepareDataResult {
            data: data.into(),
            is_error: false,
        },
        Ok(Err(error)) => PrepareDataResult {
            data: error.into_bytes().into(),
            is_error: true,
        },
        Err(error) => PrepareDataResult {
            data: format!("runtime error: {}", error).into_bytes().into(),
            is_error: true,
        },
    })
}

#[no_mangle]
pub extern "C" fn prepare_data_result_destroy(result: Box<PrepareDataResult>) {
    drop(result);
}

#[repr(C)]
#[derive(Debug)]
pub struct AggregateResult {
    pub data: crate::Buffer,
    pub is_error: bool,
    pub digest: Option<Box<crate::Buffer>>,
}

#[no_mangle]
pub extern "C" fn aggregate_result_destroy(result: Box<AggregateResult>) {
    drop(result);
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SliceArray {
    pub ptr: *mut Slice,
    pub len: usize,
}

#[no_mangle]
pub extern "C" fn instance_aggregate(
    instance: &Instance,
    config: Slice,
    data: SliceArray,
    request: Slice,
) -> Box<AggregateResult> {
    let config = unsafe { from_raw_parts(config.ptr, config.len) };
    let data = unsafe {
        (0..data.len)
            .map(|index| {
                let slice = &*data.ptr.add(index);
                from_raw_parts(slice.ptr, slice.len)
            })
            .collect::<Vec<_>>()
    };
    let request = unsafe { from_raw_parts(request.ptr, request.len) };
    Box::new(match instance.aggregate(config, &data, request) {
        Ok(Ok(aggregation)) => AggregateResult {
            data: aggregation.data.into(),
            is_error: false,
            digest: Some(Box::new(aggregation.digest.into())),
        },
        Ok(Err(error)) => AggregateResult {
            data: error.into_bytes().into(),
            is_error: true,
            digest: None,
        },
        Err(error) => AggregateResult {
            data: format!("runtime error: {}", error).into_bytes().into(),
            is_error: true,
            digest: None,
        },
    })
}
