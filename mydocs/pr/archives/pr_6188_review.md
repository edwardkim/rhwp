---
kind: pr-review
status: accepted-with-maintainer-correction
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-27
---

# PR #6188 review - #6149 low-zoom ruler and page gaps

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6188
- 작성자: `postmelee`
- 원 PR head: `192451e9a0abd0ae1d9c761cd2ccf07aee53c3d6`
- 통합 검토 브랜치: `review/open-prs-6178-6198-20260827`
- 기준: `upstream/devel@529ab90c25d5`
- 적용 커밋: non-merge 15건 적용, 원 PR의 devel merge commit 2건은 제외
- 적용 문서: `maintainer_general`, `intake_and_review`, `local_validation`,
  `multi_pr_update_branch`, `visual_fixture_evidence`

## 검토 판단

**수용 가능, 메인터너 보정 포함**. 저배율에서 페이지 경계와 ruler 밀도를 분리해 보여주는 Studio
뷰 변경이며, low zoom/compact viewport 계약이 e2e로 확인됐다.

체리픽 중 `mydocs/orders/20260827.md`에 충돌이 있었고, 기존 devel의 #6159/#6191/#6200 기록을
보존한 뒤 #6149 표를 이어 붙였다. 추가로 `git diff --check`에서 `page-gap.ts`,
`ruler-scale.ts`, `ruler-scale.test.ts`의 EOF 공백 경고가 잡혀 `81265b486`으로 제거했다. 이는
동작 보정이 아니라 CI lint/whitespace 안정화를 위한 메인터너 보정이다.

## 증적과 검증

- Studio 단위 테스트: `npm --prefix rhwp-studio test`
  - `1221 passed`, `1 skipped`
- Studio build: `npm --prefix rhwp-studio run build` pass
- responsive e2e: `npm --prefix rhwp-studio run e2e:responsive`
  - `943 passed`, `0 failed`
- 대표 시각 증적 직접 확인:
  - `mydocs/pr/assets/pr_6188_issue6149/low_zoom_10_auto.jpg`
  - `mydocs/pr/assets/pr_6188_issue6149/focus_pinned_50_scroll.jpg`
  - `mydocs/pr/assets/pr_6178_6182_6188_6193_6195_6198/pr_6188_issue6149_low_zoom_full.png`
  - `mydocs/pr/assets/pr_6178_6182_6188_6193_6195_6198/pr_6188_issue6149_compact_panel.png`
- 전체 회귀: `8438 passed`, `43 skipped`, `10 slow`
- 증적 SHA: `mydocs/pr/assets/pr_6178_6182_6188_6193_6195_6198_visual_evidence_sha256.tsv`

## 후속

통합 PR 생성 시 #6149의 UI 시각 증적과 `e2e:responsive` 통과를 근거로 수용 판단을 남긴다.
병합 후 원 PR에는 devel merge commit은 제외하고 non-merge 커밋만 보존 적용했음을 기록한다.
