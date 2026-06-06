# PDF/SVG visual sweep 가이드

## 목적

`scripts/task1274_visual_sweep.py`는 rhwp가 만든 SVG/render tree와 한컴 기준 PDF를 비교해
문항 흐름 drift, frame overflow, 줄 순서 겹침 같은 후보를 자동으로 찾는 보조 도구다.

이 도구는 메인테이너의 최종 시각 판정을 대체하지 않는다. 대신 다음을 빠르게 확인한다.

- SVG/PDF 페이지 수 일치
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
우선한다. 자세한 폰트 fallback 동작은 [export-png 명령 가이드](export_png_command.md)의
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
| `output/task1274/<target>/analysis/metrics.json` | 페이지별 후보 상세 |
| `output/task1274/<target>/analysis/question_flow.json` | 문항 marker 흐름 비교 |

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
