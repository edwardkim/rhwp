# 1단계 완료 보고서 — M100 #2267: C ABI 확장

## 한 일

`bindings/Native` 에 Quick Look 확장용 C ABI 3개를 추가했다.

| 심볼 | 역할 |
|------|------|
| `rhwp_page_count` | 페이지 수. 실패 시 음수. |
| `rhwp_render_pdf` | `first_page` 부터 `max_pages` 장을 PDF 로 렌더. `font_dir`, `embed_text` 를 인자로 받는다. |
| `rhwp_buffer_free` | 반환 버퍼 해제. |

`RhwpBuffer { data, len, error }` 로 성공/실패를 함께 표현한다. FFI 경계를 넘는 panic 은 unwind 로 잡아 `error` 문자열로 바꾼다.

Swift 쪽에도 헤더 선언(`bindings/swift/Sources/CRhwpNative/rhwp_native_ffi.h`)과 래퍼(`Rhwp.pageCount`, `Rhwp.renderPDF`)를 추가했다.

## 부수적으로 고친 것 — 크레이트가 이미 깨져 있었다

`bindings/Native` 는 **컴파일이 안 되는 상태였다.** #1161 에서 `get_control_image_mime_native` / `get_control_image_data_native` 에 `cell_path` 파라미터가 추가됐는데 이 크레이트만 따라가지 않았다.

원인은 명확하다: **`bindings/Native` 는 워크스페이스 밖이라 CI 가 컴파일하지 않는다.** 코어 시그니처가 바뀌어도 아무도 모른다.

호출부에 빈 `cell_path`(`&[]`)를 넘겨 고쳤다.

> **권고**: CI 에 `cargo check --manifest-path bindings/Native/Cargo.toml` 을 추가할 것. 이번처럼 조용히 썩는 걸 막는 유일한 방법이다. (별도 이슈로 등록 필요)

`crate-type` 에 `rlib` 을 추가했다 — 예제(벤치)가 링크할 수 있어야 한다.

## 검증

`cargo test --manifest-path bindings/Native/Cargo.toml` — 5개 통과:

- `page_count` 가 실제 페이지 수를 반환
- `render_pdf` 결과가 유효한 PDF (`%PDF` 매직 + `%%EOF` trailer)
- `first_page` 범위 초과 시 오류
- 존재하지 않는 파일 → 오류 문자열
- 버퍼 해제 후 이중 해제 없음

벤치(`bindings/Native/examples/render_pdf_bench.rs`) — 1페이지 렌더 최대 RSS **41~100MB**, 소요 **0.16~1.32초**. 확장 한도(120MB / 30초) 안이다.
