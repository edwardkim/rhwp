# PR #1985 리뷰 - 거대 셀 렌더 O(pages×cell) 재계산 제거

## 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1985 |
| 제목 | Issue #1949: 거대 셀 렌더 O(pages×cell) 재계산 제거 (cell_units 메모이즈) |
| 작성자 | planet6897 |
| base | `devel` |
| 문서 작성 시점 head SHA | `7024ea4f6af489bc74b415d1c2c2542b2c2a0102` |
| 체리픽 commit | `508cb97d4` |
| 규모 | 10 files, +394 / -0 |
| 주요 변경 파일 | `src/renderer/layout/table_layout.rs`, `src/renderer/layout.rs`, `src/document_core/queries/rendering.rs`, `tests/issue_1949_giant_cell_render_perf.rs` |
| 처리 방식 | planet6897 PR 8건 통합 체리픽 |

## 변경 범위

- 거대 셀 RowBreak 표 렌더 시 `cell_units`를 페이지마다 재계산하는 병목을 `LayoutEngine` 라이프사이클 캐시로 제거한다.
- 보정 근거는 셀 포인터 기준의 렌더 중 IR 불변성과 재조판 경계에서 캐시를 비우는 구조다.
- PR 목적은 기준 PDF 완전 일치가 아니라 병리적 렌더 타임아웃 제거와 출력 불변 성능 최적화다.

## 체리픽 검토

- 적용 순서: 4/8
- 원 commit: `7024ea4f6af489bc74b415d1c2c2542b2c2a0102`
- 로컬 commit: `508cb97d4`
- 충돌: 없음
- 선행 PR 의존: #1976과 renderer 영역은 다르며 직접 충돌 없음.

## 시각 검증

사용자가 제공한 기준 PDF를 사용해 visual sweep을 수행했다.

| 항목 | 내용 |
|---|---|
| 샘플 | `samples/issue1949_giant_cell_nested_tables_perf.hwpx` |
| 기준 PDF | `pdf/issue1949_giant_cell_nested_tables_perf-2024.pdf` |
| 실행 | `python3 scripts/task1274_visual_sweep.py --key pr1985_issue1949 --hwp samples/issue1949_giant_cell_nested_tables_perf.hwpx --pdf pdf/issue1949_giant_cell_nested_tables_perf-2024.pdf --out output/task1274` |
| 페이지 수 | SVG 112쪽 / 기준 PDF 115쪽 |
| 자동 후보 | `flagged=100/112` |
| pixel match | 평균 `91.86403%`, worst `88.307%` |
| 내용 픽셀 중심 자동 일치율 보조값 | 평균 `14.1036%`, worst `7.39774%` |
| 대표 산출물 | `output/task1274/pr1985_issue1949/review/review_103.png` |
| 대표 asset | `mydocs/pr/assets/pr_1985_issue1949_review_103.png` |

사람 판정 메모: p103 기준으로 rhwp와 기준 PDF 사이에 큰 수직 변위와 내용 배치 차이가 있다. 이 결과는 한컴 기준 시각 정합 PASS가 아니다. 다만 PR #1985의 핵심 주장은 거대 셀 렌더가 타임아웃 없이 완주하도록 하는 성능 병목 제거이며, 테스트도 페이지 수를 90..=130 범위로 보고 전체 렌더 완주와 중간 페이지 콘텐츠 존재를 가드한다. 따라서 이번 통합 PR의 merge 판단에서는 성능 수정은 수용 가능하지만, `issue1949` 기준 PDF 115쪽 정합은 별도 후속 이슈 후보로 남긴다.

## 로컬 검증

- `env CARGO_INCREMENTAL=0 cargo test --test issue_1949_giant_cell_render_perf`: 1 passed, full render performance guard 완료
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: 통과
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과
- `cargo fmt --check`: 통과
- `git diff --check`: 통과

## 검토 결과

성능 회귀 guard는 통과했다. 기준 PDF 대비 페이지/시각 정합은 통과가 아니므로 후속 추적이 필요하다. 최종 권고는 “성능 개선 PR로 merge 수용, 기준 PDF 시각 정합은 후속 이슈로 분리”다.
