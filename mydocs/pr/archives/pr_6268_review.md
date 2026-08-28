---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-28
---

# PR #6268 review - #6208 문서 인쇄 방식(모아 찍기) 메타데이터 노출

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6268
- 작성자: `planet6897`
- 원 PR head: `f58550248bdb`
- 통합 검토 브랜치: `review/open-ci-green-20260828`
- 최신 기준: `upstream/devel@1a43a507c9da`
- 원 PR 상태: non-draft, 실패·진행 check 0건
- 관련 이슈: #6208

## 검토 판단

**수용 권고.** HWP5 `DocInfo`에 저장된 인쇄 방식 값을 읽어 `rhwp info --json`에서 파생 메타데이터로
노출한다. 4-up/5-up처럼 실제 n-up 의미를 갖는 값만 `n_up`으로 해석하고, 원시 값은 authoritative
layout 설정이 아니라 provenance로 취급하는 방향이 적절하다.

## 증적과 검증

- 원 PR 증적:
  `mydocs/report/print-method-nup-6208/oracle_2up_vs_rhwp_portrait.png`,
  `samples/issue6208/print_method_nup.hwp`
- 통합 head 기준 MCP/visual sweep:
  - `rhwp info --json`: `mydocs/pr/assets/pr_6275_issue6208_info.json`
  - 저장 제품: `hancom-office-2020 11.0.0.2129` -> MCP `engine 2020`
  - 기준 PDF: `pdf/pr_6275/by_saved_version/pr6275_issue6208_print_method_nup-2020.pdf`
    (1 page)
  - visual sweep: `pr6275-issue6208-p1`, p1, flagged 0, pixel match `99.40392%`,
    visual accuracy proxy `4.86844%`
  - 장기 asset:
    `mydocs/pr/assets/pr_6275_issue6208_visual_review_p1.png`,
    `mydocs/pr/assets/pr_6275_issue6208_visual_sweep_summary.json`,
    `mydocs/pr/assets/pr_6275_issue6208_visual_overlay_metrics.json`
- 검토자가 직접 확인한 대표 증적: PNG는 2-up 문서 provenance 보조 자료이며, 수용 판단 중심은
  `rhwp info --json`/contract test의 인쇄 방식 노출이다. 통합 head 기준 PDF와 visual sweep도
  보존했지만, 이 PR은 n-up 출력 구현이 아니라 `printMethod`/`printMethodImpliesNup` 노출까지가
  범위이므로 낮은 자동 일치율을 출력 fidelity 실패로 판정하지 않는다.
- focused tests:
  - `issue_6208_hwp5_doc_data_carries_print_method` 1 pass
  - `issue_6208_only_four_and_five_imply_nup` 1 pass
  - `issue_6208_print_method_is_derived_not_authoritative` 1 pass
- 통합 head 공통 검증: fmt, unit tier, suite manifest, focused regressions, clippy, 전체 nextest,
  Native Skia 3종, WASM build 통과.

## 코멘트 처리

merge 후 원 PR/issue 코멘트에는 `rhwp info --json` 계약과 focused tests를 중심으로 설명하고, PNG와
`pr_6275_issue6208_visual_review_p1.png`는 provenance 보조 증적으로만 언급한다. 수치 문구는
"내용 픽셀 중심 자동 일치율 보조값 = 약 4.87%"로 적되, 이 값은 미구현 n-up 출력 차이를 포함하므로
이번 PR의 `printMethod` 노출 수용 판단과 분리한다고 명시한다.

## 후속

추가 보정 필요 없음.
