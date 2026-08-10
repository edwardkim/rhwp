---
kind: implementation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-10
---

# Task #3820 Stage 122 — 76076 p18--p19 rowspan tail boundary

## 입력·정답지

| 항목 | 값 |
| --- | --- |
| 시작 commit | `306b3164bd81` |
| 입력 HWP | `samples/76076_regulatory_analysis.hwp` |
| 독립 PDF oracle | `samples/issue1891/76076_regulatory_analysis-2024.pdf` |
| oracle 생성기 | Hancom PDF 1.3.0.550 / Hwp 2024 13.0.0.3622 |
| 대상 | 물리 p18--p19, source `(section=0, pi=173, ci=0)` 31×17 비-TAC `RowBreak` 표 |

## 관찰한 결함

한컴 PDF는 p18에서 row 13의 `11.영향평가 여부` 제목까지만 그린 뒤, row 14의
`해당 없음` 세 결과 칸을 p19 상단으로 온전히 넘긴다. RHWP는 row 14의 텍스트를 p18
하단에 그리고 p19에는 10.2px의 빈 tail만 남겼다. 그 결과 p19는 PDF에 있어야 할
`여부`/`해당 없음` 행 없이 `대분류`부터 시작한다.

`fidelity_compare` 0-based 17--19 direct pair와 render tree에서 RHWP의 첫 fragment는
`end_row=15`, `split_end_limit=21.8px`, continuation `start_cut=[1,1,1]`였다. PDF와
눈으로 대조한 결과 이것은 단순 raster 차이가 아니라 표 행 owner 오류다.

row 14를 온전히 p19로 이월한 뒤에는 둘째 결함도 드러났다. row 13--14를 덮는
cell `(r=13,c=1,rowspan=2)`은 저장 문단이 `11.영향평가`와 `여부`의 정확한 두 줄인데,
기존 높이 기반 rowspan cut은 첫 문단의 trailing line/문단 간격까지 row 13의 24.5px
예산에 포함했다. 그래서 p18에는 첫 문단이 사라지고 p19에는 두 문단이 통째로 다시
그려졌다. 한컴 PDF는 p18=`11.영향평가`, p19=`여부`로 source 문단 owner를 나눈다.

## 원인 분석

`scan_block_table_split_rows()`의 Stage 76 `RSPAN_BAND` 경로는 이전 행에서 시작한
rowspan이 현재 행을 덮고, 현재 행의 셀 내용이 남은 공간에 모두 들어가면 선언 행 높이의
빈 tail을 현재 쪽에 보존한다.

이 규칙은 76076 p35의 row 12 `주요내용`(declared 98.8px, visible 23.3px,
remaining 74.7px)을 위해 필요하다. 하지만 p18 row 14는 declared 32.0px,
visible 20.7px, remaining 21.8px라 visible content 뒤의 여유가 1.1px뿐이다.
즉 실제로 보존할 물리 blank tail이 없는데도 같은 경로가 `fully_consumed`를 이유로
split을 허용했다. PDF는 이 근소한 여유를 row-tail 보존으로 해석하지 않고 전체 row를
다음 페이지로 이월한다.

## 변경 계약

기존 Stage 76 경로는 다음을 모두 만족할 때만 허용한다.

1. 기존 native HWP5 `RowBreak`·prior-rowspan·비중첩·content fully consumed 조건을 유지한다.
2. 현재 쪽에 실제로 남겨지는 blank tail(`remaining - visible_height`)이
   `MIN_TOP_KEEP_PX`(25px) 이상이어야 한다.
3. 그 미만이면 row를 부분 소유하지 않고 일반 RowBreak 경계처럼 다음 페이지에서 다시
   시작한다.

이는 p35의 51.4px tail은 유지하고 p18의 1.1px pseudo-tail만 배제한다. 전역 행 높이,
font metric, row-break 일반 정책은 바꾸지 않는다.

두 행 경계에서 끊기는 HWP5 저장 pagination 계약의 exact 2행 rowspan/2문단/각 1 합성
line-unit 형상은 일반 높이 컷 대신 문단 경계를 owner로 쓴다. 이 fixture의 원문
`LINE_SEG`는 비어 있지만 두 문단의 합성 unit이 각각 하나라는 불변조건은 유지된다.
이때만 p18은 첫 unit `[0,1)`, p19는 둘째 unit `[1,2)`를 그린다. 원본 HWP와
HWP5-origin HWPX에는 같은 규칙을 적용해 roundtrip render 계약을 보존하며, 순수
HWPX·inline·중첩 control·다중줄·intra-row cut·일반 rowspan은 기존 높이 기반 경로를
유지한다.

## 검증 계획

- 새 76076 focused regression으로 p18에 row 14의 `해당 없음`이 없고 p19에 정확히
  세 칸이 있는 owner, 그리고 p18=`11.영향평가` / p19=`여부`의 cell `(13,1)` 문단
  owner를 고정한다.
- 기존 `issue_3820_rowbreak_rowspan_band`로 p35의 `주요내용` tail과 p36 재개를 유지한다.
- 코드 변경 뒤 첫 gate는 `issue_2430_cell_rewrap_threshold`, 이후 focused gates와 전체
  `cargo test --profile release-test --tests`, fmt/diff/clippy를 실행한다.
- 최신 binary로 p18--p20을 다시 PDF direct pair 비교해 p19 상단 row sequence를 판정한다.

## 구현·최종 검증 결과

- `issue_2430_cell_rewrap_threshold`: 2/2 통과.
- `issue_3820_rowbreak_rowspan_band`: 3/3 통과. 새 regression은 p18 row 14 결과칸
  미소유, p19 `해당 없음` 3개, cell `(13,1)`의 `11.영향평가` / `여부` 문단 owner를
  각각 직접 단언한다.
- `issue_1921_59043_pagination_pin`: p8 Square picture 회귀를 포함해 5/5 통과.
- HWP5-origin HWPX에도 같은 stored owner를 적용했으므로, `issue_1939` exact 회귀도
  통과했다. 순수 HWPX의 일반 rowspan 분할은 기존 높이 기반 경로를 유지한다.
- 전체 `CARGO_TARGET_DIR=target/task-3820-production-fidelity CARGO_INCREMENTAL=0
  cargo test --profile release-test --tests`는 502 test binary, 5,536 passed로 exit 0을
  확인했다. `cargo fmt --check`, `git diff --check`,
  `cargo clippy --all-targets -- -D warnings`도 exit 0이다.
- native-Skia는 lib 58/58, issue #2225 2/2, p37 관련 4/4를 통과했고,
  `wasm-pack build --target web --out-dir pkg`도 성공했다.
- 최신 release-test binary로 physical p18--p20을 Hancom 2024 PDF와 다시 대조했다.
  p18 table tail은 `11.영향평가`에서 끝나고 p19는 `여부`와 `해당 없음` 3칸 뒤
  `대분류`/`소분류`로 재개한다. 최신 direct pair의 text owner/sequence/page-boundary/
  visible-excess 후보는 모두 0건이며, p19 raster diff는 row owner 수정 전 17.96%에서
  12.02%로 낮아졌다. p18/p19 전체가 pixel-identical하다는 주장은 하지 않는다. 남은
  표 폭·글꼴/ink raster 차이는 이 stage의 row owner 결함과 별개다.

## 최종 시각 증적

- 입력 HWP SHA-256:
  `3308ba8505391bae2d0d62963e9399f4e48cdae574304cc0f89a311c6efbb6b5`
- Hancom 2024 PDF SHA-256:
  `06a389455d6b96e5f6580c9930fd8555256f9c712be85fb3cdaf31fc601a090d`
- 최종 sweep commit: `575da2a5dece0a7cedeebfd4579c58c734b6ffaf`; release-test
  binary SHA-256:
  `8718165b233567a8d411168e0d5c6681e9548c89c0a072b8e04dd85fad48a4b8`.
- direct pair(physical p18--p20):
  `output/task-3820-stage122-76076-p018-p020-575da2a-final/`. 요청 3/3 완료,
  p18/p19/p20의 owner·sequence·boundary·visible-excess 후보는 모두 0건이다.
- 3-way/OVL sweep(physical p18--p19):
  `output/task-3820-stage122-76076-p018-p019-575da2a-final/`. 82/82 SVG와 render
  tree를 export하고 요청 2/2를 raster·compare·review·overlay로 완료했다. 자동 flagged
  page는 0건이고, pixel match는 p18 89.74629%, p19 93.50343% (평균 91.62486%)다.
  visual accuracy proxy는 p18 18.92541%, p19 40.29382%로, 글꼴/ink 차이를 포함하므로
  범용 합격 점수가 아니라 보조 지표로만 기록한다.
- 직접 3-way 판독에서는 PDF와 동일하게 p18에 결과 행을 그리지 않고, p19 상단에
  `여부`와 세 `해당 없음` 칸이 먼저 나온 뒤 `대분류`/`소분류`가 재개함을 확인했다.
  이는 이 stage가 약속한 표 행·문단 owner 경계의 해결 근거다. 문서 전체의 최종
  사용자 시각 승인과는 별개다.

재현 명령:

```sh
RHWP_BIN=target/task-3820-production-fidelity/release-test/rhwp \
  venv/bin/python tools/fidelity_compare/fidelity_compare.py 17 19 \
  --source samples/76076_regulatory_analysis.hwp \
  --reference-pdf samples/issue1891/76076_regulatory_analysis-2024.pdf \
  --label task3820-stage122-76076-p018-p020-575da2a-final \
  --reference-grade '한컴 2024 기준 PDF' --layout-ledger \
  --out-dir output/task-3820-stage122-76076-p018-p020-575da2a-final

venv/bin/python scripts/visual_sweep.py \
  --key task3820-stage122-76076-p018-p019-575da2a-final \
  --hwp samples/76076_regulatory_analysis.hwp \
  --pdf samples/issue1891/76076_regulatory_analysis-2024.pdf \
  --pages 18-19 --dpi 144 \
  --rhwp-bin target/task-3820-production-fidelity/release-test/rhwp \
  --out output/task-3820-stage122-76076-p018-p019-575da2a-final
```

대표 최종 증적:

- [p18 3-way review](../pr/assets/task_m100_3820_stage122_76076_p018_p019_rowspan_boundary/review_p018_final.png),
  [p18 overlay](../pr/assets/task_m100_3820_stage122_76076_p018_p019_rowspan_boundary/overlay_p018_final.png)
- [p19 3-way review](../pr/assets/task_m100_3820_stage122_76076_p018_p019_rowspan_boundary/review_p019_final.png),
  [p19 overlay](../pr/assets/task_m100_3820_stage122_76076_p018_p019_rowspan_boundary/overlay_p019_final.png)
- [sweep provenance](../pr/assets/task_m100_3820_stage122_76076_p018_p019_rowspan_boundary/visual_sweep_manifest.json),
  [summary](../pr/assets/task_m100_3820_stage122_76076_p018_p019_rowspan_boundary/visual_sweep_summary.json),
  [overlay metrics](../pr/assets/task_m100_3820_stage122_76076_p018_p019_rowspan_boundary/overlay_metrics.json),
  [flagged pages](../pr/assets/task_m100_3820_stage122_76076_p018_p019_rowspan_boundary/flagged_pages.json)
