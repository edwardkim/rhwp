# 수행계획 — task_m100_4180_caret_meta_on_save

- **이슈**: [#4180](https://github.com/edwardkim/rhwp/issues/4180)
- **브랜치**: `task_m100_4180_caret_meta_on_save`
- **기준**: `upstream/devel` `e64c85312`
- **작성 시각**: 2026-08-08 KST

## 1. 목표

문서 캐럿 메타데이터(DOCUMENT_PROPERTIES)가 마지막 본문 편집 위치를 기록해 열기 시 캐럿이 엉뚱한 페이지로 복원되는 결함(#4180, 실측: 마지막 페이지)을 저장 시점 캐럿 기록으로 바로잡는다.

## 2. 변경 경계

- 본문 편집마다의 메타 갱신을 제거하고 저장 경로에서 현재 캐럿을 스탬프.
- 라운드트립 회귀 테스트 tests/issue_4180_caret_stamp_roundtrip.rs 동봉.

## 검증 게이트

- 레이어 자체 회귀 테스트 + `cargo nextest run --cargo-profile release-test` 전체
- fmt / clippy(root+workspace all-targets) / wasm32 check / Native Skia 3종 / studio suite

원격 push, PR 생성, 이슈 comment·close는 별도 승인 전 수행하지 않는다.
