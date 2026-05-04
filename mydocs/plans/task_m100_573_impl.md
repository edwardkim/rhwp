# Task #573 구현 계획서 — surrounding text 미렌더 정정

- **이슈**: [#573](https://github.com/edwardkim/issues/573)
- **브랜치**: `local/task573`
- **단계**: Stage 2 (구현 계획)
- **선행 산출**: `mydocs/working/task_m100_573_stage1.md` (정밀 진단)
- **작성일**: 2026-05-04

## 1. Stage 1 결론 재진술 + Stage 2 코드 추적 결과

### 1.1 본질 결함 위치 (Stage 2 trace 로 확정)

`src/renderer/layout/table_layout.rs` L1411, L1461.

```rust
let has_table_ctrl = para.controls.iter().any(|c| matches!(c, Control::Table(_)));
... (생략) ...
if !has_table_ctrl {
    // 텍스트 + 인라인 수식 렌더 (layout_composed_paragraph 호출)
    para_y = self.layout_composed_paragraph(...);
} else {
    // has_table_ctrl: 표가 포함된 문단
    // (텍스트 렌더링 SKIP — 이것이 본질 결함)
}
```

**문제**: `has_table_ctrl` 가 **block table (treat_as_char=false)** 와 **inline TAC table (treat_as_char=true)** 를 구분하지 않음. 인라인 TAC 표가 있는 셀 paragraph 도 ELSE 분기로 빠져 surrounding text 렌더가 SKIP 됨.

### 1.2 Trace 검증 (TASK573_TRACE 임시 print)

`layout_composed_paragraph` 함수 진입 trace 결과:
- pi=68 cell[5] p[0] (ㄱ.) ✓ 진입 (controls=2 = picture + 수식, has_table_ctrl=false)
- pi=68 cell[5] p[1] (ㄴ.) ✓ 진입 (controls=1 = 수식만, has_table_ctrl=false)
- **pi=68 cell[5] p[2] (ㄷ.)** ✗ **미진입** (controls=2 = 표 + 수식, has_table_ctrl=true → SKIP)

p[2] 의 surrounding text "ㄷ. ", "이다." 가 layout_composed_paragraph 내 run_tacs loop 에 도달하지 못함 → 렌더 누락.

### 1.3 ELSE 분기의 inline TAC 표 처리 (별도 경로)

`table_layout.rs` L1844-1883: ELSE 분기와 무관하게 **모든 paragraph 의 controls** 를 순회하며 처리.

```rust
for (ctrl_idx, ctrl) in para.controls.iter().enumerate() {
    match ctrl {
        Control::Table(nested_table) => {
            if nested_table.common.treat_as_char {
                // 인라인 TAC 표 — layout_table 호출 (inline_x 기준 배치)
                let table_h = self.layout_table(...);
                inline_x += tac_w;
            }
        }
        ...
    }
}
```

→ 인라인 TAC 표 자체는 이 loop 에서 **렌더된다**. 결함은 surrounding text 만 누락.

### 1.4 Equation 의 skip 가드 (참고)

`table_layout.rs` L1800-1805:
```rust
let already_rendered_inline = tree
    .get_inline_shape_position(section_index, cp_idx, ctrl_idx)
    .is_some();
if has_text_in_para || already_rendered_inline {
    inline_x += eq_w;     // 이미 layout_composed_paragraph 에서 렌더됨 → skip
} else {
    // 직접 렌더링
}
```

Equation 은 `inline_shape_position` 등록 여부로 중복 emit 방지. **Table 에는 이 가드 없음** (L1844-1883).

## 2. 정정 안 비교

### 안 A — `has_table_ctrl` 조건 좁히기 + Table 중복 가드 추가

**변경**:
1. `table_layout.rs` L1411: `has_table_ctrl` 의미를 "block table 만" 으로 한정
   ```rust
   let has_block_table_ctrl = para.controls.iter().any(|c|
       matches!(c, Control::Table(t) if !t.common.treat_as_char));
   ```
2. L1461 의 if 조건을 `has_block_table_ctrl` 로 변경
3. L1844 의 inline TAC table branch 에 Equation 과 동일한 `inline_shape_position` 중복 가드 추가
   ```rust
   if is_tac_table {
       let already_rendered = tree
           .get_inline_shape_position(section_index, cp_idx, ctrl_idx)
           .is_some();
       if already_rendered {
           inline_x += hwpunit_to_px(nested_table.common.width as i32, self.dpi);
       } else {
           // 기존 layout_table 호출
       }
   }
   ```
4. 다른 `has_table_ctrl` 사용처 (L2040, L2151 등) 의 의미 점검 — **block table 만** 인지, **모든 table** 인지 결정

**기대 효과**:
- p[2] (TAC 표 + 수식만 보유) → has_block_table_ctrl=false → IF 분기 → layout_composed_paragraph 호출 → surrounding text 정상 렌더
- p[2] 의 인라인 TAC 표는 layout_composed_paragraph 의 run_tacs (paragraph_layout.rs:1888-1903) 에서 렌더 + `set_inline_shape_position` 등록
- ELSE 분기 (이제 block table 만 진입) 는 변경 없음
- Step 3 (for ctrl loop) 의 inline TAC 표 분기에 중복 가드 추가 → 이미 렌더됐으면 skip

**장점**: 
- 변경 면적 작음 (2 곳)
- Equation 의 기존 가드 패턴 재사용 (일관성)
- block table 만 있는 paragraph 는 동작 미변경 (회귀 차단)

**단점**: 
- 다른 `has_table_ctrl` 사용처의 의미 검토 필요 (L2040 의 vpos 보정 — TAC 표에도 적용해야 하는지)

### 안 B — ELSE 분기 내부에 surrounding text 렌더 추가

ELSE 분기 (L1501) 안에서 layout_composed_paragraph 의 텍스트 렌더 부분만 수동 호출. 인라인 TAC 표는 step 3 그대로.

**단점**: layout_composed_paragraph 의 일부만 추출해야 하므로 큰 변경. 코드 중복.

### 안 C — Step 3 (for-ctrl loop) 의 inline TAC 표 처리를 layout_composed_paragraph 로 통합

cell_layout 의 inline TAC 표 처리 로직 자체를 layout_composed_paragraph 로 이전. layout_table 의 인라인 호출 형식을 통합.

**단점**: 변경 면적 매우 큼. 회귀 위험 매우 큼.

→ **안 A 권고**.

## 3. Stage 3 실행 절차

1. baseline SVG 생성 (변경 전, 광범위 fixture)
2. `table_layout.rs` 변경:
   - L1411: `has_table_ctrl` → `has_block_table_ctrl` 의미 정정
   - L1461: 조건 변경
   - L1844 (inline TAC table branch): `inline_shape_position` 가드 추가
3. 다른 `has_table_ctrl` 사용처 (L2040, L2151, L2161, L2226, L2371, L1345) 검토 — block 만 인지 확인 후 필요 시 동시 변경
4. `cargo build --release` — 빌드 통과
5. p[2] (pi=68 cell[5]) SVG 좌표 측정 — surrounding text 정상 렌더 확인
6. 동일 패턴 검증 — pi=75/82 (page 3 16/15번), page 4 19번
7. `cargo test --lib`, `cargo clippy`, `cargo test --test svg_snapshot`
8. 광범위 fixture sweep (15+ fixture 280+ 페이지)
9. Stage 3 보고서 작성

## 4. 회귀 검증 범위

- **필수**: `exam_science.hwp` 의 13/15/16/19번 보기 셀 분수 단락 외 byte-identical
- **필수**: 셀 paragraph 에 block table (treat_as_char=false) 있는 fixture — ELSE 분기 동일 동작 보존 (inline TAC 표 경로는 변경)
- **필수**: 셀 paragraph 에 inline TAC 표만 (수식 없음) 보유 — text 렌더 동작 검증
- **필수**: 셀 paragraph 에 inline TAC 수식만 보유 — Task #565 회귀 차단 검증
- **필수**: `cargo test --lib` 1125+ 통과, `clippy` 신규 0, `svg_snapshot` 6/6
- **권고**: 한컴 2010/2020 PDF 비교 (보조 ref)

## 5. 위험 요소

- **메모리 `feedback_essential_fix_regression_risk`**: 셀 paragraph 의 inline TAC 표 처리 분기 정정. 광범위 sweep 필수.
- **L2040 `has_table_ctrl` 의 vpos 보정 분기**: 인라인 TAC 표 paragraph 도 vpos 보정이 필요한지 Stage 3 첫 빌드 시 확인. 필요 시 별도 변수 (`has_any_table_ctrl`) 로 분리.
- **layout_composed_paragraph 의 인라인 TAC 표 렌더 (paragraph_layout.rs:1888-1903)** 의 cell_ctx 처리 — 이 코드는 cell 안에서도 정상 동작해야 함 (이미 동작한다고 가정 — Stage 3 검증).

## 6. 변경 LOC 추정

| 파일 | 변경 | 추정 LOC |
|------|------|---------|
| `src/renderer/layout/table_layout.rs` | L1411 의미 정정 + L1844 가드 추가 + 다른 사용처 검토 | +15 / -3 |

## 7. 산출물 (Stage 3)

- `mydocs/working/task_m100_573_stage3.md` — 구현 + 검증 결과
- 코드 diff: `src/renderer/layout/table_layout.rs`
- (필요 시) sweep 결과 요약

## 8. 승인 요청

본 구현 계획대로 Stage 3 (구현 + 검증, 안 A) 진입을 승인 요청합니다.
