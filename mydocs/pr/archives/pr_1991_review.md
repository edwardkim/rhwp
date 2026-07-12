# PR #1991 리뷰 - HWP3 탭 char 위치 단위 통일

## 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1991 |
| 제목 | Issue #1950: HWP3→HWP5 변환 탭 char 위치 단위 통일 (376px 이탈 해소) |
| 작성자 | planet6897 |
| base | `devel` |
| 문서 작성 시점 head SHA | `01c74c4607f7a2089e3c0f99c57cf15b96684e89` |
| 체리픽 commit | `b1cd472b8` |
| 규모 | 8 files, +364 / -1 |
| 주요 변경 파일 | `src/parser/hwp3/mod.rs`, `tests/issue_1950_hwp3_tab_charoffset.rs`, `samples/issue1950_hwp3_tab_charoffset.hwp` |
| 처리 방식 | planet6897 PR 8건 통합 체리픽 |

## 변경 범위

- HWP3 탭 문자의 char offset 단위를 HWP5 직렬화 의미와 맞춘다.
- 탭을 1 code-unit로 보던 HWP3 파서 계산을 8 code-unit 의미로 맞춰 HWP3→HWP5 변환 후 탭 run 이탈을 줄인다.
- 특정 샘플명 분기가 아니라 탭 문자 단위 처리 규칙에 근거한다.

## 체리픽 검토

- 적용 순서: 7/8
- 원 commit: `01c74c4607f7a2089e3c0f99c57cf15b96684e89`
- 로컬 commit: `b1cd472b8`
- 충돌: 없음
- 선행 PR 의존: 없음

## 시각 검증

사용자가 제공한 기준 PDF를 사용해 visual sweep을 수행했다.

| 항목 | 내용 |
|---|---|
| 샘플 | `samples/issue1950_hwp3_tab_charoffset.hwp` |
| 기준 PDF | `pdf/issue1950_hwp3_tab_charoffset-2024.pdf` |
| 실행 | `python3 scripts/task1274_visual_sweep.py --key pr1991_issue1950 --hwp samples/issue1950_hwp3_tab_charoffset.hwp --pdf pdf/issue1950_hwp3_tab_charoffset-2024.pdf --out output/task1274` |
| 페이지 수 | SVG 1쪽 / 기준 PDF 1쪽 |
| 자동 후보 | `flagged=0/1` |
| pixel match | `95.90899%` |
| 내용 픽셀 중심 자동 일치율 보조값 | `7.41154%` |
| 대표 산출물 | `output/task1274/pr1991_issue1950/review/review_003.png` |
| 대표 asset | `mydocs/pr/assets/pr_1991_issue1950_review_003.png` |

사람 판정 메모: p3 대응 산출물에서 표 구조와 탭 위치는 기준 PDF와 맞는다. 농도/폰트 차이 때문에 내용 픽셀 중심 보조값은 낮지만, 자동 후보 0/1이고 PR 주장인 탭 char offset 이탈 해소에는 부합한다.

## 로컬 검증

- `env CARGO_INCREMENTAL=0 cargo test --test issue_1950_hwp3_tab_charoffset`: 1 passed
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: 통과
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과
- `cargo fmt --check`: 통과
- `git diff --check`: 통과

## 검토 결과

targeted test와 기준 PDF visual sweep이 모두 PR 목적과 맞는다. 최종 권고는 통합 PR merge 후보다.

