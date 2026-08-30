---
kind: pr-review
status: accepted-with-ci-condition
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-31
pr: 6483
author: planet6897
---

# PR #6483 review - U+300C/U+300D 전각 측정으로 복원

## 검토 기준

- 원 PR head: `0906d7c08a67bae354e53c18ce84ed625a651fef`
- 통합 적용 commits: `4e4bafc2cc49800869140eee76a83a8303058381`,
  `696ce085539aee23533499798b1ff99fe9ddbcde`
- 통합 기준 base: `upstream/devel@77bcaaa49c89dc12761282c759717188a880064c`
- 작성 시점 원 PR은 Open/non-draft, `MERGEABLE/CLEAN`이며 Build & Test는 success, CodeQL은 neutral이다.
  merge 직전에 최신 head와 required check를 재확인한다.

## 검토와 검증

- 등록 고정폭 face라는 이유만으로 `「」`를 반각 overlay로 축소하던 분기를 제거한 것을 확인했다.
  실제 반각 `｢`(U+FF62)은 기존 별도 unicode-halfwidth 처리 대상이며 바뀌지 않는다.
- `issue_6060_cjk_quote_paint_measure_parity`, `issue_2020`를 포함한 focused nextest 22건이 통과했다.
  고정폭 및 비례 face에서 U+300C의 measure/paint 판정이 전각으로 일치함을 검증한다.
- 2020 저장 HWP를 2020 기준 PDF `pdf/issue2020/passport_application_lawgo-lawgo-2020.pdf`
  (2쪽, SHA-256 `02ecef8dd99a59331476ac2bf755d5b3194899f83d62420e56bfe107ff0e686f`)로 1쪽 visual sweep 했다.
- 결과: pixel match 84.26276%, proxy 45.74855%, 자동 flag 1건이다. flag는 하단
  `210mm×297mm[백상지 120g/㎡]` footer가 frame 밖 13px인 기존 paper-size footer bleed이며,
  render tree에서도 `paper_size_footer_bleed`로 억제 후보로 분류됐다. 대표 form과 낫표 glyph는 사람이
  직접 확인해 누락·반각 축소·겹침이 없음을 확인했다.
- 안정 증적: `mydocs/pr/assets/pr_6483_issue6478_p001_review.png`,
  `mydocs/pr/assets/pr_6483_issue6478_visual_sweep_summary.json`,
  `mydocs/pr/assets/pr_6483_issue6478_overlay_metrics.json`.

## Merge 후 contributor PR comment 계획

- [Visual Sweep GitHub merge comment 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을
  링크하고, 1쪽의 13px footer bleed는 이번 낫표 변경과 무관한 기존 관측값이라고 명시한다.
- flagged 1건, pixel match 84.26276%, proxy 45.74855%, 사람의 glyph 판정과 proxy의 font/layout 의존 한계를
  함께 적는다.
- merge 후 asset이 devel에 존재하면
  `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr_6483_issue6478_p001_review.png`
  를 표시하고, `--body-file` 게시 뒤 API 응답을 재확인한다.

## 최종 판정

**승인.** 전각 oracle 근거, measure/paint parity 계약, 원 문서 PDF 비교가 같은 결론을 지지한다. 최신
통합 head의 CI 성공과 mergeability 재확인이 merge 조건이다.
