# Task #768 Stage 3 (GREEN) 완료 보고서

**Issue**: [#768](https://github.com/edwardkim/rhwp/issues/768)
**Stage**: 3 — GREEN (수정 적용 + RED PASS + 회귀 0)
**작성일**: 2026-05-10

---

## 정정 요약

`src/renderer/typeset.rs:417-446` 의 column-break 분기에 **다단 유형 (ColumnType)** 가드 추가.

### 본질 식별

다단 유형(ColumnType) 별 column-break 의 의미:

- **Distribute (배분) / Parallel (병행) 다단**:
  - column-break = col div (zone) 마감 신호
  - 마지막 단에서 발생 시 같은 페이지에 새 col div 추가 (좌단/우단 추가행 시작)
  - 페이지 잔여 공간 부족 시 페이지 break 폴백
- **Normal (일반/신문형) 다단**:
  - column-break = 일반 텍스트 연속과 등가
  - 다음 단으로 advance, 마지막 단이면 페이지 break (기존 동작)

### 변경 코드

```rust
// 단 나누기
if para.column_type == ColumnBreakType::Column {
    if has_diff_col_def {
        // [Task #702] 단나누기 + 새 ColumnDef = zone 재정의 (MultiColumn 등가)
        self.process_multicolumn_break(...);
    } else if !st.current_items.is_empty() {
        // [Task #768] 다단 유형(ColumnType) 별 column-break 분기
        let is_last_col_in_multi = st.col_count > 1
            && st.current_column + 1 >= st.col_count;
        let is_distribute_or_parallel = matches!(
            st.current_zone_column_type,
            ColumnType::Distribute | ColumnType::Parallel
        );
        if is_last_col_in_multi && is_distribute_or_parallel {
            self.advance_column_break_wrap_in_multi(&mut st);
        } else {
            st.advance_column_or_new_page();
        }
    }
}
```

### 보조 함수 신규

```rust
/// [Task #768] Distribute/Parallel 다단의 column-break wrap-around.
fn advance_column_break_wrap_in_multi(&self, st: &mut TypesetState) {
    st.flush_column();
    let zone_used_max = st.pages.last()
        .map(|p| p.column_contents.iter()
            .filter(|cc| (cc.zone_y_offset - st.current_zone_y_offset).abs() < 0.1)
            .map(|cc| cc.used_height)
            .fold(0.0_f64, f64::max))
        .unwrap_or(0.0);
    let new_zone_y = st.current_zone_y_offset + zone_used_max;
    let body_height = st.layout.body_area.height;
    const MIN_WRAP_HEIGHT_PX: f64 = 13.3;
    if new_zone_y + MIN_WRAP_HEIGHT_PX > body_height {
        st.push_new_page();
    } else {
        st.current_zone_y_offset = new_zone_y;
        st.current_column = 0;
        st.current_height = 0.0;
    }
}
```

`process_multicolumn_break` 의 zone 진입 로직과 동일하나 ColumnDef / col_count / layout / zone_column_type 은 그대로 유지.

---

## RED 테스트 결과

```
$ cargo test --test issue_768 -- --nocapture

[issue_768] pi=94 등장 페이지 인덱스 = 2 (page_count=7), PDF 권위 = 2
test issue_768_pi94_appears_on_page3_not_page4 ... ok

test result: ok. 1 passed; 0 failed.
```

→ pi=94 등장 페이지 인덱스 3 → 2 (PDF 정합 ✓)

## 페이지 수 PDF 정합

| 샘플 | 다단 유형 | Before | After | PDF 권위 |
|------|---------|--------|-------|---------|
| `samples/basic/shortcut.hwp` | 배분 (Distribute) | 8 | **7** | **7** ✓ |
| `samples/exam_math.hwp` | 일반 (Normal) | 20 | **20** | **20** ✓ |
| `samples/21_언어_기출_편집가능본.hwp` | 일반 (Normal) | 15 | **15** | (변동 없음) ✓ |

## 회귀 검증

```
$ cargo test --release
test result: ok. 1217 passed; 0 failed; 2 ignored;
... (모든 통합/스냅샷/issue 테스트 PASS)
```

→ **회귀 0**. test_539 / test_548 / test_exam_math_page_count 모두 PASS.

## 영향 분석

### 본 정정의 효과

1. **Distribute (배분) 다단의 column-break wrap-around 활성화** — shortcut.hwp 페이지 3 의 "<편집 화면 분할에서>" 행이 PDF 권위와 같이 같은 다단 영역에 추가
2. **Normal (일반/신문형) 다단 동작 보존** — exam_math.hwp / 21_언어_기출_편집가능본.hwp 등 신문형 다단의 페이지 분배는 기존 동작 유지

### Distribute / Parallel / Normal 식별

ColumnDef.column_type (`src/model/page.rs::ColumnType`) 의 값을 통해 구분. Distribute = 배분(Balanced), Parallel = 병행, Normal = 일반(Newspaper).

본 정정 가드 `is_distribute_or_parallel` 는 Parallel 도 포함 (Parallel 다단도 column-break 가 col div 마감 신호로 동작).

### Newspaper 다단 보호

```rust
let is_distribute_or_parallel = matches!(
    st.current_zone_column_type,
    ColumnType::Distribute | ColumnType::Parallel
);
```

이 가드로 Newspaper 다단의 column-break 는 본 분기 진입 안 함 → `advance_column_or_new_page` 기존 동작 유지.

## 다음 단계 (Stage 4 — 회귀)

1. 골든 SVG 회귀 0 확인 (이미 cargo test 에 포함)
2. shortcut.hwp 페이지 3 SVG 시각 점검 (`output/svg/`)
3. 보고서 + 커밋

## 다음 단계 (Stage 5 — 광범위)

1. 169 샘플 페이지 수 비교
2. Distribute / Parallel 다단 보유 샘플 횡단 시각 검증
3. 보고서

## 승인 요청

Stage 3 GREEN 완료. RED PASS, 회귀 0, PDF 정합 확보. Stage 4/5 (회귀 + 광범위) 진입.
