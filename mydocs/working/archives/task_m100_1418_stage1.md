# Stage 1 완료 보고서 — Task M100-1418 기준점 확정

- 이슈: #1418
- 제목: 첫 페이지 글상자와 표 상단 중앙 overlap 정합
- 작성일: 2026-06-16
- 브랜치: `local/task_m100_1418`
- 참조 PDF: `pdf-large/hwpx/2026_oss_rst.pdf`
- 참조 PDF SHA-256: `bec53a601cc7714a40ca340d26f971d1ab49eeb355682fc9db7b15cd5e04c86e`

## 1. 기준 산출물

참조 PDF가 교체되어 산출물을 다시 생성했다. 교체 후 PDF와 현재 rhwp 출력을 비교하기 위해
다음 산출물을 사용한다.

| 구분 | 경로 |
|---|---|
| 참조 PDF 1페이지 PNG | `output/poc/task1418-reference-pdf/page001.png` |
| 참조 PDF 1페이지 SVG 변환 | `output/poc/task1418-reference-pdf/page001.svg` |
| 참조 PDF 6페이지 PNG 확인 | `output/poc/task1418-reference-pdf/page006.png` |
| HWP 현재 SVG(오버레이 없음) | `output/poc/task1418-stage1-current-hwp/2026_oss_rst_001.svg` |
| HWP 현재 PNG(오버레이 없음) | `output/poc/task1418-baseline-png/hwp_page001_no_overlay.png` |
| HWP debug SVG | `output/poc/task1418-baseline-hwp/2026_oss_rst_001.svg` |
| HWP render tree | `output/poc/task1418-render-tree-hwp/render_tree_001.json` |
| HWPX 현재 SVG | `output/poc/task1418-stage1-current-hwpx/2026_oss_rst.svg` |
| HWPX render tree | `output/poc/task1418-render-tree-hwpx/render_tree_001.json` |

참조 PDF 메타데이터:

- Author: `edward`
- Creator: `Hwp 2020 11.0.0.9083`
- Producer: `Hancom PDF 1.3.0.550`
- A4 세로, `595 x 841 pt`
- 6페이지

현재 rhwp 로드 결과:

- `samples/2026_oss_rst.hwp`: 6페이지
- `samples/hwpx/2026_oss_rst.hwpx`: 1페이지

## 2. 정답 배치

참조 PDF 1페이지는 큰 1x1 안내 표의 상단 테두리 중앙을 흰 글상자가 덮고,
그 위에 제목 `< 결과보고서 작성 안내 >`를 그리는 구조다. 글상자는 표를 밀어내는 flow
객체가 아니라 표 상단선 위에 올라가는 `InFrontOfText` 성격의 객체로 판단된다.

PDF SVG 좌표를 rhwp SVG 좌표계(96dpi px, `pt * 4 / 3`)로 환산하면 다음과 같다.

| 항목 | PDF 좌표 | rhwp px 환산 |
|---|---:|---:|
| 큰 표 좌측 | `84.914pt` | `113.2px` |
| 큰 표 우측 | `510.207pt` | `680.3px` |
| 큰 표 상단 | `841 - 725.926 = 115.074pt` | `153.4px` |
| 큰 표 하단 | `841 - 87.625 = 753.375pt` | `1004.5px` |
| 제목 흰 배경 x | `203.770pt` | `271.7px` |
| 제목 흰 배경 y | `99.852pt` | `133.1px` |
| 제목 흰 배경 w | `187.102pt` | `249.5px` |
| 제목 흰 배경 h | `28.527pt` | `38.0px` |

따라서 정답 기준에서는 제목 흰 배경이 `y≈133.1..171.1px` 영역을 차지하고,
큰 표 상단선은 그 가운데쯤인 `y≈153.4px`를 지나야 한다.

## 3. 현재 HWP 배치

`dump-pages` 기준 첫 페이지:

```text
body_area: x=113.4 y=132.3 w=566.9 h=876.8
FullParagraph  pi=0  h=21.3
Shape          pi=0 ci=2  wrap=InFrontOfText tac=false
Table          pi=1 ci=0  1x1  566.9x852.0px  wrap=TopAndBottom tac=true
```

`dump` 기준 글상자:

- 위치: 가로 `Paper + Center`, 세로 `Paper + Top + 35.3mm`
- 크기: `66.0mm x 10.0mm`
- 배치: `InFrontOfText`, `treat_as_char=false`, `z=10`
- render tree 배경 박스: `x=271.9 y=133.4 w=249.4 h=37.8`
- 내부 TextBox: `x=275.6 y=137.2 w=241.9 h=30.2`

`dump` 기준 큰 표:

- `pi=1 ci=0`
- 1행 1열, `150.0mm x 225.4mm`
- `treat_as_char=true`
- `wrap=TopAndBottom`
- `vpos=1600`

현재 render tree의 큰 표:

- Table bbox: `x=113.4 y=132.3 w=566.9 h=853.8`
- 상단 Line bbox: `x=113.4 y=132.3 w=566.9 h=0.5`
- 하단 Line bbox: `x=113.4 y=986.1 w=566.9 h=0.5`

즉 제목 흰 배경은 참조 PDF와 거의 같은 위치에 있으나, 큰 표 상단선은 참조 PDF의
`y≈153.4px`가 아니라 현재 `y=132.3px`에 그려진다. 차이는 약 `21.1px`이며,
이는 직전 빈 문단 `pi=0`의 높이 `21.3px`와 일치한다.

## 4. HWPX 샘플 판정

`samples/hwpx/2026_oss_rst.hwpx`는 첫 페이지가 `출품작 중복수혜 여부 확인서`로 시작한다.
교체 후 참조 PDF의 1페이지는 `결과보고서 작성 안내`이고 6페이지는 `새로 생성된 가중치`
항목으로 시작하므로, HWPX 샘플은 참조 PDF와 직접 대응하지 않는 별도 문서 조각으로 판단한다.

`ir-diff samples/hwpx/2026_oss_rst.hwpx samples/2026_oss_rst.hwp --summary --max-lines 80`
결과도 총 103건 차이를 보였다. 주요 차이는 section count, controls count, text,
line segment, table size 등이다.

따라서 이번 이슈의 primary fixture는 다음 조합으로 고정한다.

- 입력: `samples/2026_oss_rst.hwp`
- 정답 참조: `pdf-large/hwpx/2026_oss_rst.pdf` 1페이지

HWPX 샘플은 이번 overlap 정합의 직접 기준이 아니라 별도 fixture 또는 보조 조사 대상으로 둔다.

## 5. 결론

Stage 1 기준 결론:

1. 글상자 자체의 paper-relative 위치와 크기는 참조 PDF와 거의 맞다.
2. 정답은 글상자 흰 배경 중앙을 큰 표 상단선이 지나가는 형태다.
3. 현재 rhwp는 큰 표를 `body_area.y=132.3px`에서 시작시켜, 표 상단선이 글상자 배경의 위쪽 가장자리로 올라간다.
4. 표 시작 위치는 직전 빈 문단 높이 `21.3px`만큼 내려간 `y≈153.4px`가 되어야 한다.
5. 결함 후보는 글상자 anchor/z-order가 아니라, `InFrontOfText` shape가 있는 빈 문단 뒤의 TAC/TopAndBottom 표 배치에서 앞 문단 높이 또는 line segment `vpos=1600`을 반영하지 않는 경로다.

다음 단계는 Stage 2에서 HWP5 IR과 렌더러 배치 경로를 분리한 뒤, 구현 계획서에서 수정 지점을
`src/renderer/layout.rs`, `src/renderer/typeset.rs`, `src/renderer/layout/shape_layout.rs` 중
어디로 한정할지 결정한다.

## 6. 실행 명령

```bash
pdfinfo pdf-large/hwpx/2026_oss_rst.pdf
shasum -a 256 pdf-large/hwpx/2026_oss_rst.pdf
pdftoppm -f 1 -singlefile -png -r 144 pdf-large/hwpx/2026_oss_rst.pdf output/poc/task1418-reference-pdf/page001
pdftoppm -f 6 -l 6 -singlefile -png -r 144 pdf-large/hwpx/2026_oss_rst.pdf output/poc/task1418-reference-pdf/page006
pdftocairo -svg -f 1 -l 1 pdf-large/hwpx/2026_oss_rst.pdf output/poc/task1418-reference-pdf/page001.svg
target/debug/rhwp dump-pages samples/2026_oss_rst.hwp -p 0
target/debug/rhwp dump samples/2026_oss_rst.hwp -s 0 -p 0
target/debug/rhwp dump samples/2026_oss_rst.hwp -s 0 -p 1
target/debug/rhwp export-render-tree samples/2026_oss_rst.hwp -p 0 -o output/poc/task1418-render-tree-hwp
target/debug/rhwp export-svg samples/2026_oss_rst.hwp -p 0 -o output/poc/task1418-stage1-current-hwp
target/debug/rhwp export-render-tree samples/hwpx/2026_oss_rst.hwpx -p 0 -o output/poc/task1418-render-tree-hwpx
target/debug/rhwp ir-diff samples/hwpx/2026_oss_rst.hwpx samples/2026_oss_rst.hwp --summary --max-lines 80
```
