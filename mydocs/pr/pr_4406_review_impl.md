---
kind: pr_review_impl
status: active
canonical: mydocs/pr/pr_4406_review.md
last_verified: 2026-08-10
---

# PR #4406 메인터너 보정 실행 기록

## 커밋 경계

| 구분 | SHA / 내용 |
| --- | --- |
| contributor source | `d69abf374fffe40f5ac99c66171fc4876075b263` |
| maintainer code·test | `7584408f` — 계보·receipt fail-closed와 replay 입력 스냅샷 |
| maintainer review | `487e0700` — 최초 보정 검토 기록 |
| maintainer follow-up code·test | `37d866ed` — plan 결속, 경로 경계, private scratch |
| maintainer follow-up code·test 2 | `2d593563` — shallow step 결속과 audit 열거 fail-closed |
| maintainer review update 1 | `9bfc81fb09649270daa5618a1398e009ae91628b` — 후속 보정 검토 갱신 |
| maintainer follow-up code·test 3 | `5012f519e7367aa276463a6d30a216950efb159d` — strict capsule JSON과 parent 자기덮어쓰기 차단 |
| maintainer review update 2 | 이 문서와 `pr_4406_review.md`의 trailing 갱신 커밋 |

## 단계

1. 원 PR head, fork source ref, 수정 권한을 대조했다.
2. contributor history 위에 code·test 보정을 추가하고 독립 재검토 finding을 별도 후속 코드 커밋으로 닫았다.
3. strict UTF-8 파싱과 capsule/parent 동일 실파일 차단을 세 번째 code·test 커밋으로 추가했다.
4. Windows replay·audit·lineage·unit focused 회귀 14개를 통과하고 review 갱신은 코드와 분리된
   trailing commit으로 만든다.
5. push 직전 source SHA 재확인, LFS 비대상 판독, dry-run을 수행한다.
6. push 뒤 최신 head full CI, Unix symlink 회귀와 mergeability, #4392·#4399 선행 상태를 확인한다.
7. merge는 작업지시자의 별도 승인 전에는 수행하지 않는다.

## rollback

문제가 생기면 review update 2, `5012f519`, review update 1, `2d593563`, `37d866ed`,
`487e0700`, `7584408f`를 역순 revert한다. contributor commit은 amend, rebase 또는
force-push하지 않는다.
