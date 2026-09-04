---
kind: investigation
status: active
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-10
---

# Task #3820 Stage 101 — 표 경계 침범·SVG 글자 절단 자동 후보

## 목적

Stage 100에서 분리한 두 사용자-visible 결함을 `fidelity_compare.py`의 빠른 전수 원장으로 만든다.

1. 표 셀 문단이 우측선 등 자신의 Cell 경계를 넘어가는 결함
2. page/body/cell SVG clip이 첫·마지막 glyph band를 부분적으로 잘라내는 결함

자동 검출은 결함을 확정하지 않는다. 후보 페이지와 bbox를 줄인 뒤 한컴 기준 PDF raster로 판정한다.

## 시작 상태

- 시작 commit: `edf823777`
- 기존 text multiset 원장은 같은 글자가 같은 페이지에 있으면 표 우측선 침범을 볼 수 없다.
- 기존 overflow/hidden-text 원장은 clip 밖에 완전히 남은 stale continuation과 실제로 일부만 잘린
  glyph를 구분하지 못했다.
- Stage 96·#4138 작업 파일은 별도 변경이므로 stage/revert하지 않는다.

## 검출 계약

### `table-cell-text-boundary-candidates.tsv`

- render tree의 `Cell` bbox와 visible `TextLine` bbox를 먼저 비교한다.
- `TextLine` 자체가 좌·우·상·하 중 2px 이상 넘으면 `line_boundary_overflow`를 기록한다.
- 자연 `TextRun` 폭은 저장 자간/justify 적용 전 값일 수 있으므로 `natural_visible_width_risk`로
  분리한다. overflowing edge가 실제로 그리지 않는 선행/후행 공백뿐이면 후보에서 제외한다.
- 중첩 `Cell`은 외부 셀에 귀속하지 않고 자기 소유로 다시 검사한다.
- 공백/private marker와 셀에서 완전히 분리된 stale continuation은 제외한다.

### `svg-text-band-clip-candidates.tsv`

- SVG의 명시적 `clipPath` rect 교집합과 `<text>`의 근사 ink band
  `(baseline - 0.8em)..(baseline + 0.2em)`을 비교한다.
- 상·하단이 2px 이상 **부분 절단**된 text만 기록한다.
- 완전히 clip 밖인 stale text, hidden/blank text, x 범위가 분리된 text, 안전하게 축 정렬 좌표를
  계산할 수 없는 transform text는 제외한다.
- 한 줄이 glyph별 `<text>`로 나뉘면 여러 후보 행이 생길 수 있으므로 같은 baseline·clip·edge로 묶어
  시각 검토한다.

## 실물 검증

### 76076 p34 — 해결 상태와 detector 오탐 교정

다음 direct pair를 0-based page 33에 실행했다.

```bash
RHWP_BIN=target/stage98-clean/release-test/rhwp \
venv/bin/python tools/fidelity_compare/fidelity_compare.py 33 33 \
  --source samples/76076_regulatory_analysis.hwp \
  --reference-pdf samples/issue1891/76076_regulatory_analysis-2024.pdf \
  --label stage101-76076-p34 --reference-grade '한컴 2024 기준 PDF' \
  --text-only --export-all-svg --layout-ledger \
  --out-dir /tmp/rhwp-stage101-76076-p34
```

첫 구현은 aggregate `TextRun.bbox`를 실제 최종 paint 폭으로 잘못 간주해 physical p34에서 4건을
냈다. 직접 SVG/PDF 대조 결과 이는 현재 renderer 결함이 아니라 detector 오탐이었다.

- nested Cell right: `701.3px`
- visible TextLine right: `694.5px` — 저장 margin `6.8px` 유지
- 오탐 run의 자연 bbox right: 최대 `708.8px`
- 같은 줄의 최종 SVG 마지막 visible glyph right: 약 `693.7px`

원인은 justify 문단의 후행 공백까지 포함한 자연 run 폭이었다. SVG/Canvas는 저장 문자 위치를
적용해 실제 glyph를 선 안쪽에 그리고, 기존 `issue_2308_nested_non_tac_table_keeps_saved_horizontal_cell_margin`
회귀도 visible TextLine이 border보다 6px 이상 안쪽임을 고정한다. 따라서 renderer를 바꾸지 않고 detector를
위의 line/visible-ending 2단계 계약으로 좁혔다. 재실행한 p34 원장은 header만 남아 현재 해결 상태와
일치한다.

### Stage 96 p65 — 보수적 band 오탐 교정

보존 SVG
`mydocs/pr/assets/task_m100_3820_stage96_issue2279_r27_continuation/stage96-issue2279-r27-final/svg/86712_regulatory_analysis_065.svg`
에 첫 구현을 적용했을 때 `body-clip-3` 상단 `y=75.6`에 대해 baseline `y=91.6`, font-size 20인
첫 줄이 상단 4.0px 부분 절단 후보로 기록됐다. 그러나 이 값은 visibility용
`baseline-font_size` envelope를 실제 ink로 오인한 결과였다. 현재 head와 한컴 PDF physical p65를
직접 비교하면 제목 glyph 상단은 잘리지 않는다. partial-clip용 band를 `-0.8em..+0.2em`으로 좁히고
동일 수치를 반례로 고정한 뒤 재실행하면 후보가 0건이다.

## 회귀 검증

- synthetic Cell 경계: TextLine 직접 침범과 visible-ending 자연 폭 위험 검출
- current p34 수치 반례: 자연 run bbox는 넘지만 line에 6.8px 저장 margin이 있어 후보 제외
- 2px 미만/blank/detached/nested 반례 제외
- synthetic SVG clip: 0.8em/0.2em top/bottom partial clip 검출, wholly clipped/transform 반례 제외
- Stage 96 p65 body clip 수치 반례: 실제 ink band가 clip 안이면 후보 제외
- Python fidelity test suite 55건과 `py_compile`을 통과시킨다.

## 다음 stage

Stage 98 전수 PDF 후보에서 다음 실제 불일치 페이지를 선택한다. p34는 renderer를 추가 보정하지 않고
현재 saved-margin 회귀와 clean 후보 원장으로 보호한다.
