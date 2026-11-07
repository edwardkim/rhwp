#define _POSIX_C_SOURCE 200809L
#include "owner_dispatcher.h"

#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#ifdef _WIN32
#include <windows.h>
typedef CRITICAL_SECTION Mutex;
typedef CONDITION_VARIABLE Condition;
typedef HANDLE Thread;
#define THREAD_RESULT DWORD WINAPI
#define THREAD_DONE 0
static void mutex_init(Mutex* m) { InitializeCriticalSection(m); }
static void mutex_destroy(Mutex* m) { DeleteCriticalSection(m); }
static void mutex_lock(Mutex* m) { EnterCriticalSection(m); }
static void mutex_unlock(Mutex* m) { LeaveCriticalSection(m); }
static void condition_init(Condition* c) { InitializeConditionVariable(c); }
static void condition_destroy(Condition* c) { (void)c; }
static void condition_wait(Condition* c, Mutex* m) { SleepConditionVariableCS(c, m, INFINITE); }
static void condition_signal(Condition* c) { WakeConditionVariable(c); }
static void pause_owner(void) { Sleep(2); }
static SRWLOCK loader_mutex = SRWLOCK_INIT;
static void loader_lock(void) { AcquireSRWLockExclusive(&loader_mutex); }
static void loader_unlock(void) { ReleaseSRWLockExclusive(&loader_mutex); }
static void* library_open(const char* path) {
  int count = MultiByteToWideChar(CP_UTF8, MB_ERR_INVALID_CHARS, path, -1, NULL, 0);
  if (count <= 0) return NULL;
  wchar_t* wide = malloc((size_t)count * sizeof(*wide));
  if (!wide) return NULL;
  (void)MultiByteToWideChar(CP_UTF8, MB_ERR_INVALID_CHARS, path, -1, wide, count);
  void* library = (void*)LoadLibraryW(wide);
  free(wide);
  return library;
}
static void library_close(void* library) { (void)FreeLibrary((HMODULE)library); }
static void* symbol(void* library, const char* name) { return (void*)GetProcAddress((HMODULE)library, name); }
#else
#include <dlfcn.h>
#include <pthread.h>
#include <time.h>
typedef pthread_mutex_t Mutex;
typedef pthread_cond_t Condition;
typedef pthread_t Thread;
#define THREAD_RESULT void*
#define THREAD_DONE NULL
static void mutex_init(Mutex* m) { (void)pthread_mutex_init(m, NULL); }
static void mutex_destroy(Mutex* m) { (void)pthread_mutex_destroy(m); }
static void mutex_lock(Mutex* m) { (void)pthread_mutex_lock(m); }
static void mutex_unlock(Mutex* m) { (void)pthread_mutex_unlock(m); }
static void condition_init(Condition* c) { (void)pthread_cond_init(c, NULL); }
static void condition_destroy(Condition* c) { (void)pthread_cond_destroy(c); }
static void condition_wait(Condition* c, Mutex* m) { (void)pthread_cond_wait(c, m); }
static void condition_signal(Condition* c) { (void)pthread_cond_signal(c); }
static void pause_owner(void) { struct timespec t = {0, 2000000}; (void)nanosleep(&t, NULL); }
static pthread_mutex_t loader_mutex = PTHREAD_MUTEX_INITIALIZER;
static void loader_lock(void) { (void)pthread_mutex_lock(&loader_mutex); }
static void loader_unlock(void) { (void)pthread_mutex_unlock(&loader_mutex); }
static void* library_open(const char* path) { return dlopen(path, RTLD_NOW | RTLD_LOCAL); }
static void library_close(void* library) { (void)dlclose(library); }
static void* symbol(void* library, const char* name) { return dlsym(library, name); }
#endif

#define MAX_RPC 64u
#define MAX_OWNED_BYTES (64u * 1024u * 1024u)
#define MAX_PAYLOAD_BYTES (16u * 1024u * 1024u)
#define MAX_JSON_BYTES (1024u * 1024u)
#define MAX_WORKERS 4u
#define MAX_BUFFERS 64u

typedef char* (*HostRequest)(const uint8_t*, size_t, const uint8_t*, size_t);
typedef char* (*Capabilities)(void);
typedef char* (*PresentationRequest)(const uint8_t*, size_t);
typedef char* (*BufferCopy)(uint64_t, uint8_t*, size_t);
typedef char* (*BufferRelease)(uint64_t);
typedef void (*StringFree)(char*);
typedef struct NativeApi {
  HostRequest host;
  Capabilities capabilities;
  BufferCopy copy;
  BufferRelease release;
  StringFree string_free;
  PresentationRequest presentation;
} NativeApi;
static NativeApi pinned_api;
static char* pinned_path;
/* The single native module is pinned for process lifetime. In particular its
 * Rust TLS destructors must remain mapped through owner-thread teardown. */
static void* pinned_library;

uint32_t rhwp_flutter_owner_abi_version(void) { return 2; }

typedef struct Command {
  RhwpFlutterReply* reply;
  uint8_t* json;
  size_t json_length;
  uint8_t* data;
  size_t data_length;
  uint64_t handle;
  size_t buffer_length;
  struct Command* next;
} Command;

struct RhwpFlutterOwner {
  Mutex mutex;
  Condition ready;
  Thread thread;
  char* library_path;
  NativeApi api;
  Command* commands;
  Command* commands_tail;
  RhwpFlutterReply* replies;
  RhwpFlutterReply* replies_tail;
  size_t outstanding;
  size_t owned_bytes;
  int closing;
  int available;
  int finished;
  int cleanup_error;
  uint64_t workers[MAX_WORKERS];
  size_t worker_count;
  uint64_t buffers[MAX_BUFFERS];
  size_t buffer_count;
};

static void mark_cleanup_error(RhwpFlutterOwner* owner) {
  mutex_lock(&owner->mutex); owner->cleanup_error = 1; mutex_unlock(&owner->mutex);
}

static char* duplicate(const char* value) {
  size_t length = strlen(value);
  char* result = malloc(length + 1);
  if (result) memcpy(result, value, length + 1);
  return result;
}

/* These readers inspect small, trusted native ABI reply objects. They are not
 * an input JSON parser and never interpret HWP/HWPX bytes. */
static const char* field(const char* json, const char* key) {
  char name[64];
  if (!json || strlen(key) > 50) return NULL;
  (void)snprintf(name, sizeof(name), "\"%s\":", key);
  const char* value = strstr(json, name);
  if (!value) return NULL;
  value += strlen(name);
  while (*value == ' ' || *value == '\t' || *value == '\n') ++value;
  return value;
}
static int native_true(const char* json, const char* key) {
  const char* value = field(json, key);
  return value && strncmp(value, "true", 4) == 0;
}
static uint64_t native_handle(const char* json, const char* key) {
  const char* value = field(json, key);
  if (!value || *value < '0' || *value > '9') return 0;
  errno = 0;
  char* end = NULL;
  unsigned long long number = strtoull(value, &end, 10);
  return errno || end == value || (*end != ',' && *end != '}') ? 0 : (uint64_t)number;
}
static void remove_handle(uint64_t* list, size_t* count, uint64_t handle) {
  for (size_t i = 0; i < *count; ++i) {
    if (list[i] == handle) { list[i] = list[--*count]; return; }
  }
}
static int load_api(RhwpFlutterOwner* owner) {
  int ok = 0;
  loader_lock();
  if (pinned_path) {
    ok = strcmp(pinned_path, owner->library_path) == 0;
  } else {
    void* library = library_open(owner->library_path);
    NativeApi api = {0};
    if (library) {
      /* The supported desktop loader ABIs use equally sized code/data pointers.
       * memcpy avoids type-punning through incompatible pointer aliases. */
#define LOAD_API(member, name) do { \
        void* address = symbol(library, name); \
        _Static_assert(sizeof(api.member) == sizeof(address), "unsupported loader ABI"); \
        memcpy(&api.member, &address, sizeof(address)); \
      } while (0)
      LOAD_API(host, "rhwp_native_host_request");
      LOAD_API(capabilities, "rhwp_session_capabilities");
      LOAD_API(copy, "rhwp_session_buffer_copy");
      LOAD_API(release, "rhwp_session_buffer_release");
      LOAD_API(string_free, "rhwp_string_free");
      LOAD_API(presentation, "rhwp_presentation_request");
#undef LOAD_API
      if (api.host && api.capabilities && api.copy && api.release && api.string_free) {
        pinned_path = duplicate(owner->library_path);
        if (pinned_path) { pinned_api = api; pinned_library = library; ok = 1; }
      }
      if (!ok) library_close(library);
    }
  }
  if (ok) owner->api = pinned_api;
  loader_unlock();
  return ok;
}
static int charge(RhwpFlutterOwner* owner, size_t bytes) {
  int ok;
  mutex_lock(&owner->mutex);
  ok = bytes <= MAX_OWNED_BYTES - owner->owned_bytes;
  if (ok) owner->owned_bytes += bytes;
  mutex_unlock(&owner->mutex);
  return ok;
}
static void uncharge(RhwpFlutterOwner* owner, size_t bytes) {
  mutex_lock(&owner->mutex);
  owner->owned_bytes -= bytes;
  mutex_unlock(&owner->mutex);
}
static char* copy_json(RhwpFlutterOwner* owner, RhwpFlutterReply* reply, char* native) {
  if (!native) { reply->error = -5; return NULL; }
  size_t length = strlen(native) + 1;
  char* copy = NULL;
  if (length <= MAX_JSON_BYTES && charge(owner, length)) {
    copy = malloc(length);
    if (copy) { memcpy(copy, native, length); reply->charged_bytes += length; }
    else uncharge(owner, length);
  }
  owner->api.string_free(native);
  if (!copy) reply->error = -4;
  return copy;
}
static char* worker_call(RhwpFlutterOwner* owner, const char* op, uint64_t worker) {
  char json[128];
  int length = snprintf(json, sizeof(json), "{\"op\":\"%s\",\"worker\":%llu}", op, (unsigned long long)worker);
  return owner->api.host((const uint8_t*)json, (size_t)length, NULL, 0);
}
static void queue_reply(RhwpFlutterOwner* owner, RhwpFlutterReply* reply) {
  mutex_lock(&owner->mutex);
  if (owner->replies_tail) owner->replies_tail->next = reply;
  else owner->replies = reply;
  owner->replies_tail = reply;
  mutex_unlock(&owner->mutex);
}
static void release_native_buffer(RhwpFlutterOwner* owner, uint64_t buffer) {
  char* result = owner->api.release(buffer);
  if (!native_true(result, "ok")) mark_cleanup_error(owner);
  owner->api.string_free(result);
  remove_handle(owner->buffers, &owner->buffer_count, buffer);
}
static void release_native_snapshot(RhwpFlutterOwner* owner, uint64_t snapshot) {
  if (!owner->api.presentation) { mark_cleanup_error(owner); return; }
  char json[128];
  int length = snprintf(json, sizeof(json), "{\"op\":\"snapshotRelease\",\"snapshot\":%llu}", (unsigned long long)snapshot);
  char* reply = owner->api.presentation((const uint8_t*)json, (size_t)length);
  if (!native_true(reply, "ok")) mark_cleanup_error(owner);
  owner->api.string_free(reply);
}
static void execute(RhwpFlutterOwner* owner, Command* command) {
  RhwpFlutterReply* reply = command->reply;
  char* native = NULL;
  if (reply->operation == RHWP_FLUTTER_COPY_RELEASE_BUFFER) {
    uint8_t* bytes = malloc(command->buffer_length);
    if (bytes) {
      native = owner->api.copy(command->handle, bytes, command->buffer_length);
      if (native_true(native, "ok")) {
        reply->bytes = bytes;
        reply->byte_length = command->buffer_length;
      } else free(bytes);
      reply->json = copy_json(owner, reply, native);
    } else reply->error = -4;
    char* released = owner->api.release(command->handle);
    if (native_true(released, "ok")) remove_handle(owner->buffers, &owner->buffer_count, command->handle);
    reply->release_json = copy_json(owner, reply, released);
  } else if (reply->operation == RHWP_FLUTTER_RELEASE_BUFFER) {
    native = owner->api.release(command->handle);
    if (native_true(native, "ok")) remove_handle(owner->buffers, &owner->buffer_count, command->handle);
    reply->json = copy_json(owner, reply, native);
  } else if (reply->operation == RHWP_FLUTTER_CREATE_WORKER && owner->worker_count >= MAX_WORKERS) {
    /* Reject before Native allocates another worker that cannot be tracked. */
    reply->error = -3;
  } else {
    if (reply->operation == RHWP_FLUTTER_POLL_WORKER)
      native = worker_call(owner, "workerPoll", command->handle);
    else if (reply->operation == RHWP_FLUTTER_RELEASE_WORKER)
      native = worker_call(owner, "workerRelease", command->handle);
    else
      native = owner->api.host(command->json, command->json_length, command->data, command->data_length);
    if (reply->operation == RHWP_FLUTTER_CREATE_WORKER && native_true(native, "ok")) {
      uint64_t worker = native_handle(native, "worker");
      if (worker && owner->worker_count < MAX_WORKERS) owner->workers[owner->worker_count++] = worker;
      else reply->error = -5;
    }
    if (reply->operation == RHWP_FLUTTER_RELEASE_WORKER && native_true(native, "ok"))
      remove_handle(owner->workers, &owner->worker_count, command->handle);
    uint64_t buffer = reply->operation == RHWP_FLUTTER_POLL_WORKER ? native_handle(native, "buffer") : 0;
    if (buffer) {
      if (owner->buffer_count < MAX_BUFFERS) owner->buffers[owner->buffer_count++] = buffer;
      else { release_native_buffer(owner, buffer); buffer = 0; reply->error = -3; }
    }
    const char* kind = field(native, "kind");
    if (reply->operation == RHWP_FLUTTER_POLL_WORKER && kind && strncmp(kind, "\"renderSnapshot\"", 16) == 0) {
      reply->snapshot_handle = native_handle(native, "snapshot");
    }
    reply->json = copy_json(owner, reply, native);
    if (reply->error && reply->snapshot_handle) {
      release_native_snapshot(owner, reply->snapshot_handle); reply->snapshot_handle = 0;
    }
    if (reply->error && buffer) release_native_buffer(owner, buffer);
  }
  uncharge(owner, command->json_length + command->data_length);
  free(command->json);
  free(command->data);
  free(command);
  queue_reply(owner, reply);
}
static void cleanup_native(RhwpFlutterOwner* owner) {
  for (size_t i = 0; i < owner->worker_count; ++i) {
    char* result = worker_call(owner, "workerClose", owner->workers[i]);
    if (!native_true(result, "ok")) mark_cleanup_error(owner);
    owner->api.string_free(result);
  }
  while (owner->buffer_count) release_native_buffer(owner, owner->buffers[owner->buffer_count - 1]);
  while (owner->worker_count) {
    size_t index = 0;
    while (index < owner->worker_count) {
      uint64_t worker = owner->workers[index];
      char* status = worker_call(owner, "workerStatus", worker);
      int ok = native_true(status, "ok"), finished = native_true(status, "isFinished");
      owner->api.string_free(status);
      if (!ok || finished) {
        if (ok) {
          char* released = worker_call(owner, "workerRelease", worker);
          if (!native_true(released, "ok")) mark_cleanup_error(owner);
          owner->api.string_free(released);
        } else mark_cleanup_error(owner);
        remove_handle(owner->workers, &owner->worker_count, worker);
      } else ++index;
    }
    if (owner->worker_count) pause_owner();
  }
}
static THREAD_RESULT owner_main(void* argument) {
  RhwpFlutterOwner* owner = argument;
  RhwpFlutterReply* startup = calloc(1, sizeof(*startup));
  int loaded = load_api(owner);
  mutex_lock(&owner->mutex);
  owner->available = loaded && startup;
  mutex_unlock(&owner->mutex);
  if (startup) {
    if (loaded) startup->json = copy_json(owner, startup, owner->api.capabilities());
    else startup->error = -5;
    queue_reply(owner, startup);
  }
  if (loaded && startup) {
    for (;;) {
      mutex_lock(&owner->mutex);
      while (!owner->commands && !owner->closing) condition_wait(&owner->ready, &owner->mutex);
      Command* command = owner->commands;
      if (command) {
        owner->commands = command->next;
        if (!owner->commands) owner->commands_tail = NULL;
      }
      int closing = owner->closing;
      mutex_unlock(&owner->mutex);
      if (command) execute(owner, command);
      else if (closing) break;
    }
    cleanup_native(owner);
  }
  mutex_lock(&owner->mutex);
  owner->finished = 1;
  if (!startup) owner->outstanding = 0;
  mutex_unlock(&owner->mutex);
  return THREAD_DONE;
}

RhwpFlutterOwner* rhwp_flutter_owner_create(const char* path) {
  if (!path || !*path || strlen(path) > 4096) return NULL;
  RhwpFlutterOwner* owner = calloc(1, sizeof(*owner));
  if (!owner) return NULL;
  owner->library_path = duplicate(path);
  if (!owner->library_path) { free(owner); return NULL; }
  mutex_init(&owner->mutex);
  condition_init(&owner->ready);
  owner->outstanding = 1;
#ifdef _WIN32
  owner->thread = CreateThread(NULL, 0, owner_main, owner, 0, NULL);
  int failed = owner->thread == NULL;
#else
  int failed = pthread_create(&owner->thread, NULL, owner_main, owner) != 0;
#endif
  if (failed) {
    condition_destroy(&owner->ready); mutex_destroy(&owner->mutex);
    free(owner->library_path); free(owner); return NULL;
  }
  return owner;
}
int rhwp_flutter_owner_submit(RhwpFlutterOwner* owner, uint64_t id, int operation,
    const uint8_t* json, size_t json_length, const uint8_t* data, size_t data_length,
    uint64_t handle, size_t buffer_length) {
  if (!owner || !id || operation < RHWP_FLUTTER_HOST || operation > RHWP_FLUTTER_RELEASE_BUFFER
      || json_length > MAX_JSON_BYTES || data_length > MAX_PAYLOAD_BYTES
      || buffer_length > MAX_PAYLOAD_BYTES || (json_length && !json) || (data_length && !data)
      || (operation == RHWP_FLUTTER_COPY_RELEASE_BUFFER && (!buffer_length || !handle))) return -1;
  size_t bytes = json_length + data_length + (operation == RHWP_FLUTTER_COPY_RELEASE_BUFFER ? buffer_length : 0);
  mutex_lock(&owner->mutex);
  int error = owner->closing || owner->finished ? -2 : !owner->available ? -6 : owner->outstanding >= MAX_RPC ? -3
              : bytes > MAX_OWNED_BYTES - owner->owned_bytes ? -4 : 0;
  if (!error) { ++owner->outstanding; owner->owned_bytes += bytes; }
  mutex_unlock(&owner->mutex);
  if (error) return error;
  Command* command = calloc(1, sizeof(*command));
  RhwpFlutterReply* reply = calloc(1, sizeof(*reply));
  if (command) {
    command->json = json_length ? malloc(json_length) : NULL;
    command->data = data_length ? malloc(data_length) : NULL;
  }
  if (!command || !reply || (json_length && !command->json) || (data_length && !command->data)) {
    if (command) { free(command->json); free(command->data); free(command); }
    free(reply); uncharge(owner, bytes);
    mutex_lock(&owner->mutex); --owner->outstanding; mutex_unlock(&owner->mutex); return -4;
  }
  if (json_length) memcpy(command->json, json, json_length);
  if (data_length) memcpy(command->data, data, data_length);
  command->json_length = json_length; command->data_length = data_length;
  command->handle = handle; command->buffer_length = buffer_length; command->reply = reply;
  reply->request_id = id; reply->operation = operation;
  reply->charged_bytes = operation == RHWP_FLUTTER_COPY_RELEASE_BUFFER ? buffer_length : 0;
  mutex_lock(&owner->mutex);
  if (owner->closing || owner->finished) {
    owner->owned_bytes -= bytes; --owner->outstanding;
    mutex_unlock(&owner->mutex);
    free(command->json); free(command->data); free(command); free(reply); return -2;
  }
  if (owner->commands_tail) owner->commands_tail->next = command;
  else owner->commands = command;
  owner->commands_tail = command;
  condition_signal(&owner->ready);
  mutex_unlock(&owner->mutex);
  return 0;
}
RhwpFlutterReply* rhwp_flutter_owner_poll(RhwpFlutterOwner* owner) {
  mutex_lock(&owner->mutex);
  RhwpFlutterReply* reply = owner->replies;
  if (reply) { owner->replies = reply->next; if (!owner->replies) owner->replies_tail = NULL; reply->next = NULL; }
  mutex_unlock(&owner->mutex);
  return reply;
}
uint64_t rhwp_flutter_reply_claim_snapshot(RhwpFlutterOwner* owner, RhwpFlutterReply* reply) {
  (void)owner;
  if (!reply) return 0;
  uint64_t snapshot = reply->snapshot_handle;
  reply->snapshot_handle = 0;
  return snapshot;
}
void rhwp_flutter_reply_free(RhwpFlutterOwner* owner, RhwpFlutterReply* reply) {
  if (!reply) return;
  if (reply->snapshot_handle) release_native_snapshot(owner, reply->snapshot_handle);
  free((void*)reply->json); free((void*)reply->release_json); free((void*)reply->bytes);
  mutex_lock(&owner->mutex);
  owner->owned_bytes -= reply->charged_bytes; --owner->outstanding;
  mutex_unlock(&owner->mutex);
  free(reply);
}
void rhwp_flutter_owner_close(RhwpFlutterOwner* owner) {
  mutex_lock(&owner->mutex); owner->closing = 1; condition_signal(&owner->ready); mutex_unlock(&owner->mutex);
}
int rhwp_flutter_owner_finished(RhwpFlutterOwner* owner) {
  mutex_lock(&owner->mutex); int finished = owner->finished; mutex_unlock(&owner->mutex); return finished;
}
size_t rhwp_flutter_owner_retained_bytes(RhwpFlutterOwner* owner) {
  mutex_lock(&owner->mutex); size_t bytes = owner->owned_bytes; mutex_unlock(&owner->mutex); return bytes;
}
int rhwp_flutter_owner_destroy(RhwpFlutterOwner* owner) {
  mutex_lock(&owner->mutex);
  int can_destroy = owner->finished && !owner->outstanding && !owner->commands && !owner->replies;
  mutex_unlock(&owner->mutex);
  if (!can_destroy) return -1;
#ifdef _WIN32
  (void)WaitForSingleObject(owner->thread, INFINITE); (void)CloseHandle(owner->thread);
#else
  (void)pthread_join(owner->thread, NULL);
#endif
  int result = owner->cleanup_error;
  condition_destroy(&owner->ready); mutex_destroy(&owner->mutex);
  free(owner->library_path); free(owner);
  return result;
}
