//! Bridge for WASM `PrepareDataResult` type.

use wasmer::Value;

use crate::{buffer::Buffer, instance::Instance, Error};

pub fn convert_prepare_data_result(
    instance: &Instance,
    prepare_data_result: &Value,
) -> Result<Result<Vec<u8>, String>, Error> {
    let result = convert_without_cleanup(instance, prepare_data_result);
    let destroy = instance
        .inner()
        .exports
        .get_function("prepare_data_result_destroy")?;
    destroy.call(&mut instance.store().lock(), &[prepare_data_result.clone()])?;
    result
}

fn convert_without_cleanup(
    instance: &Instance,
    prepare_data_result: &Value,
) -> Result<Result<Vec<u8>, String>, Error> {
    let exports = &instance.inner().exports;
    let get_data = exports.get_function("prepare_data_result_get_data")?;
    let is_error = exports.get_function("prepare_data_result_is_error")?;

    let data = get_data.call(&mut instance.store().lock(), &[prepare_data_result.clone()])?;
    let data = Buffer::from_raw(instance, data[0].clone())?.read()?;

    let is_error = is_error.call(&mut instance.store().lock(), &[prepare_data_result.clone()])?;
    let is_error = is_error[0].unwrap_i32() != 0;

    Ok(if is_error {
        Err(String::from_utf8(data).expect("result data should be valid utf-8"))
    } else {
        Ok(data)
    })
}
