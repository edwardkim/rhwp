# Task #495 구현 계획서

**이슈**: #495
**브랜치**: `local/task495`
**전제**: 단계 1 (원인 진단) 완료 — `mydocs/working/task_m100_495_stage1.md`

---

## 1. 결함 위치 (재확인)

`src/renderer/layout/table_layout.rs:1568~1647` 의 셀 paragraph 인라인 Shape 분기:

```rust
Control::Shape(shape) => {
    if shape.common().treat_as_char {
        let shape_w = ...;
        if let Some(&(tac_pos, _, _)) = composed.tac_controls.iter().find(|&&(_, _, ci)| ci == ctrl_idx) {
            // (B) Shape 앞 텍스트 추출 — composed.lines.first() 만 사용
            let text_before: String = composed.lines.first().map(|line| { ... }).unwrap_or_default();
            if !text_before.is_empty() {
                // (C) text_before 를 사각형 옆에 별도 TextRun 으로 발행
                ... cell_node.children.push(text_node);
            }
        }
        ... self.layout_cell_shape(...);
    } else { ... }
}
```

결함: 사각형이 두번째 이후 줄에 있을 때 `composed.lines.first()` 의 첫 줄 전체 텍스트가 text_before 로 추출되어 layout_composed_paragraph 결과와 중복 발행.

## 2. 비교 — Picture 분기

같은 함수의 `Control::Picture` 인라인 분기 (라인 1483~1530) 는 다음 로직을 갖는다:

```rust
let target_line = composed.tac_controls.iter()
    .find(|&&(_, _, ci)| ci == ctrl_idx)
    .map(|&(abs_pos, _, _)| {
        composed.lines.iter().enumerate().rev()
            .find(|(_, line)| abs_pos >= line.char_start)
            .map(|(li, _)| li).unwrap_or(0)
    }).unwrap_or(0);

if target_line > current_tac_line {
    // 줄 변경: inline_x 리셋, y 를 LINE_SEG vpos 기반으로 이동
    current_tac_line = target_line;
    inline_x = ...;
    if let Some(seg) = para.line_segs.get(target_line) {
        tac_img_y = para_y_before_compose + hwpunit_to_px(seg.vertical_pos, self.dpi);
    }
}
```

→ **Shape 분기에는 줄 추적 로직이 누락**. 이것이 본 결함의 근본 원인.

## 3. 수정 방향 — 단계 2 베이스라인 검증으로 결정

### 후보 (A) — Picture 와 정합: target_line 산출 + 줄별 text_before

Picture 분기와 동일 구조로 Shape 도 target_line 산출, 해당 줄 안에서만 text_before 추출.

장점: 디자인 일관성, Picture 분기에서 검증된 구조 재사용.
위험: 단일 줄 paragraph 케이스에서 동작 변화 확인 필요.

### 후보 (B) — 분기 자체 제거 (text_before 발행 삭제)

`layout_composed_paragraph` 가 이미 paragraph 의 모든 줄을 발행하므로 text_before 발행 자체가 중복일 가능성. 단계 1 진단 결과 첫 줄은 `layout_composed_paragraph` (line_idx=0) 로 발행되었고, 해당 발행이 사각형 자리(인라인 컨트롤 위치)를 비웠는지가 관건.

장점: 코드 단순화.
위험: 분기가 도입된 사유(예: layout_composed_paragraph 가 사각형 자리를 비우지 못하는 케이스)를 모르고 제거하면 회귀.

### 후보 (C) — 가드만 추가 (최소 변경)

`composed.lines.len() == 1 && tac_pos < line.char_start + first_line_chars` 일 때만 text_before 발행.

장점: 변경 최소, 회귀 위험 낮음.
단점: 다중 줄 + 사각형이 첫 줄에 있는 정상 케이스 처리 모호 — Picture 분기 수준의 정합성 미달.

### 결정 절차

단계 2 에서 후보 (B) 를 검증한다 — `composed.lines.first()` 의 텍스트가 layout_composed_paragraph 발행 결과와 동일한지 확인하여 **이 분기가 실제로 보강 역할을 하는지 / 단순 중복인지** 결정. 그 결과에 따라 (A)/(B)/(C) 중 채택.

## 4. 단계 구성

### 단계 2 (Stage 2) — 회귀 베이스라인 + 분기 필요성 검증

**목표**: 수정 전 상태의 SVG 베이스라인 수집 + Shape 인라인 분기의 의도 식별 + 수정 방향 (A)/(B)/(C) 확정.

작업:
1. 회귀 검증용 샘플 SVG 베이스라인 생성 (수정 전):
   - `samples/exam_science.hwp` 페이지 1, 2 (결함 재현 페이지 포함)
   - `samples/synam-001.hwp` (인라인 Shape 가능성 있는 표 다수)
   - `samples/k-water-rfp.hwp` (셀 안 그림/도형)
   - 기타 셀 안 인라인 사각형/도형 포함 샘플 (검색으로 식별)
2. `composed.lines.first()` 의 폭과 layout_composed_paragraph 가 ls[0] 에 발행한 글자 영역 폭 비교 — text_before 가 실제로 비어있는 자리 채우기인지 vs 단순 중복인지 식별
3. git history (table_layout.rs 1568 부근) 에서 본 분기 도입 커밋의 원래 의도 확인
4. (A)/(B)/(C) 중 채택 결정 + 근거 기록

산출물: `mydocs/working/task_m100_495_stage2.md`

### 단계 3 (Stage 3) — 코드 수정 + 결함 검증

**목표**: 단계 2 에서 결정한 방향으로 코드 수정 후 결함 재현이 사라졌는지 검증.

작업:
1. table_layout.rs Shape 분기 수정 (단계 2 결정 사항 반영)
2. `cargo build --release` 통과
3. 결함 검증:
   - exam_science.hwp 2페이지 SVG 재생성 → cell-clip-21 영역에서 동일 텍스트 중복 라인(y=224.26) 사라짐 확인
   - 박스 안 글자 분포가 PDF 정답(`samples/pdf/hwp2022/exam_science.pdf` 페이지 2) 과 정합
4. SVG 좌표 자동 검증 스크립트 (Stage 1 의 grep 패턴 활용)

산출물: `mydocs/working/task_m100_495_stage3.md`

### 단계 4 (Stage 4) — 회귀 검증 + 최종 정리

**목표**: 광범위 회귀 없음 검증 + 최종 보고서.

작업:
1. 단계 2 베이스라인과 수정 후 SVG 비교:
   - `samples/synam-001.hwp` (전체)
   - `samples/k-water-rfp.hwp` (전체)
   - 기타 식별된 회귀 위험 샘플
   - 차이 발생 시 의도된 변화인지 회귀인지 판정
2. `cargo test --release` 전체 통과
3. `cargo clippy --release` 경고 0
4. 최종 보고서 작성: `mydocs/report/task_m100_495_report.md`
5. `mydocs/orders/{yyyymmdd}.md` 갱신 (오늘 할일 상태 업데이트)

산출물: `mydocs/report/task_m100_495_report.md`, orders 갱신

## 5. 위험 관리

- **회귀 위험**: 메모리 `feedback_essential_fix_regression_risk.md` 에 따라 layout 변경은 회귀 위험 큼. 단계 2 의 베이스라인 + 단계 4 의 비교 검증 필수.
- **PDF 절대 기준 X**: `feedback_pdf_not_authoritative.md` — PDF 와의 시각 비교는 보조 기준. 핵심은 "동일 텍스트 중복 발행이 사라짐" + "회귀 없음".
- **룰 vs 휴리스틱**: `feedback_rule_not_heuristic.md` — Picture 분기와 Shape 분기는 동일 패턴(인라인 tac 컨트롤 처리)이므로 단일 룰(target_line 산출 + 줄별 처리) 로 정합. 후보 (A) 가 가장 룰-정합적.

## 6. 자동승인 진행 정책

작업지시자 "승인 자동승인" 지시에 따라 단계 2~4 의 _stageN.md 보고서는 작성과 동시에 다음 단계 진입. 다만 단계마다 결과를 명확히 사용자에게 보고하고, 회귀 위험이 식별되면 그 단계에서 진행 중단 후 별도 승인 요청.

## 7. 단계 2 즉시 착수

본 구현 계획서 작성 직후 단계 2 (회귀 베이스라인 + 분기 필요성 검증) 에 착수합니다.
