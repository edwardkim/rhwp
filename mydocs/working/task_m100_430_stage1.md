# Task M100 #430 Stage 1 완료 보고서 — 원인 정밀 조사

**브랜치**: `local/task430` ← `local/devel`

## 1. 조사 대상

`samples/exam_kor.hwp` 1쪽 상단 가운데 헤더 이미지(BinData bin_id=27).
SVG: "국어 영역(A 형)" / PDF: "국어 영역" (PDF 정답).

## 2. 조사 방법

`src/main.rs:1643` 의 표 셀 내부 그림 dump 형식에 `crop`, `orig`, `cur` 필드를 추가하여 `rhwp dump samples/exam_kor.hwp | grep "bin_id=27"` 로 실측.

## 3. 실측 결과

```
ctrl[0] 그림: bin_id=27, w=16378 h=4253 (57.8×15.0mm), tac=true,
        wrap=TopAndBottom, vert=Para(off=0), horz=Para(off=0),
        orig=174000×26580, cur=16378×4253,
        crop=(0,0,102366,26580)
```

| 필드 | 값(HWPUNIT) | px 환산(75 HU/px) |
|------|-------------|-------------------|
| original_width × original_height | 174000 × 26580 | 2320 × 354 (= 실제 JPEG 크기) |
| current(display) width × height | 16378 × 4253 | (57.8 × 15.0 mm 박스) |
| crop (left, top, right, bottom) | (0, 0, 102366, 26580) | (0, 0, 1365, 354) |

**해석**: HWP는 원본 2320×354 px 중 좌측 1365×354 px(가로 58.8%)만 표시하라고 명시. 이 영역은 "국어 영역"만 포함하고 "(A 형)" 부분은 잘린다.

## 4. 원인 — 시나리오 (A) 확정

HWP에 명시 crop 있음. 그러나 SVG/Canvas 렌더러 양쪽 모두 스케일 공식이 잘못돼 crop이 무효화된다.

### 4.1 버그 위치

| 파일 | 라인 | 동일 버그 |
|------|------|-----------|
| `src/renderer/svg.rs` | 1098-1109 | `scale_x = cr as f64 / img_w` |
| `src/renderer/web_canvas.rs` | 2048-2052 | 동일 |

### 4.2 잘못된 공식의 동작

```rust
let scale_x = cr as f64 / img_w;          // 102366 / 2320 = 44.12 HU/px
let src_w  = (cr - cl) as f64 / scale_x;  // 102366 / 44.12 = 2320 (= img_w 그대로!)
```

`cr` 는 crop 우경계의 HU 좌표인데, 마치 원본 이미지 우경계인 것처럼 다뤘다. 그 결과 어떤 crop 값이 와도 `src_w`가 항상 이미지 전체 폭(`img_w`)이 되어 crop이 무력화된다.

### 4.3 올바른 공식

원본 이미지 크기(HU)와 이미지 픽셀 크기 사이의 실제 변환비(HU/px)를 사용해야 한다.

```rust
let scale_x = original_width_hu  / img_w_px;   // 174000 / 2320 = 75 HU/px
let scale_y = original_height_hu / img_h_px;   //  26580 /  354 = 75.08 HU/px
let src_x = cl as f64 / scale_x;
let src_y = ct as f64 / scale_y;
let src_w = (cr - cl) as f64 / scale_x;        // 102366 / 75 = 1365 px
let src_h = (cb - ct) as f64 / scale_y;        //  26580 / 75 = 354  px
```

## 5. 데이터 흐름 점검

### 5.1 crop은 이미 전달되고 있음

`samples/exam_kor.hwp` 의 헤더 그림 경로:
- `table_layout.rs:1539, 1563` → `picture_footnote.rs::layout_picture` (line 21)
- `picture_footnote.rs:85-92` 에서 `crop` 추출, `ImageNode.crop` 으로 전달 ✓

### 5.2 original_size_hu 가 전달되지 않음

`render_tree.rs::ImageNode` 에는 `original_size: Option<(f64, f64)>` 만 있고, 그마저도:

- 주석은 "HWPUNIT 기반, SVG 좌표 변환 후"로 모호하지만,
- 실제 사용처(`web_canvas.rs:2066`, `svg.rs::render_positioned_image` 등)는 px(display) 단위로 사용.
- 모든 ImageNode 생성 사이트에서 default(`None`)로 둠 → 채워지지 않음.

따라서 crop 보정에 필요한 "원본 크기(HU)"가 렌더러에 도달하지 못한다.

### 5.3 ImageNode 생성 사이트 (참고)

| 파일:라인 | 용도 | crop 전달 | original_size_hu 필요 |
|-----------|------|-----------|----------------------|
| `picture_footnote.rs:98` | 일반 picture (본 케이스) | ✓ | **필요** |
| `picture_footnote.rs:295` | body picture | ✓ | **필요** |
| `layout.rs:2610` | TAC + 텍스트 없는 문단 | ✓ | **필요** |
| `paragraph_layout.rs:1700,1950,2033` | 인라인 TAC | ✗ (누락) | (옵션) |
| `shape_layout.rs:959,1133` | 그룹/도형 내 picture | ✗ | (옵션) |
| `table_layout.rs:308`, `table_cell_content.rs:635` | 셀 배경 이미지 채우기 | (해당 없음) | (해당 없음) |

본 이슈 수정 범위는 위 표의 "필요" 행만으로 충분. 인라인 TAC 등 누락 경로는 후속 이슈로 분리.

## 6. 수정 방침

`ImageNode` 에 `original_size_hu: Option<(i32, i32)>` 신규 필드를 추가하고, `pic.shape_attr.original_width/height` 값을 채워 보낸다. 렌더러는 이 값을 사용해 정확한 HU/px 스케일로 src 좌표를 계산.

세부 단계는 [`task_m100_430_impl.md`](./../plans/task_m100_430_impl.md) 참조.

## 7. 산출물

- (조사용 일시 변경) `src/main.rs:1643` dump 형식 보강 — 유지하기로 결정 (디버깅에 유용).
- 본 보고서.
- 구현 계획서: `mydocs/plans/task_m100_430_impl.md` (별도 작성).
