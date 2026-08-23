---
kind: pr-review-implementation
status: code-ci-success-review-only-fast-pass-pending
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-24
---

# PR #5913, #5914 체리픽 통합 검토 기록

## 기준과 적용 순서

| 순서 | 원본 / SHA | 통합 branch commit | 처리 |
| --- | --- | --- | --- |
| 기준 | `upstream/devel` `343ed2c013606319b6418dd8c637c5e04047e304` | - | 최신 devel fast-forward 후 branch 생성 |
| 1 | #5913 `7c7e51594a1dd3179721951a8a157396f02d27c9` | `21c55a882` | p122 기능 commit 적용 |
| 2 | #5914 `95890d376d513bf1e21591cf99e593ba7dac9c99` | `59c4c6a38` | rowspan 기능 commit 적용 |
| 3 | #5914 `9fb1e199b1349f61bf25f6555cc6020bfec0c2d4` | `40423b836` | 77쪽 정정에 따른 기존 nested table 계약 page index 적용 |
| 4 | 메인터너 | `da2a3123c` | p122 vpos reset 범위 축소 |
| 5 | 메인터너 | `046e7da61` | TAC 그림의 PAPER/PAGE 크기 기준 해석 |

통합 branch는 `review/kevin9327-20260823`이다. #5914 source history의 `5ae9b1d`는 devel merge commit이므로
기능 중복을 막기 위해 체리픽하지 않았다. 적용 과정의 conflict는 없었다.

## 메인터너 보정

#5913 원 predicate는 저장 vpos 0의 일반 텍스트 연쇄도 쪽 경계로 취급했다. 원 CI에서 실패한
outline navigation 2건과 #1510 3건을 candidate에서 재현한 뒤, `stored_vpos_top_collision`을
빈-text control-anchor 충돌로 한정했다. p122의 세 stored-vpos 0 문단은 SectionDef/ColumnDef 빈 문단과
글자처럼 취급한 그림의 조합이어서 이 조건을 만족하지만, 일반 본문은 만족하지 않는다.

#5914 기능은 직접 수정하지 않았다. 다만 78쪽에서 77쪽으로 바뀐 결과에 맞춰 source follow-up test를
적용했고, #4698의 앞/연속 조각 expectation도 현재 candidate 64/65쪽에서 실행해 확인했다.

추가 시각 검토에서 p122 2쪽 TAC 그림의 `common.width=42520`, `height=22238`은 HWPUNIT가 아니라
`PAPER` 기준 425.20% x 222.38%임을 확인했다. 일반 object 경로와 달리 inline TAC 경로가 이 기준을
우회해 그림을 축소하던 결함을 보정했고, 그림의 확대ㆍclip 결과를 `p122-2022.pdf`와 다시 비교했다.

## 완료한 검증

| 게이트 | 결과 |
| --- | --- |
| #5913 focused gate | `p122_stored_vpos_page_break` 3 passed, 132 skipped |
| #5914/#1073/#4698 focused gate | 통과 |
| 전체 nextest | 8,208 passed, 41 skipped |
| Native-Skia lib | 현재 보정 작업 트리에서 exit code 0 |
| visual sweep | p122 1-3 재실행: blocker 0, 평균 pixel match 99.91133%, 2쪽 99.73398% |

## Code candidate CI

통합 PR [#5954](https://github.com/edwardkim/rhwp/pull/5954)의 code candidate는
`046e7da61fcb6b529f0f96b9f492c037f2abf579`이다. 이 exact head에서 GitHub Full CI(Build & Test,
lint, Native Skia, build archive와 모든 test shard), CodeQL, Canvas visual diff, Proptest roundtrip,
Adapter inter-diff가 모두 성공했다. CI 결과는 code candidate에 대한 과거형 증적이며, 뒤따르는
review-only commit의 결과를 미리 단정하지 않는다.

`upstream/devel@5057a7fcaf055b928e76115cdee4bc20bf0936f9`과의 merge-tree는
`8f91cf6909e0ed64c005769e04411fc404770600`으로 충돌 없이 생성됐고 `git diff --check`도 통과했다.

## 다음 단계와 merge 조건

1. #5913과 #5914를 같은 candidate에 유지할 수 있다. p122 2쪽 그림의 확대ㆍclip fidelity는 기준 PDF와
   다시 비교해 수용 가능한 수준으로 회복됐다.
2. archive reviewㆍasset과 오늘할일을 code candidate 뒤 review-only trailing commit으로 같은 PR에 추가한다.
3. trailing head의 fast-pass aggregate가 성공하고 `MERGEABLE/CLEAN`이며, merge 직전 해당 원 PR의 최신
   source head와 작업지시자 승인을 재확인한다.
4. merge 후 원 PR에는 통합 PR과 merge SHA를 연결한 comment를 남기고 close한다.

통합 PR 번호만을 위한 별도 review 문서는 만들지 않는다. 각 원 PR의 review 기록과 이 통합 구현 기록에
수용 근거와 메인터너 보정 이유를 함께 보존한다.
