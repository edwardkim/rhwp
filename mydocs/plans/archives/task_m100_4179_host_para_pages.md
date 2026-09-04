# 수행계획 — task_m100_4179_host_para_pages

- **이슈**: [#4179](https://github.com/edwardkim/rhwp/issues/4179)
- **브랜치**: `task_m100_4179_host_para_pages`
- **기준**: `upstream/devel` `e64c85312`
- **작성 시각**: 2026-08-08 KST

## 1. 목표

텍스트 있는 표 호스트 문단의 getCursorRect 가 후보 전 페이지를 비캐시 렌더 트리로 빌드하는 O(pages) 경로(#4126 미커버 자매 케이스, 115쪽 문서 열기 2.5s 실측)를 pagination 메타데이터만으로 제거한다.

## 2. 변경 경계

- 순수-중간 연속 컷(cont=true && end_cut 비어있지 않음) 페이지를 후보에서 제외 — 렌더 트리 빌드 없이 판정.
- 기존 캐럿 좌표 출력 불변(회귀 테스트 tests/issue_4179_cursor_rect_text_host_para_pages.rs 동봉).

## 검증 게이트

- 레이어 자체 회귀 테스트 + `cargo nextest run --cargo-profile release-test` 전체
- fmt / clippy(root+workspace all-targets) / wasm32 check / Native Skia 3종 / studio suite

원격 push, PR 생성, 이슈 comment·close는 별도 승인 전 수행하지 않는다.
