# PR #2100 리뷰 — #2004 부동 표 셀 이미지 스택 콘텐츠 페이지네이션

- 작성 시각: 2026-07-09 16:30 KST
- PR: https://github.com/edwardkim/rhwp/pull/2100
- 작성자: `planet6897`
- base / head: `devel` / `fix/2004-cell-image-stack-pagination`
- 문서 작성 시점 참고 head: `90b86e1975dc80ecde61501af4ec27d3985d80b5`
- 문서 작성 시점 참고 merge state: `BEHIND`
- reviewer assign: `jangster77` 요청 완료
- 처리 경로: `codex/planet6897-prs-review-20260709` 에서 여러 PR 누적 체리픽 검토

## 변경 범위

- `samples/issue2004_cell_image_stack.hwp`, `samples/issue2004_cell_image_stack.hwpx` 추가.
- `pdf/issue2004_cell_image_stack-2022.pdf` 추가.
- `src/document_core/queries/rendering.rs`: 부동 표 셀 이미지 스택을 콘텐츠 높이에 맞게 분할/정규화.
- `tests/issue_2004_cell_image_stack_pagination.rs`: HWP/HWPX 모두 8쪽 고정 검증.
- 관련 계획/보고/working 문서 추가.

## 체리픽 검토

- 누적 체리픽 순서: 10/11.
- 적용 커밋: `8956e2c2d` (`Issue #2004: 부동 표 셀 이미지 스택 콘텐츠 페이지네이션 ...`).
- 충돌: 없음.
- 선행 PR 의존: #2092 이후 적용했으나 충돌 없음.

## 기준 PDF

PR에 기준 PDF가 포함되어 있어 MCP 변환은 별도로 수행하지 않았다.

| 항목 | 내용 |
|---|---|
| 기준 PDF | `pdf/issue2004_cell_image_stack-2022.pdf` |
| PDF SHA-256 | `927090a49dd8c9d570b06833129732de2a3a40cf99c244c2786e08a3e8d03e8d` |
| pdfinfo | 8 pages, Creator `Hwp 2022 12.0.0.4547`, Producer `Hancom PDF 1.3.0.550` |

## 로컬 검증

- GitHub Actions: 원 PR head 기준 `Build & Test`, `CodeQL`, `Canvas visual diff` 등 성공 확인.
- `git diff --check upstream/devel...HEAD`: 통합 브랜치 fixup 이후 통과.
- `cargo fmt --check`: 통과.
- cargo 검증 전 `/Users/tsjang/rhwp/target` 하위 산출물 삭제 후 진행.
- `CARGO_INCREMENTAL=0 cargo test --features native-skia --test issue_1842 --test issue_2004_cell_image_stack_pagination --test issue_2083_hide_fill_page_background --test issue_2093_saved_single_line_spacing_after --test issue_2097_none_table_declared_fits`: 통과.
  - #2100 관련 `tests/issue_2004_cell_image_stack_pagination.rs`: 2 passed, HWP/HWPX 모두 8쪽.

## 시각 검증

요약:

- rhwp SVG pages: 8
- 기준 PDF pages: 8
- flagged pages: 0/8
- overlay average pixel match: 83.29703%
- overlay average visual accuracy proxy: 18.95654%
- 대표 asset: `mydocs/pr/assets/planet6897_prs_20260709/pr2100_issue2004_p008_review.png`
- metrics: `mydocs/pr/assets/planet6897_prs_20260709/pr2100_issue2004_overlay_metrics.json`

사람 판정: 대표 p8은 폰트/스케일 차이는 있으나 페이지 수와 흐름은 8쪽 기준과 맞고, 자동 후보도 0/8이다.

## 판단

- 체리픽 가능 여부: 가능.
- blocking finding: 없음.
- 기준 PDF가 PR에 포함되어 있고, HWP/HWPX regression test 및 visual sweep이 PR의 핵심 8쪽 주장을 뒷받침한다.
