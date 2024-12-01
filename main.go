package main

import (
	"bytes"
	"encoding/binary"
	"log"
	"os"

	"github.com/IntelliXLabs/iwasm/api"
)

// LD_LIBRARY_PATH=lib go run main.go
func main() {
	runtime := api.NewRuntime()
	defer runtime.Dispose()
	if err := runtime.Err(); err != nil {
		log.Fatalf("failed to create runtime: %v", err)
	}

	wasmFile := "testutils/data/processor.wasm"
	wasm, err := os.ReadFile(wasmFile)
	if err != nil {
		log.Fatalf("failed to read WASM file %s: %v", wasmFile, err)
	}

	instance, err := runtime.CreateInstance(wasm)
	if err != nil {
		log.Fatalf("failed to create instance: %v", err)
	}
	defer instance.Dispose()
	if err := instance.Err(); err != nil {
		log.Fatalf("failed to create instance: %v", err)
	}

	config := []byte("IntelliX")
	dataResult, err := instance.PrepareData(config, []byte("https://api.coingecko.com/api/v3/coins/bitcoin/history?date=24-11-2024"))
	if err != nil {
		log.Fatalf("failed to prepare data: %v", err)
	}

	data, err := dataResult.Data()
	dataResult.Dispose()
	if err != nil {
		log.Fatalf("failed to prepare data: %v", err)
	}

	aggregateResult, err := instance.Aggregate(config, [][]byte{data}, []byte("first"))
	if err != nil {
		log.Fatalf("failed to aggregate: %v", err)
	}

	aggregation, _, err := aggregateResult.Data()
	aggregateResult.Dispose()
	if err != nil {
		log.Fatalf("failed to aggregate: %v", err)
	}

	var result float64
	err = binary.Read(bytes.NewReader(aggregation), binary.LittleEndian, &result)
	if err != nil {
		log.Fatalf("failed to read result: %v", err)
	}
	log.Printf("result: %f", result)
}
