---
kind: implementation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-08
---

# Task #3820 Stage 75 — 한양중고딕 실제 face 연결 및 PDF 재검증

## 입력 계약

Stage 74가 현재 `76076_regulatory_analysis.hwp` p33--p36에서 다시 확정한 결함은
table box width가 아니라 cell text paint의 폭이다. HWP layout은 `한양중고딕`의
`HanyangJungGothic` 실측 advance로 줄을 나누지만, macOS SVG paint는 해당 face가 없어
`Malgun Gothic` fallback glyph를 그린다. p34 nested cell의 마지막 paint line은 outer
border보다 0.8px 안쪽까지만 layout 여유를 갖기 때문에 fallback glyph 폭이 그대로
표시되면 border를 침범하고 행 높이·page fragment가 PDF와 달라진다.

2026-08-08에 Windows 기준 font `C:\\Windows\\Fonts\\H2GTRM.TTF`를 확인해 Mac,
`ubuntu-hwp`, `ubuntu-ted`의 사용자 font directory에 같은 파일을 설치하고 font cache를
갱신했다. 파일 내부 family는 `HYGothic-Medium`/`HY중고딕`이며, 원 문서가 요청하는
`한양중고딕`은 fontconfig에서 자동 동의어가 아니다. 설치 직후 `fc-match 한양중고딕`은
Mac에서 Verdana, Ubuntu에서 기본 sans를 반환했다. 즉 설치만으로는 PDF 기준 face가
선택되지 않는다.

## 현재 경로와 제약

`render_font_family_chain()`은 종전에 `한양중고딕`과 `HY중고딕` 모두를 `Malgun Gothic`으로
보완했다. 실제 Hanyang font가 설치된 host에서는 이 순서가 원 face를 선택하지 못하므로,
`한양중고딕 → HY중고딕 → 기존 generic fallback`으로 바꾼다. `HY중고딕` 자체가 요청된
경우에는 종전처럼 `Malgun Gothic`을 fallback으로 남긴다. 이는 native face가 없는 host의
마지막 fallback 순서를 바꾸지 않고 실제 face가 있는 host에서만 원 glyph를 우선한다.

처음에는 한글 cluster에 SVG `textLength`를 강제하는 가설도 준비했지만, 실제 font를 연결하기
전에 glyph 폭을 인위적으로 늘이거나 줄이면 정확한 원인을 가릴 수 있다. 따라서 그 실험 코드는
커밋하지 않고 되돌렸으며, 실제 face 연결의 PDF 대조 결과가 남은 차이를 보일 때만 별도 Stage에서
다룬다.

## 구현·검증 계획

1. SVG/Canvas font family chain에 `한양중고딕 → HY중고딕` 별칭을 추가하고 chain unit test를
   갱신한다.
2. Mac과 두 Ubuntu host에서 `fc-scan H2GTRM.TTF` 및 `fc-match HY중고딕`으로 설치를 확인한다.
3. p33--p36 180 DPI PDF sweep을 Stage 74와 같은 input/provenance로 다시 수행한다.
   p34의 table border와 visible lines를 PDF panel로 판정한다.
4. focused font-chain test와 `issue_2308_render_normalized_derived_state`,
   `issue_1891`, `overflow_cell_baseline`을 실행한다. 원 face 선택이 page owner를 악화시키거나
   overflow baseline을 늘리면 별칭을 폐기한다.

## 중단 조건

- p34 PDF 직접 대조에서 실제 face 선택이 table outer border 침범을 줄이지 못하면, 이 Stage의
  별칭 가설을 폐기하고 source-charshape 또는 cell padding 경로를 별도 분석한다.
- `HY중고딕`이 설치되지 않은 host의 generic fallback 또는 기존 ASCII `textLength` snapshot이
  바뀌면 범위를 다시 좁힌다.

## 실행 결과 (2026-08-08)

### 실제 font 설치

Windows의 font registry에서 `HY중고딕 (TrueType) => H2GTRM.TTF`를 확인했다. 동일 파일을
다음 사용자 font directory에 설치하고 `fc-cache -f`를 완료했다.

| host | 설치 경로 | `fc-scan` family | `fc-match HY중고딕` |
| --- | --- | --- | --- |
| macOS | `/Users/tsjang/Library/Fonts/H2GTRM.TTF` | `HYGothic-Medium`, `HY중고딕` | 설치 파일 |
| ubuntu-hwp | `/home/ubuntu/.local/share/fonts/rhwp-hancom/H2GTRM.TTF` | 동일 | 설치 파일 |
| ubuntu-ted | `/home/tsjang/.local/share/fonts/rhwp-hancom/H2GTRM.TTF` | 동일 | 설치 파일 |

`한양중고딕`은 세 host의 fontconfig에서 이 파일의 자동 동의어가 아니었다. 이 때문에
SVG/Canvas chain에 `한양중고딕 → HY중고딕`을 명시했다. 독립 SVG probe에서는
`한양중고딕`과 `HY중고딕` text raster가 동일하고 `Malgun Gothic`과는 다름을 확인해,
실제 Hanyang glyph 선택 여부를 분리했다.

### PDF 직접 대조

새 release-test binary로 아래 sweep을 완주했다.

```text
python3 scripts/visual_sweep.py \\
  --rhwp-bin target/task-3820-stage75-hanyang-glyph-fit/release-test/rhwp \\
  --hwp samples/76076_regulatory_analysis.hwp \\
  --pdf samples/issue1891/76076_regulatory_analysis-2024.pdf \\
  --pages 33-36 --dpi 180
```

- 요청 4쪽과 SVG/PDF/PNG 4쪽 모두 산출됐다. `review_033.png`--`review_036.png`,
  `metrics.json`, `overlay_metrics.json`, `summary.json`은
  `mydocs/pr/assets/task_m100_3820_stage75_hanyang_font_environment/`에 보관한다.
- 기존 자동 후보는 0쪽이었다. 이 문서의 행 밀도·page fragment 차이는 여전히 해당 자동
  판정 범위 밖이며, 자동 통과를 fidelity 통과로 해석할 수 없다.
- Stage 74와 Stage 75의 각 p33--p36 RHWP panel을 동일 좌표로 비교한 결과가 모두
  pixel-identical이다. 따라서 실제 font 설치·명시 alias는 host의 font 선택을 재현 가능하게
  만들었지만, Stage 74에서 관측된 PDF page/row 차이를 줄이지 못했다.

### 회귀 게이트

- `renderer::tests::test_base_family_without_weight_suffix`: 통과.
- `issue_2308_render_normalized_derived_state`, `issue_1891`,
  `overflow_cell_baseline`: 모두 통과. overflow-cell baseline은 `678 fixtures`,
  `17 nonzero documents`, `691 lines`로 기존 기준을 유지했다.

## 결론·다음 Stage 인계

이 Stage는 font가 없는 환경이라는 가설을 배제했다. `HYGothic-Medium`을 사용해도 p33--p36
page/row fidelity는 변하지 않았다. 다음 Stage는 SVG paint가 아니라 nested table의
fragment-height·row owner 결정 경로와 PDF의 table fragment boundary를 분석 대상으로 삼는다.
