---
kind: guide
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-28
---

# 첫 기여자 외부 PR 처리

이 문서는 rhwp에 처음 기여하는 외부 contributor의 PR을 검토·보정·merge할 때 적용하는
공식 절차다. 일반 외부 PR 절차의 기준 문서는
[PR 검토 workflow](../pr_review_workflow.md)와
[외부 contributor PR](collaborator_external_pr.md)다.

## 적용 범위

다음 조건을 모두 만족하면 이 절차를 적용한다.

- PR 작성자가 rhwp에 처음 기여한다.
- PR head가 contributor fork에 있고 maintainer가 보정 또는 merge를 진행한다.
- 변경은 코드, 테스트, 문서 또는 이들의 조합일 수 있다.

첫 기여자라는 이유로 검증 기준을 낮추거나 branch protection을 우회하지 않는다. 환영과
안전성은 함께 지킨다.

## merge 전 검토

1. PR head, fork remote branch, 검토 branch의 시작 SHA가 일치하는지 확인한다.
2. fork base와 최신 `upstream/devel`의 차이를 확인한다. base가 뒤처져 사실상 revert 또는
   충돌 위험이 있으면 즉시 merge하지 않고 rebase 또는 분리 PR을 안내한다.
3. PR 목적에 맞는 focused 검증을 수행하고, 실행하지 않은 PDF·시각·전체 회귀 검증을
   실행한 것처럼 기록하지 않는다.
4. source head에 review-only tail이 추가된 경우에도 code candidate와 trailing head의 CI를
   각각 최신 SHA 기준으로 확인한다.

## maintainer 보정

원 기여 구현이 수용 가능하지만 테스트, 문서 또는 최소한의 안전장치가 부족할 수 있다.
이 경우 다음을 지킨다.

1. contributor의 원래 변경과 maintainer 보정의 diff·commit을 분리한다.
2. 보정은 원 PR head 위에만 올린다. 관련 없는 `devel` 이력이나 다른 PR 변경을 fork branch에
   섞지 않는다.
3. 보정 사유, 보정 범위, 검증 결과와 검증하지 못한 범위를 review 문서와 PR comment에
   명시한다.
4. 사용자 또는 maintainer가 실제 동작을 확인한 경우에는 그 사실과 확인 주체를 기록하되,
   자동화 검증 결과로 바꾸어 표현하지 않는다.

## review·오늘할일 trailing commit

코드 후보의 최신 required CI가 성공한 뒤에만 다음 문서를 같은 source branch의 trailing
commit으로 추가한다.

- `mydocs/pr/archives/pr_N_review.md`
- `mydocs/orders/YYYYMMDD.md`

검토 정책을 함께 보완해야 하면 `mydocs/manual/pr_review_workflow.md`와 이 문서를 같은
trailing commit에 포함할 수 있다. trailing commit은 review, 오늘할일, 절차 문서만 포함해야
하며 source, test, fixture, workflow, baseline 변경을 섞지 않는다. push 전에는 LFS 대상 여부와
remote dry-run을 확인한다.

trailing head가 생성한 CI에서 fast pass가 허용되는지 preflight 결과로 확인한다. 허용되지 않으면
전체 CI를 정상적으로 완료해야 하며, 문서 변경이라는 이유로 required check를 우회하지 않는다.

## merge 및 후속 처리

1. trailing head의 최신 CI, `MERGEABLE`, `CLEAN` 상태를 확인한 뒤 merge한다.
2. merge SHA를 확인하고 `devel`을 `upstream/devel`에 fast-forward한다.
3. issue의 자동 close 여부를 확인하고, 필요한 경우 실제 merge·검증 결과를 담은 maintainer
   comment를 게시한다.
4. 원 PR에는 따뜻한 감사와 rhwp 첫 기여 환영을 명시한다. contributor의 기여 내용과 maintainer
   보정 사유를 구분하고, 실제 CI·로컬 검증·시각 검증 결과만 적는다.
5. contributor fork branch는 삭제하지 않는다. 이번 처리에서 만든 clean local branch, worktree,
   검토 전용 산출물만 [merge 후속 처리](post_merge.md)의 정리 절차로 정리한다.

## 기록 예시

```markdown
rhwp 첫 기여를 보내주셔서 감사합니다. 검토와 merge를 완료했습니다.

- 기여 구현: <원 기여 범위>
- maintainer 보정: <보정이 필요한 이유와 최소 범위>
- CI: <실제로 성공한 최신 head check>
- 로컬 검증: <실제로 실행한 focused 검증>
- 동작 확인: <사용자 또는 maintainer가 확인한 범위, 해당하는 경우>

contributor fork branch는 유지했습니다. 다음 기여도 환영합니다.
```
