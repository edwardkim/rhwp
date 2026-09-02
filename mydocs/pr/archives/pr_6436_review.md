---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6436
issue: 6312
author: kevin9327
---

# PR #6436 review - TOP_AND_BOTTOM 부동 표 host 자기 글줄 보존

## Metadata

| 항목 | 확인값 |
| --- | --- |
| 원 PR head / 통합 적용 | `f2d95f476df950c9deebc93b48484abafaa7097c` / `662d9a6` |
| 규모 | 5 files, `+211/-16`, 1 commit |
| 작성 시점 원 PR 상태 | Open, non-draft, `mergeStateStatus=CLEAN`, 원 PR CI 성공 |
| 메인터너 보정 | `dbbb8d8`에서 `cargo fmt`가 요구한 import wrapping만 정리 |

## 변경 검토

- `TOP_AND_BOTTOM` 부동 표의 host 문단에 실제 글줄이 있으면 그 줄의 높이를 흐름에 남겨, 다음 본문 문단이 표에 붙지 않게 한다.
- `tab_host_own_line.hwpx`와 `issue_6312_tab_host_line_kept` contract는 표 band와 이후 본문 사이의 간격을 `27.2px`로 고정한다.
- 빈 host용 기존 경로는 유지하고, 보이는 host 글줄에만 새 advance를 적용하므로 대상이 분리되어 있다.

## 시각 증적

| 항목 | 결과 |
| --- | --- |
| 입력 | `samples/issue6312/tab_host_own_line.hwpx` (2 pages, `lastSavedWith=hancom-office-2024 13.0.0.3189`) |
| 기준 PDF | `pdf/issue6312/tab_host_own_line-2024.pdf`, Hancom 2024 MCP job `90cd7ee6-87a4-48fa-8fd7-f344bf402d2d`, 2 pages, SHA-256 `be7d7894777ed6ca791f4c1309e12922090b3e455c8c5ae0ed16dacbc2110bda` |
| 대상 페이지 | p1, 변경된 TOP_AND_BOTTOM 표와 뒤따르는 본문이 함께 보이는 페이지 |
| visual sweep | complete, `flagged_page_count=0`, frame/content-bottom/line-band 후보 모두 0 |
| render tree | 표 band bottom `492.9px`, 본문 top `520.1px`, gap `27.2px` |
| 직접 확인 | `pr_6436_issue6312_p001_review.png`에서 한글 글리프 누락, 표-본문 겹침, 표 밖 clipping을 발견하지 못함 |

- 보관 증적: `mydocs/pr/assets/pr_6436_issue6312_tab_host_own_line_info.json`, `pr_6436_issue6312_visual_sweep_summary.json`, `pr_6436_issue6312_p001_review.png`.
- 이 호스트에는 Chrome이 없어 허용된 `rsvg` rasterizer를 사용했다. Hancom과 rsvg의 글꼴 rasterization 차이로 overlay ink match가 낮으므로, 그 수치는 fidelity 합격/불합격 판단에 사용하지 않았다.

## 검증과 판단

- 원 PR CI는 성공했다. 통합 candidate의 `cargo fmt --all -- --check`도 통과했다.
- 통합 candidate에서 `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`를 실행해 424.788초 동안 `8763 passed`, `3 slow`, `43 skipped`로 통과했다. 이 결과에는 `issue_6312_visible_tab_host_keeps_its_own_line`도 포함된다.

**수용.** 정확한 2024 기준 PDF 및 p1 시각 검증은 결함의 대상 영역을 직접 커버하며, 자동 후보와 직접 확인 모두 이상이 없다. 전체 `release-test` 회귀도 통과했다.

## Merge 후 contributor PR comment 계획

- 자산이 `devel`에 반영된 뒤, 원 PR에 실제 줄바꿈을 담은 UTF-8 `--body-file`로 게시한다. 현재는 통합 PR 번호와 merge SHA가 없으므로 외부 comment를 게시하지 않는다.
- comment에는 [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment), p1 `flagged=0/1`, `pixel match=80.95624%`, `visual_accuracy_proxy=8.91946%`, 그리고 표 band-본문 `27.2px` gap 및 직접 확인 결론을 넣는다.
- 수치는 `rsvg`와 Hancom의 글꼴 rasterization 차이에 영향을 받는 자동 일치율 보조값이며 사람의 최종 판단을 대체하지 않는다고 명시한다.
- 이미지 표시는 실제 통합 merge SHA로 고정한 다음 URL을 사용한다.

  ```text
  https://raw.githubusercontent.com/edwardkim/rhwp/<integration-merge-sha>/mydocs/pr/assets/pr_6436_issue6312_p001_review.png
  ```
