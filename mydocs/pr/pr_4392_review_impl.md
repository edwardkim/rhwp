---
kind: pr_review_impl
status: active
canonical: mydocs/pr/pr_4392_review.md
last_verified: 2026-08-10
---

# PR #4392 메인터너 보정 실행 기록

## 커밋 경계

| 구분 | SHA / 내용 |
| --- | --- |
| contributor source | `3d2a18cbae00c873ab1508bfaf5b3c0270793f15` |
| maintainer code·test | `f3f12e65` — 해시 입력 스냅샷 실행과 nextest 경로 보정 |
| maintainer review | 이 문서와 `pr_4392_review.md`의 후속 기록 커밋 |

## 단계

1. API head, fork source ref, local source SHA를 대조했다.
2. contributor history를 유지한 채 code·test 보정 커밋을 추가하고 focused 테스트를 통과했다.
3. review 기록은 별도 trailing commit으로 추가한다.
4. push 직전 source SHA 재확인, LFS 판독, dry-run을 수행한다.
5. push 뒤 최신 head full CI와 mergeability를 확인한다.
6. #4399, #4406에는 해당 head 구조에 맞는 별도 보정을 적용하며 contributor commit을 재작성하지 않는다.
7. merge는 작업지시자의 별도 승인 전에는 수행하지 않는다.

## rollback

문제가 생기면 review commit과 `f3f12e65`만 역순 revert한다. contributor source는 amend,
rebase 또는 force-push하지 않는다.

