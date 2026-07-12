# PR #1934 검토 — Task #1920: pi-page 검증 도구 오탐 분리 + 72건 분류

- 작성일: 2026-07-05 / planet6897 → devel / 시간순 6/7 (05:49)

## 요지
소스 무접촉(도구 PR). `tools/verify_pi_page_vs_hangul.py`에 오탐 유형(쪽 하단 빈 문단
캐럿, PI_MISMATCH_CARET) 분리 추가 + 72건 전수 분류(NOW_MATCH 6 / EMPTY_CARET 6 /
REAL 60). REAL 60은 개별 결함이 아니라 stored-vs-재조판 드리프트(#1759/#1763/#1858 계열
캘리브레이션 프로그램)로 재분류 — 조사 가치가 큰 판별.

## 판단
머지 권고 (도구 정확성 개선, 페이지 오라클 게이트의 신뢰도 향상). #1920/#1921 이슈
처리는 분류 결과 기준 작업지시자 판단.

## 결합 게이트 결과 (devel `d982067b` + 7건 시간순 통합, 2026-07-05)

fmt 통과 / clippy 경고 0 / `cargo test --profile release-test --tests` **2,889 통과·실패 0**
(신규 핀 11건 포함) / OVR baseline 5샘플 **개체 회귀 0건** (기준 00014ecf)
