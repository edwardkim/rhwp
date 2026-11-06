#ifndef RHWP_FLUTTER_OWNER_DISPATCHER_H
#define RHWP_FLUTTER_OWNER_DISPATCHER_H

#include <stddef.h>
#include <stdint.h>

#ifdef _WIN32
#define RHWP_FLUTTER_EXPORT __declspec(dllexport)
#else
#define RHWP_FLUTTER_EXPORT __attribute__((visibility("default")))
#endif

#ifdef __cplusplus
extern "C" {
#endif

typedef struct RhwpFlutterOwner RhwpFlutterOwner;
RHWP_FLUTTER_EXPORT uint32_t rhwp_flutter_owner_abi_version(void);
enum RhwpFlutterOperation {
  RHWP_FLUTTER_STARTUP = 0,
  RHWP_FLUTTER_HOST = 1,
  RHWP_FLUTTER_CREATE_WORKER = 2,
  RHWP_FLUTTER_POLL_WORKER = 3,
  RHWP_FLUTTER_RELEASE_WORKER = 4,
  RHWP_FLUTTER_COPY_RELEASE_BUFFER = 5,
  RHWP_FLUTTER_RELEASE_BUFFER = 6
};

typedef struct RhwpFlutterReply {
  uint64_t request_id;
  int32_t operation;
  int32_t error;
  const char* json;
  const char* release_json;
  const uint8_t* bytes;
  size_t byte_length;
  /* Private queue/accounting fields; clients must not modify them. */
  size_t charged_bytes;
  struct RhwpFlutterReply* next;
  uint64_t snapshot_handle;
} RhwpFlutterReply;

/* One pinned native library path per process. Creation and all operations are
 * asynchronous; the startup reply (id 0) contains native capabilities. */
RHWP_FLUTTER_EXPORT RhwpFlutterOwner* rhwp_flutter_owner_create(const char* library_path);
/* Returns 0 on acceptance, -1 invalid input, -2 closing, -3 count credit,
 * -4 byte credit/allocation failure, -6 startup not ready.
 * id 0 is reserved for startup. */
RHWP_FLUTTER_EXPORT int rhwp_flutter_owner_submit(
    RhwpFlutterOwner*, uint64_t id, int operation,
    const uint8_t* json, size_t json_length,
    const uint8_t* data, size_t data_length,
    uint64_t handle, size_t buffer_length);
RHWP_FLUTTER_EXPORT RhwpFlutterReply* rhwp_flutter_owner_poll(RhwpFlutterOwner*);
/* ABI2: claim an immutable presentation handle only when delivering its JSON
 * to the caller. Unclaimed snapshots are released by reply_free, including
 * decode failure and close races. This operation transfers no legacy TLS state. */
RHWP_FLUTTER_EXPORT uint64_t rhwp_flutter_reply_claim_snapshot(RhwpFlutterOwner*, RhwpFlutterReply*);
/* Frees shim copies and unclaimed cross-thread presentation snapshots. Legacy
 * strings/buffers stay on the owner thread. */
RHWP_FLUTTER_EXPORT void rhwp_flutter_reply_free(RhwpFlutterOwner*, RhwpFlutterReply*);
RHWP_FLUTTER_EXPORT void rhwp_flutter_owner_close(RhwpFlutterOwner*);
RHWP_FLUTTER_EXPORT int rhwp_flutter_owner_finished(RhwpFlutterOwner*);
/* Returns -1 until finished AND all replies consumed, else cleanup status.
 * This joins an already finished thread; it never waits for document work. */
RHWP_FLUTTER_EXPORT int rhwp_flutter_owner_destroy(RhwpFlutterOwner*);
RHWP_FLUTTER_EXPORT size_t rhwp_flutter_owner_retained_bytes(RhwpFlutterOwner*);

#ifdef __cplusplus
}
#endif
#endif
