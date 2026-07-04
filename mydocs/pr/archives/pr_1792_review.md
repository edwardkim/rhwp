# PR #1792 리뷰 — Task #1591 v2 first-para hidden slot / fieldEnd roundtrip 정합

## PR 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1792 |
| 작성자 | @planet6897 |
| base / head | `devel` / `planet6897:pr/devel-1591` |
| 관련 이슈 | #1591, #1593 |
| reviewer assign | @jangster77 요청 완료 |
| 적용 방식 | planet6897 open PR 묶음 누적 cherry-pick |

## 변경 범위

- `src/serializer/hwpx/mod.rs`
- `src/serializer/hwpx/section.rs`
- `tools/roundtrip_control_compare.py`
- `tests/fixtures/opengov_snapshot.tsv`
- `samples/hwpx/opengov/*.hwpx`
- 관련 계획/작업/보고 문서

HWPX serializer가 first paragraph의 hidden slot, char shape 위치, `fieldEnd`, bookmark를 입력 순서대로 보존해
roundtrip IR diff를 줄이도록 한다.

## 검토 결과

이 PR의 목적은 renderer 결과를 직접 고치는 것이 아니라 HWPX serializer/roundtrip 구조 보존이다. 따라서 기준
PDF와 rhwp renderer의 visual sweep 차이는 참고 자료로만 기록하고, 그 차이만으로 merge 보류 사유로 삼지
않았다.

코드 변경은 hidden slot을 앞단에서 임의 hoist하지 않고 문단 run/control 순서를 보존하는 방향이다. focused
unit test가 bookmark hoist 방지와 first paragraph hidden slot의 char shape 위치를 직접 확인한다.

## 시각 참고 자료

| target | SVG/PDF pages | flagged | 판단 |
|---|---:|---:|---|
| `pr1792-36384689` | 1/1 | 1 | renderer 직접 비교 차이 있음. serializer PR 범위 밖 참고 자료 |
| `pr1792-36385445` | 1/2 | 0 | 페이지 수 차이 참고. serializer PR 범위 밖 참고 자료 |
| `pr1792-36388711` | 9/9 | 1 | 일부 페이지 drift 있음. serializer PR 범위 밖 참고 자료 |

대표 참고 asset:

- `mydocs/pr/assets/pr1792_reference_36384689_review_p298.png`
- `mydocs/pr/assets/pr1792_reference_36388711_review_p007.png`

## 검증

- `git diff --check upstream/devel..HEAD`: 통과
- `cargo fmt --check`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --lib task1591`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: 통과
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과

## 결론

PR 내용 기준으로 serializer/roundtrip 구조 보존 테스트가 통과했다. visual sweep 차이는 renderer 직접 비교 참고값으로
분리했고 merge blocker로 보지 않는다. merge 후보로 판단한다.
