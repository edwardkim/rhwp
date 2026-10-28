---
kind: pr-review
status: accepted-with-ci-condition
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-31
pr: 6484
author: jeong-sik
---

# PR #6484 review - KoPub돋움체 한글 advance 0.872em 복원

## 검토 기준

- 원 PR head: `a405c3855fee02b16da1bc04147be9ae968a1988`
- 통합 적용 commit: `6c79824b60423a44cb841b91fb2e5a75c57dd4a4`
- 통합 기준 base: `upstream/devel@77bcaaa49c89dc12761282c759717188a880064c`
- 작성 시점 원 PR은 Open/non-draft, `MERGEABLE/CLEAN`이며 Build & Test와 CodeQL이 성공했다. merge 직전에
  최신 head의 CI와 mergeability를 다시 확인한다.

## 검토와 검증

- KoPub돋움체 한글 face 상수를 1.0em 치환값이 아니라 embedded CIDFont `/W` 실측인 0.872em으로
  변경한 범위를 확인했다. 비-KoPub 바탕체 상수와 반각 punctuation 경로는 바꾸지 않는다.
- `issue_5804_dash_leader_glyphs`, `issue_5830_dash_leader_last_line`, `issue_1891`을 포함한 focused
  nextest 22건이 통과했다. page-count baseline의 86712 두 항목은 새 advance로 재산출한 64쪽에 맞춘
  동반 기준선 갱신이다.
- 2024 저장 HWP를 2024 기준 PDF `pdf/2025 행정업무운영 편람(최종)-hwp-2024.pdf`
  (384쪽, SHA-256 `34db2aeefa4ae00b38c464571e7e17eef375ffd3ec29eb6d89ddcd67b63bb670`)로 68쪽 visual sweep 했다.
- 결과: flagged 후보 0건, pixel match 89.44578%, proxy 74.33477%. line-band 진단 후보는 font raster와
  grey panel 경계의 차이를 포함하지만 자동 flag는 아니며, 사람이 대표 PNG에서 KoPub 한글, paragraph flow,
  box frame의 누락·overflow·깨진 glyph가 없음을 확인했다.
- 안정 증적: `mydocs/pr/assets/pr_6484_issue6389_p068_review.png`,
  `mydocs/pr/assets/pr_6484_issue6389_visual_sweep_summary.json`,
  `mydocs/pr/assets/pr_6484_issue6389_overlay_metrics.json`.

## Merge 후 contributor PR comment 계획

- [Visual Sweep GitHub merge comment 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을
  링크한다.
- 68쪽, flagged 0건, pixel match 89.44578%, proxy 74.33477%, 사람의 paragraph/glyph 판정과
  pixel proxy가 완전한 typography oracle은 아니라는 한계를 함께 적는다.
- merge 후
  `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr_6484_issue6389_p068_review.png`
  를 표시하고 `--body-file` 게시 뒤 API를 재조회한다.

## 최종 판정

**승인.** face-specific advance의 출처, related baseline 변경, focused regression, 2024 기준 PDF 시각 검증이
서로 일치한다. 통합 PR #6490의 code head `2024da02cde595a179655b8e697d6f0ab8f8b509`에서 Build & Test,
Lint, Native Skia, CodeQL을 포함한 34 check-run이 실패 없이 끝났고 mergeability가 `clean`임을 확인했다.
이 trailing 문서 head의 CI만 마저 확인하면 merge 조건이 충족된다.
