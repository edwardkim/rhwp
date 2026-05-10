# Task #768 최종 결과 보고서

**제목**: shortcut.hwp 페이지 3 끝 column-break 행이 페이지 4 첫 줄로 밀림 (다단 zone 분할 결함)
**Issue**: [#768](https://github.com/edwardkim/rhwp/issues/768)
**브랜치**: `local/task768` (stream/devel 베이스)
**작업 기간**: 2026-05-10 (단일 세션)
**최종 상태**: ✅ closes #768

---

## 1. 결함 요약

`samples/basic/shortcut.hwp` 페이지 3 끝의 다단 영역에 있어야 할 pi=94 ("<편집 화면 분할에서>") + pi=95 ("화면 이동 Ctrl+W,N") 가 페이지 4 첫 줄로 밀림.

**PDF 권위 (한글 2022)**: 페이지 3 의 다단 영역 좌단 7행 + 우단 7행. 마지막 1행이 "<편집 화면 분할에서>" + "화면 이동". 페이지 4 시작은 입력 표 (pi=96 [다단나누기]).

**결과**: 본 정정 후 페이지 3 끝에 column-break 행 표시 (PDF 정합), 전체 페이지 수 8→7 (PDF 권위 7 정합).

## 2. Root cause 분석

### 분석 흐름 (3 단계 가설 → 한계 도달 → ColumnType 식별자 발견 → 본질 식별)

**가설 1** (정적 분석): typeset.rs:417 의 column-break + has_diff_col_def=false + 마지막 단 분기에서 push_new_page 강제. 페이지 잔여 공간 무시.

**시도 1-3** (단순 가드 추가): 무가드 wrap → vpos=0 가드 → vpos=0 + controls=0 가드. 모두 회귀 다수 (페이지 수 절반 수준). 본질 본 단순 가드로 구분 불가.

**한계 도달** (Stage 2 보고): shortcut.hwp pi=94 (wrap 의도) 와 21_언어_기출 pi=44 (push_new_page 의도) 가 IR 의 단순 필드 (column_type, vpos, controls) 만으로 안정적 구분 불가. 다단 분배 algorithm 의 깊은 재설계 필요로 판단, revert.

**ColumnType 식별자 발견** (Stage 3): 다단 유형 별 column-break 의 의미 식별:

- **Distribute (배분) / Parallel (병행) 다단**:
  - column-break = col div (zone) 마감 신호
  - 마지막 단에서 발생 시 같은 페이지에 새 col div 추가 (좌단/우단 추가행 시작)
  - 페이지 잔여 공간 부족 시 페이지 break 폴백
- **Normal (일반/신문형 = Newspaper) 다단**:
  - column-break = 일반 텍스트 연속과 등가
  - 다음 단으로 advance, 마지막 단이면 페이지 break

### 다단 유형별 ColumnDef 검증

| 샘플 | 다단 유형 | 본 정정 영향 |
|------|---------|-------------|
| `samples/basic/shortcut.hwp` | **배분 (Distribute)** | wrap-around (정정) |
| `samples/exam_math.hwp` | 일반 (Normal) | 영향 없음 |
| `samples/21_언어_기출_편집가능본.hwp` | 일반 (Normal) | 영향 없음 |

본 결함이 **Distribute 다단 단독 케이스**임이 정확히 식별됨.

## 3. 정정 (`src/renderer/typeset.rs:417-446`)

```rust
// 단 나누기
if para.column_type == ColumnBreakType::Column {
    if has_diff_col_def {
        // [Task #702] 단나누기 + 새 ColumnDef = zone 재정의 (MultiColumn 등가)
        self.process_multicolumn_break(...);
    } else if !st.current_items.is_empty() {
        // [Task #768] 다단 유형(ColumnType) 별 column-break 분기:
        // - Distribute/Parallel: col div 마감 신호 → wrap-around (zone 추가)
        // - Normal: 일반 텍스트 연속 등가 → 기존 동작 (advance / push_new_page)
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

`process_multicolumn_break` 의 zone 진입 로직과 동일하나 ColumnDef / col_count / layout / zone_column_type 은 그대로 유지 (MultiColumn 신호가 없으므로).

## 4. 검증 결과

### RED → GREEN

```
$ cargo test --test issue_768 -- --nocapture

[issue_768] pi=94 등장 페이지 인덱스 = 2 (page_count=7), PDF 권위 = 2
test issue_768_pi94_appears_on_page3_not_page4 ... ok
```

### 회귀 검증

```
$ cargo test --release
test result: ok. 1217 passed; 0 failed; 2 ignored;
```

→ **회귀 0**. test_539 / test_548 / test_exam_math_page_count 모두 PASS. 골든 SVG 7개 PASS.

### 광범위 (205 샘플)

| 메트릭 | Before | After | Δ |
|--------|--------|-------|---|
| 샘플 수 | 205 | 205 | — |
| `LAYOUT_OVERFLOW_DRAW` 총 | 226 | 225 | -1 |
| `LAYOUT_OVERFLOW` 총 | 354 | 355 | +1 |
| 페이지 수 변동 샘플 | — | — | **1 (shortcut.hwp 8→7)** |

샘플별 변경 (`diff before.tsv after.tsv`):
- `shortcut.hwp`: 페이지 8→7, DRAW 12→11 (-1, 정정), FLOW 13→14 (+1, trailing-ls)
- 그 외 204 샘플: 변동 없음

→ **shortcut.hwp 단독 영향**. Newspaper (일반) 다단 샘플 회귀 0.

### PDF 정합

| 샘플 | rhwp (after) | PDF |
|------|-------------|-----|
| shortcut.hwp | **7페이지** | **7페이지** ✓ |
| exam_math.hwp | 20페이지 | 20페이지 ✓ |

## 5. 영향 분석

### 본 정정의 효과

1. **Distribute (배분) 다단의 column-break wrap-around 활성화** — col div 마감 신호로 동작
2. **shortcut.hwp 페이지 3 끝 행 정합** — PDF 권위와 같이 같은 다단 영역에 추가
3. **페이지 수 정정** — 8→7 (PDF 정합)

### Newspaper 다단 보호

`is_distribute_or_parallel` 가드로 Newspaper (일반/신문형) 다단의 column-break 는 본 분기 진입 안 함 → `advance_column_or_new_page` 기존 동작 유지. exam_math.hwp / 21_언어_기출_편집가능본.hwp 등 영향 0.

### 페이지 잔여 검사 폴백

`advance_column_break_wrap_in_multi` 의 `MIN_WRAP_HEIGHT_PX = 13.3` 가드로 페이지 잔여 1 line 미만 시 push_new_page 폴백. 무한 wrap 또는 영역 음수 height 차단.

## 6. 단계별 산출물

| Stage | 커밋 | 산출물 |
|-------|------|--------|
| 0 | (계획서) | `plans/task_m100_768.md`, `task_m100_768_impl.md` |
| 1 (RED) | (RED) | `tests/issue_768.rs` + FAIL 확인 |
| 2 (분석) | (분석) | 정정 시도 3차 + 한계 도달 보고 + revert |
| 3 (GREEN) | (GREEN) | ColumnType 가드 + advance_column_break_wrap_in_multi + RED PASS |
| 4-5 (회귀+광범위) | (회귀) | cargo test 0 failed + 205 샘플 1건 정정 |
| 6 (최종) | (본 커밋) | 최종 보고서 + closes #768 |

## 7. PR 정보

- 브랜치 (origin push 예정): `pr-task768` (stream/devel 베이스)
- conflict 점검: `git merge-tree --write-tree origin/stream/devel...HEAD`

## 8. 학습 / 노트

### Stage 2 한계 도달 후 ColumnType 식별자 발견의 가치

정적 분석 + 단순 IR 필드 가드 (vpos, controls) 만으로는 column-break 의 의도 (wrap vs page break) 구분 불가. **다단 유형(ColumnType) 이 단독 식별자** 임을 발견 후 회귀 3건 → 0건 정정.

→ 깊은 결함은 IR 의 단일 필드 (column_type) 가 아닌 **컨텍스트 필드 (current_zone_column_type)** 의 조합이 본질을 결정. ParaShape / ColumnDef / ColumnType 등 분배 컨텍스트 전체를 고려하는 것이 정확한 정정의 지름길.

### Distribute / Parallel / Normal 의 column-break 의미

- Distribute / Parallel: col div 마감 신호 (paragraph-level 분배 마감)
- Normal: 일반 텍스트 연속과 등가 (paragraph-level 분배 동작)

이 차이는 ColumnDef.column_type 으로 구분되며, 본 task 의 가드 핵심.

### 한컴 다단 분배 algorithm 정합

본 정정은 `process_multicolumn_break` 의 zone 진입 로직을 ColumnDef 변경 없이 재사용. 다단 zone 추가 매커니즘 자체는 기존 코드 그대로, ColumnType 분기로 trigger 시점만 추가.

## 9. 관련 자료

- 수행 계획서: `mydocs/plans/task_m100_768.md`
- 구현 계획서: `mydocs/plans/task_m100_768_impl.md`
- Stage 보고서: `mydocs/working/task_m100_768_stage{1,2,3,4}.md`
- 회귀 테스트: `tests/issue_768.rs`
- 정정 위치: `src/renderer/typeset.rs:417-446`, `2316-2348`
- 관련 task: Task #321 (vpos-reset), Task #470 (다단 vpos-reset 완화), Task #702 (shortcut.hwp p2/p3 헤더 패턴)
