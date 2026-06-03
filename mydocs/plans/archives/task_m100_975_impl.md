# 구현 계획서 — Task #975: PageBackground 이미지 fill_mode 렌더링 보정

- 이슈: [#975](https://github.com/edwardkim/rhwp/issues/975)
- 수행 계획서: [task_m100_975.md](task_m100_975.md)
- Stage 1 보고서: [task_m100_975_stage1.md](../working/task_m100_975_stage1.md)
- 브랜치: `task-975-page-background-fill-mode`

## 1. Stage 1 결론

`143E433F503322BD33.hwp` 샘플에서 페이지 배경/BorderFill 이미지가 `Center` 모드와 워터마크 톤 값을 갖는 것은 확인됐다.

```text
border_fill[2] fill_type=Image  image(bin_id=3, mode=Center, brightness=-50, contrast=70, effect=0)
```

기존 문제는 두 갈래다.

```text
1. fill_mode는 layout까지 보존됐지만 SVG/Web Canvas가 bbox 전체 stretch로 렌더링했다.
2. ImageFill의 brightness/contrast/effect는 parser에서 읽었지만 PageBackground 렌더 트리로 전달되지 않았다.
```

| 경로 | 현재 상태 | 판단 |
|------|----------|------|
| SVG `src/renderer/svg.rs` | PageBackground 이미지가 bbox 전체로 stretch, tone 미적용 | 수정 대상 |
| Web Canvas `src/renderer/web_canvas.rs` | PageBackground 이미지가 bbox 전체로 stretch, tone 미적용 | 수정 대상 |
| Style/Layout/RenderTree | ImageFill tone 정보 손실 | 수정 대상 |
| Skia `src/renderer/skia/renderer.rs` | `Some(image.fill_mode)` 전달 | fill_mode 수정 제외, 구조체 필드 보강만 |
| legacy Canvas / HTML | PageBackground 이미지 실질 렌더링 없음 | 수정 제외 |

## 2. 구현 원칙

```text
1. parser/doc_info의 ImageFill 파싱은 유지한다.
2. style resolver, layout, render_tree는 ImageFill의 brightness/contrast/effect를 보존한다.
3. FitToSize는 기존 PageBackground 출력과 동일하게 유지한다.
4. Center/위치 모드는 이미지 원본 픽셀 크기 기준으로 bbox 안에 배치한다.
5. Tile 계열은 일반 ImageNode와 같은 타일링 의미를 따른다.
6. 이미지 크기 파싱 실패 시 기존처럼 bbox 전체 렌더링으로 폴백한다.
7. SVG와 Web Canvas의 동작 의미를 맞춘다.
8. `brightness=-50, contrast=70` 조합은 한컴 색상 워터마크 톤 preset으로 취급한다.
9. `effect=0(RealPic)`인 워터마크 preset은 기존 brightness/contrast 필터를 적용하지 않고, 추출 워터마크 이미지와 한컴 뷰어 watermark-only 스크린샷 비교로 산정한 공통 톤 보정 `saturate(0.9165) + contrast(0.9313) + brightness(2.0972) + opacity(0.21729612)`를 적용한다.
10. 이 규칙은 PageBackground와 셀/문단/도형 배경 ImageFill에 동일하게 적용한다.
11. `effect != RealPic`이거나 preset이 아닌 tone 값은 기존 일반 이미지 필터 경로를 유지한다.
```

## 3. 구현 범위

### 3.1 SVG 렌더러

파일:

```text
src/renderer/svg.rs
```

현재 `RenderNodeType::PageBackground` 이미지 분기는 다음처럼 직접 출력한다.

```text
<image x="{bbox.x}" y="{bbox.y}" width="{bbox.width}" height="{bbox.height}" preserveAspectRatio="none" .../>
```

수정 방향:

1. PageBackground 이미지용 helper를 추가하거나 기존 로직을 분기 안에 정리한다.
2. BMP → PNG 재인코딩과 data URI 생성은 현재 PageBackground 경로의 호환성을 유지한다.
3. `img.fill_mode` 기준으로 다음 처리:
   - `FitToSize` 또는 `None`: 기존과 같은 bbox 전체 출력
   - `TileAll`, `TileHorzTop`, `TileHorzBottom`, `TileVertLeft`, `TileVertRight`: 기존 `render_tiled_image()` 재사용
   - `Center`, `CenterTop`, `CenterBottom`, `LeftCenter`, `LeftTop`, `LeftBottom`, `RightCenter`, `RightTop`, `RightBottom`: 기존 `render_positioned_image()` 재사용
4. `original_size`는 PageBackgroundImage에 없으므로 `None`을 전달하고, helper가 이미지 바이너리에서 크기를 파싱하게 한다.
5. `effect=0(RealPic) + brightness=-50 + contrast=70` PageBackground 워터마크 preset은 brightness/contrast 필터를 적용하지 않는다.
6. 위 preset은 원본 색상을 유지하되 `saturate(0.9165) + contrast(0.9313) + brightness(2.0972) + opacity(0.21729612)`로 한컴 색상 워터마크에 근사한다.
7. 그 외 `brightness/contrast/effect` 조합은 일반 ImageNode와 같은 필터 경로를 유지한다.

예상 변경 형태:

```text
self.render_page_background_image(img, &node.bbox)
```

또는 직접:

```text
match img.fill_mode {
    ImageFillMode::FitToSize | ImageFillMode::None => 기존 출력,
    ImageFillMode::Tile... => self.render_tiled_image(..., None),
    _ => self.render_positioned_image(..., img.fill_mode, None),
}
```

### 3.2 Web Canvas 렌더러

파일:

```text
src/renderer/web_canvas.rs
```

현재 `RenderNodeType::PageBackground` 이미지 분기는 다음처럼 직접 호출한다.

```text
self.draw_image(&img.data, node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height)
```

수정 방향:

1. 기존 `draw_image_with_fill_mode()`를 PageBackground 이미지에도 사용한다.
2. PageBackground에는 crop과 HWP shape 원본 크기가 없으므로 다음 값을 전달한다.

```text
fill_mode = Some(img.fill_mode)
original_size = None
crop = None
original_size_hu = None
```

예상 변경 형태:

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

이 함수는 `FitToSize | None`에서 기존 bbox 전체 `draw_image()`로 폴백하므로 기존 기본 동작을 유지한다.

3. `effect=0(RealPic) + brightness=-50 + contrast=70` PageBackground 워터마크 preset은 기존 `compose_image_filter()`를 건너뛴다.
4. 위 preset은 CSS `saturate(92%) contrast(93%) brightness(210%)`와 `globalAlpha=0.21729612`를 적용하고 렌더 후 reset한다.
5. 그 외 `brightness/contrast/effect` 조합은 기존 `compose_image_filter()` 경로를 유지한다.

### 3.3 Style/Layout/RenderTree

파일:

```text
src/renderer/style_resolver.rs
src/renderer/layout.rs
src/renderer/render_tree.rs
```

수정 방향:

```text
ResolvedImageFill: bin_data_id, fill_mode + brightness, contrast, effect 보존
PageBackgroundImage: data, fill_mode + brightness, contrast, effect 보존
LayoutEngine::build_page_background(): ResolvedImageFill 값을 PageBackgroundImage로 전달
table_layout/shape_layout: ResolvedImageFill 또는 Drawing ImageFill의 tone 속성을 ImageNode로 전달
```

### 3.4 Skia

파일:

```text
src/renderer/skia/renderer.rs
```

현재 이미 다음 경로가 있다.

```text
draw_image(&image.data, *bbox, Some(image.fill_mode), None, None, ImageEffect::RealPic)
```

따라서 코드 변경 대상에서 제외한다.

가능하면 별도 검증:

```text
cargo build --release --features native-skia
target/release/rhwp export-png ...
```

단, `native-skia` 빌드 비용이나 환경 제약이 크면 Stage 4에서 미실행 사유를 기록한다.

## 4. 테스트 계획

### 4.1 구조 검증

대상 샘플:

```text
/private/tmp/rhwp-task975/143E433F503322BD33.hwp
```

수정 전 SVG 배경 이미지:

```text
<image x="0" y="0" width="793.706..." height="1122.506..." preserveAspectRatio="none" .../>
```

수정 후 기대:

```text
PageBackground 이미지가 bbox 전체 크기가 아니라 이미지 원본 크기 기준으로 중앙 배치된다.
```

정확한 수치는 Stage 3 구현 후 생성 SVG에서 확인한다.

### 4.2 명령 검증

```text
target/debug/rhwp export-svg /private/tmp/rhwp-task975/143E433F503322BD33.hwp -o /private/tmp/rhwp-task975/svg-after
```

확인 항목:

```text
1. PageBackground `<image>`의 x/y가 중앙 배치 좌표로 바뀐다.
2. PageBackground `<image>`의 width/height가 페이지 bbox 전체가 아니라 PNG 원본 크기다.
3. `preserveAspectRatio="none"`은 원본 크기 렌더링에서는 왜곡을 만들지 않는다.
4. 본문 그림 등 일반 ImageNode 출력은 기존 경로 그대로 유지된다.
```

### 4.3 자동 테스트

우선 실행:

```text
cargo test --lib
```

필요 시 추가:

```text
cargo test --test svg_snapshot
```

Web Canvas는 wasm/browser 경로라 로컬 Rust 테스트로 직접 픽셀 검증하기 어렵다. Stage 4에서 코드 경로 검증과 필요 시 WASM 빌드 여부를 기록한다.

## 5. 위험 및 완화

| 위험 | 완화 |
|------|------|
| SVG/Web Canvas fill mode 의미 불일치 | 기존 일반 이미지 helper 재사용 |
| Center 이미지가 너무 큰 경우 bbox 밖으로 넘침 | 기존 helper처럼 bbox clip 적용 |
| PNG 크기 파싱 실패 | 기존 bbox 전체 렌더링으로 폴백 |
| FitToSize 회귀 | `FitToSize | None`은 기존 출력 유지 |
| Skia와 SVG 결과 차이 | Skia는 이미 fill_mode 전달 중, 가능하면 native-skia 검증 |

## 6. 구현 후 산출물

```text
mydocs/working/task_m100_975_stage3.md
mydocs/working/task_m100_975_stage4.md
mydocs/report/task_m100_975_report.md
```

## 7. 승인 요청

본 구현 계획 승인 후 다음 source 변경을 수행한다.

```text
1. src/renderer/svg.rs — PageBackground image fill_mode 적용
2. src/renderer/web_canvas.rs — PageBackground image fill_mode 적용
```

소스 수정은 이 구현 계획 승인 후에만 진행한다.
