# PR #3019 검토 — 가운뎃점 합성 원 반지름 실측 보정

| 항목 | 내용 |
| --- | --- |
| 원 PR | [#3019](https://github.com/edwardkim/rhwp/pull/3019) |
| 작성자 | planet6897 |
| 관련 이슈 | [#2999](https://github.com/edwardkim/rhwp/issues/2999) |
| base / 규모 | devel / 5 files, +37 -15 |
| 문서 작성 시점 참고값 | 원 PR head 786b2842f80a789cd4292a0f467bce1827251449, BEHIND 및 MERGEABLE. 이전 head의 Build & Test, CodeQL, Canvas visual diff는 성공으로 확인했다. |
| 통합 적용 | 최신 upstream/devel 866925fa6 위 적용 commit 2a6011dd14ef85ae8e1ae40f8df4148413c7f527 |

## 관련 이슈와 변경 범위

- #2999의 목적은 폰트 대체를 피하려고 벡터 원으로 그리는 U+00B7 가운뎃점의 반지름을 한글 COM PDF 실측값에 맞추는 것이다.
- 핵심 변경은 render tree의 반지름 상수와 SVG 출력 경로다. 반지름을 0.080em에서 0.060em으로 보정하고, form-002 golden을 함께 갱신한다.
- 메타 변경이나 범위 밖 리팩터링은 없다.

## 렌더 영향과 검증

- SVG 출력 geometry가 바뀌므로 visual 확인 대상이다. form-002 golden snapshot으로 수정값을 고정했고, 통합 렌더 visual sweep도 함께 수행했다.
- 최신 통합 브랜치에서 cargo fmt --check, cargo test --test svg_snapshot을 실행했다. svg snapshot 8건이 모두 통과했으며 form-002 page 0도 포함된다.
- 검토 시작 전 분리 target과 CARGO_INCREMENTAL=0을 사용했다. Clippy, doctest, release 전체 검증은 원격 통합 PR의 최신 head CI에서 최종 확인한다.

## 리스크와 판단

- 상수 변경은 모든 SVG 가운뎃점의 모양에 영향을 준다. 그러나 해당 출력의 golden을 갱신했고, 합성 원의 위치 계산과 색상 경로는 바꾸지 않아 범위가 좁다.
- 원 PR의 base가 뒤처져 있으므로 개별 merge 대신 최신 devel 위 통합 브랜치에 원 작성자 commit을 보존해 적용했다.

## 최종 권고

- planet6897 렌더 4건 통합 PR에 포함해 수용 권고.
- 최종 merge 조건은 통합 PR 최신 head의 GitHub Actions 통과와 작업지시자 승인이다. 원 PR 상태와 CI는 merge 직전에 다시 확인한다.
