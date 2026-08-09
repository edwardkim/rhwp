---
kind: pr_review_impl
status: active
canonical: mydocs/pr/pr_4410_review.md
last_verified: 2026-08-10
---

# PR #4410 메인터너 보정 실행 기록

## 커밋 경계

| 구분 | SHA / 내용 |
| --- | --- |
| contributor source | `cc0d4678e2141ad4f04cf904a4117c59cf18d2a3` |
| maintainer design correction | `67e7c3bcdb29753532c4f4d500cbf2d5d003b6d0` — edge input 결속, 안전한 traversal/cache, 사실 보정 |
| maintainer review 1 | `2c0a70e8bc69cfaeccaf338c1ea4c552678745c7` — 최초 보정 검토 기록 |
| maintainer follow-up design correction | `f42bf99fb39de80f8ff48d03bff2130aceec7ef5` — access path identity, 완전한 DAG 예시, C2PA §7.3.2 경계 |
| maintainer review 2 | `047d286ef28b847d85c65eb8c35097cf5581dfb2` — 첫 후속 검토 갱신 |
| maintainer follow-up design correction 2 | `58cf7d4c` — root external input과 비-root binding 경계 |
| maintainer review 3 | `fe33191b` — root-input과 C2PA review 갱신 |
| maintainer record closure | 이 문서와 `pr_4410_review.md`의 trailing 기록 정합 commit |

## 단계

1. PR API head, fork `task_m100_4407` ref, fetch한 local ref가 모두 contributor
   source SHA와 같은지 확인했다.
2. `devel`이 source의 조상이고 원 diff가 문서 2개뿐임을 확인해 같은 가시성
   브랜치에서 검토를 이어갔다.
3. #4406 실제 replay/lineage 계약과 원문 자료를 대조해 material edge, cache key,
   방문 identity, C2PA 설명의 차단 오류를 보정했다.
4. 후속 독립 검토 finding에 따라 hardlink dedup 금지, 5-node/6-edge 예시,
   C2PA active asset/ingredient validation 경계를 별도 문서 commit으로 보정했다.
5. 마지막 독립 검토에서 root source input과 비-root parent binding 규약의 모순을 닫고
   review의 C2PA 요약도 같은 경계로 정렬했다.
6. 내부 링크, 문서 metadata, roadmap 집계, v1.1 JSON graph 불변식,
   `git diff --check`를 통과했다.
7. 후속 review 갱신을 design correction과 분리된 trailing commit으로 만든다.
8. 상위 작업이 push를 결정하면 직전 remote SHA, LFS 대상, dry-run을 다시 확인하고
   push 뒤 최신 문서-only aggregate를 기다린다.
9. merge는 작업지시자의 별도 승인 전에는 수행하지 않는다.

## rollback

문제가 생기면 record closure, `fe33191b`, `58cf7d4c`, `047d286e`, `f42bf99f`, `2c0a70e8`,
`67e7c3bc`를 역순 revert한다. contributor commit은 amend, rebase 또는 force-push하지
않는다.
