---
kind: implementation
status: active
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-09
---

# Task #3820 Stage 82 — 76076 p81–82 표 소유 경계 정합

## 범위와 기준

- 대상 HWP: `samples/76076_regulatory_analysis.hwp`
- 기준 PDF: `samples/issue1891/76076_regulatory_analysis-2024.pdf`
- 판정 기준: 한컴 2024 PDF의 실제 페이지 소유·줄바꿈이다. 페이지 수 또는 이전 RHWP
  산출물과의 비교로 대체하지 않는다.

## 재현 증적

다음 명령으로 RHWP와 기준 PDF의 p81–82를 직접 대조했다.

```sh
venv/bin/python tools/fidelity_compare/fidelity_compare.py 80 81 \
  --source samples/76076_regulatory_analysis.hwp \
  --reference-pdf samples/issue1891/76076_regulatory_analysis-2024.pdf \
  --label task3820-p081-p082 --reference-grade '한컴 2024 기준 PDF' \
  --text-only --export-all-svg --layout-ledger \
  --out-dir /tmp/rhwp-fidelity-76076-p081-p082

python3 scripts/visual_sweep.py \
  --rhwp-bin target/task-3820-stage80-hanyang-space/release-test/rhwp \
  --key task3820-stage82-pdf-audit \
  --hwp samples/76076_regulatory_analysis.hwp \
  --pdf samples/issue1891/76076_regulatory_analysis-2024.pdf \
  --pages 81-82 --dpi 180 --out /tmp/rhwp-stage82-pdf-audit
```

빈 문단 보정 전에는 p81의 `pi=842, ci=0` 표가 PDF보다 늦게 시작했다. 보정 뒤 PDF와 같이
p81은 `일시적/반복적` 행과 `근거설명` 첫 줄(`○ 구내운반차 … 사고`)까지 그린다. 그러나 p82는
그 첫 줄을 이어 그리지 않고 다시 paint한다. 따라서 남은 결함은 빈 문단 높이가 아니라
**중첩 표 행의 물리 페이지 continuation state가 부모 `PartialTable`에 전파되지 않는 중복**이다.

`fidelity_compare.py`에는 기존의 text·sequence·table fragment 원장을 하나로 결합한
`page-boundary-fidelity-candidates.tsv`를 추가했다. 현재 p81→82는 다음처럼 자동 후보화된다.

```text
81  82  table_fragment_text_owner_drift  rhwp_later_than_reference  39  39
pi=842,ci=0,rows=5,cols=2,same_pi_ci_adjacent_fragment|page_bottom_near_material_text_delta
```

같은 검사로 사용자가 지적한 p70→71도 9자 `text_owner_shift` 후보로 자동 수집된다. 이 원장은
후보 queue이며, 최종 결함 판정은 기준 PDF의 페이지 소유 대조로 유지한다.

현 visual sweep의 `layout-candidates.tsv`는 0건으로 남았으므로, 이 결함은 단순
line-flow/overlay 기준만으로는 자동 차단되지 않는다. Stage 82의 검증에는 표 행·줄의
페이지 소유를 직접 확인하는 회귀를 추가한다.

## 원인 가설과 분리

저장 `LINE_SEG`가 없고 공백 문자만 가진 문단에서 pagination은 글자모양·줄간격으로 계산한
32px을 사용했지만, SVG layout의 `ComposedLine` 0개 전용 분기는 400HU(약 5.3px)만 전진했다.
이 둘의 차이가 앞선 문단의 물리 y와 p842 시작 위치를 갈라놓았다. `height_measurer`,
`paragraph_layout`, `typeset` 모두에서 같은 true-empty/no-LineSeg fallback metric을 사용하도록
통일했다. HWP3의 기존 호환 cap은 HWP3 profile에만 유지한다.

이후 드러난 잔여 p81→82 중복은 `p842`의 outer `PartialTable`이 `startRow=0,endRow=3` / 다음 쪽
`startRow=3,endRow=5`로 분할되지만, 그 행 안의 nested table continuation은 `startCut/endCut=[]`로
남는 구조다. 즉 nested 셀의 paint는 p81까지 보이는데 부모 row는 p82에서 처음부터 다시 배치한다.
다음 단계는 이 재귀 RowCut/height 전파를 원인 정정한다. 표에 남은 공간을 느슨하게 하거나 PDF
페이지 소유를 baseline으로 바꾸는 방식은 사용하지 않는다.

## 수행 계획

1. `p842` 중첩 셀의 실제 paint 높이와 parent row의 declared height가 갈라지는 지점을 재귀
   `RowCut` 생성 경로에서 계측한다.
2. parent `PartialTable`이 nested continuation을 소유하도록 일반화하고, p81의 첫 줄이 p82에서
   중복되지 않으며 p82가 `를 예방함으로써 …`로 이어지는 회귀를 추가한다.
3. p70–71의 owner 이동도 PDF 대조해 같은 원인인지 분리하고, Stage 80 p35–36 및 전체
   `release-test` 검증을 함께 통과시킨다.
