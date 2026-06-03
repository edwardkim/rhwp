# Task #470 Stage 3 완료 보고서

## 단계 목표

회귀 검증 완료 + pagination/engine.rs (RHWP_USE_PAGINATOR=1) 동기 보강 검토.

## 회귀 검증

### 전체 cargo test

```
test result: ok. 1071 passed; 0 failed; 1 ignored (lib unit tests)
test result: ok. 14 passed (exam_eng_multicolumn)
test result: ok. 25 passed (hwpx_roundtrip_integration)
test result: ok. 1 passed (hwpx_to_hwp_adapter)
test result: ok. 1 passed (issue_301)
test result: ok. 1 passed (issue_418)        ← 이전 시도에서 fail 했던 것 PASS
test result: ok. 2 passed (page_number_propagation)
test result: ok. 6 passed (svg_snapshot)
test result: ok. 1 passed (tab_cross_run)
```

총 **1122 PASS / 0 FAIL**.

### 다단 / 단일 단 샘플 OVERFLOW 비교

| 샘플 | 단 수 | BEFORE | AFTER | Δ |
|------|------|--------|-------|---|
| 21_언어_기출_편집가능본 | 다단 | 13 | 10 | -3 ✓ |
| exam_kor | 다단 | 19 | 19 | 0 |
| exam_eng | 다단 | 11 | 11 | 0 |
| exam_math | 다단 | 0 | 0 | 0 |
| exam_science | 다단 | 5 | **0** | -5 ✓ |
| exam_social | 다단 | 4 | **1** | -3 ✓ |
| hwpspec | 단일 단 | 45 | 45 | 0 |

다단 샘플에서 추가 OVERFLOW 11건 해소. hwpspec (단일 단, issue_418 시나리오) 회귀 0.

## pagination/engine.rs 검토

`src/renderer/pagination/engine.rs` 는 default 가 아닌 opt-in 경로 (`RHWP_USE_PAGINATOR=1`). 현재 cross-paragraph vpos-reset 검출 로직이 부재.

해당 엔진으로 21_언어_기출 페이지 1 dump-pages 실행 시 다른 형태의 split 문제 발견 (pi=9 가 col 0/col 1 로 분할됨). 본 task 범위와 다른 별도 이슈.

**결정**: 본 task 에서는 typeset.rs (default 엔진) 에만 적용. pagination/engine.rs 동기 보강은 별도 follow-up task 로 분리. 사용자가 `RHWP_USE_PAGINATOR=1` 옵션을 활성화하지 않는 한 영향 없음.

## 최종 변경 요약

`src/renderer/typeset.rs`:

1. **L415 cross-paragraph vpos-reset 검출** — 다단/단일 단 분기:
   - 다단: `cv < pv && pv > 5000` (Task #470)
   - 단일 단: `cv == 0 && pv > 5000` (Task #321 보수적 유지, issue_418 회귀 차단)

2. **L439 next-paragraph vpos-reset look-ahead** — 동일 패턴 적용:
   - 다단: `nv < cl && cl > 5000`
   - 단일 단: `nv == 0 && cl > 5000`

`src/renderer/layout/integration_tests.rs`:

3. 신규 통합 테스트 `test_470_cross_paragraph_vpos_reset_with_column_header_offset` 추가.

## 다음 단계

최종 보고서 작성 + orders 갱신 + commit + merge.
