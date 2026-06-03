# Task #311 1단계 완료 보고서: PaginationOpts 구조체 도입

상위: 구현 계획서 `task_m100_311_impl.md`, Epic #309

## 변경 요약

`paginate_with_measured_opts`의 bool 인자(`hide_empty_line`)를 `PaginationOpts` 구조체로 마이그레이션. 기능 변경 없음.

## 변경 파일

- `src/renderer/pagination.rs` — `PaginationOpts` 구조체 신설 (`hide_empty_line`, `respect_vpos_reset`)
- `src/renderer/pagination/engine.rs` — `paginate_with_measured_opts` 시그니처 `(..., opts: PaginationOpts)` 로 변경, 내부에서 분해
- `src/document_core/queries/rendering.rs:824` — 호출 측 마이그레이션

`respect_vpos_reset` 필드는 1단계에서는 미사용 (2단계에서 활용). `_respect_vpos_reset` 으로 명명하여 unused 경고 방지.

## 검증

- `cargo build` 성공
- `cargo test`: **992 passed; 0 failed**
- 4개 샘플 페이지 수:
  - 21_언어: 19쪽 (변화 없음)
  - exam_math: 20쪽 (변화 없음)
  - exam_kor: 25쪽 (변화 없음)
  - exam_eng: 11쪽 (변화 없음)

## 다음 단계

2단계: `paginate_text_lines`에 vpos-reset 검출 + 강제 분리 로직 + `--respect-vpos-reset` CLI 플래그
