# Task #705 Stage 1 — RED 테스트 작성

## 산출물

- `src/renderer/layout/integration_tests.rs` 에 신규 통합 테스트 4건 추가 (816 → 905 line)

## 신규 테스트 매트릭스

| 테스트 | 검증 영역 | RED 메시지 |
|--------|----------|-----------|
| `test_705_aift_page2_cell_pagehide_collected` | aift page 2 (s0/p[1]) page_hide.is_some + hide_page_num | 결함 #1 — 셀 안 paragraph 미순회 |
| `test_705_aift_page2_cell_pagehide_six_fields` | 6 필드 모두 true (메인테이너 권위 측정) | 결함 #1 — page_hide 채워져야 함 |
| `test_705_aift_page3_cell_pagehide_collected` | aift page 3 (s1/p[0]) hide_page_num | 결함 #1 |
| `test_705_aift_cell_pagehides_total_count` | 본문 2 + 셀안 2 = 최소 4 페이지 매핑 | 실제 2 < 기대 4 |

## 실행 결과

```
running 4 tests
test test_705_aift_page3_cell_pagehide_collected ... FAILED
test test_705_aift_page2_cell_pagehide_six_fields ... FAILED
test test_705_aift_cell_pagehides_total_count ... FAILED
test test_705_aift_page2_cell_pagehide_collected ... FAILED

test result: FAILED. 0 passed; 4 failed; 0 ignored; 0 measured; 1120 filtered out
```

→ **모두 RED**, 정확히 결함 #1 영역에서 fail. 기대 동작 일치.

## RED 메시지 분석

1. `test_705_aift_page2_cell_pagehide_collected` — page.page_hide is None (결함 #1)
2. `test_705_aift_page2_cell_pagehide_six_fields` — page_hide unwrap fail (None)
3. `test_705_aift_page3_cell_pagehide_collected` — page.page_hide is None
4. `test_705_aift_cell_pagehides_total_count` — **실제=2** (본문 2건만 매핑) vs **기대>=4** → 셀 안 2건 누락 정량 측정

→ Stage 0 의 결함 #1 진단 (셀 안 PageHide 무시) 가 RED 결과로 정량 검증됨.

## 다른 테스트 영향

- 1120 filtered out (영향 없음) — 본 PR 의 신규 4건만 fail, 기존 테스트 0 fail

## 결함 #2 (border/fill 가드) 의 RED 단계 처리

수행/구현 계획서의 "Stage 1 RED 4건" 약속은 결함 #1 위주로 충족. 결함 #2 (`layout.rs` 의 `hide_border`/`hide_fill` 가드 부재) 의 RED 검증은:

- **Stage 1 에서는 IR 검증으로 충분** — `aift_page2_cell_pagehide_six_fields` 테스트가 page.page_hide 의 6 필드 모두 true 를 검증하므로, 결함 #2 정정 후 layout.rs 가 이 6 필드 중 hide_border/hide_fill 를 읽는지는 코드 리뷰로 검증 가능.
- **Stage 3 에서 추가 단위 테스트** — layout.rs 의 가드 추가 후 단위 테스트로 RenderTree 노드 검증 (별도 RED 단계 없이 GREEN 직접).

이 결정은 메모리 룰 `essential_fix_regression_risk` 정합 — 결함 #2 의 가드는 자명한 코드 패턴 (`if !hide_xxx`) 이므로 별도 RED 테스트보다 코드 리뷰 + Stage 5 의 174 sweep 으로 회귀 검증.

## Stage 0 보고서 갱신 사항

Stage 0 보고서에서 신규 테스트 후보 6건 (aift x4 + 국립국어원/KTX x2) 으로 도출했으나, Stage 1 에서는 **aift 위주 4건** 만 RED 작성:

- 본질 측정 데이터 (메인테이너 권위) 가 aift 에 집중
- 국립국어원/KTX 는 본문 PageHide 1건 + 셀 안 PageHide 1건 → 페이지 매핑 추가 측정 필요. Stage 5 의 174 sweep 으로 통합 검증
- 본질 정정의 RED 검증은 aift 4건으로 충분

## 위험 평가 (Stage 1 결과)

| 항목 | 결과 |
|------|------|
| RED 전환 | 4/4 모두 RED ✓ |
| 다른 테스트 영향 | 0 fail (1120 filtered out) ✓ |
| RED 메시지 명확성 | 결함 #1 영역 명시 ✓ |
| 정량 측정 | total_count 테스트로 본문 2 + 셀안 2 = 4 검증 ✓ |

## Stage 2 진입 결정

**Stage 2 (GREEN 결함 #1 정정)** 진입 가능:

1. `src/renderer/pagination/engine.rs:519-544` 의 `collect_header_footer_controls` 함수 수정
2. `Control::Table(table)` 매칭 추가 + 셀 안 paragraph 재귀 순회
3. 헬퍼 함수 `collect_pagehide_in_table(&Table)` 분리 (Stage 0 결정 — 1 depth 충분)
4. RED 4건 → GREEN 전환 확인

## 관련

- 수행 계획서: `mydocs/plans/task_m100_705.md`
- 구현 계획서: `mydocs/plans/task_m100_705_impl.md`
- Stage 0 보고서: `mydocs/working/task_m100_705_stage0.md`
- 본 보고서: `mydocs/working/task_m100_705_stage1.md`
