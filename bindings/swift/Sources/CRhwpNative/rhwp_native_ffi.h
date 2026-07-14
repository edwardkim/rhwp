#ifndef RHWP_NATIVE_FFI_H
#define RHWP_NATIVE_FFI_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

char *rhwp_export_text(const char *input_path, const char *output_dir, int page);
char *rhwp_export_markdown(const char *input_path, const char *output_dir, int page);
char *rhwp_read_text(const char *input_path, int page);
void rhwp_string_free(char *value);

/// 바이너리 결과 버퍼 (PDF 등).
///
/// `data` 가 NULL 이면 실패이며 `error` 에 사유가 담긴다.
/// 성공/실패와 무관하게 반드시 `rhwp_buffer_free` 로 해제해야 한다.
typedef struct RhwpBuffer {
  uint8_t *data;
  size_t len;
  char *error;
} RhwpBuffer;

void rhwp_buffer_free(RhwpBuffer buffer);

/// 문서의 페이지 수. 실패 시 -1.
int32_t rhwp_page_count(const char *input_path);

/// 문서를 PDF 로 렌더링한다.
///
/// - first_page:  0-based 시작 페이지
/// - max_pages:   렌더할 최대 페이지 수. 0 이하면 문서 끝까지.
/// - font_dir:    폰트 탐색 절대경로. NULL 가능.
///                코어의 기본 폰트 탐색은 작업디렉터리 상대경로라
///                샌드박스된 확장에서는 잡히지 않는다. 번들 Resources
///                절대경로를 넘겨야 한다.
/// - embed_text:  0 이면 글리프를 path 로 변환한다 (메모리 대폭 절감,
///                텍스트 선택·검색 불가).
///
/// 반환 버퍼는 반드시 `rhwp_buffer_free` 로 해제한다.
RhwpBuffer rhwp_render_pdf(const char *input_path, uint32_t first_page, int32_t max_pages,
                           const char *font_dir, int32_t embed_text);

#ifdef __cplusplus
}
#endif

#endif
