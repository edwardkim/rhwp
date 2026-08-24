---
kind: pr-review
status: accepted-pending-integration-pr-approval
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-24
---

# PR #6008 review - HWP3 셀 안여백 2022 사상 규칙 제거 (#5916)

## 접수 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#6008](https://github.com/edwardkim/rhwp/pull/6008) |
| 작성자 | [@planet6897](https://github.com/planet6897) |
| 원 head | `72001e836dc0626d59725015ed86ae180d2086de` |
| 통합 commit | `5bd0ed8aa` |
| GitHub 상태 | non-draft, `MERGEABLE/CLEAN`, CI·CodeQL 성공 |
| 판정 | **수용 권고** |

## 변경 검토

HWP3 셀 안여백을 한글 2022 사상 규칙으로 재해석하던 경로를 제거해 원본 padding을 그대로 보존한다.
`05434_vehicle_log_form.hwp`의 되살린 기호가 2쪽 서식 영역에서 3쪽으로 넘치는 문제를 회귀 테스트로
고정한다.

## 로컬 검증

- `issue_5557_hwp3_cell_margin`: 2 passed
- `issue_5916_hwp3_cell_margin_pagefit`: 2 passed
- #5970 제외 후 `issue_1880` focused 회귀도 2 passed로 복구됨을 확인했다.
- 통합 후보 전체 검증에서 `cargo fmt`, suite manifest, unit-tier, clippy, 전체 nextest가 통과했다.
- 전체 nextest: `8304 tests run: 8304 passed (3 slow), 42 skipped`

## 시각 증적

`rhwp info --json samples/issue5916/05434_vehicle_log_form.hwp` 기준 `format=hwp3`, `version=3.0.0.0`,
`lastSavedWith=null`, `pageCount=2`로 판정했다. HWP3에는 마지막 저장 제품 메타데이터가 없어 자동 서비스
선택은 불가능하므로, 레거시 HWP3 기준 PDF는 HWP 2024 MCP 통합 service의 `engine 2020`으로 명시 산출하고
이 예외 판단을 증적에 기록했다.

- 기준 PDF: `pdf/05434_vehicle_log_form-2020.pdf`
- PDF SHA-256: `aefc8043d43bbdd3076260ef507e3104af89f04effc5ae440197637facc8bb0d`
- MCP job: `1190e5b0-3ba5-4aa2-908f-61f3185ec80d`, `succeeded`, download `success`
- visual sweep: `python3 scripts/visual_sweep.py --key pr6008_issue5916 --hwp samples/issue5916/05434_vehicle_log_form.hwp --pdf pdf/05434_vehicle_log_form-2020.pdf --page 2 --out output/visual_sweep_pr_5999_6002_6005_6008/pr6008 --rhwp-bin target/pr-review/release-test/rhwp --dpi 120`
- metrics: `mydocs/pr/assets/pr_6008_visual_sweep_overlay_metrics.json`
- 대표 review asset: `mydocs/pr/assets/pr_6008_visual_sweep_review_p002.png`
- visual_accuracy_proxy_percent: `14.90723`

review PNG에서 2쪽 차량일일점검표가 rhwp와 기준 PDF 양쪽에 유지되고, 3쪽으로 넘어가지 않는 핵심 계약을
확인했다. 자동 보조값은 폰트·자간·선 위치 차이를 크게 잡으므로 정밀 시각 정합 PASS로 쓰지 않고, 2쪽 유지
증적으로만 사용한다.

## 권고

원 PR CI와 로컬 focused/전체 회귀가 모두 통과했다. #5970 대신 이번 통합 PR에 포함해 수용 가능하다.
