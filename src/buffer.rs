//! Wrapper for WASM `Buffer` type.

use std::sync::Arc;

use parking_lot::Mutex;
use wasmer::{Memory, Store, Value};

use crate::{instance::Instance, Error};

#[derive(Debug)]
pub struct Buffer {
    memory: Memory,
    buffer: Value,
    ptr: u64,
    len: usize,
    store: Arc<Mutex<Store>>,
}

impl Buffer {
    pub fn new(instance: &Instance, len: usize) -> Result<Self, Error> {
        let buffer_create = instance.inner().exports.get_function("buffer_create")?;
        let buffer =
            buffer_create.call(&mut instance.store().lock(), &[Value::I32(len as i32)])?[0].clone();
        Self::from_raw(instance, buffer)
    }

    pub fn new_with_data(instance: &Instance, data: &[u8]) -> Result<Self, Error> {
        let buffer = Self::new(instance, data.len())?;
        buffer.write(data)?;
        Ok(buffer)
    }

    pub fn from_raw(instance: &Instance, buffer: Value) -> Result<Self, Error> {
        match Self::from_raw_without_cleanup(instance, buffer.clone()) {
            Ok(buffer) => Ok(buffer),
            Err(error) => {
                let buffer_destroy = instance.inner().exports.get_function("buffer_destroy")?;
                buffer_destroy.call(&mut instance.store().lock(), &[buffer])?;
                Err(error)
            }
        }
    }

    fn from_raw_without_cleanup(instance: &Instance, buffer: Value) -> Result<Self, Error> {
        let exports = &instance.inner().exports;
        let buffer_get_mut_ptr = exports.get_function("buffer_get_mut_ptr")?;
        let buffer_len = exports.get_function("buffer_len")?;

        let Value::I32(buffer_ptr) =
            buffer_get_mut_ptr.call(&mut instance.store().lock(), &[buffer.clone()])?[0]
        else {
            panic!("buffer_get_mut_ptr did not return an i32")
        };
        let ptr = buffer_ptr as u64;
        let Value::I32(buffer_len) =
            buffer_len.call(&mut instance.store().lock(), &[buffer.clone()])?[0]
        else {
            panic!("buffer_len did not return an i32")
        };
        let len = buffer_len as usize;

        Ok(Buffer {
            memory: instance.memory().clone(),
            buffer,
            ptr,
            len,
            store: instance.store().clone(),
        })
    }

    pub fn inner(&self) -> &Value {
        &self.buffer
    }

    pub fn read(&self) -> Result<Vec<u8>, Error> {
        let mut data = vec![0; self.len];
        self.memory
            .view(&self.store.lock())
            .read(self.ptr, &mut data)?;
        Ok(data)
    }

    pub fn write(&self, data: &[u8]) -> Result<(), Error> {
        self.memory
            .view(&self.store.lock())
            .write(self.ptr, data)
            .map_err(Into::into)
    }
}
