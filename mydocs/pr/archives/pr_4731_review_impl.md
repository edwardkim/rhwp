---
kind: pr-review-implementation
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-13
---

# PR #4731 메인터너 보정 구현 기록

## 라우팅

```text
base route: collaborator_external_pr
modifiers: intake_and_review, local_validation, review_only_fast_pass, post_merge
```

외부 contributor source branch `kevin9327/rhwp:fix_hwpx_container_recursion`는 변경하지
않는다. 최신 `upstream/devel` `b5c14346d0` 위 review branch
`review/kevin9327-pr4731-20260813`에 원 head `f22568721e`의 기능 commit을
`3c033c537`으로 cherry-pick했다. 그 위에 메인터너 보정 `3de1aff81`을 추가하고,
결과를 [통합 PR #4738](https://github.com/edwardkim/rhwp/pull/4738)으로 올렸다.

## stage

1. 원 PR의 최신 head, #4730의 네 재귀 경로, 기존 HWP3/HML 깊이 제한과 현재 `devel`을 대조했다.
2. 256 상한이 `parse_container`의 큰 기본 debug 프레임보다 안전하다는 증거가 없고, 원 회귀가
   32MiB 전용 스레드에서만 실행됨을 확인했다.
3. 한계를 64개 그룹으로 낮추고 `>=` 경계로 65번째를 거부하게 했다. 기본 스레드의 상한 초과와
   정확히 상한인 정상 입력을 각각 시험하도록 회귀를 고쳤다.
4. #4730이 아직 완료되지 않았으므로 원 PR description의 `closes #4730`을 관련 이슈 표현으로 정정했다.
5. Windows PowerShell에서 debug·release-test focused 회귀, 대상 파일 Rustfmt, 공백 검사를 통과시켰고,
   code candidate `3de1aff81`의 Full CI·CodeQL·Build & Test aggregate 성공을 확인했다.

## 경계와 rollback

이 통합은 #4730의 HWPX container 경로만 수정하며, 표 재귀와 HWP5 재귀 경로를 해결 또는 close하지
않는다. 보정을 되돌려야 하면 contributor 원 commit·fork branch를 건드리지 않고 통합 branch에
되돌림 commit을 추가한다.

이 문서와 오늘할일은 code candidate 뒤의 review-only trailing commit이다. push 뒤 최신 head의
fast-pass preflight와 Build & Test aggregate, mergeability를 확인한 다음 self-review와 merge로 진행한다.
