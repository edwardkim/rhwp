# PR #2107 리뷰 — 구역 첫 쪽 배경 이미지 적용

- 작성 시각: 2026-07-10 KST
- PR: https://github.com/edwardkim/rhwp/pull/2107
- 작성자: `planet6897`
- base / head: `devel` / `pr-task2102`
- 문서 작성 시점 참고 head: `48da1531dc0a71c4138d8e766834c8e5082aacf3`
- 문서 작성 시점 참고 mergeable: `MERGEABLE`
- 처리 경로: `codex/planet6897-cherrypick-20260710` 에서 여러 PR 누적 체리픽 검토

## 변경 범위

- `LayoutEngine`에 현재 페이지가 구역 첫 쪽인지 나타내는 컨텍스트를 추가한다.
- `build_page_background`에서 그림 채우기 배경만 구역 첫 쪽으로 제한한다.
- 색/그라데이션 채우기, 쪽 테두리선, 쪽번호 배치는 기존 동작을 유지한다.
- `page_bg_image_only_on_section_first_page` 단위 테스트와 task #2102 문서를 추가한다.

## 체리픽 검토

- 누적 체리픽 순서: 1/4.
- 적용 커밋: `7edc2a924` (`5078da3f59c4f3c0ca4cbb5ac35211cfca047341`에서 `-x` 체리픽).
- 충돌: `src/renderer/layout.rs`.
- 충돌 해소: #2083의 `hide_fill` 처리와 #2107의 `allow_bg_image` 처리를 병합했다. `hide_fill`이면 흰 종이 바탕만 유지하고, 일반 경로에서는 색/그라데이션은 유지하되 이미지만 구역 첫 쪽 조건을 적용한다.
- 선행 PR 의존: 없음.

## 검증

- 원 PR GitHub Actions: 문서 작성 시점 기준 `CI`, `CodeQL`, `Render Diff` 계열 check 성공 확인.
- `git diff --check upstream/devel...HEAD`: 통과.
- `cargo fmt --check`: 통과.
- `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과.
- `CARGO_INCREMENTAL=0 cargo test page_bg_image_only_on_section_first_page --lib`: 1 passed.
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: 통과(exit 0).

## 판단

- 체리픽 통합 가능.
- 충돌은 기존 `hide_fill` 정책과 신규 첫 쪽 이미지 배경 정책을 모두 보존하는 방식으로 해소했다.
- 원 PR은 통합 PR이 merge된 뒤 supersede close/comment 처리 대상이다.
