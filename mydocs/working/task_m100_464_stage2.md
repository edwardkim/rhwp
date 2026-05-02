# Task #464 단계 2 보고서 — 베이스라인 + 구현 검증

**이슈**: #464
**브랜치**: `local/task464_v2`
**전제**: 단계 1 완료

---

## 1. 베이스라인 SVG

`/tmp/baseline_464/` 에 5종 다단 샘플 보존:

| 샘플 | 페이지 |
|---|---|
| exam_kor | 20 |
| exam_eng | 8 |
| exam_math | 20 |
| exam_science | 4 |
| exam_social | 4 |
| **합계** | **56 SVG** |

체크섬: `/tmp/baseline_464/checksums.txt`. 단계 4 회귀 검증에 사용.

## 2. typeset.rs 메서드 식별

`advance_column_or_new_page` (line 244-257) 가 정확히 필요한 동작:

```rust
fn advance_column_or_new_page(&mut self) {
    self.flush_column();
    if self.current_column + 1 < self.col_count {
        self.current_column += 1;
        self.current_height = self.pending_body_wide_top_reserve;
    } else {
        self.push_new_page();
    }
}
```

- 다음 단 있으면 → 단 이동
- 없으면 → 새 페이지 (`push_new_page` → `reset_for_new_page` 가 `current_column = 0`, `current_height = 0.0` 처리)

→ `typeset_multicolumn_paragraph` 의 라인 1975~1981 의 직접 처리를 `st.advance_column_or_new_page()` 호출로 단순화.

## 3. 수정 코드 (단계 3 적용)

### typeset.rs:904 가드 완화
```rust
let col_breaks = if st.col_count > 1 {
    Self::detect_column_breaks_in_paragraph(para)
} else {
    vec![0]
};
```

### typeset.rs:1975~1981 페이지 break 처리 추가
```rust
if bi + 1 < col_breaks.len() {
    st.advance_column_or_new_page();
}
```

기존 코드 (flush_column + 직접 column 변경 + height reset) 를 통합 메서드 호출로 대체.

## 4. pagination/engine.rs 동기화 확인 필요

```bash
grep -n "current_column == 0\|detect_column_breaks" src/renderer/pagination/engine.rs
```

단계 3에서 typeset.rs 변경 후 engine.rs 도 동일 패턴 검출 + 수정. 단계 4 에서 결합 검증.

## 5. 잠재 위험 — `reset_for_new_page` 의 zone_layout 초기화

`push_new_page` → `reset_for_new_page` 이 `current_zone_layout = None` 처리. 다단 layout 이 페이지 break 시 깨질 수 있음.

다만 일반 페이지 break (typeset 일반 흐름) 시에도 같은 동작을 거쳐 다단 페이지가 정상 작동한다 (10+ 페이지 다단 문서 정상). 페이지 break 후 layout 이 다단으로 복원되는 메커니즘이 별도로 존재함을 의미. 단계 3 회귀 검증으로 확인.

## 6. 단계 3 진입

자동승인 모드. 단계 3 (typeset.rs 수정 + 결함 검증) 즉시 진입.
