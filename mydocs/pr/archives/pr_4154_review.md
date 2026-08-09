---
kind: pr_review
status: merged-post-review
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-08
---

# PR #4154 사후 검토 — tracked `output/poc` 정리

## 결론

[PR #4154](https://github.com/edwardkim/rhwp/pull/4154)의 merge 자체에는 code·test·fixture·렌더
회귀가 없다. 실제 merge tree는 최신 `devel`에 이미 `/output/` ignore 대상인 tracked POC 10개를
삭제한 것뿐이며, 삭제 상태를 유지한다. 삭제 자료는 복원하지 않는다.

다만 Draft 본문에 남아 있던 “참조된 5개 유지 또는 전체 삭제” 선택을 merge 전에 기록하지 않았고,
reviewer·review 문서·오늘할일 게이트도 생략했다. 사후 결정은 **전체 10개를 임시 POC로 보고 삭제 유지**다.
활성 문서가 가리키던 5개 경로는 역사적·비추적 산출물로 명시하고, 현행 재현 근거는 repository fixture,
test와 최종 보고서로 연결한다.

## 검토 경로

```text
base route: collaborator_self_merge.md
modifiers: intake_and_review.md, local_validation.md,
           review_only_fast_pass.md, post_merge.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
                  collaborator_self_merge.md, intake_and_review.md,
                  local_validation.md, review_only_fast_pass.md, post_merge.md
original PR head: 445b6a936fcd23926c16b99f2146cb76ff808e90
merge commit: dc7d7adcc6e6b40b4a3822594e525382742bdea9
post-review base: devel @ 30bad7c1d4c3799d2e4df4027c465a972dc9559d
```

## 메타데이터와 merge

| 항목 | 값 |
| --- | --- |
| PR | [#4154](https://github.com/edwardkim/rhwp/pull/4154) |
| 작성자 / merge 실행자 | `edwardkim` / `edwardkim` |
| base / head | `devel` / `local/poc-output-cleanup-20260807` |
| source head | `445b6a936fcd23926c16b99f2146cb76ff808e90` |
| 생성 시점 base | `cb0a3b86ddb5dcdc46af58bb51fb2a1cb580b7e6` |
| 실제 merge 첫 부모 | `5a4f26d0d0a4e2fc96f4b73510d2aecdad916722` |
| merge commit | `dc7d7adcc6e6b40b4a3822594e525382742bdea9` |
| merge 시각 | 2026-08-08 10:26 KST |
| 규모 | 1 commit, 10 files, +0 / -1,397 |
| metadata | assignee `edwardkim`, milestone `v1.0.0`, label `documentation` |
| 관련 이슈 | 없음 |

GitHub timeline에서 Ready 전환은 10:26:23, merge는 10:26:30이었다. review request·review·PR comment는
없었고 assignee·label·milestone은 merge 뒤에 설정됐다. 이번 사후 기록은 이 절차 누락을 성공 사례로
정당화하지 않으며, 이후 collaborator self PR에는 merge 전 reviewer와 review 기록 게이트를 적용한다.

## 변경 범위와 삭제 결정

삭제된 파일은 모두 루트 `.gitignore`의 `/output/` 규칙 아래에 있었다.

- `output/poc/glyphoverlap_20260803/`: 진단 TSV·일회성 스캔 스크립트 2개
- `output/poc/textfidelity_r30_20260803/`: 진단 TSV 1개
- `output/poc/textorder_20260803/`: 진단 TSV·일회성 스캔 스크립트 2개
- `output/poc/task2004/`: 역사적 페이지 기준선 1개
- `output/poc/task2019/`: 역사적 기준선·표본 목록·캡처 하네스·오라클 메모 4개

현재 source, test, script와 package에서는 이 경로를 실행 입력으로 참조하지 않는다. `mydocs`에 남아 있던
task2004·task2019 참조는 검증 당시 산출 위치를 기록한 역사 문맥이다. 삭제 전 내용은 merge 첫 부모의
[`output/poc`](https://github.com/edwardkim/rhwp/tree/5a4f26d0d0a4e2fc96f4b73510d2aecdad916722/output/poc)
Git 이력에서 확인할 수 있지만 active tree로 복원하지 않는다.

현행 장기 회귀 근거는 다음 tracked 자산이다.

- [#2004 HWP/HWPX 페이지네이션 래칫](../../../tests/issue_2004_cell_image_stack_pagination.rs)과
  `samples/issue2004_cell_image_stack.{hwp,hwpx}`
- [#2019 과분할 래칫](../../../tests/issue_2019_floating_form_overpagination.rs),
  `samples/hwpx/issue2019_floating_form_74312.hwpx`,
  [최종·정정 보고서](../../report/task_m100_2019_report.md)

## 사후 검증

| 검증 | 결과 |
| --- | --- |
| merge commit 첫 부모 대비 범위 | 삭제 10개만 존재 |
| `git diff --check <merge^1> <merge>` | 통과 |
| `.gitignore` | 5개 역사적 참조 경로 모두 `/output/`에 match |
| 현재 tracked `output/` | 0개 |
| source·test·script 실행 참조 검색 | 0건 |
| 영향 문서 targeted link check | 6개 문서, 내부 상대 링크 이상 없음 |
| GitHub CI | run `31159210894`, success |
| GitHub CodeQL | run `31159210173`, success |
| `devel` 포함 | merge commit이 현재 `upstream/devel`의 조상 |

원 PR CI는 source head에서 merge 전 완료됐다. 그 뒤 base가 전진했지만 실제 merge 결과는 base 변경과
겹치지 않는 `output/poc` 삭제 10개뿐이다. Render Diff는 실행되지 않았으나 renderer, layout, sample,
PDF, golden과 baseline fixture를 바꾸지 않았으므로 별도 시각 게이트 대상이 아니다.

## 사후 보완과 남은 처리

- [#2004 Stage 1](../../working/task_m100_2004_stage1.md)과 계획 문서의 `output/poc/task2004`를 역사적
  산출물로 명시하고 tracked test·fixture로 연결한다.
- [#2019 Stage 1](../../working/task_m100_2019_stage1.md)과 구현계획의 `output/poc/task2019`를 역사적
  산출물로 명시하고 tracked report·test·fixture로 연결한다.
- [#2220 보고서](../../report/task_m100_2220_report.md)의 “복원”은 당시 사건으로 보존하되, #4154에서
  이후 의도적으로 제거했음을 덧붙인다.
- `mydocs/metrics/frontend/2026-07-11/metrics.json`의 경로 목록은 당시 inventory snapshot이므로
  수정하지 않는다.
- 원 PR branch와 `/tmp/rhwp-output-cleanup-pr` worktree는 별도 삭제 승인 전까지 유지한다.

이 보완은 완료된 원 PR의 기록만 담는 `mydocs/` 전용 후속 변경이다. 원 PR의 issue·comment를 반복하지
않고, 후속 PR을 만들면 review-only B 경로 preflight와 최종 aggregate만 확인한다.
