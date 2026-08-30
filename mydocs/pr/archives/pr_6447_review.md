---
kind: pr-review
status: approved-via-integration
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-31
pr: 6447
issue: 6300
author: planet6897
---

# PR #6447 검토 기록

## 결정

| 구분 | 현재 결정 |
|---|---|
| 원 PR head 직접 병합 | 하지 않음 |
| 수용 대상 | 통합 PR #6481에 포함된 `16ee5f750f8ce76ba9879eed00c2597423b1b0c6` |
| 현재 상태 | 승인: #6481 통합 후보의 N-up 원본 physical-sheet 판정과 계약 검증에 차단 finding 없음 |
| 승인 뒤 처리 | #6481 수용·병합 뒤 #6447을 포함 수용 근거와 함께 close |

## 식별과 provenance

| 항목 | 값 |
|---|---|
| 원 PR | https://github.com/edwardkim/rhwp/pull/6447 |
| 원 head | `73f906ca5a88434a2740580ac7b4499b741be83d` |
| 통합 기준 | `upstream/devel@8a150f9a8bb19a9918e195da3a646690f68f4328` |
| 통합 commit | `16ee5f750f8ce76ba9879eed00c2597423b1b0c6` |
| 통합 순서 | 4/8 |

## 검토

line 끝의 forced break가 다음 행과 합쳐지는 #6300 페이지 회귀를 보정한다. 원 PR CI는 수집 시점에 비성공 check 없이 완료됐고 cherry-pick 충돌도 없었다.

통합 후보에서 `forced_break_at_line_end_does_not_merge_two_rows`, `page_count_moves_toward_the_hangul_oracle`가 통과했다. 공통 필수 clippy·build·manifest·format 검증도 통과했다.

원 PR의 비교 이미지는 변경 의도의 보조 근거로만 확인했다. hidden 원본 `156464313`은 repo fixture와 SHA-256이 같음을 확인했고, HWP MCP 2020 기준 PDF physical sheet 9의 좌측 slot을 통합 code head `export-svg -p 17` logical page와 직접 비교했다. `pixel_match=86.17842`이며 review PNG에서 `농수산식품` 문단의 4개 줄과 주석 block은 Hancom sheet와 같이 분리돼 있고, 빈 줄 뒤 과긴 line이나 우측 clip은 보이지 않았다. 대표 증적은 [N-up claim review PNG](../assets/pr_6481_issue6300_nup_claim_review.png)이며, physical-slot mapping과 원본/PDF SHA는 [PR #6481 visual sweep 기록](pr_6481_planet6897_visual_sweep.md)에 보존했다.

## #6481 당시 결론 (역사)

**당시 판정: 승인.** forced break와 페이지 수는 사용자-visible 결과이므로 contract test만으로 수용하지 않고, N-up sheet의 실제 slot에 쟁점 문단을 매핑해 직접 판정했다. 차단 finding은 없으며 수치는 forced-break claim 근거에 한정한다. 원 PR은 직접 merge하지 않고 #6481 통합 결과로만 수용한다. remote push, merge, #6447 close는 별도 지시가 있을 때만 수행한다.

## #6485 최신 통합 판정

현재 수용 대상은 PR #6485 code candidate `f47d5b3586d470c99ed38f155af18175801f3c85`의 `03c1d494c`이며 원 PR head는 직접 병합하지 않는다. focused 2건, 전체 nextest 8,785건, Native Skia, WASM, lint가 실제 통과했다. private original `156464313`의 HWP MCP 2020 PDF `pdf/pr6485-visual/pr6485-issue6300-trade-report-forced-break-object-2020.pdf`(SHA-256 `a5e20780244dba528173ce3b35c85657e4c10ccf3751276d203db1b386360437`)에서 physical sheet 9 left와 logical p17을 대조했다. `pixel_match=86.17842`이며 쟁점 문단의 4개 줄과 주석 block이 보존되고 우측 clip이 없다. footer `-17-` 대 `-16-` 차이는 남은 전체 pagination 차이로 forced-break 주장 범위 밖이다. 대표 PNG는 [N-up claim review PNG](../assets/pr_6485_issue6300_nup_claim_review.png)다.

**최종 판정: 승인.** #6485 최신 trailing head CI와 명시적 merge 승인이 남은 조건이다.

## Merge 후 contributor PR comment 계획

#6485 merge SHA와 실제 PR/devel CI, N-up slot 및 위 수치와 pagination 잔여 범위를 [Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment), `<merge-commit-sha>` 고정 raw PNG URL로 한 번 게시한다.
