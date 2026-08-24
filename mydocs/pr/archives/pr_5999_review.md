---
kind: pr-review
status: accepted-pending-integration-pr-approval
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-24
---

# PR #5999 review - 직접 HWPX RowBreak 칸 조각 회계 정합 (#5880)

## 접수 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#5999](https://github.com/edwardkim/rhwp/pull/5999) |
| 작성자 | [@planet6897](https://github.com/planet6897) |
| 원 head | `c8da82ba7222ecfc6f7e80a523b7063e706851d8` |
| 통합 commit | `420f6e27c` |
| GitHub 상태 | non-draft, `MERGEABLE/CLEAN`, CI·CodeQL 성공 |
| 판정 | **수용 권고** |

## 변경 검토

직접 HWPX RowBreak 칸에서 저장 사다리와 조각 회계를 맞춰 2737927 fixture의 소실 줄과 쪽수 기준을
복구한다. 변경은 table layout partial/row accounting과 해당 sample·계약 테스트에 한정된다.

## 로컬 검증

- `issue_5880_rowbreak_fragment_overfill`: 3 passed
- 통합 후보 전체 검증에서 `cargo fmt`, suite manifest, unit-tier, clippy, 전체 nextest가 통과했다.
- 전체 nextest: `8304 tests run: 8304 passed (3 slow), 42 skipped`

## 시각 증적

`rhwp info --json samples/issue5880/rowbreak_fragment_overfill.hwpx` 기준
`lastSavedWith.product=hancom-office-2020`, `pageCount=8`로 판정했다. 이에 따라 HWP 2024 MCP 통합
service의 `engine 2020`으로 기준 PDF를 산출했다.

- 기준 PDF: `pdf/rowbreak_fragment_overfill-2020.pdf`
- PDF SHA-256: `21360a4d1058a6120931fd14f5957b8ec9c45faac1765840efefe54be3ea4bf1`
- MCP job: `2529b8df-18aa-411a-8274-4d4dadda8c37`, `succeeded`, download `success`
- visual sweep: `python3 scripts/visual_sweep.py --key pr5999_issue5880 --hwp samples/issue5880/rowbreak_fragment_overfill.hwpx --pdf pdf/rowbreak_fragment_overfill-2020.pdf --pages 2,5,6,8 --out output/visual_sweep_pr_5999_6002_6005_6008/pr5999 --rhwp-bin target/pr-review/release-test/rhwp --dpi 120`
- metrics: `mydocs/pr/assets/pr_5999_visual_sweep_overlay_metrics.json`
- evidence summary: `mydocs/pr/assets/pr_5999_6002_6005_6008_visual_sweep_evidence.json`

| page | visual_accuracy_proxy_percent | 대표 review asset |
| --- | ---: | --- |
| 2 | 8.84199 | `mydocs/pr/assets/pr_5999_visual_sweep_review_p002.png` |
| 5 | 7.95011 | `mydocs/pr/assets/pr_5999_visual_sweep_review_p005.png` |
| 6 | 5.56413 | `mydocs/pr/assets/pr_5999_visual_sweep_review_p006.png` |
| 8 | 26.79218 | `mydocs/pr/assets/pr_5999_visual_sweep_review_p008.png` |

자동 일치율 보조값은 낮다. 다만 review PNG에서 PR의 핵심 계약인 8쪽 구성과 말미 표 내용의 보존은 확인된다.
폰트·자간·세부 배치 차이는 잔여 fidelity 후보로 보며, 이 PR의 조각 회계 수용 판단을 단독 차단하지 않는다.

## 권고

독립 focused test와 통합 전체 회귀가 모두 통과했다. #5970 제외 후에도 회귀가 재발하지 않아 이번 통합
PR에서 수용 가능하다.
