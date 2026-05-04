# Task #565 구현 계획서 — `layout_inline_table_paragraph` 인라인 수식 미처리 정정

- **이슈**: [#565](https://github.com/edwardkim/rhwp/issues/565)
- **단계**: Stage 2 (구현 계획서, **본질 결함 식별 완료**)
- **작성일**: 2026-05-04

## 1. 본질 결함 (Stage 2 디버그 결과)

### 1.1 결함 위치

`src/renderer/layout.rs::layout_column_item` (L2056) 의 `PageItem::FullParagraph` 분기에서:

```rust
let has_inline_tables = para.controls.iter()
    .any(|c| matches!(c, Control::Table(t) if t.common.treat_as_char
        && is_tac_table_inline(t, seg_width, &para.text, &para.controls)));

if has_inline_tables {
    // → layout_inline_table_paragraph 호출 (인라인 표 + 텍스트 세그먼트만 처리)
} else {
    // → 일반 layout_paragraph (인라인 수식/그림/글상자 정상 처리)
}
```

### 1.2 결함 메커니즘

`layout_inline_table_paragraph` (`paragraph_layout.rs:88`) 는 **인라인 표(treat_as_char Table) + 텍스트 세그먼트만 배치**하고, **다른 인라인 컨트롤(Equation/Picture/Shape)은 처리하지 않음**.

12번 본문(0.61) 의 controls = `[표 1, 수식 9]` (모두 treat_as_char). 따라서:

1. `has_inline_tables = TRUE` → `layout_inline_table_paragraph` 진입
2. 9 개 인라인 수식은 무시 → `tree.set_inline_shape_position` 등록 안 됨
3. paginator 가 `PageItem::Shape pi=61 ci=1..9 (수식)` 을 별도로 등록함
4. `shape_layout::layout_shape_item` 진입 시 `inline_pos.is_none()` → fallback 경로 (L140-181):
   - `eq_x = col_area.x` (정렬=Justify 이지만 좌측 fallback)
   - `eq_y = para_y` (paragraph 시작 y)
5. 9 개 수식 모두 동일 좌표 (534.8, 1218.106) 에 겹쳐 그려짐 (실제 SVG 결과와 일치)

### 1.3 디버그 로그 증거 (제거됨)

```
[T565-DISPATCH] FullParagraph pi=60 y_offset=1040.19
[T565-DISPATCH]   pi=60 no special table → plain layout_paragraph
[T565-A] line=0 ci=1 script="rmA SIM D" x=656.80 ...   ← 정상 분산
...
[T565-DISPATCH] FullParagraph pi=61 y_offset=1130.32
[T565-DISPATCH]   pi=61 has_inline_tables=TRUE → layout_inline_table_paragraph
                                              ← 인라인 수식 처리 누락 (T565-A/B 둘 다 안 찍힘)
```

대조: 0.60 (12번 그림 문단, 인라인 표 없음) 은 plain layout_paragraph 정상 동작.

## 2. 정정 방향 비교 (3 안)

### 안 A — `has_inline_tables` 조건 강화 (분기 전환)

```rust
// 인라인 표 + 다른 인라인 컨트롤 (Equation/Picture/Shape) 가 같이 있으면
// 일반 layout_paragraph 로 보낸다 (인라인 표는 layout_paragraph 의 TAC 표
// 인라인 경로 L1888-1906 가 처리).
let has_inline_tables = para.controls.iter()
    .any(|c| matches!(c, Control::Table(t) if t.common.treat_as_char
        && is_tac_table_inline(...)));
let has_other_inline_ctrls = para.controls.iter()
    .any(|c| matches!(c,
        Control::Equation(_)
        | Control::Picture(p) if p.common.treat_as_char
        | Control::Shape(s) if s.common().treat_as_char
    ));

if has_inline_tables && !has_other_inline_ctrls {
    // 기존 specialized 경로
} else {
    // 일반 layout_paragraph
}
```

- **장점**: 정정 위치 한 곳, 영향 범위 명확.
- **위험**: 일반 layout_paragraph 의 인라인 표 처리 (L1886-1906) 가 기존 `layout_inline_table_paragraph` 와 동등한 출력을 내는지 확인 필요. 12번 표 (2x1 TAC) 는 한컴 정답지 대비 정정되어야 하지만, `layout_inline_table_paragraph` 가 처리하던 다른 케이스 (Task #517 등) 와 동등 출력 보장 필요.
- **대안**: 인라인 수식 + 인라인 표 동시 케이스 한정으로 우회 분기.

### 안 B — `layout_inline_table_paragraph` 에 인라인 수식 처리 추가

`layout_inline_table_paragraph` 의 segment 배치 루프 안에서, 표 사이 갭 위치에 등장하는 `Control::Equation` (treat_as_char) 도 함께 처리.

- **장점**: 기존 분기 구조 유지, specialized 경로 강화.
- **위험**: `layout_inline_table_paragraph` 의 char_offsets 갭 분석 + 표/텍스트 배치 로직과 인라인 수식 배치를 통합해야 함. 코드 복잡도 ↑. char_offsets 와 controls 의 매칭 로직이 layout_composed_paragraph 의 `tac_offsets_px` 와 다르므로 별도 산출 필요.
- **회귀 위험**: 기존 layout_inline_table_paragraph 사용 케이스 (Task #517 등) 의 byte-identical 보장 까다로움.

### 안 C — `shape_layout` fallback 좌표 정정

shape_layout fallback 경로 (`shape_layout.rs:140-181`) 가 9 개 수식이 같은 좌표를 받지 않도록 하단의 inline 위치를 추정하여 분산.

- **장점**: `layout.rs` 분기와 `layout_inline_table_paragraph` 무수정.
- **위험**: 추정 좌표는 실제 paragraph layout 결과와 다를 수밖에 없음. 본질 정정이 아님 (shape_layout 의 fallback 자체가 본 케이스 처리용으로 설계되지 않음). **메모리 `feedback_rule_not_heuristic` 에 정합 — 휴리스틱 fallback 회피**.

### 권장: 안 A

1. 본질 정정 (분기 조건의 정확한 케이스 분리).
2. 변경 범위 한 곳 (`layout.rs::layout_column_item` 의 `has_inline_tables` 가드).
3. `layout_paragraph` 의 인라인 표 처리 (L1886-1906) 와 인라인 수식 처리 (L1830-1885) 가 같은 줄/위치 체계 (run_tacs / inline_x) 를 공유하므로 자연스러운 통합.

**조건**: 안 A 적용 전 광범위 fixture sweep 으로 일반 layout_paragraph 의 인라인 표 처리가 기존 `layout_inline_table_paragraph` 사용 케이스와 동등 출력을 내는지 확인. **다르면 안 B 로 전환**.

## 3. 단계별 구현 (Stage 3 세부 단계)

### Stage 3-1 — 안 A 적용 가능성 검증 (no-op 변경 사전 확인)

1. `has_other_inline_ctrls` 조건 추가 후, 안 A 를 12번 본문 한정으로 강제 적용 (예: `para_index == 61`).
2. 12번 본문 SVG 산출 → 9 개 수식 정상 분산 + 인라인 표 위치 정상 + 표 셀 내부 텍스트 정상 표시 확인.
3. **표 위치/셀 내부 텍스트가 회귀하면 → 안 A 부적합 → Stage 3-2 안 B 로 전환**.

### Stage 3-2 — 정정 적용 (안 A 채택 시)

1. `layout.rs::layout_column_item` 의 `has_inline_tables` 가드를 `has_inline_tables && !has_other_inline_ctrls` 로 변경.
2. `has_other_inline_ctrls` 정의 (Equation + treat_as_char Picture/Shape).
3. (안 B 채택 시) `layout_inline_table_paragraph` 에 인라인 수식 + Picture/Shape 처리 통합.

### Stage 3-3 — 검증 (필수)

광범위 fixture sweep (인라인 수식 + 인라인 표 동시 사용 페이지 식별):

1. `cargo test --lib` 1113 통과 (현재 baseline)
2. `cargo clippy` 0 경고
3. `svg_snapshot` 6/6
4. `samples/exam_science.hwp` 4 페이지 — 12/15/18/19 번 본문 의도 정정 + 그 외 byte-identical
5. 다른 sample 들 byte-identical 확인:
   - `samples/exam_kor*.hwp`, `samples/issue-505-equations.hwp`
   - `samples/aift.hwp`, `samples/treatise sample.hwp`, `samples/2010-01-06.hwp` 등 포함
6. WASM 빌드 성공 + 크기 확인

### Stage 3-4 — 단계별 보고서 작성 + 승인 요청

`mydocs/working/task_m100_565_stage3.md` — 적용 안, 변경 LOC, 검증 결과, 회귀 0 입증.

### Stage 4 — 시각 판정 + 최종 보고

작업지시자 시각 판정 (SVG + rhwp-studio web Canvas) → `task_m100_565_report.md` + `orders/20260504.md` 갱신.

## 4. 회귀 위험 요소 (재정리)

| 위험 | 완화 |
|------|------|
| `layout_paragraph` 의 인라인 표 처리 vs `layout_inline_table_paragraph` 의 출력 차이 | Stage 3-1 사전 검증 + 광범위 fixture sweep |
| 인라인 표 + 인라인 그림/글상자 동시 케이스 (다른 fixture 가능) | `has_other_inline_ctrls` 에 모든 treat_as_char 컨트롤 포함 — 동일 분기로 통일 |
| Task #287 (display equation as own LINE_SEG) 회귀 | layout_paragraph 의 L2245 분기는 그대로 유지 |
| 다단/페이지 분할 회귀 | 안 A 는 분기 조건만 변경 — y/x 계산 변경 없음 |

## 5. 변경 LOC 추정

- 안 A: `layout.rs` ~5 LOC 추가 (조건절 1 곳)
- 안 B: `paragraph_layout.rs::layout_inline_table_paragraph` ~50 LOC 추가
- 안 C: 회피 (휴리스틱)

## 6. 승인 요청

본 구현 계획대로 **Stage 3 (구현 + 검증) 진입**을 승인 요청합니다.

진입 시점에 **Stage 3-1 (안 A 적용 가능성 사전 검증)** 부터 시작하며, 12번 본문에서 안 A 가 부적합으로 판명되면 안 B 로 전환합니다. 어느 안을 채택했는지는 Stage 3 완료 보고서에 명기합니다.
