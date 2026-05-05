# Task #573 Stage 3 보고서 — 구현 + 검증

- **이슈**: [#573](https://github.com/edwardkim/issues/573)
- **브랜치**: `local/task573`
- **단계**: Stage 3 (구현 + 검증)
- **선행 산출**: Stage 1 (`task_m100_573_stage1.md`), Stage 2 (`task_m100_573_impl.md`)
- **작성일**: 2026-05-04

## 1. 변경 요약

`src/renderer/layout/table_layout.rs` 변경 (+19 / -2 LOC):

### 1.1 L1411 — 변수 추가 (`has_block_table_ctrl`)

```rust
let has_table_ctrl = para.controls.iter().any(|c| matches!(c, Control::Table(_)));
// [Task #573] inline TAC 표(treat_as_char=true) 와 block 표(treat_as_char=false) 를 분리.
// 인라인 TAC 표가 있는 셀 paragraph 의 surrounding text (예: "ㄷ. ", "이다.") 가
// layout_composed_paragraph 호출 미진입으로 미렌더되던 결함 정정.
let has_block_table_ctrl = para.controls.iter().any(|c|
    matches!(c, Control::Table(t) if !t.common.treat_as_char));
```

### 1.2 L1461 — IF 분기 조건 정정

```rust
if !has_block_table_ctrl {
    // 텍스트 + 인라인 수식 + 인라인 TAC 표 모두 layout_composed_paragraph 에서 렌더
    para_y = self.layout_composed_paragraph(...);
} else {
    // block table 만 있는 paragraph: 텍스트 흐름 외부 — 기존 ELSE 분기 유지
}
```

### 1.3 L1844 — inline TAC table 중복 가드

Equation 의 기존 가드 패턴 (L1800) 재사용:

```rust
if is_tac_table {
    // [Task #573] layout_composed_paragraph 의 run_tacs 가 인라인 TAC 표를 이미 렌더하고
    // set_inline_shape_position 등록했다면 중복 emit 방지 (Equation 의 L1800 가드와 동일 패턴).
    let already_rendered_inline = tree
        .get_inline_shape_position(section_index, cp_idx, ctrl_idx)
        .is_some();
    let tac_w = hwpunit_to_px(nested_table.common.width as i32, self.dpi);
    if already_rendered_inline {
        inline_x += tac_w;
    } else {
        // 기존 layout_table 호출 (변경 없음)
    }
}
```

### 1.4 L2040 `if has_table_ctrl` 보존

vpos 보정 분기는 `has_table_ctrl` (any table) 그대로 유지. block + inline TAC 모두 paragraph 의 lh 가 표 높이를 포함하므로 다음 paragraph vpos 보정 동일 필요.

## 2. 검증 결과

### 2.1 자동 테스트

| 테스트 | 결과 |
|--------|------|
| `cargo test --lib` | **1125 passed**, 0 failed, 2 ignored |
| `cargo test --test svg_snapshot` | **6/6 passed** |
| `cargo clippy --release --lib` | 본 변경 신규 경고 0 (사전 결함 2건 변경 전후 동일) |

### 2.2 의도된 정정 검증 — exam_science.hwp

**pi=68 cell[5] p[2] (page 3 13번 보기 ㄷ. 단락)**:

| 항목 | Before | After |
|------|--------|-------|
| "ㄷ" 위치 | **미렌더** | x=97.07 y=715.56 ✓ |
| "." 위치 | **미렌더** | x=111.15 y=715.56 ✓ |
| 분수 (cell-clip-175) x | 97.07 | **122.07** (ㄷ. 텍스트 다음 위치) |
| "이" 위치 | **미렌더** | x=347.29 y=715.56 ✓ |
| "다" 위치 | **미렌더** | x=361.38 y=715.56 ✓ |
| "." 위치 | **미렌더** | x=375.46 y=715.56 ✓ |

**완벽 정정**: surrounding text 가 모두 정상 렌더, 분수 위치는 "ㄷ. " 텍스트 폭(25 px)만큼 우측으로 조정.

**다른 영향 단락** (pi=75/82 등 동일 패턴 추정):
- 페이지 3 의 ㄷ glyph 다수 출현 (y=715.56, y=1219.55 등) — pi=68/75 모두 정상 렌더 확인
- 페이지 4 19번 보기 셀 동일 패턴 정정 추정 (Stage 4 시각 판정)

### 2.3 광범위 fixture sweep 결과

| Fixture | 페이지 | 변경 | 비고 |
|---------|------|------|------|
| `exam_science.hwp` | 4 | **4 페이지 모두 변경** | 의도된 정정 — Page 1 외곽 표 셀 헤더 sub-tables (item ① 자동 정정), Page 2 pi=61 (Task #568), Page 3/4 보기 셀 분수 단락 |
| `21_언어_기출_편집가능본.hwp` | 15 | 1 페이지 변경 | Page 1 헤더 sub-tables 위치 변동 (Justify slack 분배) — Stage 4 시각 판정 |
| `atop-equation-01.hwp` | 1 | byte-identical | ✓ 비회귀 |
| `equation-lim.hwp` | 1 | byte-identical | ✓ 비회귀 |
| `exam_eng.hwp` | 8 | 1 페이지 (page 4) | inline TAC 표 보유 셀 paragraph — 시각 판정 필요 |
| `exam_math.hwp` | 20 | byte-identical | ✓ 비회귀 |
| `exam_kor.hwp` | 20 | 1 페이지 (page 18) | 동일 패턴 — 시각 판정 |
| `biz_plan.hwp` | 6 | byte-identical | ✓ 비회귀 |
| `aift.hwp` | 77 | 5 페이지 (003, 031, 075-077) | 동일 패턴 — 시각 판정 |

**byte-identical 페이지 수**: 60 / 152 (39.5%) — 본 정정이 영향을 주는 paragraph 가 다수 존재

**의도된 정정 외 변경 페이지**: 8 페이지 (21_언어 1, exam_eng 1, exam_kor 1, aift 5). Stage 4 에서 시각 판정.

### 2.4 인접 효과 — Page 1 header item ① 자동 정정

작업지시자 보고 item ① ("성명/수험 번호/제 [ ] 선택" LEFT-shift) 가 **본 fix 로 자동 정정**:

```
exam_science page 1 header (외곽 1×1 표 셀 paragraph p[3] 의 sub-tables):
  Before: "성" x=86.39 (cell 좌단)
  After:  "성" x=152.39 (RIGHT shift +66 px, Justify slack 분배 결과)
```

이전 routing (step 3 for-ctrl loop) 은 `inline_x = inner_area.x + line_margin` 직배치 (slack 미고려). 새 routing (layout_composed_paragraph) 은 paragraph alignment + Justify slack 으로 sub-tables 를 분배 → cell halign=Center 의도에 가까운 결과.

**별도 issue #572 (item ①) 자동 close 가능 여부 — Stage 4 시각 판정 후 결정**.

## 3. 잔여 / 우려

### 3.1 21_언어_기출 page 1 헤더 변동
"성" sub-table x=339.12 → x=310.12 (LEFT shift 29 px). exam_science 와 반대 방향. 원인:
- 셀 paragraph text "    " (4 spaces) + sub-table 갯수 차이 → Justify slack 분배 결과 다름
- exam_science 는 11 spaces + 3 sub-tables, 21_언어 는 4 spaces + 2 sub-tables

작업지시자 시각 판정 필요 — 한컴 정답지 비교.

### 3.2 exam_eng/exam_kor/aift 영향
inline TAC 표 보유 셀 paragraph 의 routing 변경으로 위치 변동. 정확성 시각 판정 필요.

### 3.3 회귀 위험 — 메모리 정합

- `feedback_essential_fix_regression_risk`: 광범위 sweep 에서 60 페이지 byte-identical, 8 페이지 의도된 외 변경 — Stage 4 시각 판정 필수.
- `feedback_pdf_not_authoritative`: PDF 비교는 보조 ref. 한컴 2010/2020 환경 차이 점검 권고.
- `feedback_rule_not_heuristic`: HWP 표준 룰 (block table = 텍스트 흐름 외, inline TAC = 텍스트 흐름 내) 단일 룰 적용. 임계값/허용오차 미도입.

## 4. 변경 LOC

`src/renderer/layout/table_layout.rs`: **+19 / -2**

## 5. 산출물

- `mydocs/working/task_m100_573_stage3.md` — 본 보고서
- 변경 전 baseline: `/tmp/task573/sweep_before/`
- 변경 후 산출: `/tmp/task573/sweep_after/`

## 6. Stage 4 권고

작업지시자 시각 판정:
1. **의도된 정정** 검증 — exam_science page 3/4 13/15/16/19번 보기 셀 분수 단락
2. **인접 효과** 검증 — exam_science page 1 header (item ①) 자동 정정 효과
3. **회귀 검증** — 21_언어/exam_eng/exam_kor/aift 변경 페이지 8장
4. issue #572 (item ①) 자동 close 여부 결정

## 7. 승인 요청

본 Stage 3 검증 결과를 바탕으로 Stage 4 (시각 판정 + 최종 보고) 진입을 승인 요청합니다.
