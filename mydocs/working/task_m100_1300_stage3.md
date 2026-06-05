# Task #1300 Stage 3: 회귀 검증 + 마무리

- 브랜치: `local/task1300` / 일자: 2026-06-05

## 전체 테스트

`cargo test` — **2037 passed, 0 failed**.

회귀 테스트 `test_superscript_tall_base_no_overshoot`(상단 정렬 핀):
- 위첨자 상단이 base 상단 위로 치솟지 않음(`sup.y >= base.y`).
- 합성 baseline = base 자연 baseline(이중 가산 없음).
- 짧은 base(`x^4`)·키 큰 base(`(1/6)^4`) 양쪽 검증.
- 기존 `test_superscript_layout`, `test_superscript_fraction_baseline`(#532) 통과.

## 정리

- studio 임시 캡처 스크립트 제거.
- 임시 프로브 → 정식 회귀 테스트 승격.
