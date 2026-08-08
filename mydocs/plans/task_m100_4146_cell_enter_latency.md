# 수행계획 — task_m100_4146_cell_enter_latency

- **이슈**: [#4146](https://github.com/edwardkim/rhwp/issues/4146)
- **브랜치**: `task_m100_4146_cell_enter_latency`
- **기준**: `upstream/devel` `e64c85312`
- **작성 시각**: 2026-08-08 KST

## 1. 목표

거대 분할 표 셀의 Enter(문단 분리) 후 전체 fragment 재조판으로 인한 지연(#4146)을 stale deferred pagination 의 무계산 취소와 split 완료 소유 이전으로 해소한다.

## 2. 변경 경계

- 3단계 리프트: 기준선 probe(테스트) → stale deferred pagination 무계산 취소(perf) → split 완료 소유를 command effects 선언으로 이전(fix) + 셀 Enter pagination 계약 e2e.
- pagination 결과 계약 불변 — e2e 가 실브라우저 검증 보고 포함.

## 검증 게이트

- 레이어 자체 회귀 테스트 + `cargo nextest run --cargo-profile release-test` 전체
- fmt / clippy(root+workspace all-targets) / wasm32 check / Native Skia 3종 / studio suite

원격 push, PR 생성, 이슈 comment·close는 별도 승인 전 수행하지 않는다.
