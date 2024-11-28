//! Bridge for WASM `AggregateResult` type.

use wasmer::Value;

use crate::{
    buffer::Buffer,
    instance::{Aggregation, Instance},
    Error,
};

pub fn convert_aggregate_result(
    instance: &Instance,
    aggregate_result: &Value,
) -> Result<Result<Aggregation, String>, Error> {
    let result = convert_without_cleanup(instance, aggregate_result);
    let destroy = instance
        .inner()
        .exports
        .get_function("aggregate_result_destroy")?;
    destroy.call(&mut instance.store().lock(), &[aggregate_result.clone()])?;
    result
}

fn convert_without_cleanup(
    instance: &Instance,
    aggregate_result: &Value,
) -> Result<Result<Aggregation, String>, Error> {
    let exports = &instance.inner().exports;
    let get_data = exports.get_function("aggregate_result_get_data")?;
    let is_error = exports.get_function("aggregate_result_is_error")?;
    let get_digest = exports.get_function("aggregate_result_get_digest")?;

    let data = get_data.call(&mut instance.store().lock(), &[aggregate_result.clone()])?;
    let data = Buffer::from_raw(instance, data[0].clone())?.read()?;
    let is_error = is_error.call(&mut instance.store().lock(), &[aggregate_result.clone()])?;
    let error = is_error[0].unwrap_i32() != 0;

    Ok(if error {
        Err(String::from_utf8(data).expect("result data should be valid utf-8"))
    } else {
        let digest = get_digest.call(&mut instance.store().lock(), &[aggregate_result.clone()])?;
        let digest = Buffer::from_raw(instance, digest[0].clone())?.read()?;
        Ok(Aggregation { data, digest })
    })
}
