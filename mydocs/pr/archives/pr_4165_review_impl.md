---
kind: pr_review_implementation
status: pending-push-approval
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-08
---

# PR #4165 메인터너 보정 이행 기록

## 고정한 기준

- contributor source head: `39c535583cb5e18bc4b54e79f825d73a0ac8a265`
- 최신 검토 기준: `upstream/devel` `d9c530ee8ed4bd0830ff35bc47e552bb0f32274f`
- 보정 뒤 candidate: `ae63f789860ca1af3131d6d644cfd5dc1fda7055`
- 가시성 검토 branch: `review/kevin9327-4165-20260808`
- contributor 원 고유 commit: `100868aa9`, `7efa8c18d`, `801cfced7`
- collaborator 보정 commit: `303c550e3`, `7470966f9`, `9cbb8dd35`, `ae63f7898`

## 완료한 단계

1. contributor head와 최신 `devel`의 merge tree를 확인해 `mydocs/orders/20260807.md` 한 파일 충돌을
   식별했고, PR 고유 변경이 없는 과거 오늘할일은 최신 기준 내용을 유지해 해소했다.
2. Markdown 링크 검사가 코드 영역의 대괄호 파일명을 내부 링크로 오인하는 문제를 별도 code/test
   commit으로 보정했다.
3. 최신 기준 r1 보고서의 다섯 열 검증 표와 `<code>` 문서 경로를 검증기가 처리하지 못하는 문제를
   별도 code/test commit으로 보정했다. 최초 실행 ERR과 재실행 ERR을 분리해 보고서의 2/4/1 분할을
   그대로 검증한다.
4. 정합성 보고서가 이전 기준의 `39 PASS · 2 CONTRADICTION`을 유지하던 문제를 고쳐 실제 실행 결과
   `41 PASS · 0 FAIL · 0 CONTRADICTION`과 일치시켰다.
5. 오라클 단위 3건, r1 전수 41건, 링크 단위 5건, scripts 발견형 118건, Python 구문, Markdown 링크,
   merge tree와 diff 검사를 완료했다.

## 남은 순서

1. push 직전 contributor 원격 SHA, PR head SHA, LFS 대상 여부를 다시 확인한다.
2. 작업지시자의 push 승인 뒤에만 `kevin9327/verify/oracle-r1-2020-consistency`에 candidate를 push한다.
3. 최신 remote head의 Full CI와 CodeQL, mergeable 상태를 확인한다. code/test 보정이 포함되므로
   fast-pass를 적용하지 않는다.
4. 작업지시자의 merge 승인 뒤에만 merge, contributor 결과 comment, `devel` 동기화와 검토 branch 정리를
   수행한다.
