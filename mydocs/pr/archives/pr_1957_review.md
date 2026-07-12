# PR #1957 리뷰

## 메타

| 항목 | 내용 |
| --- | --- |
| PR | #1957 |
| 작성자 | planet6897 |
| 제목 | Issue #1945: PartialParagraph start_line 슬라이스 범위 밖 패닉 방어 |
| base | devel |
| 문서 작성 시점 참고값 | mergeable: MERGEABLE, mergeStateStatus: BEHIND |
| 변경 규모 | 2 files, +81/-3 |

## 변경 범위

- `paragraph_layout.rs`의 `composed.lines[start_line..end]` 직접 슬라이스를 `get(start_line..end)` 기반 방어 코드로 바꾼다.
- `start_line`이 실제 라인 수를 넘는 PartialParagraph 경로에서 패닉하지 않도록 한다.
- 동일 패닉을 재현하는 단위 테스트를 추가했다.

## visual sweep 판정

- 렌더러 레이아웃 경로 변경이므로 시각 영향 PR로 분류한다.
- 다만 이 PR의 핵심 목적은 실문서 export 중 패닉 방어이며, 유효 범위 레이아웃 동작은 기존과 동일하게 유지하는 패치다.
- GitHub Actions의 Render Diff `Canvas visual diff`가 성공했고, 로컬 `cargo test --profile release-test --tests`에 포함된 `svg_snapshot`도 통과했다.

## 로컬 검증

누적 검토 브랜치 `review/planet-1940-1960`에서 오래된 순서로 cherry-pick했다.

- `git diff --check upstream/devel...HEAD`: 통과
- `cargo fmt --check`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --lib`: 통과, 2123 passed / 0 failed / 6 ignored
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: 통과, `svg_snapshot` 포함
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과
- GitHub Actions: Build & Test, CodeQL, Render Diff `Canvas visual diff` 성공

## 검토 메모

- 범위 밖 슬라이스를 "가시 run 없음"으로 처리하는 방어 코드이며, `start_line..end`가 유효한 정상 경로의 렌더링은 바꾸지 않는다.
- 근본적인 start_line 오버슛 추적은 별도 후속 축으로 남길 수 있으나, 패닉 방어 자체는 merge 가치가 있다.

## 결론

merge 후보로 판단한다. #1953 이후 순서로 merge한다.
