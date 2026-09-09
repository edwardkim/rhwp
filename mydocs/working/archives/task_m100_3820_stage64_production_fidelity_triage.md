---
kind: investigation
status: active
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-08
---

# Task #3820 Stage 64 — 잔여 production PDF fidelity triage

## 목적과 기준 상태

`#3820`은 PR #4257(`e919655a`)에서 해결한 정책 연구 문서의 page-owner와
issue2007 p7–p17 중첩 표 범위 때문에 자동으로 닫혔다가, 별도 실문서 fidelity 범위를
분리하기 위해 다시 열렸다. 이 Stage는 해결된 두 범위를 다시 수정하지 않고, 현재
`upstream/devel`(`e919655a78d5928cdf7236152fce04d6aa6f6377`) 위에서 남은 입력을
독립적으로 재현·분류한다.

상위 절차는 bug-hunter다. 한컴 PDF는 provenance가 기록된 비교 기준이며, 자동
fidelity ledger와 visual sweep은 후보를 좁히는 수단이다. 자동 지표·페이지 수·회귀
통과만으로 PDF fidelity를 확정하지 않고, 후보마다 PDF review를 한다.

## 입력·기준 PDF provenance

| 축 | 입력 | 기준 PDF | SHA-256 |
| --- | --- | --- | --- |
| A — 알려진 nested fragment | `samples/76076_regulatory_analysis.hwp` | `samples/issue1891/76076_regulatory_analysis-2024.pdf` | HWP `3308ba8505391bae2d0d62963e9399f4e48cdae574304cc0f89a311c6efbb6b5`; PDF `06a389455d6b96e5f6580c9930fd8555256f9c712be85fb3cdaf31fc601a090d` |
| B — production HWP | `samples/2025 행정업무운영 편람(최종).hwp` | `pdf/2025 행정업무운영 편람(최종)-2024.pdf` | HWP `40d6d05eac4d55bdc4b0c62c42d93af104d5123b447581246f36fd15de7bd46f`; PDF `2cf19014c2835d3ca14014cc7f08c03850c2dc3b85c606bf4d70d864b1c568ef` |
| C — production HWPX | `samples/2025 행정업무운영 편람(최종).hwpx` | 같은 PDF | HWPX `c6dd7e847a99f219681afc5a29c80a9665c04df9cda4d820a3350d739664fdf6`; PDF 위와 같음 |

기준 PDF는 383쪽, 555×752pt 한컴 2024 출력물이다. HWP/HWPX를 같은 원인으로
가정하지 않으며, source format·render tree·PDF owner가 모두 맞을 때만 공통 보정으로
합친다.

## 먼저 확정할 재현

축 A의 기존 PDF 직접 관찰은 다음과 같다.

- p33→p34: 연속 표 하단/상단의 문단이 각 physical page 경계에서 잘린다.
- p34: 첫 표의 우측 외곽선까지 문단이 침범한다.

Stage 30은 동일 fixture에서 비글자 중첩 표의 fragment 단위 폭을 실제 padding과 맞춰
p33/p34 clip을 보정했다. 이후 direct HWPX canonical projection과 issue2007/3637 보정이
여러 차례 병합되었으므로, **Stage 30 결론을 현행 결론으로 재사용하지 않는다.** 현재
release-test binary와 독립 PDF에서 먼저 다시 재현한다.

## triage 순서

1. 전용 target(`target/task-3820-stage64-production-fidelity`)으로 현행 binary를 빌드한다.
2. 축 A p33–p34를 `fidelity_compare` ledger와 180 DPI visual sweep으로 동시에 대조한다.
   `svg-table-*-clip`, `table-cell-text-overlap`, `table-fragment`, `overflowCellLines`와
   review PNG를 함께 보존한다.
3. 축 B/C는 383쪽 전체에 대해 먼저 `--text-only --export-all-svg --layout-ledger`를 실행해
   page-count, text owner, table fragment/border/overlap 후보 원장을 만든다. 이 pass의
   후보만 raster/visual sweep으로 확대한다.
4. PDF review로 실제 결함이 남음을 확인하고 source→layout→SVG paint 경로가 하나로
   좁혀지면, 이 분석 Stage를 커밋하고 다음 Stage에서 최소 코드를 수정한다.

전수 원장은 오래 걸려도 process 종료만으로 성공을 추정하지 않는다. 각 도구의 final exit
status, requested/completed/missing와 manifest를 기록한다.

## 완료 조건

- 축 A p33–p34는 PDF와 source owner, visible text, 표 외곽선·cell clip이 모두 일치하는지
  판정한다.
- 축 B/C는 383쪽 전체 SVG/PDF/render-tree page count와 후보 ledger를 하나의 provenance
  묶음으로 남긴다. 후보 0을 곧바로 합격으로 쓰지 않는다.
- 발견된 실제 결함은 재현 명령, 기준 PDF evidence, source/layout/paint 코드 경로를 모두
  이 Stage에 기록한 뒤 다음 Stage 구현으로 넘긴다.
- 이번 Stage에서 코드를 수정하지 않는다. 범위가 확정되면 이 문서를 먼저 커밋해 단계 경계를
  남긴다.

## 실행 결과

### 축 A — `76076_regulatory_analysis.hwp` p33–p34

전용 release-test binary로 `fidelity_compare` ledger와 180 DPI visual sweep을 각각
완료했다. SVG/render tree 82/82, 요청 페이지 2/2가 모두 생성되었다. 현재 p33–p34의
직접 PDF review에서는 과거 결함이었던 첫 표 우측 외곽선의 문단 침범과 physical page
경계의 content clip이 재현되지 않았다. 자동 pixel proxy(p33 7.82%, p34 9.26%)는 이
문서의 font raster 차이를 크게 반영하므로 불합격 근거로 사용하지 않았다.

- evidence: `/tmp/rhwp-task3820-stage64-76076-visual/task3820-stage64-76076-p33-p34/review/review_033.png`,
  `review_034.png`
- 판정: Stage 30의 nested fragment 보정은 현행 `upstream/devel`에서도 이 두 쪽의
  알려진 구조 결함을 해소한다. 이 Stage에서 76076 renderer를 다시 수정하지 않는다.

### 축 B/C — production 전체 ledger

두 source format 모두 PDF 383쪽 전체를 `--text-only --export-all-svg --layout-ledger`로
완주했다. HWP는 393쪽(+10), HWPX는 387쪽(+4)이었다. `table-cell-text-overlap` 및
vertical-border-clip 후보가 0이더라도 자동 원장만으로 합격 처리하지 않았다. 수평선 후보
다수는 physical page 경계에서 발생하는 후보였으며 PDF review로 좁혔다.

HWPX p138–p141의 row-owner 후보는 180 DPI direct sweep(4/4)에서 실제 page-owner
결함이 아니었다. p139의 차이는 주로 font/ink raster였으므로 보정 대상으로 승격하지
않았다.

### 확정 결함 — HWPX p144부터 시작하는 조기 table fragment

HWPX p143–p146 180 DPI direct sweep(4/4)과 한컴 PDF를 직접 대조했다. PDF p144는
`활용하면 좋은 기능 / 온나라 문서 붙임 파일에 직인 날인 방법` 1×N 표의 후반 본문과
두 예시 이미지 caption을 같은 p144에 둔다. 현행 rhwp는 같은 table의 앞부분만 p144에
두고 후반을 p145로 조기 이월한다. 그 결과 p145 이후가 한 페이지씩 밀린다.

- review evidence:
  `/tmp/rhwp-task3820-stage64-manual-hwpx-p143-p146/task3820-stage64-manual-hwpx-p143-p146/review/review_144.png`,
  `review_145.png`
- overlay proxy: p144 18.67%, p145 50.57%, p146 11.89%. 이 수치는 font 차이도 포함하나,
  표 내부 이미지/caption의 물리적 owner 이동은 PDF 직접 review로 독립 확인했다.
- render/layout evidence: `Contents/section3.xml`의 `id=1723619577`, para index 71,
  3×1 표이다. `treatAsChar=1`, `flowWithText=0`, `pageBreak=NONE`이며 inline TAC 표가
  아니다. 진단에서 저장된 row 합은 712.3px, 이 표가 시작되는 잔여 공간은 717.6px으로
  실제로 fit한다. 반면 generic row-cut은 마지막 row를 853.6px으로 재계산해 167.7px
  크게 보고 p145 continuation을 만든다.
- diagnostics:
  `/tmp/rhwp-task3820-stage64-diag-hwpx-p71-20260808.log`의
  `TABLE_DRIFT pi=71`, `TABLE_CUT_DRIFT pi=71`, `DIAG_SPLITSCAN pi=71` 및
  `TABLE_SPLIT_RESULT pi=71` (first fragment 715.2px ≤ 717.6px)을 보존했다.

## 기각한 가설과 기존 회귀의 재분류

- 이 결함은 p139 owner 후보나 76076 p33–p34 재발과 같은 원인이 아니다. 페이지 수만으로
  source format을 합치거나 전역 pagination tolerance를 넓히지 않는다.
- `tests/issue_3930_hwpx_hwp_save_layout.rs`는 기존 save-path 동등성 관찰을 위해
  source p144에 attachment guidance가 **없고** p145에 있다고 고정한다. 한컴 PDF가
  p144 owner를 명확히 보이므로 이 기대는 현재 #3820 PDF fidelity 기준과 충돌한다.
  Stage 65에서는 이 assertion을 보존하지 않고, direct HWPX의 PDF owner 계약과
  HWP round-trip 계약을 분리한다.

## Stage 65 이관

다음 Stage는 HWPX stored-layout에서 `treatAsChar=1`이지만 `flowWithText=0`인 block
table을 inline TAC으로 취급해 declared whole-fit을 막는 경로를 최소 범위로 검증한다.
특히 `src/renderer/typeset.rs`의 raw `table.common.treat_as_char` 조건과
`uses_tac_table_flow()`의 semantic flow 판정이 불일치하는지 실험한다. 보정은 p144의
PDF owner를 focused regression으로 고정하고, p143–p146 direct sweep으로 먼저 확인한
뒤에만 전체 production cascade와 다른 HWP profile에 넓힌다.
