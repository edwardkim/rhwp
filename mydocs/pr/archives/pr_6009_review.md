---
kind: pr-review
status: accepted-pending-integration-pr-approval
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-24
---

# PR #6009 review - 중첩 표 호스트 셀 저장 사다리 종점 정합 (#5885)

## 접수 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#6009](https://github.com/edwardkim/rhwp/pull/6009) |
| 작성자 | [@planet6897](https://github.com/planet6897) |
| 원 head | `aa56e0c13e0f661f57b819b62bd09f05779164f5` |
| 통합 commit | `8c0ab6151` |
| GitHub 상태 | non-draft, `MERGEABLE/CLEAN`, CI·CodeQL 성공 |
| 판정 | **수용 권고** |

## 변경 검토

`samples/issue5885/3171199_design_capability_criteria.hwp` 2쪽에서 셀 마지막 문단이 중첩 표
호스트일 때 문단 뒤 간격이 유닛 합에 반영되지 않아, 바깥 행 구분선이 중첩 표 마지막 행을 가로지르고
다음 행이 겹치던 문제를 보정한다.

변경은 `cell_units_uncached` 후처리에서 native HWP5 RowBreak 저장 사다리, local vpos origin,
마지막 문단의 table host, 단조·비합성 lineseg, 한 문단 간격 규모의 shortfall 조건을 모두 만족할 때만
마지막 unit 높이를 저장 사다리 종점까지 늘리는 방식이다. `use_vpos_unit_positions` 전역 게이트를 넓히지
않고 좁은 호환 후처리로 둔 판단은 타당하다.

## 로컬 검증

- `git diff --check upstream/devel...HEAD`: 통과
- `node scripts/rust-test-suite-manifest.mjs --prepare`: 통과
- `node scripts/run-rust-test.mjs issue_5885_nested_host_row_ladder_end -- --locked --cargo-profile release-test --target-dir target/pr-review`: 1 passed
- `cargo fmt --all -- --check`: 통과
- `cargo clippy --locked --target-dir target/pr-review --lib --bins --tests -- -D warnings`: 통과
- 전체 nextest: `8306 tests run: 8306 passed (4 slow), 42 skipped`, 211.485s

## 시각 증적

`rhwp info --json samples/issue5885/3171199_design_capability_criteria.hwp` 기준
`format=hwp5`, `version=5.1.0.1`, `lastSavedWith.product=hancom-office-2020`,
`lastSavedWith.version=11.0.0.8362`, `pageCount=7`로 판정했다. 이에 따라 HWP 2024 MCP 통합 service의
`engine 2020`으로 기준 PDF를 산출했다.

- 원본 fixture: `samples/issue5885/3171199_design_capability_criteria.hwp`
- 원본 SHA-256: `6980eda2b04765cc8051da8581ee7f29fc9cb4780bb42ae14ab98883c1e90a69`
- info JSON: `mydocs/pr/assets/pr_6009_info.json`
- 기준 PDF: `pdf/pr6009_issue5885_3171199_design_capability_criteria-2020.pdf`
- PDF SHA-256: `dabd9f8a992a80ff69916ff91ac0dcf65949739875712f470362aa472bec86d9`
- MCP job: `0849e766-99d7-44e4-b167-004bcad7c6d4`, `succeeded`, download `success`,
  engine/profile `2020`
- visual sweep:
  `python3 scripts/visual_sweep.py --key pr6009_issue5885 --hwp samples/issue5885/3171199_design_capability_criteria.hwp --pdf pdf/pr6009_issue5885_3171199_design_capability_criteria-2020.pdf --page 2 --rhwp-bin target/pr-review/debug/rhwp --out output/visual_sweep_pr6009_issue5885`
- summary: `mydocs/pr/assets/pr_6009_issue5885_visual_sweep_summary.json`
- 대표 review asset: `mydocs/pr/assets/pr_6009_issue5885_p2_review.png`
- 대표 review PNG SHA-256: `0992e4a45549bd69f012a12c7ab7089abb1621c8857e36d5df72561e6db078e7`
- visual sweep 결과: `run_state=complete`, requested/completed page `2`, `flagged_page_count=0`,
  `average_pixel_match_percent=91.14844`, `visual_accuracy_proxy_percent=10.67374`

대표 review PNG에서 `(ㄴ)유동비율` 중첩 표와 다음 `라. 기술개발 및` 행이 분리되어 보인다.
자동 proxy는 폰트·자간 차이를 크게 반영하므로 정밀 fidelity 통과값으로 쓰지 않고, PR 주장인
중첩 표/다음 행 겹침 해소의 대표 증적으로 사용한다.

## 권고

원 PR CI와 통합 후보의 focused/전체 회귀 및 대표 visual sweep이 모두 통과했다. 이번 통합 PR에 포함해
수용 가능하다.
