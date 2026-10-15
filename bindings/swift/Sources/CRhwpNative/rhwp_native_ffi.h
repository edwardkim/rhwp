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
void rhwp_string_free(char *value);

#ifdef __cplusplus
}
#endif

#endif
