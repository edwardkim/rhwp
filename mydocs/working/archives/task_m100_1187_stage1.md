# Stage 1 보고서 — Task M100-1187: BookReview.hwp 글상자 clip RED 테스트

## 목표

`samples/basic/BookReview.hwp` 1쪽에서 글상자 콘텐츠 clip 이 없어서 목차 뒤쪽 텍스트가 글상자 밖으로 출력되는 회귀를 자동 테스트로 고정한다.

## 변경

### tests/issue_1187_textbox_clip.rs (신규)

- `BookReview.hwp` 1쪽을 `render_page_svg_native(0)` 로 렌더링한다.
- 우측 하단 저자 정보 텍스트(`강우신지음`, `원앤원북스`)가 사라지지 않았는지 먼저 확인한다.
- SVG 에 `textbox-clip-` clipPath 가 존재하는지 확인한다.
- 큰 점선 글상자 내부 영역에 해당하는 clip rect 가 있는지 확인한다.
  - 예상 범위: `x≈47.9`, `y≈516.6`, `w≈687.5`, `h≈487.9`
  - 부동소수 출력 포맷 차이를 고려해 범위 비교로 작성했다.

## RED 검증

명령:

```bash
cargo test --test issue_1187_textbox_clip
```

결과:

- 컴파일 성공
- 테스트 1건 실패
- 실패 지점: `BookReview.hwp 글상자 콘텐츠에는 textbox clipPath 가 필요함`

이는 최신 `upstream/devel` 기준 SVG 출력에 글상자 콘텐츠용 clipPath 가 없다는 기존 진단과 일치한다.

## 다음 (Stage 2)

`layout_textbox_content` 에서 글상자 내부 콘텐츠를 기존 shape node 직속이 아니라 `RenderNodeType::TextBox` 컨테이너 아래로 모아, 이후 SVG/paint layer 에서 해당 노드를 clip 처리할 수 있게 한다.
