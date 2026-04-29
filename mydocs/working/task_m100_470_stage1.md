# Task #470 Stage 1 완료 보고서

## 단계 목표

cross-paragraph vpos-reset 미감지로 pi=10 이 col 0 에 잘못 배치되는 회귀를 검증하는 통합 테스트 추가 (red).

## 산출물

- `src/renderer/layout/integration_tests.rs::test_470_cross_paragraph_vpos_reset_with_column_header_offset`

## 테스트 시나리오

`samples/21_언어_기출_편집가능본.hwp` 페이지 1 의 `dump_page_items` 결과를 파싱하여:
- 단 0 블록에 `pi=10` 이 포함되면 안 됨
- 단 1 블록에 `pi=10` 이 등장해야 함

## 결과

**FAILED (의도된 red)** — col 0 에 `PartialParagraph pi=10 lines=0..2` 가 발견되어 fail. 분석 단계의 진단과 일치.

## 다음 단계

Stage 2 — `src/renderer/typeset.rs:415, 439` 의 cross-paragraph vpos-reset 검출 조건 완화.
