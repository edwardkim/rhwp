# Task M100 #430 Stage 2 완료 보고서 — 모델 + 코어 렌더러 수정

## 1. 변경 사항

### 1.1 `src/renderer/render_tree.rs`

`ImageNode` 에 `original_size_hu: Option<(u32, u32)>` 필드 추가. `pic.shape_attr.{original_width, original_height}` 와 동일 단위(HWPUNIT). default = `None`.

### 1.2 ImageNode 채우기 — 4곳

| 파일:라인 | 변경 |
|-----------|------|
| `src/renderer/layout/picture_footnote.rs:98` | `original_size_hu` 채움 (본 이슈 핵심 경로 — 표 셀 그림) |
| `src/renderer/layout/picture_footnote.rs:312` | 동일 (body picture) |
| `src/renderer/layout.rs:2614` | 동일 (TAC + 텍스트 없는 문단) |
| `src/renderer/layout/table_cell_content.rs:635` | 명시적 초기화 보강 (`None`) — 컴파일 에러 회피 |

### 1.3 `src/renderer/svg.rs`

- 새 헬퍼 `compute_image_crop_src` 추출 (pub(crate)).
- `render_image_node` 의 `FitToSize + crop` 분기에서 헬퍼 호출.
- 핵심: `scale_x = original_width_HU / img_w_px`, `scale_y = original_height_HU / img_h_px`.
- 폴백: `original_size_hu` 가 없으면 기존 `cr/img_w` 공식 유지(과거 동작 호환).

### 1.4 `src/renderer/svg/tests.rs` — 단위 테스트 4개

| 테스트 | 내용 |
|--------|------|
| `test_compute_image_crop_src_exam_kor_header` | 본 이슈 케이스 (174000×26580 HU, crop=(0,0,102366,26580), 2320×354 px → src=(0,0,1364.88,354)) |
| `test_compute_image_crop_src_no_crop_full_image` | crop이 원본 전체와 일치 → src도 전체 |
| `test_compute_image_crop_src_offset_top_left` | 좌상단 오프셋 + 우하단 잘림 |
| `test_compute_image_crop_src_fallback_when_original_size_missing` | 폴백 동작 검증 |

### 1.5 `src/main.rs:1643` (Stage 1에서 적용)

표 셀 그림 dump 형식에 `orig`, `cur`, `crop` 정보 추가 — 디버깅 가치로 유지.

## 2. 검증

### 2.1 빌드

```
cargo build --release   → 통과
```

### 2.2 단위 테스트

```
cargo test --release --lib renderer::svg::tests::test_compute_image_crop
running 4 tests
test renderer::svg::tests::test_compute_image_crop_src_exam_kor_header ... ok
test renderer::svg::tests::test_compute_image_crop_src_no_crop_full_image ... ok
test renderer::svg::tests::test_compute_image_crop_src_fallback_when_original_size_missing ... ok
test renderer::svg::tests::test_compute_image_crop_src_offset_top_left ... ok
test result: ok. 4 passed; 0 failed
```

### 2.3 실측 검증

```
rhwp export-svg samples/exam_kor.hwp -p 0 -o /tmp/exam_kor_fix/
```

생성된 SVG에서 헤더 이미지 영역:
```xml
<svg x="452.07" y="207.25" width="218.37" height="56.71"
     viewBox="0 0 1364.88 354" preserveAspectRatio="none">
  <image width="2320" height="354" .../>
</svg>
```

`viewBox="0 0 1364.88 354"` — 정확히 원본 좌측 1365×354 px 영역만 표시.
이미지를 PIL로 동일 영역 잘라낸 결과: "국어 영역" 만 표시되고 "(A 형)" 잘림 ✓.

## 3. 다음 단계

Stage 3 — `src/renderer/web_canvas.rs` 동일 공식 적용 + 회귀 검증 (`cargo test --release`, `cargo clippy`, exam_kor 다른 페이지 시각 확인).
