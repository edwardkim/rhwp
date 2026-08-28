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
- 검토자가 직접 확인한 대표 증적: PNG는 2-up 문서 provenance 보조 자료이며, 수용 판단 중심은
  `rhwp info --json`/contract test의 인쇄 방식 노출이다.
- focused tests:
  - `issue_6208_hwp5_doc_data_carries_print_method` 1 pass
  - `issue_6208_only_four_and_five_imply_nup` 1 pass
  - `issue_6208_print_method_is_derived_not_authoritative` 1 pass
- 통합 head 공통 검증: fmt, unit tier, suite manifest, focused regressions, clippy, 전체 nextest,
  Native Skia 3종, WASM build 통과.

## 코멘트 처리

merge 후 원 PR/issue 코멘트에는 `rhwp info --json` 계약과 focused tests를 중심으로 설명하고, PNG는
provenance 보조 증적으로만 언급한다. 만약 PDF/시각 비교를 추가로 요구받으면 문서 버전에 맞는 MCP로
기준 PDF를 만들고 `visual_sweep_guide.md#github-merge-comment` 절차에 따라 대표 review PNG와 summary를
`mydocs/pr/assets`에 보존한 뒤 merge SHA raw URL로 표시한다.

## 후속

추가 보정 필요 없음.
