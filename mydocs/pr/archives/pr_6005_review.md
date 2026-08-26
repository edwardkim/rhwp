---
kind: pr-review
status: accepted-with-maintainer-correction-pending-integration-pr-approval
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-24
---

# PR #6005 review - HWP5 메모 필드의 HWPX MEMO 승격 (#5866)

## 접수 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#6005](https://github.com/edwardkim/rhwp/pull/6005) |
| 작성자 | [@planet6897](https://github.com/planet6897) |
| 원 head | `0069733ef595b70cfc6a6e6891e080d481f2c0c3` |
| 통합 commit | `29e3f2853` |
| GitHub 상태 | 원 PR은 충돌로 checks 없음 |
| 판정 | **메인터너 충돌 보정 후 수용 권고** |

## 변경 검토

HWP5 메모 필드를 HWPX `MEMO`로 승격해 메모 대상 텍스트가 상호참조 fallback으로 잘못 보존되거나 본문을
오염시키는 경로를 막는다. serializer field/roundtrip/section과 HWP5 memo sample, 계약 테스트가 함께
추가됐다.

## 메인터너 보정

원 PR은 최신 `upstream/devel` 기준 `tests/fixtures/ir_field_sweep_baseline.tsv`에서 충돌했다. 충돌 해소는
새 #5866 memo baseline 행을 추가하면서, 이미 devel에 존재하던 #5870/#5875 baseline 행을 모두 보존하는
방식으로 처리했다. 제품 코드의 의미 변경은 하지 않았다.

## 로컬 검증

- `issue_5866_memo_field_hwpx`: 3 passed
- 통합 후보 전체 검증에서 `cargo fmt`, suite manifest, unit-tier, clippy, 전체 nextest가 통과했다.
- 전체 nextest: `8304 tests run: 8304 passed (3 slow), 42 skipped`

## 시각 증적

`rhwp info --json samples/issue5866/memo_field_hwp5.hwp` 기준
`lastSavedWith.product=hancom-office-2018`, `pageCount=40`로 판정했다. 이에 따라 HWP 2024 MCP 통합
service의 `engine 2020`으로 기준 PDF를 산출했다.

- 기준 PDF: `pdf/memo_field_hwp5-2020.pdf`
- PDF SHA-256: `d020b37edd534c41568b7347737ff882e4ee129e53053f764102a9fa045db061`
- MCP job: `c7acb0bf-ff3d-40c3-ad51-cad4080565f1`, `succeeded`, download `success`
- visual sweep: `python3 scripts/visual_sweep.py --key pr6005_issue5866 --hwp samples/issue5866/memo_field_hwp5.hwp --pdf pdf/memo_field_hwp5-2020.pdf --page 1 --out output/visual_sweep_pr_5999_6002_6005_6008/pr6005 --rhwp-bin target/pr-review/release-test/rhwp --dpi 120`
- metrics: `mydocs/pr/assets/pr_6005_visual_sweep_overlay_metrics.json`
- 대표 review asset: `mydocs/pr/assets/pr_6005_visual_sweep_review_p001.png`
- visual_accuracy_proxy_percent: `5.28158`

이 PR은 parser/serializer의 `MEMO` 필드 구조 보존이 주 계약이다. MCP 기준 PDF는 `pdfinfo` 기준 20쪽이고
`rhwp info`는 40쪽이라, p1 visual sweep은 기준 PDF 로드와 대표 화면 확인용 참고 증적으로만 사용했다. 쪽수
차이와 2-up처럼 보이는 PDF 배치는 이 PR의 `MEMO` 승격 수용 판단을 단독 차단하지 않는다.

## 권고

충돌은 baseline ledger 누적 충돌이며, 메인터너 보정으로 독립 행을 모두 보존했다. focused test와 전체
회귀가 모두 통과했으므로 이번 통합 PR에서 수용 가능하다.
