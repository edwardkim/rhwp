# PR #1960 리뷰

## 메타

| 항목 | 내용 |
| --- | --- |
| PR | #1960 |
| 작성자 | planet6897 |
| 제목 | Issue #1944: legacy 공용 도형 경로 drawText 미방출 — 도형 내 텍스트 소실 수정 |
| base | devel |
| 문서 작성 시점 참고값 | mergeable: MERGEABLE, mergeStateStatus: CLEAN |
| 변경 규모 | 1 file, +55/-1 |

## 변경 범위

- `render_common_shape_xml` 경로의 polygon/ellipse/arc/curve 계열 도형에 `drawText` 방출을 추가한다.
- rect 경로와 달리 공용 legacy 경로에서 도형 내부 텍스트가 저장 시 소실되던 문제를 해결한다.
- 빈 글상자는 기존 rect 정책과 동일하게 미방출한다.

## visual sweep 판정

- serializer 변경이지만 도형 내부 텍스트 보존은 사용자-visible 출력에 영향이 있다.
- PR 본문은 실파일의 drawText 개수와 render-diff 노드 수 정합을 검증 근거로 제시한다.
- 로컬 통합 테스트와 GitHub CI가 통과했고, 같은 누적 브랜치에서 `svg_snapshot`도 통과했다.
- 이 PR 자체에 기준 PDF asset이 추가된 것은 아니므로 별도 visual sweep 이미지는 남기지 않고, 구조 보존 게이트와 PR 본문 render-diff 결과를 근거로 삼는다.

## 로컬 검증

누적 검토 브랜치 `review/planet-1940-1960`에서 오래된 순서로 cherry-pick했다.

- `git diff --check upstream/devel...HEAD`: 통과
- `cargo fmt --check`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --lib`: 통과, 2123 passed / 0 failed / 6 ignored
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: 통과, `svg_snapshot` 포함
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과

## 검토 메모

- OWPML 순서 `shadow -> drawText -> hc:pt`에 맞춰 추가되어 구조적으로 무리 없는 변경이다.
- PR 본문이 밝힌 connectLine/line 컴포넌트 결손은 #1943 별도 축으로 남는다.

## 결론

merge 후보로 판단한다. #1959 이후 순서로 merge한다.
