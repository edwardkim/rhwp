#ifndef RHWP_NATIVE_FFI_H
#define RHWP_NATIVE_FFI_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

char *rhwp_export_text(const char *input_path, const char *output_dir, int page);
char *rhwp_export_markdown(const char *input_path, const char *output_dir, int page);
/* Print-profile direct PDF; page=-1 means all, otherwise zero-based.
 * Input <=128 MiB, selected pages <=256. Destination must not exist.
 * All pages are rendered before creating the file. Requires native-skia;
 * valid arguments in disabled builds return {"ok":false,"code":"featureDisabled",...}.
 * Free the returned UTF-8 JSON with rhwp_string_free. */
char *rhwp_export_pdf_direct(const char *input_path, const char *output_file, int page);
char *rhwp_read_text(const char *input_path, int page);
/* Native host ABI v1. All handles belong to the calling OS thread.
 * All returned JSON strings use rhwp_string_free. Inputs are borrowed only
 * during the call; nonempty byte spans must point to readable memory.
 * Empty document bytes create a blank document. Close documents explicitly.
 * Output buffers have independent lifetime and must be released separately. */
char *rhwp_session_capabilities(void);
/* Host JSON operations; data is borrowed for binary-payload operations only. */
char *rhwp_native_host_request(const uint8_t *request, size_t request_len, const uint8_t *data, size_t data_len);
char *rhwp_session_open(const uint8_t *data, size_t len, const uint8_t *password, size_t password_len);
char *rhwp_session_request(uint64_t handle, const uint8_t *request, size_t request_len);
char *rhwp_session_close(uint64_t handle);
/* destination must be writable for capacity bytes. Too-small destinations
 * are rejected before copying. This call does not release the source buffer. */
char *rhwp_session_buffer_copy(uint64_t buffer, uint8_t *destination, size_t capacity);
char *rhwp_session_buffer_release(uint64_t buffer);
/* Presentation ABI v1 is the explicit cross-thread exception: snapshot,
 * CPU frame producer, and consumer lease handles are process-wide. Legacy
 * session/worker/buffer handles above still belong to their owner OS thread.
 * Configure a live worker source through owner-thread host operation
 * presentationConfigure, then submit exportRenderSnapshot and poll it there.
 * rhwp_presentation_request accepts snapshotInfo/snapshotRelease, renderCpu,
 * frameInfo/frameAcquire/frameRelease, leaseInfo/leaseRelease, and stats JSON.
 * renderCpu is synchronous work for a render thread; acquisition/data access
 * never implicitly renders. GPU presentation is not implemented/advertised.
 * Every returned JSON string needs rhwp_string_free, on any OS thread. */
char *rhwp_presentation_request(const uint8_t *request, size_t request_len);
/* Immutable CPU bytes stay valid until this specific consumer lease is released,
 * including after producer frame release or source close/staleness. New stale
 * acquisitions are rejected. The caller coordinates pointer use versus release;
 * never release concurrently, write the bytes, or unload the library while a
 * lease/call/worker is live. byte_length must point to writable size_t storage.
 * Invalid/released lease: NULL and length=0. NULL byte_length: NULL. CPU lease
 * return does not establish GPU sampling completion or allow GPU target reuse. */
const uint8_t *rhwp_presentation_lease_data(uint64_t lease, size_t *byte_length);
void rhwp_string_free(char *value);

#ifdef __cplusplus
}
#endif

#endif
