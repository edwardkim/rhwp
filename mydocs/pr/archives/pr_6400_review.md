---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6400
issue: 6342
author: kevin9327
---

# PR #6400 review - TAC 결재 표 뒤 붙임 쪽 분할

## Metadata

| 항목 | 확인값 |
| --- | --- |
| 원 PR head | `8c6f266072f75d5ea147ea989448f146201b7799` |
| 규모 | 3 files, `+125/-0`, 3 commits |
| 작성 시점 원 PR 상태 | Open, non-draft, check `SUCCESS`, `mergeStateStatus=UNKNOWN` |
| 통합 적용 | `4ee3999` → `d15cbd7` → `a1991a3` |

## 검토와 증적

- contributor comment의 Archive B/C 회귀를 반영했다. 모든 TAC 표가 아니라 4×1 단 기준 결재 표와 40px 미만 두 붙임 줄에만 적용되도록 좁혔다.
- `issue_6342_tac_overflow_page_split`은 입력 문서가 2쪽으로 끝남을 고정하며 통합 후보 full nextest에서 통과했다.
- [info 증적](../assets/pr_6400_issue6342_approval_info.json)은 원본이 `hancom-office-2020`, 2쪽임을 확인한다. 예전 2024 이름 PDF가 아니라 #6466 canonical `pdf/hwpx/opengov/36385445_결재문서본문_화재발생종합보고서(제2026-189호, 2026. 5. 14.)-hwpx-2020.pdf`를 사용했고 새 PDF는 만들지 않았다.
- 두 쪽 `rsvg` sweep은 [요약](../assets/pr_6400_issue6342_approval_attachments_visual_sweep_summary.json)에서 완료 2/2, 후보 0건이다. [1쪽](../assets/pr_6400_issue6342_p001_review.png) 결재 표와 [2쪽](../assets/pr_6400_issue6342_p002_review.png) 붙임 두 줄을 직접 확인했다. 2쪽의 붙임이 별도 페이지에 있는 것이 이 PR의 사용자-visible 계약이다. `rsvg` 글꼴 차이로 ink match는 보조 지표로만 기록한다.

| 보존물 | SHA-256 |
| --- | --- |
| source HWPX | `5ac007ffc51958f776b655241ced079bfe311f481af84565c6284bc80b568a02` |
| canonical PDF | `a154bd38ea9b3e6b3e23511d8495e35f4e6a58ba5f660486e99d7c9415c4970a` |
| info JSON / sweep summary | `d4546adab4caa4a8307e8c7703c4c28f6844d75966a0c471e69ac2ac0e4ac229` / `ac55157dfe67e31f64fd627a038ebb2d361a6cd8852ace1dc9757a0c60369fd0` |
| p001 / p002 PNG | `02905799bfb5c5e0f6f3154f09856256e717e3cd70ef1d950e00bdb937da75ee` / `db1cda7a1b89744d12a1b45758b5b56768dc1fc983632e57acd51fcbe4b386ec` |

임시 output은 `output/visual_sweep_pr6400/`였고, 결론에 쓴 JSON/PNG만 `mydocs/pr/assets/`로 보존했다.

## 판단

**수용.** 요구한 붙임 2쪽 분할이 기존 canonical PDF와 같은 페이지 수로 확인됐고, 이전 broad 적용 회귀는 조건 제한으로 차단됐다. merge 전 최신 head CI를 다시 확인한다.

## Merge 후 contributor PR comment 계획

- 자산이 `devel`에 반영된 뒤, 원 PR에 실제 줄바꿈을 담은 UTF-8 `--body-file`로 게시한다. 현재는 통합 PR 번호와 merge SHA가 없으므로 외부 comment를 게시하지 않는다.
- comment에는 [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment), p1-2 `flagged=0/2`, `pixel match=96.05035%`, `visual_accuracy_proxy=43.65817%`와 p2 붙임 두 줄의 독립 페이지 직접 확인 결론을 넣는다.
- 수치는 `rsvg` 글꼴 환경의 자동 일치율 보조값이며 사람의 최종 판단을 대체하지 않는다고 명시한다.
- 이미지 표시는 실제 통합 merge SHA로 고정한 다음 URL을 사용한다.

  ```text
  https://raw.githubusercontent.com/edwardkim/rhwp/<integration-merge-sha>/mydocs/pr/assets/pr_6400_issue6342_p001_review.png
  https://raw.githubusercontent.com/edwardkim/rhwp/<integration-merge-sha>/mydocs/pr/assets/pr_6400_issue6342_p002_review.png
  ```
