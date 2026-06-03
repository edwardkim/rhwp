# Task M100 #430 최종 결과 보고서

**이슈**: [#430 그림 자동 크롭(박스 비율 맞춤) 미구현 — exam_kor 헤더 "국어 영역(A 형)"](https://github.com/edwardkim/rhwp/issues/430)
**브랜치**: `local/task430` ← `local/devel`
**마일스톤**: M100 (v1.0.0)
**작업 기간**: 2026-04-29 (1일)

## 1. 문제 및 결과

`samples/exam_kor.hwp` 1쪽 상단 가운데 헤더 이미지가 SVG 출력에서 `국어 영역(A 형)` 으로 표시(원본이 강제 스트레치되어 가로 폭 압축). 한컴 PDF는 `국어 영역` 만 표시 — PDF가 정답.

| 단계 | 표시 텍스트 | 이미지 폭(px) |
|------|-------------|---------------|
| 수정 전 (rhwp) | 국어 영역(A 형) | 2320 (원본 전체) |
| 수정 후 (rhwp) | 국어 영역 | 1364.88 (좌측 58.8%) |
| 한컴 PDF | 국어 영역 | 1137 (PDF 비트맵) |

## 2. 원인

HWP 그림 컨트롤(`Picture`) 의 `crop` 필드가 정상 파싱돼 `ImageNode.crop` 에 전달되고 있었으나, SVG/Canvas 렌더러의 스케일 변환 공식이 잘못돼 crop이 사실상 무력화되고 있었다.

### 잘못된 공식

```rust
let scale_x = cr as f64 / img_w;          // 스케일을 crop 우경계로부터 추정
let src_w  = (cr - cl) as f64 / scale_x;  // 결과: cr/(cr/img_w) = img_w (항상 전체 폭)
```

`cr`(crop 우경계 HU)을 마치 원본 이미지 우경계인 양 다뤘다. 결과적으로 어떤 crop 값이 와도 src 영역이 항상 이미지 전체와 일치해 crop이 무효.

### 올바른 공식

```rust
let scale_x = original_width_hu  / img_w_px;   // 진짜 HU/px 스케일
let scale_y = original_height_hu / img_h_px;
let src_x = cl as f64 / scale_x;
let src_y = ct as f64 / scale_y;
let src_w = (cr - cl) as f64 / scale_x;
let src_h = (cb - ct) as f64 / scale_y;
```

본 이슈 케이스 입력값:
- `original_size_hu = (174000, 26580)`, `img_px = (2320, 354)`
- `crop = (0, 0, 102366, 26580)`
- → `scale = (75, 75.08) HU/px`
- → `src = (0, 0, 1364.88, 354)`

이 src 영역만 SVG `<svg viewBox=...>` 로 노출 → "국어 영역" 만 보임.

## 3. 변경 사항

### 3.1 모델

| 파일 | 변경 |
|------|------|
| `src/renderer/render_tree.rs` | `ImageNode` 에 `original_size_hu: Option<(u32, u32)>` 필드 추가. `pic.shape_attr.{original_width, original_height}` 와 동일 단위(HWPUNIT) |

### 3.2 레이아웃 (ImageNode 채우기)

| 파일:라인 | 경로 | 변경 |
|-----------|------|------|
| `src/renderer/layout/picture_footnote.rs` `layout_picture` | 표 셀 그림 (본 케이스) | `original_size_hu` 채움 |
| `src/renderer/layout/picture_footnote.rs` `layout_body_picture` | body picture | 동일 |
| `src/renderer/layout.rs` (~2614) | TAC + 텍스트 없는 문단 | 동일 |
| `src/renderer/layout/table_cell_content.rs:635` | 셀 텍스트 안 picture | `None` 명시 (컴파일 호환) |

### 3.3 렌더러

| 파일 | 변경 |
|------|------|
| `src/renderer/svg.rs` | 헬퍼 `compute_image_crop_src(crop_hu, original_size_hu, img_w_px, img_h_px) -> (sx, sy, sw, sh)` 추가(pub(crate)). `render_image_node` 의 `FitToSize+crop` 분기가 헬퍼 호출. 폴백 동작 보존 |
| `src/renderer/web_canvas.rs` | `draw_image_with_fill_mode` 시그니처에 `original_size_hu` 추가 + 동일 헬퍼 호출. SVG/Canvas 두 렌더러가 단일 진실 원천 공유 |

### 3.4 디버깅 보강

| 파일 | 변경 |
|------|------|
| `src/main.rs` `dump` (라인 1643) | 표 셀 내부 그림에 `orig`, `cur`, `crop` 출력 추가. Stage 1 조사 단계에서 적용한 후 디버깅 가치로 유지 |

### 3.5 테스트

`src/renderer/svg/tests.rs` 에 단위 테스트 4건 추가:
- `test_compute_image_crop_src_exam_kor_header` — 본 이슈 케이스
- `test_compute_image_crop_src_no_crop_full_image` — crop이 원본 전체와 일치
- `test_compute_image_crop_src_offset_top_left` — 좌상단 오프셋 + 우하단 잘림
- `test_compute_image_crop_src_fallback_when_original_size_missing` — 폴백 검증

## 4. 검증

| 항목 | 결과 |
|------|------|
| `cargo build --release` (native) | 통과 |
| `cargo check --target wasm32-unknown-unknown --release --lib` (WASM) | 통과 |
| `cargo test --release --lib` | 1027 passed, 0 failed |
| `cargo test --release --tests` | 7 passed, 0 failed |
| `cargo clippy --release --lib` | 기존 선존재 에러 2건(`table_ops.rs:1007`, `object_ops.rs:298`) 외 본 변경 추가 경고 0건 |
| 시각 검증 — exam_kor 1쪽 헤더 | "국어 영역" 만 표시 (PDF 일치) |
| 회귀 — exam_kor 다른 페이지 | 본 변경 전후 동일 동작 |

### 시각 검증 상세

수정 후 SVG의 헤더 영역:
```xml
<svg x="452.07" y="207.25" width="218.37" height="56.71"
     viewBox="0 0 1364.88 354" preserveAspectRatio="none">
  <image width="2320" height="354" href="data:image/jpeg;base64,..."/>
</svg>
```
viewBox 가 정확히 좌측 1364.88×354 px(= "국어 영역" 영역)만 노출. 원본 JPEG 자체는 그대로 임베드되어 있어 향후 동일 자원 다른 crop 적용에도 재사용 가능.

## 5. 비범위 (Out of Scope)

본 이슈 수정 범위 외로 별도 이슈 분리 권장:

1. **인라인 TAC picture crop 누락** — `paragraph_layout.rs:1700, 1950, 2033` 의 ImageNode 생성 사이트는 `crop` 자체를 전달하지 않음. 본 케이스에서는 미사용 경로지만 동일 패턴 재발 가능.
2. **그룹/도형 picture** — `shape_layout.rs:959, 1133` 동일.
3. **섹션 1/2 (페이지 14, 20) 헤더 이미지 미렌더** — 셀 paragraph[1] 이 텍스트(`(화법과 작문)` 등) + 그림(bin_id=27) 동시 보유 시 그림이 SVG에 출력되지 않음. 본 이슈와 별개로, "텍스트와 그림이 함께 있는 셀 paragraph" 처리 결함.

## 6. 산출물

### 코드

```
M src/main.rs                                  (dump 보강, 디버깅 가치로 유지)
M src/renderer/layout.rs                       (ImageNode original_size_hu)
M src/renderer/layout/picture_footnote.rs      (ImageNode original_size_hu × 2 사이트)
M src/renderer/layout/table_cell_content.rs    (필드 명시 보강)
M src/renderer/render_tree.rs                  (ImageNode 필드 추가)
M src/renderer/svg.rs                          (헬퍼 추출 + 공식 교정)
M src/renderer/svg/tests.rs                    (단위 테스트 4건 추가)
M src/renderer/web_canvas.rs                   (시그니처 + 헬퍼 호출)
```

### 문서

- `mydocs/plans/task_m100_430.md` — 수행 계획서
- `mydocs/plans/task_m100_430_impl.md` — 구현 계획서
- `mydocs/working/task_m100_430_stage1.md` — Stage 1 (원인 정밀 조사)
- `mydocs/working/task_m100_430_stage2.md` — Stage 2 (모델 + svg.rs)
- `mydocs/working/task_m100_430_stage3.md` — Stage 3 (web_canvas + 회귀)
- `mydocs/report/task_m100_430_report.md` — 본 보고서
- `mydocs/orders/20260429.md` — 오늘 할일 갱신
