# 수행계획 — task_m100_4163_resize_observer_noise

- **이슈**: [#4163](https://github.com/edwardkim/rhwp/issues/4163)
- **브랜치**: `task_m100_4163_resize_observer_noise`
- **기준**: `upstream/devel` `e64c85312`
- **작성 시각**: 2026-08-08 KST

## 1. 목표

ResizeObserver 루프 경고가 uncaught error 로 보고되는 잡음(#4163)을 제거한다 — 관찰 콜백의 프레임 내 재레이아웃 루프 차단 + 잔여 합성 오류 억제.

## 2. 변경 경계

- viewport-manager 한 파일: 콜백 변이를 매크로태스크로 프레임 경계 너머로 이연, 전용 window error 리스너로 해당 메시지만 stopImmediatePropagation.
- 다른 리스너 등록 시점(부트 초기 수집기)은 영향 밖 — 진단 수집기 잔존 가능성을 기록.

## 검증 게이트

- 레이어 자체 회귀 테스트 + `cargo nextest run --cargo-profile release-test` 전체
- fmt / clippy(root+workspace all-targets) / wasm32 check / Native Skia 3종 / studio suite

원격 push, PR 생성, 이슈 comment·close는 별도 승인 전 수행하지 않는다.
