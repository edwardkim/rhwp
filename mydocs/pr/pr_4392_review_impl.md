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
| maintainer code·test 1 | `f3f12e6501b87f8f3dbecf13f05bb0aa7d6c9368` — 해시 입력 스냅샷 실행과 nextest 경로 보정 |
| maintainer review 1 | `58dbf9be4ce58d2635d12a0ac698a1eab0d199d3` — 최초 보정 검토 기록 |
| maintainer code·test 2 | `ffe7e4e1fd30cc65a097c9bad7b7f88e244afdab` — private scratch, Unix 권한, RAII 정리 |
| maintainer review 2 | 이 문서와 `pr_4392_review.md`의 trailing 갱신 커밋 |

## 단계

1. API head, fork source ref, local source SHA를 대조했다.
2. contributor history를 유지한 채 TOCTOU code·test 보정과 최초 review 기록을 분리해 추가했다.
3. 후속 보안 검토에 따라 입력·산출을 private scratch에 격리하고 focused replay/unit 회귀를 통과했다.
4. 이 후속 판단과 실제 검증은 다시 별도 trailing review commit으로 기록한다.
5. push 직전 source SHA 재확인, LFS 판독, dry-run을 수행한다.
6. push 뒤 최신 head full CI와 Unix 권한 단언, mergeability를 확인한다.
7. #4399, #4406에는 해당 head 구조에 맞는 별도 보정을 적용하며 contributor commit을 재작성하지 않는다.
8. merge는 작업지시자의 별도 승인 전에는 수행하지 않는다.

## rollback

문제가 생기면 maintainer review 2, `ffe7e4e1`, maintainer review 1, `f3f12e65`를 역순
revert한다. contributor source는 amend, rebase 또는 force-push하지 않는다.
