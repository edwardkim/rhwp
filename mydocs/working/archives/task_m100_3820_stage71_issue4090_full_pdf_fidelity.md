---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-08
---

# Task #3820 Stage 71 — issue4090 전체 PDF fidelity 재검증

## 범위

- 입력: `samples/issue4090/156492236_규제샌드박스_min.hwpx`
- 기준: `pdf/issue4090/156492236_규제샌드박스_min-hancom2020-production-verify.pdf`
- 기준 산출: HWP 2020 MCP `PrintToPDFEx`, `PrintMethod=0`, 17쪽

기존 #4090은 `pi=59`, `pi=74`, `pi=183`의 explicit page-break tail line과 내부
page count 17만 고정했다. #3820의 후속 원장에서는 같은 문서의 모든 physical page를 PDF와
직접 대조해, 표·그림·머리말을 포함한 남은 fidelity 결함이 있는지 판정한다.

## 방법과 판정

1. release-test binary로 p1--p17 SVG/render tree와 기준 PDF raster를 생성한다.
2. visual sweep의 자동 후보는 triage로만 쓰고, review PNG와 render-tree owner로 실제 결함을
   판정한다.
3. explicit tail contracts(p5→6, p7→8, p15→16)와 전체 페이지 수 17을 함께 확인한다.
4. PDF와 다른 실제 source owner·표 경계·clip이 있으면 이 문서에 source/layout/paint 경로를
   남기고 다음 Stage의 코드 범위로 분리한다. 자동 후보 0만으로는 완료 처리하지 않는다.

## 실행 결과

`scripts/visual_sweep.py`로 p1--p17을 180 DPI에서 완료했다. SVG/render-tree/PDF/raster는
각각 17/17이고 누락 페이지는 없다. 결과 원장은
`mydocs/pr/assets/task_m100_3820_stage71_issue4090_full_pdf_fidelity/summary.json`에 보존한다.

자동 지표는 `flagged_page_count=0`, frame 밖 content 0으로 보고했지만, 이는 완전한
fidelity 판정이 아니었다. 검토 PNG의 rhwp·PDF 패널을 직접 비교하면 다음 네 쪽에 같은
결함이 남아 있다.

| rhwp/PDF 쪽 | 관찰 | 판정 |
| --- | --- | --- |
| p5 | 첫 번째 점선 사각 placeholder 옆의 본문이 PDF에서는 왼쪽 열에 즉시 흐르지만, rhwp에서는 placeholder 아래에서 시작한다. | 결함 |
| p7 | `③ 도심 자율주행차…` placeholder와 그 아래 `④` 앞 본문이 p5와 같은 방식으로 아래로 밀린다. | 결함 |
| p15 | `⑰ 주류…`, `⑱ 시각장애인…`의 두 placeholder 모두 본문 wrap을 소비하지 않아 행 흐름이 PDF와 다르다. | 결함 |
| p17 | `⑲ 융복합…`, `⑳ 조제관리사…`의 두 placeholder가 같은 결함을 재현한다. | 결함 |

반면 explicit page-break tail은 PDF와 같은 p5→6, p7→8, p15→16에 남아 있으며 p6·p8·p16은
tail 한 줄과 footer만 가진다. 따라서 #4090의 page-count/tail regression은 유지되지만,
placeholder의 `Square`/behind-text 본문 감싸기 fidelity는 별도 미해결이다.

이 결함은 단순 font raster 차이가 아니다. rhwp에는 placeholder 왼쪽에 들어가야 할 문단 줄이
아예 없고 다음 y에서 시작한다. 자동 sweep은 frame overflow·line-order·square-wrap overlap만
검출하므로, 내용이 겹치지 않고 아래로 밀리는 이 **wrap exclusion** 형태를 후보로 만들지
못했다. 다음 Stage에서 HWPX control 속성, source paragraph anchor 및 `Square` flow의 left/right
available width 계산을 분석한다.

## 증적 및 다음 단계

- `review_005.png`, `review_007.png`, `review_015.png`, `review_017.png`: 직접 결함 증적
- `review_006.png`, `review_008.png`, `review_016.png`: page-break tail 대조 증적
- `review_contact_sheet.png`, `summary.json`, `analysis/metrics.json`: 전체 17쪽 실행 원장

코드 변경은 이 분석 Stage에 섞지 않는다. 다음 Stage는 각 placeholder가 `Square` wrap을
요구하는 source 계약인지 확정한 뒤, overlap 없이 PDF와 동일한 옆 본문 흐름을 복원하는 최소
layout 보정과 focused regression을 추가한다.
