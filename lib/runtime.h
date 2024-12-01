#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct Instance Instance;

typedef struct Runtime Runtime;

typedef struct Buffer {
  uint8_t *ptr;
  uintptr_t len;
} Buffer;

typedef struct RuntimeResult {
  struct Runtime *runtime;
  struct Buffer *error;
} RuntimeResult;

typedef struct Slice {
  uint8_t *ptr;
  uintptr_t len;
} Slice;

typedef struct InstanceResult {
  struct Instance *instance;
  struct Buffer *error;
} InstanceResult;

typedef struct PrepareDataResult {
  struct Buffer data;
  bool is_error;
} PrepareDataResult;

typedef struct AggregateResult {
  struct Buffer data;
  bool is_error;
  struct Buffer *digest;
} AggregateResult;

typedef struct SliceArray {
  struct Slice *ptr;
  uintptr_t len;
} SliceArray;

void runtime_result_destroy(struct RuntimeResult *result);

struct RuntimeResult *runtime_create(void);

struct Buffer *slice_to_buffer(struct Slice slice);

void buffer_destroy(struct Buffer *buffer);

struct Slice buffer_slice(const struct Buffer *buffer);

void instance_result_destroy(struct InstanceResult *result);

struct InstanceResult *runtime_create_instance(const struct Runtime *runtime,
                                               struct Slice wasm_bytes);

struct PrepareDataResult *instance_prepare_data(const struct Instance *instance,
                                                struct Slice config,
                                                struct Slice request);

void prepare_data_result_destroy(struct PrepareDataResult *result);

void aggregate_result_destroy(struct AggregateResult *result);

struct AggregateResult *instance_aggregate(const struct Instance *instance,
                                           struct Slice config,
                                           struct SliceArray data,
                                           struct Slice request);
