# Task #362 Stage 2 — 결함 origin 확정 + 수정 방안

## Stage 1 의 후속 진단

Stage 1 에서 height_measurer 측정값이 v0.7.3 와 main 동일함을 확인. 결함 origin 은 layout 단계에 있음. Stage 2 에서 layout 단계의 차이를 정량 식별.

## 정량 진단

### 비교 대상
`samples/kps-ai.hwp` p56 의 외부 셀 [0,0] (pi=535, 1x1 TAC) 의 셀 안 첫 paragraph 의 y 좌표.

### 측정 결과

| 항목 | v0.7.3 | main | 차이 |
|---|---|---|---|
| 외부 셀 cell_y | 155.48 | 148.28 | **-7.2 px** |
| 외부 셀 pad_top | 1.88 | 1.88 | 0 |
| 외부 셀 안 p[0] y_start (실제) | 157.36 | 176.83 | **+19.47 px** |

### 산술 검증

main 의 첫 paragraph y_start = 148.28 + 1.88 + ? = 176.83 → **? = 26.67 px 추가**.

26.67 px 의 정체:
- 외부 셀 p[0] 의 LINE_SEG[0]: vpos=2000 HU = **26.67 px**
- 정확히 일치

## 결함 origin 확정

### 위치: `src/renderer/layout/table_layout.rs:1287-1308`

```rust
// Task #347: HWP는 LineSeg.vertical_pos에 첫 줄의 절대 위치(셀 내부 컨텐츠 상단부터)
// 를 기록한다. 이 값을 그대로 적용하면 모든 vertical_align (Top/Center/Bottom)에서
// PDF와 일치하는 텍스트 시작 y가 자동으로 결정됨 (mechanical_offset 불필요).
let first_line_vpos = cell.paragraphs.first()
    .and_then(|p| p.line_segs.first())
    .map(|ls| hwpunit_to_px(ls.vertical_pos, self.dpi));
let text_y_start = if let Some(vpos) = first_line_vpos.filter(|&v| v > 0.0) {
    cell_y + pad_top + vpos  // ← vpos 만큼 추가
} else {
    match effective_valign {
        VerticalAlign::Top => cell_y + pad_top,
        // ...
    }
};
```

**Task #347** 의 변경 — HWP 의 LineSeg.vertical_pos 를 "셀 내부 콘텐츠 상단으로부터의 첫 줄 top 오프셋" 으로 해석하고 `cell_y + pad_top + vpos` 를 첫 줄 시작 y 로 사용.

### 외부 셀 cell_y 도 -7.2 px 이동한 origin

cell_y 자체도 v0.7.3 (155.48) → main (148.28) 으로 -7.2 px (= -540 HU) 이동. 표 자체의 위치 (table_y) 차이 또는 row_y 차이일 가능성. Task #347 의 vpos 보정과 함께 적용된 동시 변경으로 추정.

### 누적 효과

- cell_y 차이: -7.2 px (셀이 위로 이동)
- vpos 추가 적용: +26.67 px (콘텐츠가 셀 안에서 아래로 이동)
- 합계: 셀 안 콘텐츠가 v0.7.3 대비 **+19.47 px 아래** → 외부 표의 clipPath 끝을 0.12 px 초과 → 클립

## 영향 범위 (작업지시자 단서)

작업지시자 시각 확인: 표 안에 표 + 문단 + 이미지 케이스 전반에서 조판 틀어짐.

→ **모든 셀이 첫 paragraph 의 LineSeg.vpos > 0 인 경우 영향**. 외부 표 안의 표/문단/이미지 케이스 광범위.

## 수정 방안

### 옵션 A — Task #347 의 vpos 적용을 외부 표 안의 표 케이스에서 제외

조건부 vpos 적용:
- 셀 안에 표가 있는 경우 (nested table) vpos 미적용 → cell_y + pad_top 부터 시작
- 그 외에는 현재 동작 유지

**장점**: Task #347 의 다른 케이스 (단순 텍스트 셀의 PDF 일치) 효과 유지.
**단점**: 한컴의 정확한 의미 불명확. nested 외 다른 케이스에 영향 가능.

### 옵션 B — Task #347 의 vpos 적용을 전면 제거 (v0.7.3 환원)

text_y_start = cell_y + pad_top (vpos 무시) → v0.7.3 와 완전 동일.

**장점**: v0.7.3 시각으로 회귀. 본 task 의 결함 + 작업지시자가 언급한 광범위 케이스 모두 정정.
**단점**: Task #347 의 PDF 일치 효과 (exam_eng 등) 손실 가능.

### 옵션 C — vpos 적용 + cell_y 보정 (한컴 의도 유지)

vpos 가 셀 콘텐츠 상단 오프셋이라면 cell_y 의 추가 보정 -7.2 px 도 같이 제거:
- cell_y 의 -7.2 px 보정 origin 식별 후 제거
- vpos 적용은 유지

**장점**: Task #347 의 시멘틱 정확하게 적용 가능 (PDF 일치).
**단점**: cell_y -7.2 px 의 origin 추적 추가 작업 필요. 더 깊은 회귀 분석.

## 권장 — 옵션 A

이유:
1. **본 task 의 직접 결함 정정**: 외부 표 안의 콘텐츠 클립 차단
2. **광범위 영향 가능성**: nested table 케이스만 제외하므로 부수효과 최소
3. **Task #347 의 다른 효과 유지**: exam_eng 의 좌표 정합 (Task #347 본래 목적) 영향 없음
4. **회귀 검증 가능**: nested 셀과 비-nested 셀의 vpos 동작 분리 명확

작업지시자가 언급한 "표 안에 표 + 문단 + 이미지" 케이스가 모두 nested 일 가능성 높음 — 같은 origin.

### 옵션 A 의 코드 변경 미리보기

```rust
// 셀 안에 nested table 존재 여부
let has_nested_table = cell.paragraphs.iter()
    .any(|p| p.controls.iter().any(|c| matches!(c, Control::Table(_))));

let first_line_vpos = cell.paragraphs.first()
    .and_then(|p| p.line_segs.first())
    .map(|ls| hwpunit_to_px(ls.vertical_pos, self.dpi));
let text_y_start = if has_nested_table {
    // nested table 셀: vpos 적용 안 함 (콘텐츠가 셀 높이 내에 fit 되도록)
    match effective_valign {
        VerticalAlign::Top => cell_y + pad_top,
        // ...
    }
} else if let Some(vpos) = first_line_vpos.filter(|&v| v > 0.0) {
    cell_y + pad_top + vpos
} else {
    // 기존 valign 분기
};
```

## 회귀 검증 항목 (Stage 3)

- kps-ai p56 외부 표 안 콘텐츠 정상 표시 (마지막 텍스트 y 가 v0.7.3 의 1001.18 와 일치)
- 7 핵심 샘플 + form-002 페이지 수 + LAYOUT_OVERFLOW 회귀 0
- Task #347 의 효과 유지 확인 (exam_eng p2 우측 박스 등)
- `cargo test --lib`: 1008 passed
- svg_snapshot: 6/6
- 작업지시자 시각 판정 — 표 안에 표/문단/이미지 케이스 정상화

## 다음 단계 (Stage 3)

1. 옵션 A 적용 (또는 작업지시자 결정)
2. 자동 회귀 (cargo test --lib, svg_snapshot, issue_301, clippy, wasm32)
3. kps-ai p56 SVG 의 마지막 텍스트 y 좌표 검증
4. 7 샘플 회귀 검증
5. WASM 빌드 + 작업지시자 시각 판정
