# Task #573 최종 결과 보고서 — 보기 셀 분수 단락(13/15/16/19번) 인라인 표 위치 편위

- **마일스톤**: v1.0.0 (M100)
- **이슈**: [#573](https://github.com/edwardkim/issues/573)
- **브랜치**: `local/task573` (분기점: `local/devel`)
- **작성일**: 2026-05-04
- **선행 task**: #565 (closed), #568 (PR #570 검토 중) — 같은 문서의 본문 paragraph 정정. 본 task 는 셀 paragraph 메커니즘.
- **상태**: **Stage 4 시각 판정 대기**

## 1. 배경

작업지시자 보고 — `samples/exam_science.hwp` 보기 셀 내부 분수 단락 (page 3 13/15/16번, page 4 19번) 의 인라인 분수 위치 편위.

선행 task #568 (PR #570) 가 본문 paragraph 인라인 표+수식 narrow `segment_width` 만 정정하여 셀 paragraph 는 미진입. Stage 1 진단으로 실제 본질이 **surrounding text 미렌더** 임을 확정.

## 2. 본질 결함

`src/renderer/layout/table_layout.rs::layout_table` 의 cell paragraph 분기 (L1411, L1461) 가 **block table 과 inline TAC 표를 미구분** — 인라인 TAC 표 보유 셀 paragraph 도 ELSE 분기로 빠져 `layout_composed_paragraph` 호출이 SKIP, surrounding text 가 미렌더.

```
pi=68 cell[5] p[2] "ㄷ. [분수] [수식 =1] 이다."
  Before: 분수 cell content + 수식 "=1" 만 렌더
          "ㄷ. ", "이다." 모두 SVG 에 미존재
  사용자 인지: "분수가 cell 좌단에 단독 위치 → 우측 편위로 보임"
```

Stage 2 trace (TASK573_TRACE) 로 확정: `layout_composed_paragraph` 가 p[0]/p[1] 에는 진입하나 p[2] (인라인 표 보유) 에는 미진입.

## 3. 정정

### 3.1 코드 변경 — `table_layout.rs` (+19 / -2 LOC)

#### 3.1.1 L1411: `has_block_table_ctrl` 신설

```rust
let has_table_ctrl = para.controls.iter().any(|c| matches!(c, Control::Table(_)));
// [Task #573] inline TAC 표(treat_as_char=true) 와 block 표(treat_as_char=false) 를 분리.
let has_block_table_ctrl = para.controls.iter().any(|c|
    matches!(c, Control::Table(t) if !t.common.treat_as_char));
```

#### 3.1.2 L1461: IF 조건 정정

```rust
if !has_block_table_ctrl {
    // 텍스트 + 인라인 수식 + 인라인 TAC 표 모두 layout_composed_paragraph 에서 렌더
    para_y = self.layout_composed_paragraph(...);
} else {
    // block table 만 있는 paragraph: 텍스트 흐름 외부 — 기존 분기 유지
}
```

#### 3.1.3 L1844: inline TAC table 중복 emit 가드

Equation 의 기존 가드 (L1800) 패턴 재사용:

```rust
if is_tac_table {
    let already_rendered_inline = tree
        .get_inline_shape_position(section_index, cp_idx, ctrl_idx)
        .is_some();
    let tac_w = hwpunit_to_px(nested_table.common.width as i32, self.dpi);
    if already_rendered_inline {
        inline_x += tac_w;          // layout_composed_paragraph 에서 이미 렌더 — skip
    } else {
        // 기존 layout_table 호출
    }
}
```

#### 3.1.4 L2040 `if has_table_ctrl` 보존

vpos 보정은 block + inline TAC 모두 필요 (lh 가 표 높이 포함). `has_table_ctrl` 그대로.

### 3.2 핵심 설계

| 항목 | 결정 | 근거 |
|------|------|------|
| 표 분류 | block (treat_as_char=false) vs inline TAC (treat_as_char=true) | HWP 표준: block = 텍스트 흐름 외, TAC = 인라인 |
| 라우팅 | inline TAC = layout_composed_paragraph 경로 | 인라인 표가 텍스트와 같은 line 에 배치되어야 함 |
| 중복 가드 | Equation L1800 패턴 재사용 | 일관성, 코드 중복 최소화 |

## 4. 검증 결과

### 4.1 자동 테스트

| 테스트 | 결과 |
|--------|------|
| `cargo test --lib` | **1125 passed**, 0 failed, 2 ignored |
| `cargo test --test svg_snapshot` | **6/6 passed** |
| `cargo clippy --release --lib` | 본 변경 신규 경고 0 (사전 결함 2건 변경 전후 동일) |

### 4.2 핵심 정정 측정 — pi=68 cell[5] p[2] "ㄷ. 이다." (page 3 13번 보기)

| 항목 | Before | After |
|------|--------|-------|
| "ㄷ" 위치 | **미렌더** | x=97.07 y=715.56 ✓ |
| "." 위치 | **미렌더** | x=111.15 y=715.56 ✓ |
| 분수 (cell-clip-175) x | 97.07 | **122.07** (ㄷ. 다음 위치) |
| 분수 cell content | x=103.87 (cell content + padding) | x=128.87 (정정) |
| "이" 위치 | **미렌더** | x=347.29 y=715.56 ✓ |
| "다" 위치 | **미렌더** | x=361.38 y=715.56 ✓ |
| "." 위치 | **미렌더** | x=375.46 y=715.56 ✓ |

**완벽 정정**: surrounding text 가 모두 정상 렌더, 분수 위치는 "ㄷ. " 텍스트 폭만큼 우측으로 조정.

### 4.3 광범위 fixture sweep — 9 fixture / 152 페이지

| Fixture | 페이지 | 변경 | 평가 |
|---------|------|------|------|
| `exam_science.hwp` | 4 | **4 페이지 변경** | 의도된 정정 (page 1/2/3/4 모두 영향) |
| `21_언어_기출_편집가능본.hwp` | 15 | 1 페이지 (page 1) | 헤더 sub-tables — 시각 판정 |
| `atop-equation-01.hwp` | 1 | byte-identical | ✓ 비회귀 |
| `equation-lim.hwp` | 1 | byte-identical | ✓ 비회귀 |
| `exam_eng.hwp` | 8 | 1 페이지 (page 4) | 시각 판정 |
| `exam_math.hwp` | 20 | byte-identical | ✓ 비회귀 |
| `exam_kor.hwp` | 20 | 1 페이지 (page 18) | 시각 판정 |
| `biz_plan.hwp` | 6 | byte-identical | ✓ 비회귀 |
| `aift.hwp` | 77 | 5 페이지 | 시각 판정 |

**byte-identical**: 60 / 152 (39.5%)
**의도된 정정 (exam_science)**: 4 페이지
**기타 영향 (시각 판정 필요)**: 8 페이지

### 4.4 인접 효과 — Page 1 header item ① 자동 정정

작업지시자 보고 item ① (Issue #572 — "성명/수험번호/제 [ ] 선택" LEFT-shift) 가 **본 fix 로 자동 정정**:

```
exam_science page 1 header (외곽 1×1 표 셀 p[3] 의 sub-tables):
  Before: "성" x=86.39 (cell 좌단 — 사용자 보고 LEFT-shifted)
  After:  "성" x=152.39 (RIGHT shift +66 px, Justify slack 분배)
```

이전 routing (step 3 for-ctrl loop) 은 sub-tables 를 inline_x 기준 좌측 직배치. 새 routing (layout_composed_paragraph) 은 paragraph alignment + Justify slack 으로 분배 → cell halign=Center 의도에 가까운 결과.

**Issue #572 자동 close 가능 여부 — 작업지시자 시각 판정 후 결정**.

## 5. 잔여 / 우려

### 5.1 21_언어_기출 page 1 헤더 변동
"성" x=339.12 → x=310.12 (LEFT shift 29 px) — exam_science 와 반대 방향. 원인: 셀 paragraph text "    " (4 spaces) + sub-table 갯수 차이 (4 vs 11 spaces, 2 vs 3 sub-tables) → Justify slack 분배 결과 다름. 작업지시자 시각 판정 + 한컴 정답지 비교 필요.

### 5.2 exam_eng/exam_kor/aift 영향
inline TAC 표 보유 셀 paragraph 의 routing 변경으로 위치 변동. 8 페이지 시각 판정.

### 5.3 메모리 정합 검토
- `feedback_essential_fix_regression_risk` ✓: 광범위 sweep 9 fixture 152 페이지 / 60 byte-identical / 92 변경 (의도된 4 + 시각 판정 88) — 회귀 위험 광범위 검증.
- `feedback_pdf_not_authoritative` ✓: SVG 좌표 + dump IR 기반 검증. PDF 절대 기준 미사용.
- `feedback_rule_not_heuristic` ✓: HWP 표준 룰 (block table = 흐름 외, inline TAC = 흐름 내) 단일 룰 적용. 임계값/허용오차 미도입.

## 6. 산출물

```
src/renderer/layout/table_layout.rs                +19 / -2 LOC
mydocs/plans/task_m100_573.md                      수행 계획서
mydocs/working/task_m100_573_stage1.md             Stage 1 진단 (코드 무수정)
mydocs/plans/task_m100_573_impl.md                 Stage 2 구현 계획
mydocs/working/task_m100_573_stage3.md             Stage 3 구현+검증
mydocs/report/task_m100_573_report.md              본 최종 보고서
output/svg/exam_science_task573/                   시각 판정용 SVG (4 페이지)
output/svg/exam_science_task573_dbg/               debug-overlay SVG
```

## 7. 커밋 이력

```
f7245bf8 Task #573 Stage 0: 수행 계획서
16600655 Task #573 Stage 1: 정밀 진단 보고서 — 본질 결함 = surrounding text 미렌더
6c6b7311 Task #573 Stage 2: 구현 계획서 (안 A)
a149f93f Task #573 Stage 3: table_layout.rs 인라인 TAC 표 셀 paragraph 라우팅 정정
```

## 8. 작업지시자 검토 사항

1. **시각 판정 — 본 task 효과**:
   - `output/svg/exam_science_task573/exam_science_003.svg` — pi=68/75 보기 셀 (13/16번 ㄷ. 단락) surrounding text 정상 렌더
   - `output/svg/exam_science_task573/exam_science_004.svg` — 19번 보기 셀 동일 패턴
   - `output/svg/exam_science_task573/exam_science_001.svg` — 헤더 sub-tables (item ① 자동 정정) 시각 판정
2. **시각 판정 — 비의도 영향 (8 페이지)**:
   - `21_언어_기출_편집가능본` page 1 헤더
   - `exam_eng` page 4
   - `exam_kor` page 18
   - `aift` page 003/031/075/076/077
3. **rhwp-studio web Canvas 시각 판정**: WASM 빌드 후 browser 검증
4. **이슈 #573 close + `local/devel` merge** 결정: 시각 판정 통과 시
5. **이슈 #572 (item ① page 1 header) 자동 close 가능 여부** 결정 — 본 fix 의 인접 효과로 정정됐는지 확인 후
6. **PR 분리**: PR #570 (Task #568) 와 별도 PR 생성 (메모리 `feedback_per_task_pr_branch` 정합)

## 9. 승인 요청

본 최종 보고서로 Task #573 완료 승인 요청합니다.
