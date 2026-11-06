#define _POSIX_C_SOURCE 200809L
#include <stdatomic.h>
#include <stdint.h>
#include <stddef.h>
#include <stdlib.h>
#include <string.h>

static _Thread_local int on_owner;
static _Thread_local int worker;
static _Thread_local int closing;
static _Thread_local int buffer_live;
static atomic_int wrong_thread;
static atomic_int worker_releases;
static atomic_int buffer_releases;
static atomic_int allow_finish = 1;
static atomic_int snapshot_poll;
static atomic_int snapshot_releases;
static atomic_int snapshot_live;
typedef struct StringResult { int metadata; char bytes[]; } StringResult;

static int check(void) {
  if (on_owner) return 1;
  atomic_fetch_add(&wrong_thread, 1);
  return 0;
}
static char* result_kind(const char* value, int metadata) {
  size_t length = strlen(value) + 1;
  StringResult* copy = malloc(sizeof(*copy) + length);
  if (!copy) return NULL;
  copy->metadata = metadata;
  memcpy(copy->bytes, value, length);
  return copy->bytes;
}
static char* result(const char* value) { return result_kind(value, 0); }

char* rhwp_session_capabilities(void) {
  on_owner = 1;
  return result("{\"ok\":true,\"worker\":true,\"nativeSkia\":true}");
}
void rhwp_string_free(char* string) {
  if (!string) return;
  StringResult* allocated = (StringResult*)((char*)string - offsetof(StringResult, bytes));
  if (!allocated->metadata) (void)check();
  free(allocated);
}
char* rhwp_native_host_request(const uint8_t* input, size_t length, const uint8_t* data, size_t data_length) {
  (void)data; (void)data_length;
  if (!check()) return result("{\"ok\":false,\"code\":\"wrongThread\"}");
  if (length >= 2048) return result("{\"ok\":false}");
  char json[2048]; memcpy(json, input, length); json[length] = 0;
  if (strstr(json, "workerCreate")) { worker = 1; return result("{\"ok\":true,\"worker\":11}"); }
  if (strstr(json, "workerClose")) { closing = 1; return result("{\"ok\":true,\"isFinished\":false}"); }
  if (strstr(json, "workerStatus")) return result(closing && atomic_load(&allow_finish)
      ? "{\"ok\":true,\"isFinished\":true}" : "{\"ok\":true,\"isFinished\":false}");
  if (strstr(json, "workerRelease") && closing && atomic_load(&allow_finish)) {
    worker = 0; atomic_fetch_add(&worker_releases, 1); return result("{\"ok\":true}");
  }
  if (strstr(json, "workerPoll") && worker) {
    if (atomic_load(&snapshot_poll)) {
      atomic_fetch_add(&snapshot_live, 1);
      return result("{\"ok\":true,\"completion\":{\"ok\":true,\"requestId\":1,\"kind\":\"renderSnapshot\",\"snapshot\":77}}");
    }
    buffer_live = 1;
    return result("{\"ok\":true,\"completion\":{\"ok\":true,\"requestId\":1,\"kind\":\"frame\",\"buffer\":42,\"byteLength\":64,\"width\":4,\"height\":4,\"stride\":16,\"originX\":0,\"originY\":0,\"revision\":0,\"fontGeneration\":0}}");
  }
  return result("{\"ok\":true,\"requestId\":1}");
}
char* rhwp_session_buffer_copy(uint64_t buffer, uint8_t* output, size_t length) {
  if (!check() || buffer != 42 || !buffer_live || length != 64) return result("{\"ok\":false,\"code\":\"copyFailed\"}");
  for (size_t i = 0; i < length; ++i) output[i] = (uint8_t)i;
  return result("{\"ok\":true}");
}
char* rhwp_session_buffer_release(uint64_t buffer) {
  if (!check() || buffer != 42 || !buffer_live) return result("{\"ok\":false,\"code\":\"invalidBuffer\"}");
  buffer_live = 0; atomic_fetch_add(&buffer_releases, 1); return result("{\"ok\":true}");
}
int mock_wrong_thread_calls(void) { return atomic_load(&wrong_thread); }
int mock_worker_releases(void) { return atomic_load(&worker_releases); }
int mock_buffer_releases(void) { return atomic_load(&buffer_releases); }
void mock_allow_finish(int allow) { atomic_store(&allow_finish, allow); }

char* rhwp_presentation_request(const uint8_t* input, size_t length) {
  if (length >= 256) return result_kind("{\"ok\":false}", 1);
  char json[256]; memcpy(json, input, length); json[length] = 0;
  if (strstr(json, "snapshotRelease") && strstr(json, "77") && atomic_load(&snapshot_live) > 0) {
    atomic_fetch_sub(&snapshot_live, 1); atomic_fetch_add(&snapshot_releases, 1);
    return result_kind("{\"ok\":true}", 1);
  }
  return result_kind("{\"ok\":false}", 1);
}
void mock_snapshot_poll(int enabled) { atomic_store(&snapshot_poll, enabled); }
int mock_snapshot_releases(void) { return atomic_load(&snapshot_releases); }
int mock_snapshot_live(void) { return atomic_load(&snapshot_live); }
