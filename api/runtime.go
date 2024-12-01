package api

/*
#cgo LDFLAGS: -L${SRCDIR}/../lib -lruntime
#cgo CFLAGS: -I${SRCDIR}/../lib
#include "runtime.h"
*/
import "C"
import (
	"errors"
	"unsafe"
)

// Runtime represents a WASM runtime or an error during runtime creation.
type RuntimeResult struct {
	result *C.struct_RuntimeResult
}

// NewRuntime creates a new WASM runtime.
func NewRuntime() RuntimeResult {
	return RuntimeResult{C.runtime_create()}
}

// Dispose releases the resources associated with the RuntimeResult.
func (runtime RuntimeResult) Dispose() {
	C.runtime_result_destroy(runtime.result)
}

// Err returns an error if the runtime creation failed.
func (runtime RuntimeResult) Err() error {
	if runtime.result.error != nil {
		return errors.New(bufferToString(*runtime.result.error))
	}
	return nil
}

// InstanceResult represents a WASM instance or an error during instance creation.
type InstanceResult struct {
	result *C.struct_InstanceResult
}

// CreateInstance creates a new WASM instance from raw WASM bytes.
func (runtime RuntimeResult) CreateInstance(wasm []byte) (InstanceResult, error) {
	if err := runtime.Err(); err != nil {
		return InstanceResult{}, err
	}
	return InstanceResult{C.runtime_create_instance(runtime.result.runtime, bytesToSlice(wasm))}, nil
}

// Dispose releases the resources associated with the InstanceResult.
func (instance InstanceResult) Dispose() {
	C.instance_result_destroy(instance.result)
}

// Err returns an error if the instance creation failed.
func (instance InstanceResult) Err() error {
	if instance.result.error != nil {
		return errors.New(bufferToString(*instance.result.error))
	}
	return nil
}

// PrepareDataResult is the result of an instance's prepare_data function.
type PrepareDataResult struct {
	result *C.struct_PrepareDataResult
}

// PrepareData runs the WASM instance's prepare_data function.
func (instance InstanceResult) PrepareData(config, request []byte) (PrepareDataResult, error) {
	if err := instance.Err(); err != nil {
		return PrepareDataResult{}, err
	}
	return PrepareDataResult{C.instance_prepare_data(instance.result.instance, bytesToSlice(config), bytesToSlice(request))}, nil
}

// Dispose releases the resources associated with the PrepareDataResult.
func (result PrepareDataResult) Dispose() {
	C.prepare_data_result_destroy(result.result)
}

// Data returns the data produced by the prepare_data function.
func (result PrepareDataResult) Data() ([]byte, error) {
	if result.result.is_error {
		return nil, errors.New(bufferToString(result.result.data))
	}
	return bufferToBytes(result.result.data), nil
}

// AggregateResult is the result of an instance's aggregate function.
type AggregateResult struct {
	result *C.struct_AggregateResult
}

// Aggregate runs the WASM instance's aggregate function.
func (instance InstanceResult) Aggregate(config []byte, data [][]byte, request []byte) (AggregateResult, error) {
	if err := instance.Err(); err != nil {
		return AggregateResult{}, err
	}

	cData := make([]*C.struct_Buffer, len(data))
	for index, datum := range data {
		cData[index] = C.slice_to_buffer(bytesToSlice(datum))
	}

	cDataSlice := make([]C.struct_Slice, len(data))
	for index, datum := range cData {
		cDataSlice[index] = C.buffer_slice(datum)
	}

	result := C.instance_aggregate(instance.result.instance, bytesToSlice(config), sliceArray(cDataSlice), bytesToSlice(request))

	for _, datum := range cData {
		C.buffer_destroy(datum)
	}

	return AggregateResult{result}, nil
}

// Dispose releases the resources associated with the AggregateResult.
func (result AggregateResult) Dispose() {
	C.aggregate_result_destroy(result.result)
}

// Data returns the data produced by the aggregate function.
func (result AggregateResult) Data() ([]byte, []byte, error) {
	if result.result.is_error {
		return nil, nil, errors.New(bufferToString(result.result.data))
	}
	return bufferToBytes(result.result.data), bufferToBytes(*result.result.digest), nil
}

func bufferToBytes(buffer C.struct_Buffer) []byte {
	return C.GoBytes(unsafe.Pointer(buffer.ptr), C.int(buffer.len))
}

func bufferToString(buffer C.struct_Buffer) string {
	return string(bufferToBytes(buffer))
}

func bytesToSlice(bytes []byte) C.struct_Slice {
	return C.struct_Slice{
		ptr: (*C.uint8_t)(unsafe.Pointer(&bytes[0])),
		len: C.uintptr_t(len(bytes)),
	}
}

func sliceArray(slices []C.struct_Slice) C.struct_SliceArray {
	return C.struct_SliceArray{
		ptr: &slices[0],
		len: C.uintptr_t(len(slices)),
	}
}
