---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6399
issue: 6366
author: kevin9327
---

# PR #6399 review - flowWithText 글앞으로 표 쪽 분할

## Metadata

| 항목 | 확인값 |
| --- | --- |
| 원 PR head | `cbb61173b56a5892a128ea0998e10f0535508d98` |
| 규모 | 5 files, `+98/-3`, 3 commits |
| 작성 시점 원 PR 상태 | Open, non-draft, check `SUCCESS`, `mergeStateStatus=UNKNOWN` |
| 통합 적용 | `77827da` → `0c7e532` → `4b3cfad` |

## 검토와 증적

- contributor comment의 Archive B/D page-count 회귀를 확인했다. 적용 범위는 원본 HWPX, 글앞으로, 문단 기준, 40행 이상·6열 이상인 pi=9 표로 제한됐고, src test는 policy에 맞춰 integration contract로 이동했다.
- `issue_6366_infront_flow_paginate`는 해당 문서가 한컴 2020 정답지와 같이 6쪽임을 고정한다. 통합 후보 full nextest에도 포함돼 통과했다.
- [info 증적](../assets/pr_6399_issue5792_animal_info.json)은 `hancom-office-2020`, 6쪽을 확인한다. 기존 canonical `pdf/issue5792/2700727_animal_facility_standards-hwpx-2020.pdf`만 사용했다.
- 전체 1-6쪽 `rsvg` sweep은 [요약](../assets/pr_6399_issue6366_infront_flow_visual_sweep_summary.json)에서 완료 6/6, 후보 0건이다. 결함이 드러나던 마지막 [6쪽 review PNG](../assets/pr_6399_issue6366_p006_review.png)를 직접 확인해 표 꼬리가 독립 6쪽으로 존재함을 확인했다. 글꼴 환경 차이로 ink match 8.656%는 정량 충실도 판정에 사용하지 않았다.

| 보존물 | SHA-256 |
| --- | --- |
| source HWPX | `43f4d4a0e6134c787278b139da49ea88e560aee199f2c19f37c787ecb71aeb86` |
| canonical PDF | `3bd076dfedc98d52daf416fabf6d6b8e3acfa10473e3edd4a3e26c9b075a4039` |
| info JSON / sweep summary / p006 PNG | `12910fe5db71e84a50da469adafc9973a1dec3df213197039c7996f4742a056a` / `070e9e55fcfd3f5dcfd429c7dea2d81a658f4364ffaef8b1f82f1438fa5f9e57` / `0d649c6792c792a0ab3279611f7eb1bcd07ee52eff30b5e99781d786c7151e9c` |

임시 output은 `output/visual_sweep_pr6399/`였고, 결론에 쓴 JSON/PNG만 `mydocs/pr/assets/`로 보존했다.

## 판단

**수용.** 실제 HWPX와 버전에 맞는 기존 한컴 2020 PDF에서 6쪽 분할을 확인했고, 좁힌 형상 조건과 회귀 contract가 함께 있다. merge 전 최신 head CI 재확인이 필요하다.

## Merge 후 contributor PR comment 계획

- 자산이 `devel`에 반영된 뒤, 원 PR에 실제 줄바꿈을 담은 UTF-8 `--body-file`로 게시한다. 현재는 통합 PR 번호와 merge SHA가 없으므로 외부 comment를 게시하지 않는다.
- comment에는 [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment), p1-6 `flagged=0/6`, `pixel match=86.67575%`, `visual_accuracy_proxy=9.20725%`와 p6의 표 꼬리 독립 페이지 직접 확인 결론을 넣는다.
- 수치는 `rsvg` 글꼴 환경의 자동 일치율 보조값이며 사람의 최종 판단을 대체하지 않는다고 명시한다.
- 이미지 표시는 실제 통합 merge SHA로 고정한 다음 URL을 사용한다.

  ```text
  https://raw.githubusercontent.com/edwardkim/rhwp/<integration-merge-sha>/mydocs/pr/assets/pr_6399_issue6366_p006_review.png
  ```
