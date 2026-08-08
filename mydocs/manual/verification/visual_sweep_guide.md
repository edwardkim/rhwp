---
kind: guide
status: active
canonical: mydocs/manual/verification/visual_verification_governance.md
last_verified: 2026-08-08
---

# PDF/SVG visual sweep 가이드

## 목적

`scripts/visual_sweep.py`는 rhwp가 만든 SVG/render tree와 한컴 기준 PDF를 비교해
문항 흐름 drift, frame overflow, 줄 순서 겹침 같은 후보를 자동으로 찾는 보조 도구다.

이 도구는 메인테이너의 최종 시각 판정을 대체하지 않는다. 대신 다음을 빠르게 확인한다.

- SVG/PDF 페이지 수 일치
- PNG/PDF raster overlay 차이 위치
- 페이지별 픽셀/잉크 영역 일치율
- 문항 marker y drift 후보
- frame/tail overflow 후보
- 수식/본문 겹침 후보
- 줄 band/order drift 후보
- **그림·float 주변의 본문이 좁은 세로 열로 재흐름한 단별 text-flow collapse 후보**
- **Square/Tight/Through 그림의 물리 box를 본문 TextLine이 3행 이상 가로지르거나 edge에 맞닿는 후보**
- **우측 Body 표의 좌측 strip은 PDF에 본문 잉크가 있지만 rhwp에서 거의 비는 non-inline table wrap 후보**
- **구조 heuristic에 걸리지 않는 glyph·PUA·제품명 표시 차이**의 review 후보

이 도구의 절차상 지위는 [시각 검증 거버넌스의 라우팅 표](visual_verification_governance.md)를
따른다. 독립 기준 PDF와 실제 사용자-visible 실패를 조사할 때는 bug-hunter가 상위이고, sweep은
후보 검출·재현 범위 축소·수정 전후 무회귀만 담당한다. 이미 원인과 발동 페이지가 확정된 renderer/layout
PR에서는 이 가이드를 직접 시작점으로 쓴다.

한컴 기준 PDF가 있는 실물 문서에서는 먼저
`tools/fidelity_compare/fidelity_compare.py --text-only --export-all-svg --layout-ledger`로 전수 후보를
수집한다. visual sweep은 그 중 Square/Tight/Through 그림↔본문 기하 규칙을 같은 render tree에서
직접 재사용하여 `square_wrap_text_overlap` flag와 annotation으로 남긴다. 반면 PDF↔SVG text owner,
그림 앞 문단의 `float-owner-shift`, 표 fragment, page-count ledger는 sweep이 다시 계산하지 않으므로
fidelity 원장을 함께 보존해야 한다.
이 bridge의 render tree가 없거나 JSON이 손상되면 sweep은 `flagged=0`을 보고하지 않고 실패해야 한다.
`fidelity_compare`의 Python 환경과 실행 명령은
[도구 README](../../../tools/fidelity_compare/README.md)를 따른다. 저장소 로컬 `venv/`의 공통
계약은 그 문서가 연결하는 [개발 환경 가이드](../dev_environment_guide.md)가 정의한다.

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

현재 `scripts/visual_sweep.py`는 `export-svg` 호출에 `--font-path`를 전달하지
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
python3 scripts/visual_sweep.py --target all
```

특정 target만 실행:

```bash
python3 scripts/visual_sweep.py --target 2024-09-between20
```

특정 페이지만 비교:

```bash
python3 scripts/visual_sweep.py \
  --target 2024-09-between20 \
  --page 22 \
  --out output/visual-p22
```

여러 페이지 또는 범위만 비교:

```bash
python3 scripts/visual_sweep.py \
  --hwp /path/to/input.hwpx \
  --pdf /path/to/baseline.pdf \
  --pages 43-46 \
  --out output/visual-p43-46
```

`--page`는 여러 번 지정할 수 있고, `--pages`는 `1,3,5-7` 형식을 허용한다. 페이지 번호는
사용자가 PDF viewer에서 보는 1-based 번호다. `export-svg`와 render tree 추출은 문서 단위로 수행하지만,
`--page`/`--pages`가 지정되면 `rsvg-convert`와 `pdftoppm`의 raster 생성부터 선택 페이지로 제한한다.
비교·overlay·analysis도 동일한 선택 페이지로만 수행한다. 따라서 `compare/compare_022.png`,
`overlay/overlay_022.png`, `analysis/annotated_022.png`처럼 실제 페이지 번호가 파일명에 남는다.

저장소 preset에 없는 일반 파일을 실행:

```bash
python3 scripts/visual_sweep.py \
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
python3 scripts/visual_sweep.py \
  --file-target so-sueop samples/SO-SUEOP.hwpx pdf/SO-SUEOP-2024.pdf \
  --file-target pr1674 samples/pr-1674.hwpx pdf/pr-1674-2024.pdf \
  --out output/visual-custom
```

preset target과 일반 파일 target을 섞을 수도 있다.

```bash
python3 scripts/visual_sweep.py \
  --target 2024-09-between20 \
  --file-target so-sueop /path/to/SO-SUEOP.hwpx /path/to/SO-SUEOP-2024.pdf
```

일반 파일에서도 특정 페이지만 비교할 수 있다.

```bash
python3 scripts/visual_sweep.py \
  --key so-sueop-p22 \
  --hwp samples/SO-SUEOP.hwpx \
  --pdf pdf/SO-SUEOP-2024.pdf \
  --page 22 \
  --out output/visual-so-sueop-p22
```

작은 글자나 셀 clip 경계를 확대 판정할 때는 `--dpi 144`처럼 목표 DPI를 높일 수
있다. 이 값은 PDF raster와 rhwp SVG raster 양쪽에 같은 배율로 적용된다. 기본값은
96dpi이며 0 이하 값은 허용하지 않는다.

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
python3 scripts/visual_sweep.py \
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

### glyph·PUA·제품명 표시 차이도 잡는 방법

기존 `analysis.flagged`는 frame overflow, line/order drift처럼 구조적으로 규칙화한 후보만 집계했다.
현재 sweep은 여기에 render tree의 옛자모·PUA `TextRun` bbox를 PDF/SVG raster에 대조하는
`legacy_glyph_visual_mismatch`도 더한다. 해당 bbox의 잉크가 충분하고 국소 일치율이 80% 이하이면
페이지를 flag하고, `legacy_glyph_visual_candidates`에 text·code point·render-tree/raster bbox·국소
잉크 지표를 남긴다.

그래도 **`flagged=0`은 모든 glyph·글자폭·자간·제품명 convention이 PDF와 같다는 뜻이 아니다.**
이 자동 후보는 옛자모·PUA로 범위를 의도적으로 좁혔으므로, 일반 글꼴·색·자간 차이는 별도 review가
필요하다. 같은 bbox 안에서 `ᄒᆞᆫ글`과 `한글`처럼 다른 glyph를 paint하거나, PUA/글머리표의 의미가
달라질 때 구조 heuristic만으로는 0건일 수 있다.

다음은 합격 임계값이 아니라, 이런 차이를 놓치지 않기 위한 필수 triage다.

1. 대상 페이지의 `overlay_metrics.json`을 `visual_accuracy_proxy_percent` 오름차순으로 정렬한다.
   가장 낮은 페이지와 옛자모·PUA·목록 marker가 있는 페이지는 `flagged` 값과 무관하게 review PNG를
   연다.
2. review의 좌우 비교에서 glyph 모양, 글자폭, baseline, bullet/번호가 다르면 overlay가 구조 flag를
   내지 않아도 **시각 fidelity 후보**로 기록한다.
3. 해당 SVG/render tree의 `TextRun.text`, raw code point, CharShape/font family, 문단 context를
   확인한다. 원문 IR의 보존과 paint-time display projection은 별도 계약이다.
4. PDF text layer 추출이 실패했으면 문자 멀티셋 대조를 "차이 없음"으로 처리하지 않는다. raster
   review와 raw/render-tree 대조만으로 후보를 남기고, text layer 한계를 함께 기록한다.

`legacy_glyph_visual_mismatch`는 불합격을 자동 확정하는 규칙이 아니라 review 우선순위다. `analysis/`
annotated PNG에는 해당 후보 bbox가 보라색으로 표시된다. 한 페이지에 후보가 여러 개면 국소
`ink_match_percent`가 낮은 순으로 최대 20개를 기록한다.

페이지별 보조값의 낮은 순서를 보는 예시는 다음과 같다.

```bash
jq -r '.pages[] | [.page, .visual_accuracy_proxy_percent, .ink_match_percent, .pixel_match_percent] | @tsv' \
  output/task1274/<target>/overlay/overlay_metrics.json \
  | sort -t $'\t' -k2,2n
```

예를 들어 HWP 97 안내문의 `가. ᄒᆞᆫ글 드라이버 사용`은 rhwp render tree에 표준 옛자모가
남아 있고, 기준 PDF에는 제품명 `한글`로 보일 수 있다. 이 경우 HWP3 parser만 바꾸거나 모든
`ᄒᆞᆫ`을 현대화하면 실제 옛한글 문서를 훼손할 수 있다. **문서·CharShape·PDF로 증명된 제품명
context만** paint-time projection 후보로 삼고, 일반 옛한글과 다른 문서는 negative regression으로
보호한다.

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
| `flowcollapse` | 같은 단에서 PDF와 line-band 수와 y 흐름이 함께 크게 달라진 본문 flow 붕괴 후보 |
| `eq` | 수식/본문 겹침 후보 |
| `title` | 문항 제목/본문 겹침 후보 |
| `order` | 줄 순서 겹침 후보 |
| `tail` | render tree 기준 tail overflow 후보 |
| `question` | PDF/rhwp 문항 marker y drift 후보 |
| `glyph` | 옛자모·PUA TextRun의 국소 raster 불일치 후보 |
| `wrap` | fidelity와 같은 Square/Tight/Through 그림↔본문 physical-overlap 또는 edge-clearance-loss 후보 |
| `tablewrap` | 우측 Body table 좌측 strip의 PDF 본문 잉크가 rhwp에서 소실된 non-inline wrap 후보 |

권장 판정 기준:

- `svg_pages == pdf_pages`는 기본 조건이다.
- `overlay_contact_sheet.png`에서 빨강/파랑/주황이 본문 흐름에 집중되면 우선 검토한다.
- `visual_accuracy_proxy_percent`는 자동 일치율 지표일 뿐 최종 시각 판정을 대체하지 않는다.
- `flagged=0`이어도 낮은 `visual_accuracy_proxy_percent` 또는 옛자모·PUA·목록 marker가 있으면
  review/overlay를 반드시 확인한다. glyph·제품명 표시 차이는 이 경로에서 후보가 된다.
- PR 의 실제 변경 목적을 먼저 확인한다. 렌더링 개선 PR 이 아니면 visual sweep 차이는 참고 자료이며,
  그 차이만으로 merge 보류나 reject 결론을 내리지 않는다.
- `frame`, `question`, `title`, `tail`, `eq` 후보는 우선 검토 대상이다.
- `tail`은 render tree의 page bbox를 **현재 raster DPI 좌표**로 투영한 뒤, 해당 bbox에
  실제 rhwp 잉크가 있는 TextLine만 세어 만든다. 페이지 밖에만 남은 continuation node나
  ancestor clip으로 보이지 않는 node는 tail 후보가 아니다. 따라서 고 DPI sweep의 `tail`은
  render-tree 논리 좌표만으로는 재현·판정하지 않는다.
- `wrap`은 Square/Tight/Through 그림이 본문 흐름 영역과 outer clearance를 예약해야 하는 계약의 강한
  후보다. annotation의 `candidate_kind`가 `physical_overlap`이면 image/첫·마지막 교차 line bbox를,
  `edge_clearance_loss`이면 image edge와 최소 clearance를 PDF review와 즉시 대조한다. 후자는 HWP
  outer margin 유실로 glyph와 그림 테두리가 맞닿는 결함을 포착한다. 의도된 overlay·zero-margin source와
  render-tree source 정보의 한계가 있으므로 자동 불합격이나 PDF 정답 판정으로 승격하지 않는다.
- `wrap` 판정에 필요한 render tree가 빠지거나 손상된 run은 clean 결과가 아니라 **infrastructure
  failure**다. tree export를 복구한 뒤 다시 실행한다.
- `tablewrap`은 `Table`이 Body의 오른쪽에 있고, 해당 table의 세로 범위에서 Body 좌측 strip의 PDF
  content ink density가 `0.025` 이상인데 rhwp ink가 그 15% 이하일 때만 후보가 된다. 따라서 단순
  우측 정렬 standalone table(양쪽 strip이 비어 있음)과 소폭 font raster 차이는 제외한다. HWPX
  non-inline Square table이 다음 문단 prefix를 소실하는 형상을 빠르게 찾는 용도이며, PDF review로
  wrap 의도를 확인하기 전에는 자동 결함 확정이 아니다.
- `flowcollapse`은 본문이 그림 옆의 비정상적인 세로 열로 분해되는 회귀를 우선 올리는 강한 후보다.
  자동 불합격은 아니지만 `review`와 PDF를 즉시 대조한다.
- `flowcollapse` 계산은 render tree의 **Body bbox**를 우선 frame으로 쓰고, Body table 영역을 양쪽
  raster에서 제외한다. 테두리 없는 페이지의 넓은 table rule이나 cell raster 분할이 본문 line band로
  오인되는 false positive를 막기 위한 것이며, 표 row fragment·owner가 같다는 뜻은 아니다. 표가 관련된
  페이지는 `fidelity_compare` text ledger, table/footer/frame 후보와 3-way review를 별도로 확인한다.
- `line`, `column`, `order` 후보는 실제 시각 차이인지 false positive인지 비교 이미지를 열어 확인한다.
- 후보가 남아도 메인테이너 SVG/웹/한컴 시각 판정이 통과하면 blocker가 아닐 수 있다.

요약만 빠르게 보기:

```bash
jq -r '.[] | [.key, .svg_pages, .pdf_pages, (.visual_metrics.flagged_page_count // 0), (.visual_metrics.frame_overflow_pages|join(",")), (.visual_metrics.line_band_drift_pages|join(",")), (.visual_metrics.column_line_band_drift_pages|join(",")), (.visual_metrics.column_text_flow_collapse_pages|join(",")), (.visual_metrics.square_wrap_text_overlap_pages|join(",")), (.visual_metrics.right_table_left_strip_text_deficit_pages|join(",")), (.visual_metrics.line_order_overlap_pages|join(",")), (.visual_metrics.question_marker_drift_pages|join(",")), (.visual_metrics.legacy_glyph_visual_pages|join(","))] | @tsv' output/task1274/summary.json
```

## PR에 기록할 때

PR 리뷰/보고서에는 다음을 분리해 적는다.

- 설치/환경 문제로 실행하지 못한 경우: 어떤 CLI가 없는지 명시
- 실행 완료한 경우: target별 페이지 수와 후보 페이지를 표로 기록
- 후보가 남은 경우: 메인테이너 시각 판정과 blocker 여부를 별도로 기록

예:

```markdown
| target | SVG/PDF pages | flagged | frame | line | column | wrap | tablewrap | order | question | glyph |
|---|---:|---:|---|---|---|---|---|---|---|---|
| `2024-09-between20` | 24/24 | 1 | `[]` | `[11]` | `[11]` | `[]` | `[]` | `[11]` | `[]` | `[]` |
```

## 한계

- PDF는 한컴 편집기 직접 시각 판정의 완전한 대체물이 아니다.
- 폰트/anti-aliasing 차이 때문에 line/column/order 후보가 false positive로 남을 수 있다.
- 표의 같은-page geometry·row fragment는 `flowcollapse`만으로 판정하지 않는다. 이 신호는 표 영역을
  의도적으로 mask하므로, PDF text owner 차이와 render tree/table geometry를 함께 대조해야 한다.
- `wrap`은 80px 이상 Square/Tight/Through 이미지와 image 폭의 절반 이상을 가로지르는 Body TextLine
  3행 이상(`physical_overlap`), 또는 image 왼쪽/오른쪽 edge에서 `≤1px`로 맞닿거나 얕게 침범하는 3행
  이상(`edge_clearance_loss`)을 후보화한다. 1–2행·Body 밖 text·PDF와 위치만 다른 경우는 놓칠 수 있어,
  fidelity text/table 원장 및 PDF review의 대체물이 아니다.
- `tablewrap`은 PDF/rhwp raster와 render-tree Table bbox를 함께 요구하므로 PDF가 없는 자체 회귀
  검증에는 적용할 수 없다. 우측 표의 left strip에 의도적인 빈 여백이 있더라도 PDF도 비어 있으면
  후보가 되지 않지만, PDF의 그림·색면처럼 본문 이외 잉크가 strip을 채운 경우는 review에서 제외한다.
- 반대로 glyph·글자폭·PUA/제품명 convention 차이는 구조 후보가 0건이어도 실제 fidelity 결함일 수
  있다. 옛자모·PUA는 `legacy_glyph_visual_mismatch`로 우선 후보화하지만, 낮은 잉크 일치율과
  review의 반복 차이는 raw/IR/paint 경로로 분리한다.
- PDF text layer가 손상되거나 추출기에 실패하면 text 기반 자동 분류를 생략할 수 있다. 이 경우
  raster overlay와 render tree를 사용하되, 문자 멀티셋 무차이를 주장하지 않는다.
- 최종 수용 여부는 자동 sweep + 회귀 테스트 + 메인테이너 시각 판정을 함께 보고 결정한다.
