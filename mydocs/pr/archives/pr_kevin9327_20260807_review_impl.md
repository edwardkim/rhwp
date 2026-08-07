---
kind: review-implementation
status: completed-local-pending-push-approval
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-07
---

# kevin9327 누적 PR 검토와 메인터너 보정 기록

## 누적 적용

| 순서 | 원 PR | 원 SHA | 로컬 cherry-pick |
| --- | --- | --- | --- |
| 1 | #4105 | `be71772f47ac71ede7ba868ecca7a873b753ba31` | `7f637f594` |
| 2 | #4106 | `b7dd83c6c275d7217df0b730372ae69130fe0312` | `2a2aa3b26` |
| 3 | #4108 | `4c9f79d455e5f081fc378682f4ad50bfd42063fe` | `07adef498` |
| 4 | #4114 | `594398d0fcc121e06ba1b9781e1b4fbaefe274bb` | `15bea52e5`, `c3feec6c7` |
| 5 | #4116 | `958158d994bed68d54f2c4a58777bf5df041b13a` | `4e537c688` |

모든 cherry-pick은 최신 기준선 `9f564bbe` 위에서 충돌 없이 적용됐다. 가시성·통합 검토 브랜치는
`review/kevin9327-20260807`이다.

## 메인터너 보정

`1171d57c5`는 다음 보정을 로컬 통합 branch에 추가했다.

1. preflight의 착수 잠금은 단어 포함이 아니라 `착수합니다 — <범위>` 형식만 인식한다.
2. 단어 인용과 실제 착수 형식의 Python 회귀 2건을 추가했다.
3. #4106 구현에 맞춰 protocol §8-6의 구현 상태를 정렬했다.
4. R7, R74, R79, R94와 README 기계 집계를 누적 구현 상태로 정렬했다.
5. 최초 node-binding CI가 발견한 세 capabilities JSON 명령의 Node wrapper를 추가하고,
   capabilities 생성 타입을 31개에서 34개 봉투로 재생성했다.

`5db125857`은 CI 워크플로에 Python 회귀를 추가했던 변경을 작업지시자 지시에 따라 별도 revert했다.
따라서 통합 branch의 최종 CI 워크플로 diff는 없다.

## 검증과 후속 단계

- Python queue regression 2 passed, roadmap generator 검증, preflight static-only, workflow contract
  3 passed, `actionlint`, `git diff --check`를 통과했다.
- capabilities 계약 4+17 passed, MCP resources·server 계약 3+22 passed, `cargo fmt --check`,
  `cargo clippy --all-targets -- -D warnings`를 통과했다.
- Node binding은 실제 `target/release-test/rhwp`와 17 files·445 tests, typecheck, build,
  생성 타입 최신 검사를 통과했다.
- Python·문서 보정 뒤 Cargo 전체 회귀는 실행하지 않았다.
- 원격 push, 통합 PR 생성, GitHub comment, merge는 아직 수행하지 않았다.
- 통합 PR을 만들려면 현재 branch의 review 문서 commit을 추가한 뒤, 최신 `upstream/devel`과 원 PR
  heads를 재확인하고 작업지시자 승인을 받아야 한다. #4114는 원 source head에 CI가 없으므로 통합 PR의
  새 CI가 성공해야 한다.
