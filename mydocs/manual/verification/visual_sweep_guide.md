---
kind: guide
status: active
canonical: mydocs/manual/verification/visual_verification_governance.md
last_verified: 2026-07-16
---

# PDF/SVG visual sweep 가이드

## 목적

`scripts/task1274_visual_sweep.py`는 rhwp가 만든 SVG/render tree와 한컴 기준 PDF를 비교해
문항 흐름 drift, frame overflow, 줄 순서 겹침 같은 후보를 자동으로 찾는 보조 도구다.

이 도구는 메인테이너의 최종 시각 판정을 대체하지 않는다. 대신 다음을 빠르게 확인한다.

- SVG/PDF 페이지 수 일치
- PNG/PDF raster overlay 차이 위치
- 페이지별 픽셀/잉크 영역 일치율
- 문항 marker y drift 후보
- frame/tail overflow 후보
- 수식/본문 겹침 후보
- 줄 band/order drift 후보

## 필수 도구

스크립트는 실행 시작 시 다음 CLI가 `PATH`에 있는지 확인한다.

| CLI | 용도 | Ubuntu/WSL/Debian 패키지 |
|---|---|---|
| `rsvg-convert` | SVG를 PNG로 변환 | `librsvg2-bin` |
| `pdftoppm` | PDF 페이지를 PNG로 변환 | `poppler-utils` |
| `pdftotext` | PDF bbox-layout 추출 | `poppler-utils` |

주의: 패키지명은 `libsvg2-bin`이 아니라 `librsvg2-bin`이다.

설치 예:

```bash
sudo apt update
sudo apt install librsvg2-bin poppler-utils
```

macOS Homebrew 환경:

```bash
brew install librsvg poppler
```

Fedora 계열:

```bash
sudo dnf install librsvg2-tools poppler-utils
```

설치 확인:

```bash
which rsvg-convert
which pdftoppm
which pdftotext
```

## 폰트 환경

visual sweep은 SVG를 PNG로 변환한 결과와 한컴 기준 PDF raster를 비교한다. 따라서
폰트 환경이 다르면 실제 레이아웃 회귀가 없어도 `line`, `column`, `order` 후보가
false positive로 남을 수 있다.

권장 기본 폰트:

```bash
sudo apt install fonts-noto-cjk fonts-nanum
fc-list :lang=ko | head
```

한컴/HY 계열 전용 폰트는 라이선스가 있는 로컬 환경에서만 사용하고, 저장소나 PR
첨부물에 포함하지 않는다. 정확한 한컴 기준 재현이 필요한 경우 프로젝트 외부의 폰트
디렉터리를 사용한다.

```bash
rhwp export-svg samples/exam_kor.hwp \
  --font-path /path/to/ttfs \
  --output output/font-check/
```

`--font-path`는 여러 번 지정할 수 있으며, 기본 탐색 경로(`ttfs/`, 시스템 폰트)보다
우선한다. 자세한 폰트 fallback 동작은 [export-png 명령 가이드](../export_png_command.md)의
폰트 섹션을 참고한다.

현재 `scripts/task1274_visual_sweep.py`는 `export-svg` 호출에 `--font-path`를 전달하지
않는다. 자동 sweep은 시스템 fontconfig와 기본 탐색 경로 기준으로 실행되므로,
폰트 민감 문서는 다음 중 하나로 판정한다.

- 컨트리뷰터와 메인테이너가 동일한 공개 한글 폰트 환경을 맞춘 뒤 sweep 실행
- `rhwp export-svg --font-path ...`로 수동 SVG를 내보내고 별도 시각 판정
- 필요 시 후속 작업으로 sweep 스크립트에 반복 가능한 `--font-path` 전달 옵션 추가

PR 보고서에는 폰트 민감 판정일 경우 OS, 공개 한글 폰트 설치 여부, 한컴/HY 전용
폰트 사용 여부를 함께 적는다.

## 사전 빌드

현재 checkout 기준 `target/debug/rhwp`가 필요하다.

```bash
cargo build
```

## 실행

전체 교육 통합 target sweep:

```bash
python3 scripts/task1274_visual_sweep.py --target all
```

특정 target만 실행:

```bash
python3 scripts/task1274_visual_sweep.py --target 2024-09-between20
```

특정 페이지만 비교:

```bash
python3 scripts/task1274_visual_sweep.py \
  --target 2024-09-between20 \
  --page 22 \
  --out output/visual-p22
```

여러 페이지 또는 범위만 비교:

```bash
python3 scripts/task1274_visual_sweep.py \
  --hwp /path/to/input.hwpx \
  --pdf /path/to/baseline.pdf \
  --pages 43-46 \
  --out output/visual-p43-46
```

`--page`는 여러 번 지정할 수 있고, `--pages`는 `1,3,5-7` 형식을 허용한다. 페이지 번호는
사용자가 PDF viewer에서 보는 1-based 번호다. 현재 구현은 문서 전체 SVG/PDF raster 산출물을 만든 뒤
비교·overlay·analysis 단계만 선택 페이지로 좁힌다. 따라서 `compare/compare_022.png`,
`overlay/overlay_022.png`, `analysis/annotated_022.png`처럼 실제 페이지 번호가 파일명에 남는다.

저장소 preset에 없는 일반 파일을 실행:

```bash
python3 scripts/task1274_visual_sweep.py \
  --key so-sueop \
  --hwp samples/SO-SUEOP.hwpx \
  --pdf pdf/SO-SUEOP-2024.pdf \
  --out output/visual-so-sueop
```

`--hwp`에는 `.hwp`와 `.hwpx` 모두 지정할 수 있다. 파일을 `samples/`나 `pdf/`로 복사하지 않아도
된다. 절대 경로와 현재 checkout 기준 상대 경로를 모두 허용한다. `--key`를 생략하면 문서 파일명
stem을 target 이름으로 사용한다.

여러 일반 파일을 한 번에 실행:

```bash
python3 scripts/task1274_visual_sweep.py \
  --file-target so-sueop samples/SO-SUEOP.hwpx pdf/SO-SUEOP-2024.pdf \
  --file-target pr1674 samples/pr-1674.hwpx pdf/pr-1674-2024.pdf \
  --out output/visual-custom
```

preset target과 일반 파일 target을 섞을 수도 있다.

```bash
python3 scripts/task1274_visual_sweep.py \
  --target 2024-09-between20 \
  --file-target so-sueop /path/to/SO-SUEOP.hwpx /path/to/SO-SUEOP-2024.pdf
```

일반 파일에서도 특정 페이지만 비교할 수 있다.

```bash
python3 scripts/task1274_visual_sweep.py \
  --key so-sueop-p22 \
  --hwp samples/SO-SUEOP.hwpx \
  --pdf pdf/SO-SUEOP-2024.pdf \
  --page 22 \
  --out output/visual-so-sueop-p22
```

일부 공개문서 축약 샘플은 rhwp SVG/PNG 파일명이 문서 내부 원래 페이지 번호나 문서번호를 따라가고,
기준 PDF는 해당 페이지만 잘라낸 단일 페이지라 `pdf-1.png`로 생성될 수 있다. 예를 들어 rhwp 쪽은
`rhwp_177.png`인데 기준 PDF는 `pdf-1.png`인 경우다. 이때 `--page 1`처럼 사용자가 PDF viewer에서 보는
단일 페이지를 지정했고, SVG/render tree/rhwp PNG/PDF PNG 산출물이 모두 1개뿐이면 visual sweep은 자동으로
이 단일 산출물을 1:1 매칭한다. 출력 파일명은 rhwp 산출물의 실제 번호를 따라 `compare_177.png`,
`overlay_177.png`, `review_177.png`처럼 남을 수 있으므로 리뷰 문서에는 이 대응 관계를 함께 적는다.

현재 스크립트의 기본 output:

```text
output/task1274/
```

주요 산출물:

| path | 설명 |
|---|---|
| `output/task1274/summary.json` | 전체 target 요약 |
| `output/task1274/<target>/svg/` | rhwp SVG export |
| `output/task1274/<target>/rhwp_png/` | SVG를 PNG로 변환한 결과 |
| `output/task1274/<target>/pdf_png/` | PDF를 PNG로 변환한 결과 |
| `output/task1274/<target>/compare/` | rhwp/PDF 비교 이미지 |
| `output/task1274/<target>/overlay/` | rhwp/PDF PNG overlay diff 이미지와 metrics |
| `output/task1274/<target>/overlay/overlay_metrics.json` | overlay diff 페이지별 지표. manifest에는 요약이 포함됨 |
| `output/task1274/<target>/review/` | `compare`와 `overlay`를 한 장에 나란히 붙인 검토 이미지 |
| `output/task1274/<target>/analysis/metrics.json` | 페이지별 후보 상세 |
| `output/task1274/<target>/analysis/question_flow.json` | 문항 marker 흐름 비교 |
| `output/task1274/<target>/overlay_contact_sheet.png` | overlay diff 전체 요약 이미지 |
| `output/task1274/<target>/review_contact_sheet.png` | 나란히 보기 전체 요약 이미지 |

## Codex 보고 규칙

Codex가 visual sweep을 실행해 특정 페이지를 검토할 때는 결과 설명만 하지 말고, 항상 다음 세 가지를
함께 제공한다.

- `compare/compare_{page}.png` 절대 경로
- `overlay/overlay_{page}.png` 절대 경로
- `review/review_{page}.png` 절대 경로
- 해당 페이지의 `visual_accuracy_proxy_percent`

또한 Codex 화면에는 `review_{page}.png`를 먼저 열어 `compare`와 `overlay`를 한 화면에 나란히 보여준다.
필요하면 `compare_{page}.png`와 `overlay_{page}.png` 개별 파일도 추가로 연다. `compare`는 좌우 배치로
전체 시각 차이를 보고, `overlay`는 빨강/파랑/주황 차이 위치를 판단하는 용도다.
`review_{page}.png`에서는 overlay 비교 PNG 바로 아래에
`코멘트: 내용 픽셀 중심 자동 일치율 보조값 = 약 N%.` 한 줄만 포함한다.

Codex 응답에서 이미지를 보여준 바로 아래에는 반드시 한국어 코멘트를 붙인다. 코멘트는 다음 4줄 형식을 따른다.
`visual_accuracy_proxy_percent` 값은 백분율로 환산해 첫 줄에 표시한다.

보고 예:

```text
page 22
- compare: /private/tmp/.../compare/compare_022.png
- overlay: /private/tmp/.../overlay/overlay_022.png
- review: /private/tmp/.../review/review_022.png
- visual_accuracy_proxy_percent: 91.23456

코멘트: 내용 픽셀 중심 자동 일치율 보조값 = 약 91.23%.
높을수록 좋음: 기준 PDF와 rhwp PNG가 더 비슷함
낮을수록 나쁨/검토 필요: 잉크 위치나 형태 차이가 큼
단, 사람 판정 정확도가 아니라 내용 픽셀 중심 자동 일치율 보조값입니다
```

예를 들어 값이 `13.8381`이면 다음처럼 적는다.

```text
코멘트: 내용 픽셀 중심 자동 일치율 보조값 = 약 13.84%.
높을수록 좋음: 기준 PDF와 rhwp PNG가 더 비슷함
낮을수록 나쁨/검토 필요: 잉크 위치나 형태 차이가 큼
단, 사람 판정 정확도가 아니라 내용 픽셀 중심 자동 일치율 보조값입니다
```

페이지별 값을 빠르게 확인:

```bash
jq '.pages[] | {page, overlay_png, visual_accuracy_proxy_percent}' \
  output/task1274/<target>/overlay/overlay_metrics.json
```

특정 페이지 한 개만 확인:

```bash
jq '.pages[] | select(.page == 22) | {page, overlay_png, visual_accuracy_proxy_percent}' \
  output/task1274/<target>/overlay/overlay_metrics.json
```

## PNG overlay 비교

스크립트는 각 페이지에 대해 `rhwp_png`와 `pdf_png`를 같은 canvas 크기로 padding한 뒤, RGB 채널 차이가
`--pixel-diff-threshold`보다 큰 픽셀을 overlay 이미지로 표시한다. 기본 임계값은 `32`다.

```bash
python3 scripts/task1274_visual_sweep.py \
  --hwp /path/to/input.hwpx \
  --pdf /path/to/baseline.pdf \
  --pixel-diff-threshold 32 \
  --out output/visual-one
```

overlay 색상 의미:

| 색상 | 의미 |
|---|---|
| 회색 | 임계값 이하로 거의 같은 픽셀 |
| 빨강 | rhwp 쪽에만 잉크가 있거나 rhwp가 더 많이 그린 후보 |
| 파랑 | PDF 쪽에만 잉크가 있거나 PDF 기준에만 보이는 후보 |
| 주황 | 양쪽 모두 잉크가 있지만 위치/색상 차이가 큰 후보 |
| 연분홍 | 배경/anti-aliasing 계열 차이 후보 |

`overlay/overlay_metrics.json`에는 다음 보조 지표가 기록된다.

| 필드 | 의미 |
|---|---|
| `pixel_match_percent` | 전체 canvas 픽셀 중 임계값 이하로 일치한 비율 |
| `ink_match_percent` | 양쪽 중 하나라도 내용 픽셀인 영역에서 일치한 비율 |
| `visual_accuracy_proxy_percent` | 자동 시각 판정 보조 일치율. 잉크 영역이 있으면 `ink_match_percent`, 없으면 `pixel_match_percent` |
| `diff_bbox` | 차이가 난 픽셀들의 bounding box |
| `mean_abs_channel_delta` | RGB 채널 평균 절대 차이 |
| `max_channel_delta` | 페이지 내 최대 RGB 채널 차이 |

주의: `visual_accuracy_proxy_percent`는 사람이 내린 정답/오답 판정에 대한 실제 정확도가 아니다.
PDF raster와 rhwp raster가 얼마나 비슷한지를 보여주는 자동 보조 지표다. 여백이 넓은 문서는
`pixel_match_percent`가 과하게 높게 나올 수 있으므로 실제 판정에는 `overlay_contact_sheet.png`,
페이지별 `overlay_*.png`, `ink_match_percent`, 기존 `analysis/metrics.json` 후보를 함께 본다.

계산 의미:

- 각 픽셀에서 RGB 채널 최대 차이가 `--pixel-diff-threshold` 이하이면 일치로 본다.
- `pixel_match_percent = 100 * (1 - diff_pixels / total_pixels)` 이다.
- `ink_match_percent = 100 * (1 - ink_diff_pixels / ink_union_pixels)` 이다.
- `visual_accuracy_proxy_percent`는 잉크 영역이 있으면 `ink_match_percent`, 잉크 영역이 없으면
  `pixel_match_percent`를 쓴다.

따라서 이 값은 "자동 시각 판정 정확도"가 아니라 "내용 픽셀 중심 raster 일치율"에 가깝다. 폰트,
anti-aliasing, PDF rasterizer, 전체 위치 이동의 영향을 크게 받으므로, 낮은 값은 우선 검토 신호이지
그 자체로 불합격 판정은 아니다.

## 결과 해석

실행 중 출력 예:

```text
analysis: 2024-09-between20 flagged=1/24 frame=[] red=[] line=[11] column=[11] eq=[] title=[] order=[11] tail=[] question=[]
summary: /path/to/rhwp/output/task1274/summary.json
```

핵심 필드:

| 필드 | 의미 |
|---|---|
| `flagged` | 후보가 감지된 페이지 수 / 전체 분석 페이지 수 |
| `overlay_metrics` | PNG overlay 기반 픽셀/잉크 일치율 요약 |
| `frame` | 편집 frame 밖 overflow 후보 |
| `red` | 빨간 문항 marker drift 후보 |
| `line` | 페이지 전체 line band drift 후보 |
| `column` | 단별 line band drift 후보 |
| `eq` | 수식/본문 겹침 후보 |
| `title` | 문항 제목/본문 겹침 후보 |
| `order` | 줄 순서 겹침 후보 |
| `tail` | render tree 기준 tail overflow 후보 |
| `question` | PDF/rhwp 문항 marker y drift 후보 |

권장 판정 기준:

- `svg_pages == pdf_pages`는 기본 조건이다.
- `overlay_contact_sheet.png`에서 빨강/파랑/주황이 본문 흐름에 집중되면 우선 검토한다.
- `visual_accuracy_proxy_percent`는 자동 일치율 지표일 뿐 최종 시각 판정을 대체하지 않는다.
- PR 의 실제 변경 목적을 먼저 확인한다. 렌더링 개선 PR 이 아니면 visual sweep 차이는 참고 자료이며,
  그 차이만으로 merge 보류나 reject 결론을 내리지 않는다.
- `frame`, `question`, `title`, `tail`, `eq` 후보는 우선 검토 대상이다.
- `line`, `column`, `order` 후보는 실제 시각 차이인지 false positive인지 비교 이미지를 열어 확인한다.
- 후보가 남아도 메인테이너 SVG/웹/한컴 시각 판정이 통과하면 blocker가 아닐 수 있다.

요약만 빠르게 보기:

```bash
jq -r '.[] | [.key, .svg_pages, .pdf_pages, (.visual_metrics.flagged_page_count // 0), (.visual_metrics.frame_overflow_pages|join(",")), (.visual_metrics.line_band_drift_pages|join(",")), (.visual_metrics.column_line_band_drift_pages|join(",")), (.visual_metrics.line_order_overlap_pages|join(",")), (.visual_metrics.question_marker_drift_pages|join(","))] | @tsv' output/task1274/summary.json
```

## PR에 기록할 때

PR 리뷰/보고서에는 다음을 분리해 적는다.

- 설치/환경 문제로 실행하지 못한 경우: 어떤 CLI가 없는지 명시
- 실행 완료한 경우: target별 페이지 수와 후보 페이지를 표로 기록
- 후보가 남은 경우: 메인테이너 시각 판정과 blocker 여부를 별도로 기록

예:

```markdown
| target | SVG/PDF pages | flagged | frame | line | column | order | question |
|---|---:|---:|---|---|---|---|---|
| `2024-09-between20` | 24/24 | 1 | `[]` | `[11]` | `[11]` | `[11]` | `[]` |
```

## 한계

- PDF는 한컴 편집기 직접 시각 판정의 완전한 대체물이 아니다.
- 폰트/anti-aliasing 차이 때문에 line/column/order 후보가 false positive로 남을 수 있다.
- 최종 수용 여부는 자동 sweep + 회귀 테스트 + 메인테이너 시각 판정을 함께 보고 결정한다.
