---
kind: pr-review
status: accepted-with-maintainer-correction
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-27
---

# PR #6182 review - #6053 chart structure editing

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6182
- 작성자: `johndoekim`
- 원 PR head: `f74bdfe17fe52cdb19cfea01e0f724993b578558`
- 통합 검토 브랜치: `review/open-prs-6178-6198-20260827`
- 기준: `upstream/devel@529ab90c25d5`
- 원 PR 상태: non-draft, source CI 완료, source PR merge state는 `DIRTY`
- 적용 문서: `maintainer_general`, `intake_and_review`, `local_validation`,
  `multi_pr_update_branch`, `visual_fixture_evidence`

## 검토 판단

**수용 가능, 메인터너 보정 포함**. 차트 구조 편집 모델, Studio 우클릭 구조 편집 UI, OOXML chart
parser/patcher/renderer 계약을 함께 확장한다. 원 PR은 현재 GitHub 기준 `DIRTY`지만 통합 브랜치에서는
`rhwp-studio/package.json` 충돌을 수동 해소했다.

충돌 해소는 upstream의 `e2e:responsive`와 PR의 `e2e:issue-6053` 스크립트를 모두 보존하는 방식으로
처리했다. 한쪽 스크립트를 버리면 #6149 또는 #6053 검증 루트가 사라지므로, 이는 기능 변경이 아니라
검증 진입점 보존을 위한 메인터너 보정이다.

## 증적과 검증

- focused Rust:
  - `issue_6053_chart_series_identity_contract`: `4 passed`
  - `ooxml_chart_structure_contract`: `46 passed`
  - `issue_2277_stock`: `6 passed`
  - `issue_4100_chart_data_edit`: `61 passed`
  - `issue_4694_chart_list`: `5 passed`
- Studio:
  - `npm --prefix rhwp-studio test`: `1221 passed`, `1 skipped`
  - `npm --prefix rhwp-studio run build`: pass
  - `npm --prefix rhwp-studio run e2e:issue-6053`: pass
- 전체 회귀:
  - `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`
  - `8438 passed`, `43 skipped`, `10 slow`
- 대표 UI 증적:
  - `mydocs/pr/assets/pr_6178_6182_6188_6193_6195_6198/pr_6182_issue6053_context_menu.png`
  - `mydocs/pr/assets/pr_6178_6182_6188_6193_6195_6198/pr_6182_issue6053_pie_note.png`
- 차트 샘플 `rhwp info --json`: 변경 샘플 대부분 `hancom-office-2022`, `version=12.0.0.535`
- 증적 SHA: `mydocs/pr/assets/pr_6178_6182_6188_6193_6195_6198_visual_evidence_sha256.tsv`

## 후속

통합 PR 본문과 병합 후 원 PR 코멘트에는 source PR이 `DIRTY`였으나 package script 충돌만 보존형으로
해소했음을 명시한다. #6053 기능 경로는 Rust 계약과 Studio e2e 양쪽에서 통과했다.
