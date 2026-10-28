---
kind: pr-review
status: accepted-with-scope-limit
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6398
issue: 6378
author: kevin9327
---

# PR #6398 review - HWPX 표 outMargin 원점

## Metadata

| 항목 | 확인값 |
| --- | --- |
| 원 PR head | `d2bd1ef66f3803689216b45e6d31e8f8190ab7cd` |
| 규모 | 5 files, `+185/-5`, 3 commits |
| 작성 시점 원 PR 상태 | Open, non-draft, check `SUCCESS`, `mergeStateStatus=UNKNOWN` |
| 통합 적용 | `da62d2a` → `b86d6ba` → `3285e00` |

## 검토와 증적

- 원 PR의 두 후속 comment를 반영했다. 전역 outMargin 적용으로 깨진 #1133 간격 회귀를 `RowBreak` 1열·단 기준·사방 283HU 형상으로 좁혔고, src 단위 test는 suite 총량 정책에 맞춰 integration contract로 옮겼다.
- 통합 후보의 full nextest는 통과했다. `issue_6378_hwpx_outmargin_position`은 HWP/HWPX 첫 Table 원점 차이를 0.6px 미만으로 고정한다.
- `rhwp info --json` 증적은 [HWP](../assets/pr_6398_tac_img_02_hwp_info.json)에서 `hancom-office-2020`, 66쪽, [HWPX](../assets/pr_6398_tac_img_02_hwpx_info.json)에서 `hancom-office-2024`, 67쪽을 확인했다. #6466의 canonical pair를 따라 각각 `pdf/tac-img-02-hwp-2020.pdf`, `pdf/tac-img-02-hwpx-2024.pdf`를 사용했고 새 PDF는 만들지 않았다.
- `rsvg` visual sweep으로 두 형식의 1쪽을 확인했다. [요약](../assets/pr_6398_tac_img_outmargin_visual_sweep_summary.json)은 두 대상 모두 후보 0건, 1/1 페이지 완료를 기록한다. [HWP review PNG](../assets/pr_6398_tac_img_hwp_p001_review.png)와 [HWPX review PNG](../assets/pr_6398_tac_img_hwpx_p001_review.png)를 직접 열어 제목·표지선·로고의 같은 영역 흐름을 확인했다. `rsvg` 글꼴 차이로 ink match 38.785%는 충실도 합격 수치가 아니며, 자동 후보 0건과 원점 계약의 보조 증적이다.

| 보존물 | SHA-256 |
| --- | --- |
| `samples/tac-img-02.hwp` | `f8d5b42367363de0bfde553c062e8a632c222e076a52c95b36efb490b743008e` |
| `pdf/tac-img-02-hwp-2020.pdf` | `f5b56bc65f796acafe3075aadd714ec96d2e508f862dfd899ec5e5081d03e1cc` |
| `samples/tac-img-02.hwpx` | `aefaf3913470056723e8e58e0fb8f8ae4054cb57b31bc764a6dba5d6c8df2975` |
| `pdf/tac-img-02-hwpx-2024.pdf` | `ac09959a062b0d71f41ddd51f30c190e55995f5c4af2edc13e8f55ea0904604c` |
| sweep summary | `5b6c35b469e4c615f5a9f600107f71cee5bb77a1345c111041815f71cbaf4160` |
| HWP/HWPX representative PNG | `2997bc6b58832985dd5e5441c6902ae83267babe2b1dc158a496e09bde14ea27` / `320644246ebc7b8daf7c9363f718f85b72ebc1d5a887a64f61559b166d01c198` |

임시 output은 `output/visual_sweep_pr6398/`였고, 결론에 쓴 JSON/PNG만 `mydocs/pr/assets/`로 보존했다.

## 판단

**범위 제한 수용.** 이 PR은 1쪽 원점 1mm 어긋남만 닫는다. HWPX 67쪽 대 한컴 PDF 66쪽의 잔여 차이는 test도 별개 인과로 명시하므로, 이 review에서 page-count 해결로 확대 해석하지 않는다. merge 전에는 최신 원 PR head와 required check를 다시 확인한다.

## Merge 후 contributor PR comment 계획

- 자산이 `devel`에 반영된 뒤, 원 PR에 실제 줄바꿈을 담은 UTF-8 `--body-file`로 게시한다. 현재는 통합 PR 번호와 merge SHA가 없으므로 외부 comment를 게시하지 않는다.
- comment에는 [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment), HWP/HWPX 각각 p1 `flagged=0/1`, `pixel match=96.19733%`, `visual_accuracy_proxy=38.78498%`를 기록한다.
- 수치는 `rsvg` rasterizer의 자동 일치율 보조값이며 사람의 최종 판단을 대체하지 않는다고 명시하고, 제목·표지선·로고 흐름을 직접 확인한 결론을 함께 쓴다.
- 이미지 표시는 다음처럼 실제 통합 merge SHA로 고정한다.

  ```text
  https://raw.githubusercontent.com/edwardkim/rhwp/<integration-merge-sha>/mydocs/pr/assets/pr_6398_tac_img_hwp_p001_review.png
  https://raw.githubusercontent.com/edwardkim/rhwp/<integration-merge-sha>/mydocs/pr/assets/pr_6398_tac_img_hwpx_p001_review.png
  ```
