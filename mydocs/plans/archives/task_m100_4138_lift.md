# 수행계획 — task_m100_4138_lift

- **이슈**: [#4138](https://github.com/edwardkim/rhwp/issues/4138)
- **브랜치**: `task_m100_4138_stale_lineseg_rewrap`
- **기준**: `upstream/devel` `e64c85312`
- **작성 시각**: 2026-08-08 KST

## 1. 목표

셀 나누기 뒤 원본 셀에 남는 stale line_segs(옛 폭 기준)가 셀 경계 절단 렌더와 vpos 사다리 붕괴를
일으키는 결함(#4138)을 재래핑 + 사다리 단조 재구축으로 고친다.

## 2. 변경 경계

- 셀 나누기 경로에서 영향 셀 문단의 line_segs 를 현재 폭 기준으로 재래핑하고 vpos 사다리를
  단조 재구축한다. 최종 보고서 동반(mydocs/report/task_m100_4138_report.md) — 분할 전후 시각
  증적은 CONTRIBUTING 규약에 따라 PR 본문 첨부.

## 검증 게이트

- 레이어 회귀 테스트 + `cargo nextest run --cargo-profile release-test` 전체
- fmt / clippy(root+workspace all-targets) / wasm32 check / Native Skia 3종 / studio suite

원격 push, PR 생성, 이슈 comment·close는 별도 승인 전 수행하지 않는다.
