package api

import (
	"bytes"
	"encoding/binary"
	"os"
	"testing"
)

// shell: LD_LIBRARY_PATH=. go test .
func TestRuntime(t *testing.T) {
	runtime := NewRuntime()
	defer runtime.Dispose()
	if err := runtime.Err(); err != nil {
		t.Fatalf("failed to create runtime: %v", err)
	}

	wasmFile := "../wasm/processor.wasm"
	wasm, err := os.ReadFile(wasmFile)
	if err != nil {
		t.Fatalf("failed to read WASM file %s: %v", wasmFile, err)
	}

	instance, err := runtime.CreateInstance(wasm)
	if err != nil {
		t.Fatalf("failed to create instance: %v", err)
	}
	defer instance.Dispose()
	if err := instance.Err(); err != nil {
		t.Fatalf("failed to create instance: %v", err)
	}

	config := []byte("IntelliX")
	dataResult, err := instance.PrepareData(config, []byte("https://api.coingecko.com/api/v3/coins/bitcoin/history?date=24-11-2024"))
	if err != nil {
		t.Fatalf("failed to prepare data: %v", err)
	}

	data, err := dataResult.Data()
	dataResult.Dispose()
	if err != nil {
		t.Fatalf("failed to prepare data: %v", err)
	}

	aggregateResult, err := instance.Aggregate(config, [][]byte{data}, []byte("first"))
	if err != nil {
		t.Fatalf("failed to aggregate: %v", err)
	}

	aggregation, _, err := aggregateResult.Data()
	aggregateResult.Dispose()
	if err != nil {
		t.Fatalf("failed to aggregate: %v", err)
	}

	var result float64
	err = binary.Read(bytes.NewReader(aggregation), binary.LittleEndian, &result)
	if err != nil {
		t.Fatalf("failed to read result: %v", err)
	}
	t.Logf("result: %f", result)
}
