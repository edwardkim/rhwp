# Task #975 Stage 1 — 재현 및 렌더러 경로 확정

- 이슈: [#975](https://github.com/edwardkim/rhwp/issues/975)
- 브랜치: `task-975-page-background-fill-mode`
- 기준 커밋: `39d90d9d`
- 수행일: 2026-05-18

## 1. 샘플 확보

로컬 저장소에는 `143E433F503322BD33.hwp` 샘플이 없었다.

이슈 본문에 기재된 Tistory URL에서 임시 디렉터리로 다운로드했다.

```text
/private/tmp/rhwp-task975/143E433F503322BD33.hwp
```

샘플은 PR에 포함하지 않는다.

확인 결과:

```text
Hangul (Korean) Word Processor File 5.x
파일 크기: 73216 bytes
버전: 5.0.3.2
페이지 수: 1
BinData:
  [0] Embedding (ID: 1, ext: jpg, loaded: 3464 bytes)
  [1] Storage (ID: 2, ext: OLE, loaded: 11776 bytes)
  [2] Embedding (ID: 3, ext: png, loaded: 41346 bytes)
```

## 2. dump 확인

`target/debug/rhwp dump` 결과, 이슈 본문과 동일하게 페이지 배경/BorderFill 이미지가 `Center` 모드로 파싱된다.

```text
border_fill[0] fill_type=Solid bg=#FFFFFFFF pat_type=-1 pat_color=#000000
border_fill[1] fill_type=None
border_fill[2] fill_type=Image  image(bin_id=3, mode=Center)
```

따라서 parser/doc_info 단계에서는 `ImageFillMode::Center`를 읽고 있다.

## 3. SVG 재현

명령:

```text
target/debug/rhwp export-svg /private/tmp/rhwp-task975/143E433F503322BD33.hwp -o /private/tmp/rhwp-task975/svg-before
```

결과:

```text
문서 로드 완료: /private/tmp/rhwp-task975/143E433F503322BD33.hwp (1페이지)
  → /private/tmp/rhwp-task975/svg-before/143E433F503322BD33.svg
내보내기 완료: 1개 SVG 파일 → /private/tmp/rhwp-task975/svg-before/
```

생성된 SVG의 페이지 크기와 배경 이미지 출력:

```text
<svg ... width="793.7066666666667" height="1122.5066666666667" viewBox="0 0 793.7066666666667 1122.5066666666667">
<rect x="0" y="0" width="793.7066666666667" height="1122.5066666666667" fill="#ffffff"/>
<image x="0" y="0" width="793.7066666666667" height="1122.5066666666667" preserveAspectRatio="none" href="DATA"/>
```

배경 이미지가 페이지 bbox 전체 크기로 강제 stretch되므로 재현 성공.

## 4. 렌더러 경로 확인

### layout

`src/renderer/layout.rs`의 `build_page_background()`는 fill mode를 보존한다.

```text
PageBackgroundImage {
    data: c.data.clone(),
    fill_mode: img_fill.fill_mode,
}
```

layout 단계는 본 이슈의 1차 수정 대상이 아니다.

### SVG

`src/renderer/svg.rs`의 `RenderNodeType::PageBackground` 분기는 `img.fill_mode`를 사용하지 않고 전체 bbox에 `preserveAspectRatio="none"`으로 출력한다.

```text
<image x="{bbox.x}" y="{bbox.y}" width="{bbox.width}" height="{bbox.height}" preserveAspectRatio="none" .../>
```

일반 `ImageNode` 경로에는 이미 `render_positioned_image()`와 `render_tiled_image()`가 있다.

### Web Canvas

`src/renderer/web_canvas.rs`의 `RenderNodeType::PageBackground` 분기는 `img.fill_mode`를 사용하지 않고 전체 bbox로 `draw_image()`를 호출한다.

```text
self.draw_image(&img.data, node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height)
```

일반 `ImageNode` 경로에는 이미 `draw_image_with_fill_mode()`가 있다.

### Skia

`src/renderer/skia/renderer.rs`의 `PaintOp::PageBackground` 경로는 이미 `Some(image.fill_mode)`를 `draw_image()`에 전달한다.

```text
draw_image(&image.data, *bbox, Some(image.fill_mode), None, None, ImageEffect::RealPic)
```

다만 현재 `target/debug/rhwp`는 `native-skia` feature 없이 빌드되어 `export-png` 실행 검증은 불가했다.

```text
오류: export-png 명령은 native-skia feature 가 활성화되어야 합니다.
```

### legacy Canvas / HTML

`src/renderer/canvas.rs`와 `src/renderer/html.rs`는 PageBackground 이미지 자체를 실질적으로 렌더링하지 않는 경로로 보인다. 본 이슈 본문이 지목한 사용자 영향 경로는 SVG와 Web Canvas다.

## 5. 결론

Stage 1 기준 root cause는 렌더러별 PageBackground 이미지 출력 경로의 불일치다.

```text
parser/doc_info: Center 파싱 OK
layout/render_tree: PageBackgroundImage.fill_mode 보존 OK
SVG renderer: PageBackground image fill_mode 무시 → full page stretch
Web Canvas renderer: PageBackground image fill_mode 무시 → full page stretch
Skia renderer: fill_mode 전달 코드 존재, 별도 빌드 검증 필요
```

따라서 Stage 2 구현 계획서는 SVG/Web Canvas의 PageBackground 이미지 경로에 `ImageFillMode` 적용을 추가하는 방향으로 작성한다.

## 6. Stage 2 검토 포인트

1. SVG는 기존 일반 이미지 helper(`render_positioned_image`, `render_tiled_image`) 재사용 또는 PageBackground 전용 wrapper 추가를 검토한다.
2. Web Canvas는 기존 `draw_image_with_fill_mode()`를 PageBackground에서도 호출하는 방향을 우선 검토한다.
3. `FitToSize`는 기존 출력과 동일하게 유지한다.
4. 원본 크기 파싱 실패 시 기존처럼 bbox 전체 렌더링으로 폴백한다.
5. Skia는 현재 코드상 fill mode 전달이 있으므로 Stage 2에서 수정 대상 여부를 재판정한다.
