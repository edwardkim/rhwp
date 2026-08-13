---
kind: investigation
status: active
canonical: mydocs/manual/verification/visual_verification_governance.md
last_verified: 2026-08-12
---

# Task M100 #3820 Stage 169 - #4491 p26의 폰트 비교 가능 충실도 경계

## 목적

Stage 168 이후 남아 있는 #4491 physical p26 PDF 차이가 추가 페이지 나눔 변경을 정당화할
수 있는 렌더러 레이아웃 회귀인지 판단한다.

## 입력과 실행

- 원본: `samples/issue4491/30213_1.혼합단지등 제도개선 방안.hwp`
- 기준: `pdf/issue4491/30213_1-혼합단지등-제도개선-방안-hancom2020.pdf`
- 렌더러: `target/task-3820-stage168/release-test/rhwp`
- 비교: 원본/기준 쌍에 `tools/fidelity_compare/fidelity_compare.py 25 25`를 직접 적용했다.
  fidelity 도구는 zero based이므로 index `25`는 physical p26이다. 이전 도움말과 달리
  `dump-pages -p`도 zero based다.
- 산출물 루트: `output/task-3820-stage169-issue4491-p26/`

## 관찰 결과

| 점검 항목 | 결과 |
| --- | --- |
| p26 pixel 차이 | `26.31%` |
| physical page owner | HWP 2020 PDF와 페이지 및 item 순서가 같다 |
| p26 flow | 표 `pi=337`, `341`, `344`는 inline(`treatAsChar:true`)이며, empty-host RowBreak declared-height gate는 관여하지 않는다 |
| p26 geometry | 비교 시트에서 세 표 상자, 제목, 본문 문단, footer가 overlap이나 clipping 없이 원본 소유 페이지에 유지된다 |
| 기준 PDF 폰트 | `Gulim`, `HCRDotum`, `DejaVuSerif` |
| SVG 원본 face | `한양중고딕`은 `HCR Dotum` local-face 후보를 경유한다 |
| local face 가용성 | `fc-match 'HCR Dotum'`은 `Verdana`로 해석되고, `fc-match 'HY중고딕'`은 설치된 `HYGothic-Medium`(`H2GTRM.TTF`)으로 해석된다 |
| portable face 입력 | `RHWP_FONT_PATH_DIR`는 설정되지 않았고, 추적되는 `ttfs`, `fonts`, `resources` 아래에는 HCR/Haansoft Dotum asset이 없다 |

p26 비교 시트는 page ownership과 box geometry가 일치하지만 glyph weight와 raster width는
다르다. 사용할 수 없는 HCR 후보 다음으로 `HYGothic-Medium`을 선택할 수는 있지만, 이는
기준 PDF에 포함된 HCR Dotum face가 아니다. Stage 75는 이 실제 HY face만 선택해도 관련된
76076 p33--p36 fidelity gap이 해결되지 않음을 독립적으로 확인했다. PDF text layer에서는
p26 텍스트가 추출되지 않고 SVG에는 790자가 있으므로, 이 페이지의 text ledger는 owner나
line-break oracle로 사용할 수 없다.

## Stage 168 컴파일 완료

이월된 Stage 168 구현에는 새 p26 페이지 나눔 가설과 무관한 컴파일 오류 네 건이 있었다.

- `saved_bounds_fit_at_flow_tail` 호출 지점 세 곳에서 새 `bottom_spill` 인자를 빠뜨렸다.
  모든 일반 saved-line fit은 이제 명시적으로 `0.0`을 전달한다.
- mid-page RowBreak scope는 branch-local `saved_span`을 참조하는 대신, declared-height
  branch와 동일한 첫 번째 real LineSeg와 table vertical offset으로 saved object bottom을
  다시 계산한다.

## 검증

- `cargo build --profile release-test --target-dir target/task-3820-stage168`가 성공적으로
  완료됐다.
- `cargo test --profile release-test --target-dir target/task-3820-stage168 --test issue_4490_4491_anchor_flow -- --nocapture`가
  통과했다: 2 passed, 0 failed.

## 결정

이 p26 raster 비교를 근거로 pagination, table splitting, text measurement를 변경하지
않는다. 이후 p26 acceptance 비교는 문서화된 font-path contract나 동등한 라이선스 보유
local installation을 통해 Chrome에 동일한 HCR Dotum face를 제공해야 한다. 현재 가용한
font environment에서도 결함이 보이는 독립 후보로 #3820 작업을 계속한다.
