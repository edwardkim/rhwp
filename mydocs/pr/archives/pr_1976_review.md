# PR #1976 리뷰 - 각주 많은 RowBreak 표 연속 페이지 과분할 수정

## 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1976 |
| 제목 | Issue #1937: 각주 많은 RowBreak 표 연속 페이지 과분할 수정 |
| 작성자 | planet6897 |
| base | `devel` |
| 문서 작성 시점 head SHA | `d3b9ecd3a30ad82573486547b7ba2ab62995f38f` |
| 체리픽 commit | `f1bf93763` |
| 규모 | 8 files, +386 / -1 |
| 주요 변경 파일 | `src/renderer/typeset.rs`, `tests/issue_1937_rowbreak_footnote_overpagination.rs`, `samples/issue1937_rowbreak_footnote_overpagination.hwp` |
| 처리 방식 | planet6897 PR 8건 통합 체리픽 |

## 변경 범위

- 각주가 많은 RowBreak 표에서 연속 페이지가 과분할되는 조건을 보정한다.
- 신규 HWP 샘플과 회귀 테스트가 추가됐다.
- 렌더/쪽수 영향이 있는 PR이므로 기준 PDF가 있으면 visual sweep 대상이다.

## 체리픽 검토

- 적용 순서: 2/8
- 원 commit: `d3b9ecd3a30ad82573486547b7ba2ab62995f38f`
- 로컬 commit: `f1bf93763`
- 충돌: 없음
- 선행 PR 의존: #1974 뒤에 적용했으나 파일 겹침 없음.

## 시각 검증

신규 샘플 `samples/issue1937_rowbreak_footnote_overpagination.hwp`가 추가되어 시각 검증 후보지만, 문서 작성 시점에 한컴 2020/2024 기준 PDF가 없어서 visual sweep은 수행하지 않았다. 기준 PDF가 제공되면 페이지 수와 각주/표 분할 위치를 확인해야 한다.

## 로컬 검증

검토 시작 전 `/Users/tsjang/rhwp/target` 하위 항목을 삭제했다.

- `env CARGO_INCREMENTAL=0 cargo test --test issue_1937_rowbreak_footnote_overpagination`: 1 passed
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: 통과
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과
- `cargo fmt --check`: 통과
- `git diff --check`: 통과

## 검토 결과

targeted test가 PR 목적을 직접 가드하고 전체 회귀 테스트도 통과했다. 기준 PDF가 없어 시각 정합은 후속 확인 후보로 남긴다. 최종 권고는 통합 PR merge 후보다.

