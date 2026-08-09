---
kind: investigation
status: active
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-10
---

# Task #3820 Stage 106 — 정책연구 p78–p79 시각 oracle 직접 판정

## 범위와 시작 상태

- 브랜치: `task/3820-production-fidelity`
- 시작 commit: `3c87142ed`
- 입력: `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp`
- 한컴 2020 기준 PDF:
  `pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf`
- 선행 전수 원장: `output/task-3820-stage102-policy-current-ledger/`

Stage 105는 셀 분할 저장본 provenance를 별도 커밋으로 종결했다. 이 stage는
최신 215쪽 원장에서 `visible-text-excess-candidates.tsv`의 가장 큰 연속 후보인
p78–p79를 한컴 PDF raster와 직접 비교한다. 자동 원장은 후보 선별용이며,
그 자체를 결함 확정 근거로 쓰지 않는다.

## 초기 신호

- PDF/SVG/render tree 쪽수: 215/215/215
- page-boundary, text owner-shift, text owner-sequence: 0건
- `visible_svg_only`: p78 905자, p79 1734자
- p78→p79 table fragment에 text delta 신호 존재

이 조합은 실제 표 fragment 배치 결함일 수도 있고, PDF 텍스트 추출이 표 셀의
시각 문자를 누락한 것일 수도 있다. 따라서 실제 page raster, overlay, review PNG에서
다음을 판정한다.

1. 표의 행·셀 경계와 분할 순서가 PDF와 같은지
2. 문단이 표 경계를 침범하거나 잘리지 않는지
3. 각주·가로줄·쪽번호가 정확한 쪽에 있는지
4. 시각적 문자는 존재하지만 PDF 추출만 누락한 오탐인지

## 진행 계획

1. 최신 CLI로 1-based p78–p79(0-based 77–78)의 PDF/SVG raster·overlay를 생성한다.
2. 페이지별 review PNG를 분리해 확대 대조한다.
3. 오탐이면 원장 판별 한계를 기록하고 다음 후보로 이동한다.
4. 실제 결함이면 source owner/fragment 계약을 추적한 뒤 최소 수정과 회귀를
   같은 stage에 기록한다.

## 직접 비교 결과

`CARGO_TARGET_DIR=target/pr-review`, `CARGO_INCREMENTAL=0`으로 최신 CLI를 빌드한
뒤 2/2쪽 raster·overlay·layout ledger를 생성했다.

| 페이지 | pixel diff | PDF 직접 판정 |
| --- | ---: | --- |
| p78 | 15.07% | 표 25 first fragment, 각주 105–106, 표/각주 비중첩 일치 |
| p79 | 19.00% | 표 25 continuation, 각주 107–111, 표 bottom·각주 lane 일치 |

- PDF/SVG/render tree의 전체 쪽수는 215/215/215이다.
- p78→p79의 `same_pi_ci_adjacent_fragment` 신호는 정상적인 페이지 연속 표다.
- `layout-candidates.tsv`의 두 페이지는 body/footnote, table/footer, cell text overlap,
  border/text clip 모두 0건이다.
- 전수 원장의 `visible_svg_only` 905/1734자는 표 셀 문자의 PDF text extraction
  누락으로, 직접 raster에서는 PDF와 rhwp 모두 같은 문자를 표시한다.
- 글꼴·자간 raster 차이는 남아 있지만 표 행 owner, 문단 잘림, 각주 owner,
  페이지 경계 결함은 없다.

증적:

- [p78 PDF/rhwp 직접 비교](../pr/assets/task_m100_3820_stage106_policy_p078_p079_visual_oracle/compare_p078.png)
- [p79 PDF/rhwp 직접 비교](../pr/assets/task_m100_3820_stage106_policy_p078_p079_visual_oracle/compare_p079.png)
- [pixel report](../pr/assets/task_m100_3820_stage106_policy_p078_p079_visual_oracle/report.tsv)
- [page-count ledger](../pr/assets/task_m100_3820_stage106_policy_p078_p079_visual_oracle/page-count-ledger.tsv)
- [table-fragment ledger](../pr/assets/task_m100_3820_stage106_policy_p078_p079_visual_oracle/table-fragment-candidates.tsv)
- [visible-text ledger](../pr/assets/task_m100_3820_stage106_policy_p078_p079_visual_oracle/visible-text-excess-candidates.tsv)

## 판정과 다음 대상

p78·p79는 최신 산출물에서도 한컴 PDF와 같은 표/각주 물리 owner를 가지므로
재수정 대상에서 제외한다. 기존 문서·최신 render tree·PDF 직접 비교를 합친
다음 실제 잔여 결함은 **p87에 한컴 PDF의 각주 138이 있지만 rhwp
`FootnoteArea`가 없는 소실**이다. 이 원인·수정은 Stage 107로 분리한다.
