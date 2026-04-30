# Task #470 Stage 2 완료 보고서

## 단계 목표

`src/renderer/typeset.rs:415, 439` 의 cross-paragraph vpos-reset 검출 조건을 완화하여 컬럼 헤더 오프셋 (cv != 0) 케이스를 처리.

## 1차 시도와 회귀

처음에는 단순히 `cv == 0 && pv > 5000` → `cv < pv && pv > 5000` 으로 변경. 21_언어_기출 케이스는 통과했으나 `tests/issue_418.rs` 가 실패.

### 회귀 원인

`samples/hwpspec.hwp` 단일 단 섹션 2 에서:
- pi=78 (block 표 anchor, RowBreak 분할) last vpos = 56052
- pi=79 (text) first vpos = 27872

partial-table split 의 LAYOUT 잔재로 cv=27872 < pv=56052 가 발생. HWP 인코딩상 의도된 컬럼/페이지 reset 이 아닌, 표 분할 후 후속 paragraph 의 자연스러운 위치. `cv < pv` 가 잘못 트리거 → pi=79 이후 image-bearing paragraphs 가 다음 페이지로 이동 → issue_418 검증 fail.

## 최종 수정

다단 섹션에서만 `cv < pv` 적용, 단일 단은 기존 `cv == 0` 보수적 검출 유지:

```rust
// L415 (cross-paragraph vpos-reset)
let trigger = if st.col_count > 1 {
    cv < pv && pv > 5000   // 다단: 컬럼 헤더 오프셋 (cv=9014 등) 도 인정
} else {
    cv == 0 && pv > 5000   // 단일 단: partial-split 잔재 회피
};
if trigger {
    st.advance_column_or_new_page();
}

// L439 (next-vpos-reset look-ahead, Task #359 단독 항목 페이지 차단)
let multi_col = st.col_count > 1;
matches!((next_first_vpos, curr_last_vpos), (Some(nv), Some(cl))
    if (if multi_col { nv < cl } else { nv == 0 }) && cl > 5000)
```

## 검증 결과

신규 테스트 + 기존 issue_418 통합 테스트 모두 통과.

전체 cargo test: **1071 + 1 + 6 + ... = 모두 PASS, FAIL 0**

다단 샘플 OVERFLOW 비교:

| 샘플 | BEFORE | AFTER | Δ |
|------|--------|-------|---|
| 21_언어_기출 | 13 | 10 | -3 ✓ |
| exam_kor | 19 | 19 | 0 |
| exam_eng | 11 | 11 | 0 |
| exam_math | 0 | 0 | 0 |
| exam_science | 5 | **0** | -5 ✓ |
| exam_social | 4 | **1** | -3 ✓ |
| hwpspec | 45 | 45 | 0 |

다른 다단 샘플의 OVERFLOW 추가 해소 (총 -11 건). hwpspec (단일 단) 회귀 0.

## 다음 단계

Stage 3 — pagination/engine.rs (RHWP_USE_PAGINATOR=1) 동기 보강 검토 + 최종 정리.
