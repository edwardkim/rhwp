# Task #775 Stage 2 보고서 — GREEN (옵션 A 가드 추가)

## 결과 요약

- ✅ `src/renderer/typeset.rs:1553-1568` 가드 조건 정밀화 (옵션 A)
- ✅ `tests/issue_775.rs::issue_775_exam_eng_p4_pi181_table_at_column_top` GREEN (회귀 해소)
- ✅ `tests/issue_703.rs::issue_703_calendar_year_single_page` GREEN (본 케이스 보존)

## 변경 내용

### `src/renderer/typeset.rs` (+5/-1)

```diff
                Control::Table(table) => {
                    // [Issue #703] 글앞으로 / 글뒤로 표는 Shape처럼 취급 — 본문 흐름 공간 차지 없음.
                    // pagination/engine.rs:976-981 와 동일 시멘틱: 데코레이션 표는 절대 좌표로 배치되며
                    // current_height 누적에 영향을 주지 않는다.
+                   //
+                   // [Issue #775] 단일 컬럼 한정. 다단(col_count>=2) 영역에서는 InFrontOfText/BehindText
+                   // 표라도 cur_h 누적이 컬럼 분배에 필요 (exam_eng.hwp p4 27번 보기 그림 위
+                   // 데코레이션 표 회귀 차단).
                    if matches!(
                        table.common.text_wrap,
                        crate::model::shape::TextWrap::InFrontOfText
                            | crate::model::shape::TextWrap::BehindText
-                   ) {
+                   ) && st.col_count == 1
+                   {
                        st.current_items.push(PageItem::Shape {
                            para_index: para_idx,
                            control_index: ctrl_idx,
                        });
                        continue;
                    }
```

## 검증

### Issue #775 RED → GREEN

```
$ cargo test --release --test issue_775
test issue_775_exam_eng_p4_pi181_table_at_column_top ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

bbox 정상 복원: `[277.08..700.10] (height=423.03)` (Stage 1 RED 시 `[723.69..1146.72]` → +446.61 px 시프트 해소)

### Issue #703 본 케이스 보존 검증

```
$ cargo test --release --test issue_703
test issue_703_calendar_year_single_page ... ok
test issue_703_tonghap_2010_11_single_page ... ignored (Issue #704)
test issue_703_tonghap_2011_10_single_page ... ignored (Issue #704)
test result: ok. 1 passed; 0 failed; 2 ignored
```

calendar_year.hwp 단일 페이지 정합 유지 → Task #703 fix 본 케이스(단일 컬럼 BehindText 표) 영향 0.

## 메커니즘

| 케이스 | column_count | 가드 매칭 | 동작 | 결과 |
|--------|--------------|-----------|------|------|
| calendar_year.hwp (Task #703 본 케이스) | 1 | ✅ | push-only | calendar_year 1 page 유지 |
| exam_eng.hwp p4 (Issue #775 회귀) | 2 | ❌ | 종전 cur_h 누적 | y=277 정상 복원 |

## Stage 3 진행 조건

- 본 단계 보고서 승인
- Stage 3: cargo test --release 전체 회귀 검증 (1250+ 테스트, 회귀 0)
