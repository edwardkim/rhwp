# deleteRange 경계값 패닉 수정 — 처리 결과

Issue: #3187

## 증상

`delete_range_native` (src/document_core/commands/text_editing.rs)의 본문(cell_ctx=None)
경로가 `section_idx`/`start_para`/`end_para` 인덱스 범위를 전혀 검증하지 않고, 같은 문단
내 삭제 시 `end_offset - start_offset`를 무검증 뺄셈했다.

- 범위 밖 `section_idx`/`para_idx` → 인덱싱에서 그대로 패닉.
- 같은 문단에서 `start_offset > end_offset`(뒤집힌 범위) → usize 뺄셈 언더플로로 패닉
  (디버그 빌드) / 랩어라운드 (릴리스 빌드).

이 함수는 WASM `deleteRange`/`deleteRangeInCell`/`deleteRangeInCellEx` 진입점에서 직접
호출되므로, 잘못된 selection 좌표가 브라우저에서 그대로 전달되면 앱이 패닉/트랩된다.
같은 파일의 `insert_text_native`/`delete_text_native`는 이미 이 가드를 갖고 있었는데
`delete_range_native`만 빠져 있었다.

## 재현 (수정 전 red)

```
core.delete_range_native(0, 0, 4, 0, 1, None);  // start=4 > end=1 (같은 문단)
// thread panicked at ...: attempt to subtract with overflow

core.delete_range_native(0, 5, 0, 5, 1, None);   // para_idx 5, 문단 1개뿐
// thread panicked at ...: index out of bounds: the len is 1 but the index is 5
```

## 수정

`delete_range_native` 진입부에 다음 검증을 추가하고 위반 시 `Err(HwpError::RenderError(..))`를
반환하도록 했다:

1. `section_idx`가 `sections.len()` 범위 안인지
2. `start_para <= end_para`인지 (뒤집힌 문단 범위 거부)
3. 같은 문단인 경우 `start_offset <= end_offset`인지 (뒤집힌 오프셋 거부)
4. 본문 경로(cell_ctx=None)에서 `start_para`/`end_para`가 해당 구역 문단 수 범위 안인지

셀 경로(cell_ctx=Some)는 기존에 `get_cell_paragraph_mut`/`get_cell_mut`가 이미 `Result`로
범위를 검증하므로 별도 인덱스 가드를 추가하지 않았다.

## 검증

- 신규 테스트 2건 추가 (`src/document_core/commands/text_editing.rs` tests 모듈):
  - `delete_range_native_rejects_inverted_offsets_same_paragraph`
  - `delete_range_native_rejects_out_of_bounds_indices`
- 수정 전: 위 두 테스트 모두 패닉으로 FAIL 확인.
- 수정 후: 두 테스트 포함 `cargo test --lib` 전체 2554 passed / 1 failed / 7 ignored.
  실패 1건(`renderer::font_paths::tests::env_font_paths_parses_and_filters`)은 본 변경과
  무관한 기존 환경 의존 테스트(Windows에서 `/tmp` 유닉스 경로 파싱 관련)이며 되돌리지 않았다.
- `cargo fmt -- --check src/document_core/commands/text_editing.rs`: 변경분 관련 diff 없음
  (CRLF 관련 무관 경고만 존재, 기존 저장소 전반 이슈).
- `cargo clippy --lib`: 경고 없음.
