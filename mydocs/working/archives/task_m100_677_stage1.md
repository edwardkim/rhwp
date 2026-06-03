# Stage 1 — 진단 보고서 (Task #677)

## 결함 #1 — 본질 영역 확정 (정량 진단 완료)

### 임시 디버그 추적 결과

`paragraph_layout.rs::layout_paragraph_lines` 진입/종료에 임시 `eprintln!` 추가하여 pi=16 의 y 값 + ComposedLine 정보 추적:

```
[DBG677] PARA pi=16 start_line=1 end=2 composed.lines.len=2 y_in=1064.27
[DBG677]   cl[0] runs=1 char_start=0  line_height=1000  (LineSeg lh=1000  vpos=55288 th=1000)
[DBG677]   cl[1] runs=1 char_start=99 line_height=21296 (LineSeg lh=21296 vpos=56888 th=21296)
[DBG677] enter line_idx=1 y=1064.27 comp_lh=21296 seg_lh_hu=21296 runs=1 char_start=99
LAYOUT_OVERFLOW_DRAW: section=0 pi=16 line=1 y=1348.2 col_bottom=1084.7 overflow=263.5px
[DBG677] exit  line_idx=1 y=1357.81 line_height=283.95 max_fs=13.33 raw_lh=283.95
```

### 정량 분석

- pi=16 의 ComposedLine 은 cl[0] (line 0, lh=13.3px) + cl[1] (line 1, lh=283.95px) 2개
- PartialParagraph 는 **start_line=1, end=2 만 호출** — 즉 cl[1] 만 본 호출에서 처리
- **y_in = 1064.27px** (page 상대) 로 진입
- 기대 y = body_top + ls[1].vpos = 37.8 + (56888 / 7200 × 96) = 37.8 + 758.5 = **796.3px**
- **편위 = 1064.27 - 796.3 = 267.97px ≈ 표 높이 280.2px**

이 편위는 직전 `PageItem::Table` 처리에서 `y_offset` 이 표 바닥 (≈1055px) 까지 누적된 결과를 PP 가 그대로 상속받기 때문이다. HWP IR 에서 ls[1].lh=21296 (= 표 높이) 으로 line 1 자체가 표 높이만큼 차지하도록 인코딩되어 있는데, layout 은 표 높이를 (a) Table item 에서 한 번 + (b) PP item 에서 line_height 로 한 번 = **이중 누적**한다.

### 이론 기대값 (HWP IR 정합)

- pi=16 line 0 page-y: 37.8 + 55288/7200×96 = 37.8 + 737.18 = **774.98**
- pi=16 line 1 page-y: 37.8 + 56888/7200×96 = 37.8 + 758.51 = **796.31**
- pi=16 line 1 bottom: 796.31 + 21296/7200×96 = 796.31 + 283.95 = **1080.26**
- body_bottom: 37.8 + 1046.9 = **1084.70** ✅ (1080.26 < 1084.70 → fits)

표는 line 1 안에 인라인으로 위치 (table_y = line_y + baseline + om_bottom - table_h), 라인 높이 자체가 표 높이와 같으므로 표는 line 1 영역 안에 시각적으로 정합 배치되어야 한다.

### 코드 영역

**문제 영역**: `src/renderer/layout.rs:2082-2147` — `PageItem::PartialParagraph` 분기

```rust
PageItem::PartialParagraph { para_index, start_line, end_line } => {
    if let Some(para) = paragraphs.get(*para_index) {
        // ... 가드들 ...
        let comp = if *start_line == 0 { ... } else { composed.get(*para_index).cloned() };
        y_offset = self.layout_partial_paragraph(
            tree, col_node, para, comp.as_ref(), styles, col_area,
            y_offset,  // ← 이 y_offset 이 직전 Table item 의 누적값 (1055)
            *start_line, *end_line,
            ...
        );
    }
}
```

**참고**: `layout_table_item` (`layout.rs:2176-`) 의 마지막 TAC 분기에서 `clamped = line_end.min(table_y_end)` 로 y_offset 을 표 바닥으로 진행시키는데, 이 자체는 표 위치만 보면 합리적 (다음 paragraph 가 표 다음에 시작한다고 가정). 하지만 같은 paragraph 의 PP (line 1 = 표 인라인 라인) 가 별도 PageItem 으로 따라오는 경우 PP 의 y 가 LineSeg.vpos 정합 위치로 리셋되어야 한다.

### 본질 정정 영역 (Stage 2 작업 영역)

**핀셋 정정** — `layout.rs:2120-2133` 영역에 PP 의 y 리셋 로직 추가:

```rust
// [Issue #677] PP following TAC table in same paragraph: y_in 을 LineSeg.vpos 정합 위치로 리셋
let pp_y_in = if *start_line > 0
    && para.controls.iter().any(|c|
        matches!(c, Control::Table(t) if t.common.treat_as_char))
    && para_start_y.contains_key(para_index)
{
    if let (Some(seg), Some(seg0), Some(para_top)) = (
        para.line_segs.get(*start_line),
        para.line_segs.first(),
        para_start_y.get(para_index).copied(),
    ) {
        para_top + hwpunit_to_px(seg.vertical_pos - seg0.vertical_pos, self.dpi)
    } else { y_offset }
} else { y_offset };

let pp_y_out = self.layout_partial_paragraph(..., pp_y_in, ...);
y_offset = y_offset.max(pp_y_out);
```

**조건 가드**:
- `start_line > 0`: 문단 시작 PP 는 미적용
- `treat_as_char` Table 보유: 본 결함 영역 케이스만 좁힘
- `para_start_y` 등록 확인: Table item 이 선행 처리됐음을 확인 (같은 column 에 있음)

**y_offset 갱신**: `y_offset = y_offset.max(pp_y_out)` — Table item 의 누적값 (1055) 과 PP 의 자연 종료값 (1080) 중 최대값을 다음 item 으로 전파. 표 + 라인 모두 포함된 paragraph 영역의 시각 바닥을 정확히 반영.

## 결함 #2 — 본질 영역 확정

### 코드 추적 결과

- `src/model/image.rs:65,80` 에 `is_watermark()` / `watermark_preset()` helper 정의
- `src/paint/json.rs:321` (paint JSON 직렬화) 에서만 사용 — 메타정보용
- `src/renderer/svg.rs::ensure_brightness_contrast_filter` (line 1238-) 와 `src/renderer/web_canvas.rs::compose_image_filter` (line 94-) 는 **저장값 brightness/contrast 를 그대로 적용**, watermark 모드 미식별

### 정량 측정 (복학원서.hwp 워터마크)

- HWP IR: `effect=GrayScale, brightness=-50, contrast=70, watermark=custom`
- 한컴 표준 워터마크 프리셋 (`is_hancom_watermark_preset()`): `brightness=+70, contrast=-50` — **부호 반대**
- rhwp 적용 결과: brightness=-50 (어둡게) + contrast=70 (강대비) → 진한 어두운 회색
- PDF (정답): 흐릿한 회색 워터마크 (brightness 가 양수, contrast 가 음수에 가까운 효과)

### 가설 검증

- **가설 A (한컴 편집기는 워터마크 모드 시 저장값 변환 적용)** — 강한 정황증거
  - `is_hancom_watermark_preset()` 의 명세 (`brightness=70, contrast=-50`) 가 일반적인 "흐릿한 워터마크" 시각효과와 일치
  - 저장값이 -50/70 인데 PDF 가 흐린 워터마크로 출력 → 한컴이 저장값을 그대로 쓰지 않거나 변환 적용
- **가설 B (파서 부호 결함)** — 약함 (i8 부호확장은 표준)
- **가설 C (별도 alpha 필드)** — 추가 spec 확인 필요지만 본 결함은 brightness/contrast 부호만으로도 시각 결함 충분히 설명

### 본질 정정 영역 (Stage 3 작업 영역)

**가설 검증 + 정정 영역**:
1. 다른 워터마크 fixture (있다면) 와 한컴 PDF 출력 교차 검증 — 본 fixture 한정인지 광범위 영역인지 확정
2. `is_watermark() == true` + `effect == GrayScale` 케이스에서 한컴 정합 변환 적용:
   - **단순한 접근**: 워터마크 모드에서 stored brightness/contrast 의 **부호 inverse** 적용 (가설 A 의 단순 해석)
   - **보수적 접근**: 워터마크 모드에서 한컴 표준 프리셋 (brightness=+70, contrast=-50) 강제 적용
3. `src/renderer/svg.rs::ensure_brightness_contrast_filter` + `src/renderer/web_canvas.rs::compose_image_filter` 양쪽 동기 정정 (`feedback_image_renderer_paths_separate` 정합)

**선택 안**: Stage 3 시작 시 다른 fixture 교차 검증 결과로 결정. 단순한 부호 inverse 가 가장 좁은 정정. fixture 교차 결과 일관성 보이면 채택.

## BEFORE 측정값 (정량 baseline)

| 측정 항목 | BEFORE 값 |
|----------|----------|
| pi=16 LAYOUT_OVERFLOW 건수 | 3 |
| LAYOUT_OVERFLOW_DRAW pi=16 line=1 y | 1348.2 |
| LAYOUT_OVERFLOW pi=16 PartialParagraph y | 1357.8 |
| col_bottom (body bottom) | 1084.7 |
| overflow 크기 | 263.5 / 273.1 px |
| 워터마크 brightness/contrast 적용 후 | 어두운 진한 회색 (PDF: 흐린 회색) |
| cargo test --release --lib | 1155 passed (baseline 회귀 0 확인용) |

## 회귀 위험 영역 (Stage 2/3 사전 식별)

| 영역 | 위험 | 가드 |
|------|------|------|
| `layout.rs:2120` PP y reset | 다른 PP 케이스 (page break / cell 내부 등) 회귀 가능 | 조건 가드 (start_line>0 + para has TAC + para_start_y 등록) + 광범위 sweep |
| `svg.rs::ensure_brightness_contrast_filter` 변환 | 모든 brightness/contrast 적용 image 영향 | `is_watermark()` 게이트 + 다른 워터마크 fixture 교차 |
| `web_canvas.rs::compose_image_filter` 변환 | WASM 영역 동기 | 동일 가드 |

## 디버그 코드 제거 확인

`src/renderer/layout/paragraph_layout.rs` 의 임시 `eprintln!` (`[DBG677]` 태그) 4개 모두 제거 + clean build 통과.

## 승인 요청

본 Stage 1 진단 결과 승인 후 **Stage 2 (결함 #1 정정)** 진행하겠습니다.

정정 영역: `src/renderer/layout.rs:2120-2133` (`PageItem::PartialParagraph` 분기) — PP y 를 LineSeg.vpos 정합 위치로 리셋하는 핀셋 정정. 조건 가드 3개 + `y_offset.max(pp_y_out)` 누적 + 광범위 sweep 회귀 0 검증.
