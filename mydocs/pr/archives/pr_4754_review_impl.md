---
kind: pr-review-implementation
status: active
issue: 4098
pr: 4754
---

# PR #4754 메인터너 보정 기록

## 기준과 commit 경계

| 순서 | SHA | 내용 |
| --- | --- | --- |
| contributor 구현 | `22d04d32e` | `VtDataGrid` 구조 스캐너·방향 판정·코퍼스 계약 |
| contributor 형식 정리 | `fd19a9f4e` | rustfmt 정합 |
| 메인터너 보정 | `bb2366115` | 손상 치수 사전 할당 제거, window 시작 정합, 회귀 2건 |

가시성 branch `review/johndoekim-4754-20260814`는 contributor source
`fd19a9f4e2c865b35e769fe6a0bdd14242dd9bd1`에서 시작했다. `maintainerCanModify=true`와 fork remote
SHA가 일치하는 것을 확인한 뒤, contributor history는 rebase/amend/force-push하지 않고 메인터너
commit 하나만 그 위에 추가했다.

## 보정 단계

1. `rows * cols`를 `Vec::with_capacity`에 직접 전달하던 경로를 제거했다. 선언 치수는 셀 수의
   정합 검증에만 쓰고, 실제 저장소는 입력에서 찾은 셀 수에 따라 늘어난다.
2. `legacy_grid_window`의 시작을 marker 뒤 `u16 version + u32 payload`로 계산했다. scanner의
   `read_prologue`와 같은 오프셋 문법을 쓰며 checked addition으로 범위 계산도 닫았다.
3. 최대 `u16` 치수의 작은 합성 스트림이 메모리 예약 없이 구조 오류로 끝나는지, window가
   `VtDataGrid` 선언 끝에서 시작하는지를 회귀로 고정했다.
4. 모듈·통합·전체 회귀와 lint를 순차 실행했다. 결과는 [review 기록](pr_4754_review.md)에
   완료 사실로 기록했다.

## 원격 반영과 남은 단계

`bb2366115`는 LFS 대상이 없는 것을 사전 판독하고 `GIT_LFS_SKIP_PUSH=1` dry-run을 확인한 뒤,
`johndoekim/rhwp:task_m100_4098_ole_chart_grid`에 일반 push했다. fork remote, PR head, local SHA가
일치하는 것을 확인했다.

code candidate의 Full CI·CodeQL이 녹색이 된 뒤 review·오늘할일을 별도 trailing docs-only commit으로
작성한다. 이 commit은 source/test/fixture/workflow를 포함하지 않으며 fast-pass aggregate 성공 후에만
merge 후보가 된다. merge 후에는 #4098 상태, PR comment, `devel` 동기화, local review branch 정리를
순서대로 확인한다.
