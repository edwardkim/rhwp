# PR #1787 리뷰 — Task #1748 컷 걸침 rowspan 셀 높이 기반 유닛 컷

## PR 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1787 |
| 작성자 | @planet6897 |
| base / head | `devel` / `planet6897:pr/devel-1748` |
| 관련 이슈 | #1748 |
| reviewer assign | @jangster77 요청 완료 |
| 적용 방식 | 비시리즈·샘플 미포함 PR 누적 cherry-pick |

## 변경 범위

- `src/renderer/layout/table_layout.rs`
- `src/renderer/layout/table_partial.rs`
- `tests/issue_1748_rowbreak_straddle_rowspan.rs`
- `tools/compare_page_bbox.py`
- 관련 계획/보고/working 문서

RowBreak 표에서 페이지 경계가 rowspan 셀 내부를 가르는 경우, 기존 cut bookkeeping 이 row_span==1 셀만
다루어 경계에 걸친 rowspan 셀이 컷 페이지에서 넘치거나 연속 페이지에서 중복 렌더되던 문제를 높이 기반
unit cut 으로 제한한다.

## 검토 결과

새 `cell_units_fitting_height` 는 셀 유닛 누적 높이와 같은 budget 식으로 선두 유닛 수를 계산한다. 부분 표
렌더링에서는 `is_rowbreak_straddle` 조건을 RowBreak, rowspan, repeated header 제외, split row 제외로 좁혀
기존 block split 경로와 충돌하지 않게 했다. straddling 셀에는 clip 과 top align 을 적용해 컷 범위 밖 잉크가
셀 박스 밖으로 새지 않도록 한다.

## 검증

- 누적 cherry-pick 충돌 없음
- `git diff --check upstream/devel..HEAD`: 통과
- `cargo fmt --check`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: 통과
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과
- `cargo run --bin rhwp -- export-pdf samples/table_scattered_header_rowbreak.hwp -o target/tmp-pr1787/table_scattered_header_rowbreak-rhwp.pdf`: 통과, 53페이지 PDF 생성
- `pdftoppm`/PIL 기반 p6 dark-pixel bbox 게이트: 통과
  - 기준: `pdf/table_scattered_header_rowbreak-2024.pdf`
  - 결과: `p6 hancom top/bot=77/1081, rhwp top/bot=75/1077, dTop=-2, dBot=-4`
  - 게이트: `|dBot| <= 5`

주의: PR 에 포함된 `tools/compare_page_bbox.py` 는 로컬 Python 에 `pymupdf(fitz)`가 없어 직접 실행하지 못했다.
동일 목적의 bbox 비교는 `pdftoppm`과 PIL 로 재현했다.

## 결론

테스트와 기준 PDF bbox 게이트가 모두 통과했다. merge 후보로 판단한다.
