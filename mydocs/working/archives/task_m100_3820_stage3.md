---
kind: analysis
status: active
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-04
---

# Task #3820·#3821 Stage 3 — Square 그림의 paint origin과 wrap 여백 분석

## Stage 2에서 이월한 대상

Stage 2 commit `5f0c8b9b9`는 #3820의 RowBreak 표 28/29 fragment owner를 PDF와
일치시켰다. 그러나 p108 그림 52 누락과 #3821 p156 그림 64의 본문 침범은 남아 있으므로
완료로 표시하지 않는다. 이 Stage는 그림 paint origin과 wrap exclusion을 분리해 조사한다.

## p156 재현과 PDF 기준 좌표

대상은 개인정보 제거 fixture
`samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp`의
human p156, picture `pi=1692`, `ci=1`이다.

source dump는 다음 contract를 보인다.

| 항목 | 값 |
| --- | --- |
| wrap / 글자처럼 | `Square` / `false` |
| horizontal | `Column`, `Left`, offset `25139 HU` (88.7 mm) |
| common size | `16871×22713 HU` (59.5×80.1 mm) |
| outer margin | left/top/bottom `510 HU` (1.80 mm), right `0` |
| 그림 옆 마지막 문단 | `pi=1697`, line segments 모두 `cs=0`, `sw=45352` |

body left `25.0 mm`와 horizontal offset을 더한 square-frame origin은 `113.7 mm`
(`429.7px @96dpi`)다. rhwp render-tree도 image bbox left를 정확히 `429.7px`로
기록하며, p1697의 모든 TextLine right가 같은 `429.7px`다. 즉 기존 회귀는 **선이
image bbox를 넘지 않는다**만 검사해서, 간격 `0px`를 통과시켰다.

한컴 PDF raster(144dpi)의 그림 64 검은 테두리 left는 약 `654px` = `436.0px @96dpi`
이고, text bbox의 최대 right(`pdftotext -bbox`)는 `322.7pt` = `430.3px @96dpi`다.
PDF에는 약 `5.7px`의 실제 공백이 있다. source의 left outer margin `510 HU`는
`6.8px @96dpi`이므로, PDF가 보이는 origin은
`frame origin + margin.left`라는 해석과 일치한다.

## 원인

`layout_body_picture`는 non-TAC picture의 `horizontal_offset`만으로 `pic_x`를 정하고,
Square 그림의 `common.margin.left`를 paint origin에 반영하지 않는다. 반면 wrap paragraph는
이미 source `LINE_SEG` width로 오른쪽 edge를 frame origin까지만 사용한다. 결과적으로
텍스트와 그림 ink가 같은 x에서 만나며, 한컴 PDF의 left outer margin이 사라진다.

이는 p156의 그림 배치 결함이다. 그림을 전역적으로 이동하거나 줄폭을 추측으로 줄이면 다른
anchor mode를 깨뜨릴 수 있으므로 아래의 source-local gate로 고친다.

## 예정 변경과 수용 기준

1. native body picture 중 `!treat_as_char`, `Square`, `HorzRelTo::Column`,
   `HorzAlign::Left|Inside`, positive `margin.left`만 paint origin을
   `margin.left`만큼 오른쪽으로 보정한다. 기존 wrap exclusion/LINE_SEG는 변경하지 않는다.
2. p156 `pi=1692/ci=1` image left가 `429.7 + 6.8px`가 되고, p1697 visible line right와
   최소 `5px` 이상의 gap을 갖는다. 이는 reference PDF의 약 `5.7px` gap을 허용 오차 안에
   고정하는 회귀가 된다.
3. caption도 동일 origin에서 그려져 그림 64와 함께 이동한다.
4. p108 그림 52 누락은 별도 placement-owner 결함인지 같은 gate로 해소되는지 다시 직접
   비교한다. 해소되지 않으면 Stage 4 분석으로 분리한다.

## 증적 경로

- rhwp: `output/task-3820-3821-fidelity/stage3-p156-rhwp/p156_rhwp.png`
- PDF: `output/task-3820-3821-fidelity/stage3-p156-pdf/p156_pdf.png`
- PDF glyph bbox: `output/task-3820-3821-fidelity/stage3-p156-pdf/p156_pdf.xhtml`
- rhwp render tree: `output/task-3820-3821-fidelity/stage3-p156-tree/render_tree_156.json`

이 분석 문서를 커밋한 다음에만 code/test를 수정한다.

## 결과 (2026-08-04)

`layout_body_picture`에 위 gate를 적용했다. post-fix render-tree에서 `pi=1692/ci=1`
image left는 `436.5px`, p1697 TextLine max-right는 `429.7px`로 측정되어 gap은
`6.8px`이다. 직접 p156 쌍 비교에서도 본문과 그림 64가 더 이상 맞닿지 않으며, 그림과
caption이 같이 이동했다.

```text
issue_3821_page_tail_square_picture_wrap_reaches_visible_text_after_guides ... ok
issue_3821_square_picture_wrap_band_is_bounded_and_contiguous ... ok
issue_3820_rewinding_rowbreak_uses_painted_first_fragment_boundary ... ok
각각 1 passed; 0 failed
```

`output/task-3820-3821-fidelity/stage3-review-pairs/p156_rhwp_pdf.png`가 p156 직접
비교 증적이다.

그림 52는 p108에 여전히 caption만 있고 image가 누락된다. 이 문제는 left margin
paint-origin과 별개인 placement owner 결함이다. Stage 4에서 자동 탐지 도구의
text↔image overlap 판정과 함께 분석한다.
