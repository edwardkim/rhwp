---
kind: analysis
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-04
---

# Task #3820·#3821 Stage 5 — p108 TIFF 그림 52 미출력 분석

## 재현

human p108의 render-tree에는 그림 52가 누락된 것이 아니라 Body `Image(pi=1147, ci=0)`로
존재한다. bbox는 `x=105.1, y=227.3, w=482.4, h=100.8`, wrap은 `TopAndBottom`이다.
그러나 rhwp SVG는 해당 node를 다음처럼 raw `data:image/tiff`로 방출하며, rsvg/브라우저는
이를 안정적으로 decode하지 않아 raster review에는 caption만 남는다. 한컴 PDF에는 같은 위치에
청색 flow diagram과 caption이 모두 있다.

```text
<image x="105.066..." y="227.264..." width="482.4" height="100.8"
       href="data:image/tiff;base64,..."/>
```

`stage3-review-pairs/p108_rhwp_pdf.png`가 직접 증적이다. 따라서 이는 페이지 owner나
TopAndBottom 흐름 결함이 아니라 browser-compatible image emission 결함이다.

## 기존 기능과 누락 경로

`image_resolver::tiff_bytes_to_png_bytes()`와 `emitted_image_bytes()`는 이미 TIFF→PNG
변환을 구현하며, `html.rs`도 `image/tiff`를 PNG로 내보낸다. 하지만 SVG의
`render_image_node`, `render_page_background_image`, generic `draw_image`는 BMP/PCX만
변환하고 TIFF branch가 없다. Wasm `web_canvas::draw_image`도 자체 MIME 판별기에 TIFF가
없고 raw data URI를 `HtmlImageElement`로 넘긴다.

즉 paint/HTML의 정상 경로가 SVG export와 browser canvas의 fallback에 전파되지 않은 상태다.

## 예정 변경과 수용 기준

1. SVG의 세 image-emission path와 Wasm WebCanvas를 HTML과 같은 TIFF→PNG converter로
   연결한다. 변환 실패 시 기존 raw payload fallback은 유지한다.
2. SVG renderer unit은 TIFF `ImageNode`가 `data:image/png;base64,`로 방출됨을 고정한다.
3. p108 SVG의 `image/tiff`가 사라지고 PNG가 나오며, raster 쌍에서 그림 52가 PDF와 같은
   페이지에 보인다.
4. p108 및 p156과 #3820 focused regressions를 다시 확인한다. 이 문서가 commit된 뒤에만
   code/test를 수정한다.

## 구현 및 검증 결과

SVG의 세 image-emission path와 Wasm WebCanvas path 모두에서 `image/tiff` payload를
기존 `tiff_bytes_to_png_bytes()`로 정상화한 뒤 PNG data URI로 방출하도록 연결했다. 변환이
실패하면 기존 raw payload fallback을 그대로 사용한다. SVG regression
`test_image_node_tiff_converts_to_png_for_svg`는 minimal TIFF가 `data:image/png;base64,`로
방출되고 `data:image/tiff`가 남지 않는 계약을 고정한다.

실제 p108 export도 `data:image/tiff` 0건, `data:image/png` 1건으로 확인됐다.
`stage5-review-pairs/p108_rhwp_pdf.png`의 좌측 rhwp에는 이전에 사라졌던 청색 그림 52가
caption 위에 다시 나타나며, 우측 한컴 PDF의 같은 그림과 같은 페이지 위치에 놓인다.

다음 검증을 완료했다.

```text
CARGO_TARGET_DIR=target/task-3820-3821-fidelity CARGO_INCREMENTAL=0 \
  cargo test --profile release-test --lib test_image_node_tiff_converts_to_png_for_svg
# 1 passed

CARGO_TARGET_DIR=target/task-3820-3821-fidelity CARGO_INCREMENTAL=0 \
  cargo test --profile release-test --lib tiff_image_payload_is_normalized_to_png
# 1 passed

cargo fmt --check
# passed

CARGO_TARGET_DIR=target/task-3820-3821-fidelity CARGO_INCREMENTAL=0 \
  cargo test --profile release-test --lib \
  issue_3821_page_tail_square_picture_wrap_reaches_visible_text_after_guides
# 1 passed

CARGO_TARGET_DIR=target/task-3820-3821-fidelity CARGO_INCREMENTAL=0 \
  cargo test --profile release-test --lib \
  issue_3820_rewinding_rowbreak_uses_painted_first_fragment_boundary
# 1 passed
```

`visual_sweep.py --pages 108,156 --dpi 144`의 선택 스윕은 SVG/PDF 2/2, render-tree 2/2로
완료됐고 `flagged_page_count=0`이었다. p108의 TIFF 그림 복원과 p156의 Square 그림 여백
회귀가 같은 실행에서 경고 없이 통과했다. 이 결과는 두 페이지의 결함이 해결됐다는 범위의
증적이며, 문서 전체의 잔존 fidelity 결함이 없다는 선언은 아니다.
