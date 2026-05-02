# Task #464 최종 보고서

**이슈**: [#464](https://github.com/edwardkim/rhwp/issues/464) — exam_kor 15p 우측 단 본문 overflow (pi=103-105 col 1 LINE_SEG vpos-reset 미인식)
**관련**: #459 (col 0 → col 1 fix, 본 task 의 선행)
**브랜치**: `local/task464_v2`
**마일스톤**: M100 (v1.0.0)
**완료일**: 2026-05-01

---

## 1. 결과

### 결함 해결
- `samples/exam_kor.hwp` 페이지 15 우측 단 (col 1) 의 LAYOUT_OVERFLOW 16건 → **0건**
- pi=103 paragraph 내 vpos-reset (line 3) 을 정상 인식하여 lines 0..3 은 page 15 col 1, lines 3.. + pi=104, pi=105 는 page 16 col 0 에 배치
- PDF 정답과 일치

### 회귀
- **0건**. 5종 다단 샘플 56 SVG 비교 — 변경 2건 (exam_kor 15p, 16p, 의도된 결함 수정) 외 회귀 없음.

### 페이지 수
- exam_kor.hwp: 20 페이지 유지 (변동 0)

### 테스트
- `cargo test --release`: 1094 + 다수 통합 테스트 모두 통과
- `cargo clippy`: 본 task 변경부 경고 0

## 2. 결함 본질

### 결함 위치 — `typeset.rs:904`, `engine.rs:609`

```rust
let col_breaks = if st.col_count > 1 && st.current_column == 0 {
    Self::detect_column_breaks_in_paragraph(para)
} else {
    vec![0]
};
```

`current_column == 0` 가드가 col 0 만 col_break 감지 → col 1 (마지막 단) 의 paragraph 내 LINE_SEG vpos-reset 무시 → typeset_multicolumn_paragraph 미호출 → 모든 lines 가 col 1 에 누적 → overflow.

### `typeset_multicolumn_paragraph` 의 페이지 break 누락 — `typeset.rs:1975~1981`

```rust
if bi + 1 < col_breaks.len() {
    st.flush_column();
    if st.current_column + 1 < st.col_count {
        st.current_column += 1;
        st.current_height = 0.0;
    }
    // ELSE 누락: 마지막 단에서 col_break 발생 시 페이지 break 처리 안 됨
}
```

가드를 제거해도 col 1 에서 col_break 시 페이지 break 트리거가 누락되어 효과 없음.

### Task #459 와의 관계

Task #459 가 `on_first_multicolumn_page` 가드 제거 → col 0 → col 1 단 경계 정정. 그러나 `current_column == 0` 가드는 그대로 → col 1 → 다음 페이지 단 경계 미해소.

본 task 는 #459 의 잔존 가드 제거 + 페이지 break 처리 추가.

## 3. 수정

### 3.1 `typeset.rs:904` — 가드 완화
```rust
let col_breaks = if st.col_count > 1 {
    Self::detect_column_breaks_in_paragraph(para)
} else {
    vec![0]
};
```

### 3.2 `typeset.rs:1975~1981` — 페이지 break 처리 통합
```rust
if bi + 1 < col_breaks.len() {
    st.advance_column_or_new_page();
}
```

기존 직접 처리 (flush_column + col 변경 + height reset) 를 `advance_column_or_new_page` 호출로 대체. 이 메서드는 다음 단이 있으면 단 이동, 없으면 `push_new_page()` 호출 (`reset_for_new_page` 가 `current_column = 0`, `current_height = 0.0` 처리).

### 3.3 `engine.rs:609` — 동기 가드 완화
같은 패턴. `paginate_multicolumn_paragraph:929` 가 이미 `advance_column_or_new_page` 사용하므로 추가 수정 불필요.

## 4. 진행 경로

### 단계 1 — 원인 정확 진단 (`task_m100_464_stage1.md`)
- LAYOUT_OVERFLOW 로그로 결함 재현
- dump-pages 분석으로 vpos-reset 인코딩 확인 (pi=103 line 3 reset)
- 결함 코드 위치 식별

### 단계 2 — 베이스라인 + 구현 검증 (`task_m100_464_stage2.md`)
- 5종 56 SVG 베이스라인 보존
- `advance_column_or_new_page` 메서드 식별 (단계 3 적용 메서드)

### 단계 3 — typeset.rs + engine.rs 수정 + 검증 (`task_m100_464_stage3.md`)
- typeset.rs 가드 완화 + 페이지 break 통합
- engine.rs 가드 완화
- LAYOUT_OVERFLOW 16 → 0
- 56 SVG 회귀 0
- cargo test 통과

### 단계 4 — 최종 정리 (본 보고서)
- clippy 검증
- 최종 보고서 + orders 갱신
- 커밋 + merge + push + Issue close

## 5. 변경 통계

| 파일 | 변경 |
|---|---|
| `src/renderer/typeset.rs` | +9 -7 |
| `src/renderer/pagination/engine.rs` | +4 -1 |

## 6. 검증 체크리스트

- [x] exam_kor p15 LAYOUT_OVERFLOW 16 → 0
- [x] dump-pages 페이지 분리 정상 (15p → 16p)
- [x] 시각 검증 (페이지 16 col 0 정상 layout)
- [x] 5종 다단 샘플 56 SVG 회귀 0
- [x] exam_kor.hwp 페이지 수 변동 0 (20 유지)
- [x] cargo test 통과
- [x] cargo clippy 본 task 변경부 경고 0

## 7. 산출물

- 수행계획서: `mydocs/plans/task_m100_464.md`
- 구현계획서: `mydocs/plans/task_m100_464_impl.md`
- 단계 보고서: `mydocs/working/task_m100_464_stage{1,2,3}.md`
- 최종 보고서: 본 문서
- orders 갱신: `mydocs/orders/20260501.md` (별도 파일 생성)

## 8. 핵심 룰 정합 (`feedback_rule_not_heuristic.md`)

본 결함은 휴리스틱이 아닌 **룰 변경**:
- HWP 표준: paragraph 내 LINE_SEG.vpos 가 감소(reset) 하면 단/페이지 경계
- 룰: `col_count > 1` 이면 vpos-reset 항상 감지, col 자리에 따라 다음 단 또는 페이지 break

가드 (`current_column == 0`) 는 휴리스틱이었음. 룰 정합으로 단순화.
