# PR #2073 리뷰 — #1842 CellBreak synthetic 셀 라인높이 em 교정

- 작성 시각: 2026-07-09 16:30 KST
- PR: https://github.com/edwardkim/rhwp/pull/2073
- 작성자: `planet6897`
- base / head: `devel` / `fix/1842-cellbreak-synthetic-em`
- 문서 작성 시점 참고 head: `a3341b6fc1d114173c91dc7b134eda1338e241c2`
- 문서 작성 시점 참고 merge state: `BEHIND`
- reviewer assign: `jangster77` 요청 완료
- 처리 경로: `codex/planet6897-prs-review-20260709` 에서 여러 PR 누적 체리픽 검토

## 변경 범위

- `src/renderer/height_measurer.rs`: CellBreak 표의 저장 `LINE_SEG` 부재 synthetic 셀 라인높이가 160% 줄간격으로 과팽창하지 않도록 em 기반으로 조정.
- `src/renderer/layout/table_layout.rs`: CellBreak synthetic 라인높이 보정과 연동.
- `tests/issue_1842.rs`: 기존 TAC-only lineheight 검증과 CellBreak 초대형 표 페이지 수 범위 검증.
- `mydocs/report/task_m100_1842_cellbreak_report.md`: contributor 보고서.

## 체리픽 검토

- 누적 체리픽 순서: 1/11.
- 적용 커밋: `8d22453fc` (`Issue #1842: CellBreak 표 부재-LINE_SEG 셀 라인높이 em 교정 ...`).
- 충돌: 없음.
- 선행 PR 의존: 없음.

## 검증

- GitHub Actions: 원 PR head 기준 `Build & Test`, `CodeQL`, `Canvas visual diff` 등 성공 확인.
- `git diff --check upstream/devel...HEAD`: 통합 브랜치 fixup 이후 통과.
- `cargo fmt --check`: 통과.
- cargo 검증 전 `/Users/tsjang/rhwp/target` 하위 산출물 삭제 후 진행.
- `CARGO_INCREMENTAL=0 cargo test --features native-skia --test issue_1842 --test issue_2004_cell_image_stack_pagination --test issue_2083_hide_fill_page_background --test issue_2093_saved_single_line_spacing_after --test issue_2097_none_table_declared_fits`: 통과.
  - #2073 관련 `tests/issue_1842.rs`: 2 passed.

## 시각 검증

이 PR 자체에는 새 HWP/HWPX 샘플이나 기준 PDF가 추가되지 않았다. 기존 tracked sample `samples/issue2063_huge_cellbreak_table.hwp`는 성능/페이지 수 회귀 테스트 입력으로 사용되며, 이번 review에서 별도 MCP PDF 산출은 수행하지 않았다.

## 판단

- 체리픽 가능 여부: 가능.
- blocking finding: 없음.
- PR 주장은 focused test로 재현 가능하게 확인됐다. 다만 원 PR의 `mergeStateStatus`가 `BEHIND`이므로 실제 GitHub merge 전에는 최신 head/CI를 다시 확인해야 한다.
