---
kind: pr_review_impl
status: active
canonical: mydocs/pr/pr_4399_review.md
last_verified: 2026-08-10
---

# PR #4399 메인터너 보정 실행 기록

## 커밋 경계

| 구분 | SHA / 내용 |
| --- | --- |
| contributor source | `a102ee062667bcc139baff32223f5570da69b77b` |
| maintainer code·test 1 | `e73bef6f58e141d3b5c250c199c5648667cad2ea` — replay snapshot과 audit 입력 영수증 대조 |
| maintainer review 1 | `33aa9269982b2dbd4a4973f8872a4d06aec29928` — 최초 보정 검토 기록 |
| maintainer code·test 2 | `76d6580d405a9b998ac0e1e73a8716eb3e547829` — private scratch와 capsule/audit fail-closed 결합 |
| maintainer review 2 | 이 문서와 `pr_4399_review.md`의 trailing 갱신 커밋 |

## 단계

1. 원 PR head, fork source ref, 수정 권한을 대조했다.
2. contributor history 위에 TOCTOU·입력 영수증 code·test 보정과 최초 review 기록을 분리해 추가했다.
3. 후속 보안 검토에 따라 private scratch, raw plan binding, step 이중 대조, 열거 fail-closed를
   추가하고 Windows focused replay·audit·unit 회귀 10개를 통과했다.
4. 이 후속 판단과 실제 검증은 다시 별도 trailing review commit으로 기록한다.
5. push 직전 source SHA 재확인, LFS 비대상 판독, dry-run을 수행한다.
6. push 뒤 최신 head full CI와 Unix 전용 권한·비 UTF-8 이름 회귀, mergeability, #4392 선행
   상태를 확인한다.
7. merge는 작업지시자의 별도 승인 전에는 수행하지 않는다.

## rollback

문제가 생기면 maintainer review 2, `76d6580d`, maintainer review 1, `e73bef6f`를 역순
revert한다. contributor commit은 amend, rebase 또는 force-push하지 않는다.
