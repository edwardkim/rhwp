---
kind: pr-review
status: accepted-pending-integration-pr-approval
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-24
---

# PR #6014 review - co-anchored 중첩 표 TAC 적층 보정 (#5712)

## 접수 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#6014](https://github.com/edwardkim/rhwp/pull/6014) |
| 작성자 | [@planet6897](https://github.com/planet6897) |
| 원 head | `104f16952c75e7e7fd76a534dfaf4592a3556b47` |
| 통합 commit | `d2ac0e11f` |
| GitHub 상태 | non-draft, `MERGEABLE/CLEAN`, CI·CodeQL 성공 |
| 판정 | **수용 권고** |

## 변경 검토

`samples/issue5712/3184241_medical_exam_equipment.hwpx` 1쪽에서 같은 문단의 비-TAC
TopAndBottom 중첩 표와 TAC 중첩 표가 같은 대역에 완전히 포개지던 문제를 보정한다.

앞선 비-TAC TopAndBottom 표가 `para_y`를 전진시켰고, 저장 line segment가 완전 동일 vpos overlay가
아닌 부분 겹침이며, 전진 커서가 기존 TAC 앵커보다 아래일 때만 TAC 앵커를 `para_y`로 승격한다.
완전 동일 vpos는 기존 #3820 overlay 계약을 보존하도록 제외되어 있어 보정 범위가 좁다.

## 로컬 검증

- `git diff --check upstream/devel...HEAD`: 통과
- `node scripts/rust-test-suite-manifest.mjs --prepare`: 통과
- `node scripts/run-rust-test.mjs issue_5712_coanchored_nested_stack -- --locked --cargo-profile release-test --target-dir target/pr-review`: 1 passed
- `cargo fmt --all -- --check`: 통과
- `cargo clippy --locked --target-dir target/pr-review --lib --bins --tests -- -D warnings`: 통과
- 전체 nextest: `8306 tests run: 8306 passed (4 slow), 42 skipped`, 211.485s

## 시각 증적

`rhwp info --json samples/issue5712/3184241_medical_exam_equipment.hwpx` 기준
`format=hwpx`, `version=5.1.0.0`, `lastSavedWith.product=hancom-office-2020`,
`lastSavedWith.version=11.0.0.7571`, `pageCount=1`로 판정했다. 이에 따라 HWP 2024 MCP 통합 service의
`engine 2020`으로 기준 PDF를 산출했다.

- 원본 fixture: `samples/issue5712/3184241_medical_exam_equipment.hwpx`
- 원본 SHA-256: `44df8d22f30621f01a88ce981dae1a4d89d9cc417b54623a0986f286905bb991`
- info JSON: `mydocs/pr/assets/pr_6014_info.json`
- 기준 PDF: `pdf/pr6014_issue5712_3184241_medical_exam_equipment-2020.pdf`
- PDF SHA-256: `ac4cda1e7fc95fbf7b4806b39c8b33d4607e2b4ae2d078e8c3cbd59e53713502`
- MCP job: `84833a68-c22b-4a03-9989-2a2d511c0d93`, `succeeded`, download `success`,
  engine/profile `2020`
- visual sweep:
  `python3 scripts/visual_sweep.py --key pr6014_issue5712 --hwp samples/issue5712/3184241_medical_exam_equipment.hwpx --pdf pdf/pr6014_issue5712_3184241_medical_exam_equipment-2020.pdf --page 1 --rhwp-bin target/pr-review/debug/rhwp --out output/visual_sweep_pr6014_issue5712`
- summary: `mydocs/pr/assets/pr_6014_issue5712_visual_sweep_summary.json`
- 대표 review asset: `mydocs/pr/assets/pr_6014_issue5712_p1_review.png`
- 대표 review PNG SHA-256: `bff851671a4f2a26a2188d33682bd8222103131b50438595afaac2b887e165d9`
- visual sweep 결과: `run_state=complete`, completed singleton page `3184241`, `flagged_page_count=0`,
  `average_pixel_match_percent=86.92195`, `visual_accuracy_proxy_percent=34.57916`

대표 review PNG에서 중첩 표 구간들이 세로로 분리되어 보이며, PR의 핵심 주장인 완전 포개짐 해소가
확인된다. `rhwp info` 실행 중 `LAYOUT_OVERFLOW` 진단이 한 줄 출력됐지만, PR 본문이 이미 “바깥 표
선언 높이 축”으로 분리한 잔여 현상이고 focused test와 visual sweep 자동 후보는 통과했다.

## 권고

원 PR CI와 통합 후보의 focused/전체 회귀 및 대표 visual sweep이 모두 통과했다. 이번 통합 PR에 포함해
수용 가능하다.
