# PR #2065 리뷰: Issue #2063 초대형 표 O(n^2) 셀 측정 제거

- 작성 시각: 2026-07-08 21:40 KST
- PR: https://github.com/edwardkim/rhwp/pull/2065
- 작성자: `planet6897`
- base: `devel`
- head: `fix/2063-cellunits-quadratic-scan`
- 문서 작성 시점 참고 head: `c9a59d3143126191d5799860f7676d4ca06d6513`
- 문서 작성 시점 참고 merge state: `CLEAN`
- 원 PR merge 결과: 2026-07-08, merge commit `3b0790c0449f22068ea02b9b43a669118a70e3c0`
- PR 검토 코멘트: https://github.com/edwardkim/rhwp/pull/2065#issuecomment-4914910456
- review 기록 보존: 옵션 2에 따라 별도 후속 PR로 처리

## 관련 이슈

- #2063: 화성시 사무전결 처리규칙 별표2 HWP의 52,694셀 CellBreak 표에서 렌더/비교가 장시간 지연되는 성능 이슈.
- PR 본문은 `dump-pages`/`render-diff` timeout의 주원인을 `cell_units_uncached` 내부 표 전체 스캔이 셀마다 반복되는 O(n^2) hot path로 설명한다.
- 한컴 기준 대비 페이지 과분할(`rhwp 213p` vs `HWP 2020 PDF 162p`)은 이번 PR의 수정 범위 밖이며 #1937/#1842 계열 pagination fidelity 축으로 남긴다.

## 변경 범위

- `src/renderer/layout.rs`
  - `LayoutEngine`에 `table_nested_text_flag_cache: RefCell<HashMap<usize, bool>>` 추가.
  - 기존 `cell_units_cache`와 같은 `clear_layout_caches()` 경계에서 함께 clear.
- `src/renderer/layout/table_layout.rs`
  - 표 단위 불변량 `has_visible_text_with_nested_table`을 표 포인터 키로 1회 계산/캐시.
  - `cell_units_uncached`의 셀별 표 전체 스캔을 제거.
- `tests/issue_2063.rs`
  - 공개 샘플을 로드해 page_count가 timeout 없이 완주하고 기대 범위 안에 있는지 검증.
- `samples/issue2063_huge_cellbreak_table.hwp`
  - 재현 샘플 추가.
- `mydocs/plans/*`, `mydocs/report/*`, `mydocs/working/*`
  - contributor 측 계획/보고 기록 추가.

## MCP 기준 PDF 검증

PR에 별도 기준 PDF가 없어서 HWP 2020 MCP 서버로 `samples/issue2063_huge_cellbreak_table.hwp`를 PDF 변환했다.

- 산출 PDF: `pdf/issue2063_huge_cellbreak_table-2020.pdf`
- 크기: 38,501,003 bytes
- SHA-256: `6dca4c32a38fc57ff1cc5fe13699e8ddc68b58fc3d838711b85a6e99d9a1c8e6`
- MCP job id: `87174a0f-11b6-40f9-9ee7-040a14f3cc31`
- MCP run_status: `0`
- MCP validation: `ok`
- PDF page count: 162

초기 direct client 호출은 MCP SDK 기본 request timeout으로 `Request timed out`이 발생했다. 검증용 `/tmp` 클라이언트에서 `callTool` request timeout을 장문서용으로 연장한 뒤 같은 입력을 재실행해 성공했다. 서버 주소와 인증 토큰은 기록하지 않았다.

## 로컬 검증

- reviewer assign: `jangster77` 요청 완료.
- merge simulation: `git merge upstream/devel --no-edit` -> `Already up to date.`
- `git diff --check upstream/devel...HEAD`: 통과.
- cargo 검증 전 `/Users/tsjang/rhwp/target` 하위 산출물 삭제 후 진행.
- `cargo fmt --check`: 통과.
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_2063 -- --nocapture`: 1 passed, finished in 1.56s.
- `target/release-test/rhwp dump-pages samples/issue2063_huge_cellbreak_table.hwp`: 213 pages, real 1.58s.
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --lib`: 2150 passed, 0 failed, 7 ignored, finished in 6.85s.
- `CARGO_INCREMENTAL=0 cargo clippy --profile release-test --lib --tests -- -D warnings`: 통과.

## 시각 검증

MCP PDF를 기준으로 대표 페이지 1, 80, 162를 visual sweep 했다.

- 명령 결과: `output/pr2065_issue2063_visual/summary.json`
- 안정 보관 summary: `mydocs/pr/assets/pr_2065_issue2063_visual_summary.json`
- 안정 보관 이미지: `mydocs/pr/assets/pr_2065_issue2063_review_contact_sheet.png`
- rhwp SVG pages: 213
- rhwp render tree pages: 213
- MCP PDF pages: 162
- analyzed pages: 3
- flagged pages: 0
- overlay average pixel match: 88.14263%
- overlay worst pixel match: 85.35499%
- overlay average ink match: 6.0278%
- overlay worst ink match: 5.49795%

의미 기반 sweep flag는 없었다. 다만 overlay/ink match는 낮다. 이는 PR 범위 밖으로 분리한 행/줄높이 누적 드리프트와 페이지 수 차이(`213p` vs `162p`)의 영향으로 해석한다. 이번 PR의 목적은 timeout 해소와 순수 최적화 여부 확인이며, 기준 PDF 대비 전체 pagination fidelity 완료는 별도 축으로 남긴다.

## 리스크와 리뷰 판단

- blocking finding: 없음.
- `table_nested_text_flag_cache`는 기존 `cell_units_cache`와 같은 포인터 키/재조판 clear 경계를 공유한다. 현재 구조상 PR이 도입한 캐시 수명은 기존 셀 단위 캐시의 전제와 정합한다.
- `tests/issue_2063.rs`의 페이지 수 assertion은 `150..=260` 범위형이다. PR 본문의 "현재 213p 불변"을 정확히 고정하지는 않지만, 향후 #1937/#1842에서 페이지 수 개선이 들어올 가능성을 고려하면 timeout 회귀 가드로는 허용 가능하다. merge 보류 사유로 보지 않는다.
- MCP 기준 PDF로 한컴 쪽 162p를 확인했고, rhwp는 213p로 완주했다. 따라서 PR의 성능 개선 주장과 범위 밖 분류는 타당하다.

## 처리 결과

작업지시자 승인에 따라 GitHub에는 `APPROVE` review 대신 일반 검토 코멘트를 남겼고, 최신 GitHub Actions 통과와 `CLEAN` 상태를 재확인한 뒤 PR #2065를 merge했다. #2063은 성능 timeout 해결로 close하되, 페이지 과분할/pagination fidelity는 #1937/#1842 계열 이슈로 남긴다는 코멘트를 남기는 편이 좋다.
