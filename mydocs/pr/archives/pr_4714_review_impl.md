---
kind: implementation-record
status: active
canonical: mydocs/pr/archives/pr_4714_review_impl.md
last_verified: 2026-08-13
---

# PR #4714 메인터너 보정 및 처리 기록

## 목적

옵트인 플랫 스킨의 기능 변경은 유지하면서, 의존성 선언 변경 없이 삭제된 Linux native binding의
`libc` lockfile 조건만 복원한다. contributor source history를 재작성하지 않고 동일 PR head 위에
메인터너 commit을 추가한다.

## 적용 순서

| 순서 | SHA | 내용 |
| --- | --- | --- |
| 1 | `cfd6505ed` | 플랫 스킨, 설정 배선, 메뉴와 정적 테스트 추가 |
| 2 | `406171db5` | 테마 토큰·스킨 제작 가이드 추가 |
| 3 | `b63d2f3aa` | submenu clipping 수정 |
| 4 | `31d474e1d` | 메인터너: Linux `glibc`/`musl` lockfile 조건 복원 |

## 보정 근거

`package.json`에 의존성 변경이 없는데 `package-lock.json`에서 native optional package의 `libc`
필드만 삭제됐다. 이 변경은 기능 범위와 무관하며 Linux별 optional binary 선택을 넓힌다.
`31d474e1d`는 최신 `devel`의 동일 메타데이터를 복원해 설치 결정성을 유지한다.

## 실행 단계

1. contributor remote ref, PR head, visibility review branch가 `b63d2f3aa`로 같은지 확인했다.
2. `maintainerCanModify=true`와 LFS 비대상(`package-lock.json`)을 확인했다.
3. lockfile 보정 commit을 만들고 `GIT_LFS_SKIP_PUSH=1` dry-run을 통과했다.
4. 승인 뒤 contributor source branch에 `31d474e1d`를 push했다.
5. 최신 code head의 Frontend package gates, CodeQL, Canvas visual diff와 aggregate 성공을 확인했다.
6. 이 archive review·오늘할일을 trailing docs-only commit으로 추가한 뒤 fast-pass aggregate와
   mergeable 상태를 다시 확인한다.
7. 작업지시자 승인에 따라 merge한 뒤 PR comment, `devel` 동기화, review branch 정리를 수행한다.

## 되돌리기 범위

보정 자체의 회귀가 확인되면 contributor 기능 commit이 아니라 `31d474e1d`만 revert한다. 스킨 기능의
문제가 확인되면 해당 기능을 별도 후속 PR에서 다루며, contributor commit을 rebase·amend·force-push하지 않는다.
