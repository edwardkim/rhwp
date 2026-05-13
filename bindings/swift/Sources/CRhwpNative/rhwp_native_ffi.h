#ifndef RHWP_NATIVE_FFI_H
#define RHWP_NATIVE_FFI_H

#ifdef __cplusplus
extern "C" {
#endif

char *rhwp_export_text(const char *input_path, const char *output_dir, int page);
char *rhwp_export_markdown(const char *input_path, const char *output_dir, int page);
char *rhwp_read_text(const char *input_path, int page);
char *rhwp_write_text(const char *output_path, const char *text);
void rhwp_string_free(char *value);

#ifdef __cplusplus
}
#endif

#endif
