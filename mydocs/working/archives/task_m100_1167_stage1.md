# Stage 1 보고서 — Task M100-1167: layout text_wrap 채움 + RED 테스트

## 목표

ImageNode 에 wrap 정보 보존 + SVG z-order 결함 박제(RED).

## 변경

### `src/renderer/layout.rs:4642`

ImageNode 생성 시 `text_wrap: Some(pic.common.text_wrap)` 추가. 종전엔 `..ImageNode::new()` 로 `text_wrap=None` 이었다. SVG plane multi-pass z-order 판별(Stage 2)에 사용.

### `tests/issue_1167_svg_behindtext_zorder.rs` (신규)

`samples/복학원서.hwp` SVG 렌더 → BehindText 워터마크 `<image>` 출현 오프셋이 본문 첫 `<text>` 오프셋보다 **앞**(작은 값)이어야 한다는 단언. SVG 는 후순위가 위로 합성되므로 BehindText 는 본문보다 먼저(아래) 와야 한다.

## 검증 (RED 확인)

```
test behindtext_watermark_renders_before_body_text_in_svg ... FAILED
panicked: BehindText 워터마크 image(offset [453349])가 본문 첫 text(offset 66205)보다 뒤
         — SVG 후순위로 본문을 덮음 (z-order 결함)
```

→ 결함 정확히 재현. 워터마크 image(453349)가 본문 text(66205)보다 한참 뒤 = 본문 위 합성.

## text_wrap 채움 무회귀 확인

`cargo test --tests --no-fail-fast` 전수 실행 결과, 실패한 **개별 테스트는 `behindtext_watermark_renders_before_body_text_in_svg`(RED 의도) 하나뿐**. PaintOp 경로(skia/canvaskit/json) 및 기타 테스트 회귀 없음.

- 근거: PaintOp `image.text_wrap` 은 paint 변환에서 별도 set(`replay_order.rs:58`) → layout ImageNode.text_wrap 변경과 독립. json.rs:764 의 wrap 출력도 PaintOp.image.text_wrap 을 읽으므로 무관.

## 다음 (Stage 2)

`svg.rs render_node` 를 plane 별 multi-pass 순회로 변경하여 GREEN 전환.
