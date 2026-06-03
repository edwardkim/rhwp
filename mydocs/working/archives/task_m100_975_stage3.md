# Task #975 Stage 3 — PageBackground fill_mode 렌더러 보정 구현

- 이슈: [#975](https://github.com/edwardkim/rhwp/issues/975)
- 브랜치: `task-975-page-background-fill-mode`
- 기준 커밋: `39d90d9d`
- 수행일: 2026-05-18

## 1. 변경 요약

PageBackground/BorderFill 이미지가 `ImageFillMode::Center` 등 배치 모드를 갖고 있어도 SVG/Web Canvas에서 페이지 bbox 전체로 늘어나던 경로를 수정했다.

추가 확인에서 해당 Tistory 샘플은 PageBackground 이미지 워터마크 톤도 갖는 것으로 확인됐다.

```text
border_fill[2] image(bin_id=3, mode=Center, brightness=-50, contrast=70, effect=0)
```

따라서 PageBackground 경로에 `brightness/contrast/effect`를 보존했다.
추가 시각 비교에서 `effect=0(RealPic) + brightness=-50 + contrast=70` 워터마크 preset은 기존 brightness/contrast 필터가 아니라 색상 워터마크 전용 톤 보정으로 처리해야 함을 확인했다. SVG/Web Canvas는 이 preset에 `saturate(0.9165) + contrast(0.9313) + brightness(2.0972) + opacity(0.21729612)`를 적용한다.

추가 사용자 샘플 `253E164F57A1BC6934.hwp`는 PageBackground가 아니라 표 셀 `BorderFill` 이미지로 색상 워터마크를 저장한다.

```text
border_fill[3] image(bin_id=1, mode=FitToSize, brightness=-50, contrast=70, effect=0)
border_fill[4] image(bin_id=2, mode=FitToSize, brightness=-50, contrast=70, effect=0)
```

따라서 동일 preset 보정 범위를 PageBackground뿐 아니라 셀/문단/도형 배경 ImageFill 경로까지 확장했다.

변경 파일:

```text
src/main.rs
src/renderer/style_resolver.rs
src/renderer/render_tree.rs
src/renderer/layout.rs
src/renderer/svg.rs
src/renderer/svg/tests.rs
src/renderer/web_canvas.rs
src/renderer/canvaskit_policy.rs
src/renderer/skia/renderer.rs
```

## 2. SVG 렌더러 변경

`src/renderer/svg.rs`의 `RenderNodeType::PageBackground` 이미지 분기에서 직접 `<image>`를 bbox 전체로 출력하던 코드를 제거하고, PageBackground 전용 helper를 추가했다.

```text
self.render_page_background_image(img, &node.bbox);
```

새 helper는 기존 PageBackground 경로의 BMP → PNG 재인코딩과 data URI 생성을 유지하면서 `img.fill_mode`를 적용한다.

```text
FitToSize | None
  → 기존과 동일하게 bbox 전체 출력

TileAll / TileHorz* / TileVert*
  → 기존 render_tiled_image() 경로 재사용

Center / CenterTop / CenterBottom / Left* / Right*
  → 기존 render_positioned_image() 경로 재사용
```

PageBackground에는 별도 `original_size` 필드가 없으므로 `None`을 전달하고, 기존 helper가 이미지 바이너리에서 원본 크기를 파싱하게 했다.

또한 PageBackground 이미지의 `brightness/contrast/effect`를 렌더 트리에 보존하고, RealPic 워터마크 preset은 공통 색상 워터마크 보정 경로로 분리했다.

```text
effect=0(RealPic), brightness=-50, contrast=70
  → brightness/contrast filter 미적용
  → saturate(0.9165) + contrast(0.9313) + brightness(2.0972) + opacity 0.21729612
```

## 3. Web Canvas 렌더러 변경

`src/renderer/web_canvas.rs`의 PageBackground 이미지 분기에서 직접 `draw_image()`를 호출하던 코드를 기존 `draw_image_with_fill_mode()` 재사용으로 변경했다.

```text
self.draw_image_with_fill_mode(
    &img.data,
    &node.bbox,
    Some(img.fill_mode),
    None,
    None,
    None,
);
```

`FitToSize | None`은 기존처럼 bbox 전체 `draw_image()`로 처리되므로 기본 동작은 유지된다.

추가로 PageBackground 워터마크 opacity reset을 적용했다.
`effect=0(RealPic) + brightness=-50 + contrast=70` preset은 기존 `compose_image_filter()`를 건너뛰고 `saturate(92%) contrast(93%) brightness(210%)`와 `globalAlpha=0.21729612`를 적용한다. 그 외 tone/effect 조합은 기존 filter 경로를 유지한다.

## 4. Style/Layout/RenderTree 변경

parser는 이미 `ImageFill`의 `brightness`, `contrast`, `effect`를 읽고 있었다. 누락은 style resolver 이후 전달 경로였다.

```text
ResolvedImageFill
  + brightness
  + contrast
  + effect

PageBackgroundImage
  + brightness
  + contrast
  + effect
```

`src/main.rs dump` 출력도 BorderFill image 속성 확인이 가능하도록 보강했다.

표 셀 배경 ImageFill과 도형 배경 ImageFill도 동일 보정을 탈 수 있도록 `ImageNode`로 `brightness/contrast/effect`를 전달한다.

```text
table_layout: ResolvedImageFill → ImageNode
shape_layout: Drawing ImageFill → ImageNode
```

## 5. 단위 테스트 추가

`src/renderer/svg/tests.rs`에 PageBackground 이미지 전용 회귀 테스트 3개를 추가했다.

```text
test_page_background_image_fit_to_size_preserves_bbox_output
test_page_background_image_center_uses_original_image_size
test_page_background_image_realpic_watermark_preserves_color_with_opacity
test_background_image_realpic_watermark_fill_preserves_color_with_opacity
```

검증 내용:

1. `FitToSize`는 기존처럼 bbox 전체 `<image>`를 출력한다.
2. `Center`는 2x2 원본 이미지 크기를 유지하고 bbox 중앙 좌표에 배치한다.
3. `Center`는 bbox 전체 stretch 출력을 만들지 않는다.
4. `effect=0(RealPic) + brightness=-50 + contrast=70` PageBackground 워터마크 preset은 brightness/contrast 필터 없이 공통 색상 워터마크 필터와 opacity 0.21729612를 적용한다.
5. 동일 preset을 갖는 일반 ImageNode 배경 fill도 같은 공통 색상 워터마크 필터와 opacity 0.21729612를 적용한다.

`src/renderer/style_resolver.rs`에는 ImageFill 속성 보존 테스트를 추가했다.

```text
test_resolve_border_image_fill_preserves_watermark_attrs
```

## 6. Stage 3 점검 결과

### 포맷

```text
cargo fmt --all -- --check
```

결과: 통과.

### 좁은 단위 테스트

```text
cargo test --lib page_background_image
```

결과:

```text
running 3 tests
test renderer::canvaskit_policy::tests::page_background_image_and_gradient_are_policy_visible ... ok
test renderer::svg::tests::test_page_background_image_fit_to_size_preserves_bbox_output ... ok
test renderer::svg::tests::test_page_background_image_center_uses_original_image_size ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 1298 filtered out
```

보완 후 재실행:

```text
cargo test --lib page_background_image
```

결과:

```text
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 1299 filtered out
```

색상 워터마크 공통 보정 추가 후 재실행:

```text
cargo test --lib realpic_watermark
```

결과:

```text
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1302 filtered out
```

추가 속성 보존 테스트:

```text
cargo test --lib test_resolve_border_image_fill_preserves_watermark_attrs
```

결과:

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1302 filtered out
```

기존 warning 6건은 이번 변경과 무관한 기존 경고다.

### whitespace

```text
git diff --check
```

결과: 통과.

## 7. 남은 검증

Stage 4에서 다음을 확인한다.

```text
1. 대상 샘플 export-svg 후 PageBackground 이미지 좌표/크기 확인
2. Center 모드가 페이지 전체 stretch가 아닌 중앙 원본 크기 배치인지 확인
3. PageBackground RealPic 워터마크가 색상 보존 + opacity로 SVG와 Web Canvas 경로에 반영됐는지 확인
4. cargo test --lib 또는 필요한 범위의 추가 테스트
5. 가능하면 native-skia 또는 WASM 관련 검증 여부 판단
```

Stage 4 전에는 이 구현을 확정하지 않는다.
