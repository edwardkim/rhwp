# PR #1990 리뷰 - HWPX 각주/미주 모양 원본 값 보존

## 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1990 |
| 제목 | Issue #1984: HWPX 각주/미주 모양(noteLine·noteSpacing) 원본 값 보존 — 각주 페이지 표 분할 캐스케이드 해소 |
| 작성자 | planet6897 |
| base | `devel` |
| 문서 작성 시점 head SHA | `4aeea2e90f5b38515cd9fe797efceef5bdeb8b12` |
| 체리픽 commit | `1bfeabcae` |
| 규모 | 1 file, +121 / -0 |
| 변경 파일 | `src/serializer/hwpx/section.rs` |
| 처리 방식 | planet6897 PR 8건 통합 체리픽 |

## 변경 범위

- HWPX section 직렬화에서 각주/미주 모양(`noteLine`, `noteSpacing`) 원본 값을 보존한다.
- 보정 근거는 section/note shape 속성이다.
- #1987과 같은 `src/serializer/hwpx/section.rs` 영역을 수정한다.

## 체리픽 검토

- 적용 순서: 6/8
- 원 commit: `4aeea2e90f5b38515cd9fe797efceef5bdeb8b12`
- 로컬 commit: `1bfeabcae`
- 충돌: 있음
- 충돌 파일: `src/serializer/hwpx/section.rs`
- 해결 방식: #1987의 `replace_secpr_scalars`와 #1984의 `replace_footnote_shape` helper 및 호출을 모두 유지했다.

## 시각 검증

각주/미주 모양 보존은 렌더/쪽수에 영향을 줄 수 있으므로 기준 PDF가 있으면 visual sweep 대상이다. 문서 작성 시점에는 이 PR 전용 기준 PDF가 없어 직접 visual sweep은 수행하지 않았다. 대신 section 직렬화 targeted test와 통합 회귀 테스트로 확인했다.

## 로컬 검증

- `env CARGO_INCREMENTAL=0 cargo test --lib issue1984`: 1 passed
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: 통과
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과
- `cargo fmt --check`: 통과
- `git diff --check`: 통과

## 검토 결과

충돌은 동일 파일의 독립 helper 보존 충돌이었고, 두 기능을 모두 유지하는 방식으로 해결했다. 검증 통과. 최종 권고는 통합 PR merge 후보다.

