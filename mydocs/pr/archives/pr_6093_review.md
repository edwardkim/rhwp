---
kind: pr-review
status: accepted-with-maintainer-resolution
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-26
---

# PR #6093 review - #6060 낫표 반각 강제와 #6142 충돌 정리

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6093
- 작성자: `planet6897`
- 원 PR head: `53f47bba61ce`
- 통합 검토 브랜치: `review/open-prs-20260826-r1`
- 기준: `upstream/devel@1011a89475c9` (#6142 merge 포함)
- 원 PR 상태: non-draft, source CI 녹색, GitHub mergeability `CONFLICTING`, comments/reviews 0건

## 검토 판단

**메인터너 보정 포함 수용 가능**. #6142가 이미 `upstream/devel`에 병합되면서 「」 폭 판정은 더 강한
측정 기반 구현으로 교체되어 있었다. 따라서 #6093의 충돌 구간에서는 source PR의 이름 기반 보정을 그대로
되살리지 않고 #6142의 측정 기반 구현을 유지했다. 대신 #6093이 추가한 실제 샘플, 회귀 테스트,
IR field sweep baseline과 전/후 시각 증적은 보존했다.

## 메인터너 보정

- 유지한 기준 구현:
  - `src/renderer/layout/text_measurement.rs`
  - `src/renderer/skia/text_replay.rs`
  - `src/renderer/web_canvas.rs`
- 보존한 #6093 자산:
  - `samples/issue6060/30307_local_service_reform.hwp`
  - `tests/cases/issue_6060_corner_bracket_fullwidth_fonts.rs`
  - `tests/fixtures/ir_field_sweep_baseline.tsv`
  - `mydocs/report/corner-bracket-6060/before_title.png`
  - `mydocs/report/corner-bracket-6060/after_title.png`

## 증적과 검증

- #6142 merge commit `1011a89475c9`를 기준으로 rebase했고, 중복 패치는 제외했다.
- 직접 visual_sweep 증적:
  - `rhwp info --json`: `mydocs/pr/assets/sample_issue6060_30307_local_service_reform_info.json` (`lastSavedWith.product=null`, version `7.5.12.614`)
  - local Hancom 2020 PDF: `pdf/pr_6088_6144/local_hancom2020/pr_6093_6094_issue6060_local_service_reform-local2020.pdf`
  - 대표 review PNG: `mydocs/pr/assets/pr_6093_6094_issue6060_visual_review_001.png`
  - metrics: `mydocs/pr/assets/pr_6088_6144_local_visual_sweep_metrics.tsv`, `mydocs/pr/assets/pr_6088_6144_local_visual_sweep_flags.tsv` (`page=1`, `flagged_page_count=0`, pixel match `95.99198%`)
- 통합 후보 전체 검증:
  - 전체 nextest 8,399 pass, 43 skip
  - focused renderer suite 1,691 pass, 6 skip
  - rustfmt, clippy, suite manifest, diff whitespace 통과
  - WASM build와 native-Skia 공식 범위 통과

## 후속

원 PR comment에는 충돌 해결 이유를 명시한다. 결론은 원 PR 아이디어와 fixture는 수용하되, 실제 구현은
이미 병합된 #6142 측정 기반 경로를 유지한다는 것이다.
