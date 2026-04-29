# Task M100 #430 Stage 3 완료 보고서 — Web Canvas 동기화 + 회귀 검증

## 1. 변경 사항

### 1.1 `src/renderer/web_canvas.rs`

- `draw_image_with_fill_mode` 시그니처에 `original_size_hu: Option<(u32, u32)>` 추가.
- 호출부(line 315) 에서 `img.original_size_hu` 전달.
- crop 분기의 인라인 스케일 계산을 제거하고, `crate::renderer::svg::compute_image_crop_src` (Stage 2에서 추출한 헬퍼) 호출로 교체. SVG/Canvas 양쪽이 단일 진실 원천(single source of truth)을 공유.

## 2. 검증

### 2.1 빌드

```
cargo build --release                         → 통과 (native)
cargo check --target wasm32-unknown-unknown --release --lib → 통과 (WASM)
```

### 2.2 단위 테스트 + 통합 테스트

```
cargo test --release --lib   → 1027 passed; 0 failed
cargo test --release --tests → 7 passed; 0 failed (issue_*, ktx_toc, render_is_deterministic, task290_exam_math 등)
```

### 2.3 Clippy

`cargo clippy --release --lib` 에서 2건 에러:
- `src/document_core/commands/table_ops.rs:1007` (panicking_unwrap)
- `src/document_core/commands/object_ops.rs:298` (panicking_unwrap)

위 2건은 `git stash` 후 base(local/devel) 상태에서도 동일하게 발생하는 **기존 코드의 선존재 이슈**. 본 타스크 범위 밖이며 본 변경으로 새 경고 추가되지 않음.

### 2.4 시각 검증 (exam_kor.hwp)

**Before (수정 전)**: 페이지 1 SVG 헤더가 원본 2320×354 px 전체를 박스에 강제 스트레치 → "국어 영역(A 형)" 모두 보임.

**After (수정 후)**: 페이지 1 SVG의 헤더 영역:
```xml
<svg x="452.07" y="207.25" width="218.37" height="56.71"
     viewBox="0 0 1364.88 354" preserveAspectRatio="none">
  <image width="2320" height="354" href="..."/>
</svg>
```
viewBox 가 정확히 좌측 1365×354 px 영역만 노출 → "국어 영역" 만 표시.

**PDF 대조**: 한컴 PDF 출력에서 추출한 비트맵 (1137×295, 가로/세로=3.85)도 동일 영역만 포함. 결과 일치.

### 2.5 다른 페이지 (참고)

- 페이지 14, 20 (섹션 1/2 시작) 은 SVG에 헤더 이미지가 렌더되지 않는 별개 이슈가 있음. 본 변경 전후 동일하게 미렌더 → **회귀 없음**. 별도 이슈로 분리 권장(셀 paragraph 1의 그림+텍스트 동시 케이스 미처리).

## 3. 다음 단계

Stage 4 — 최종 결과 보고서 작성, `mydocs/orders/{오늘}.md` 갱신.
