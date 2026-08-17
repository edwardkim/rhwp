# PR #5183 검토 - HWP3 표 미커버 격자 완성

- PR: https://github.com/edwardkim/rhwp/pull/5183
- 작성자: `planet6897`
- base: `devel`
- 원 head: `30317e7f5b96a53e29d772ed23831472ddfe437d`
- 누적 검토 브랜치: `review/planet6897-hwp-contracts-20260818`
- 누적 통합 PR: https://github.com/edwardkim/rhwp/pull/5197
- 체리픽 커밋: `8848c6eb3`

## 결론

blocking finding 없음. HWP3 표 셀들이 덮지 않는 제한된 격자 칸을 1x1 빈 셀로
보충해 이후 HWP5 저장 경로가 완전한 표 격자를 소비하도록 한다. 빈 셀에도 문단을
넣어 기존 셀 문단 계약을 유지한다.

## 검증

- 체리픽 충돌 없음
- focused: `issue_4367_hwp3_convert_fourth_contract` 10 passed
  - `hwp3_table_cells_cover_full_grid` 포함
- 누적 전체 Rust 회귀: 6,735 passed, 38 skipped, 3 slow
- 구조 확인: `git diff --check upstream/devel...HEAD` pass

## Fixture와 시각 증적

- 관련 fixture: `samples/hwp3-table-grid-gap.hwp`
- 변경은 파서의 표 모델 정규화이며 renderer 외관 기준을 변경하지 않는다. PDF를
  새로 만들지 않았고, 실제 한글 앱에서의 열기 검증은 필요 시 별도 증적으로 보강한다.

## 리스크와 권고

비정상 HWP3 표의 해석 범위가 넓어지는 변경이다. 행·열 수가 `0x4000` 이하인 표로
경로를 제한하고 전체 회귀를 통과했으므로 누적 통합 PR 후보에 포함할 수 있다.
