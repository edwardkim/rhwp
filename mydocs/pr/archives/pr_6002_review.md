---
kind: pr-review
status: accepted-pending-integration-pr-approval
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-24
---

# PR #6002 review - 압축-단조 사다리 셀 중첩 표 높이 계상 (#5884)

## 접수 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#6002](https://github.com/edwardkim/rhwp/pull/6002) |
| 작성자 | [@planet6897](https://github.com/planet6897) |
| 원 head | `1e31a70f91be115465b682096bf53c64982ed3f1` |
| 통합 commit | `0b99b2a8a` |
| GitHub 상태 | non-draft, CI·CodeQL 성공 |
| 판정 | **수용 권고** |

## 변경 검토

압축-단조 사다리 셀에서 중첩 표 높이를 측정에 반영해 3090867 fixture의 쪽수 2 정합과 누락 내용 복원을
고정한다. 변경은 height measurer와 table layout 계산, sample·계약 테스트에 한정된다.

## 로컬 검증

- `issue_5884_nested_row_first_line_height`: 3 passed
- 통합 후보 전체 검증에서 `cargo fmt`, suite manifest, unit-tier, clippy, 전체 nextest가 통과했다.
- 전체 nextest: `8304 tests run: 8304 passed (3 slow), 42 skipped`

## 시각 증적

`rhwp info --json samples/issue5884/nested_row_first_line_height.hwpx` 기준
`lastSavedWith.product=hancom-office-2020`, `pageCount=2`로 판정했다. 이에 따라 HWP 2024 MCP 통합
service의 `engine 2020`으로 기준 PDF를 산출했다.

- 기준 PDF: `pdf/nested_row_first_line_height-2020.pdf`
- PDF SHA-256: `abe64cdac923988f8a50594dc5004c0bbfa8fc063765883bbf736a25b5be7e39`
- MCP job: `a56fc0d0-f77a-4aa1-a1a2-f59843252cab`, `succeeded`, download `success`
- visual sweep: `python3 scripts/visual_sweep.py --key pr6002_issue5884 --hwp samples/issue5884/nested_row_first_line_height.hwpx --pdf pdf/nested_row_first_line_height-2020.pdf --page 2 --out output/visual_sweep_pr_5999_6002_6005_6008/pr6002 --rhwp-bin target/pr-review/release-test/rhwp --dpi 120`
- metrics: `mydocs/pr/assets/pr_6002_visual_sweep_overlay_metrics.json`
- 대표 review asset: `mydocs/pr/assets/pr_6002_visual_sweep_review_p002.png`
- visual_accuracy_proxy_percent: `6.36583`

review PNG에서 한컴 기준 2쪽에는 중첩 예시 표가 남고, rhwp 2쪽에는 앞 설명 블록 일부도 함께 남아 자동 diff가
크다. 이 값은 시각 정밀 정합 통과가 아니라 자동 보조값이다. PR의 핵심 계약인 2쪽 생성과 예시 행 소실 방지는
focused test로 고정되지만, 세부 split 위치는 잔여 fidelity 후보로 별도 추적할 수 있다.

## 권고

focused test와 전체 회귀가 모두 통과했다. 이번 통합 PR에서 수용 가능하다.
