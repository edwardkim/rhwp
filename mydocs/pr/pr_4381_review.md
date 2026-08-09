---
kind: pr_review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-10
---

# PR #4381 검토 — 입력 SHA-256 CAS

## 라우팅

base route: `maintainer_general.md`

modifiers: `intake_and_review.md`, `local_validation.md`,
`multi_pr_update_branch.md`, `rework_and_exceptions.md`

## 메타데이터와 범위

| 항목 | 값 |
| --- | --- |
| 원 PR / 작성자 | [#4381](https://github.com/edwardkim/rhwp/pull/4381) / @kevin9327 |
| base | `devel` |
| 원 PR head | `026b947f64bffe807fda98a46c0aba2b7ba2c7c1` (2026-08-10 접수 시점) |
| 기준 devel | `e48fe86947fbf9a44b1b98c7037150751af541ab` |
| 가시성 브랜치 | `review/kevin9327-20260810-pr4381` |
| 원 변경 규모 | 22파일, `+1091/-83`, 커밋 3개 |
| 적층 관계 | PR #4330의 schema registry 커밋을 포함 |

원 변경은 plan `preconditions.inputSha256`와 단발 edit의 `--expect-sha256`을 추가해
동시 편집의 유실을 차단한다. renderer, pagination, fixture, baseline은 바꾸지 않으므로 visual
sweep 대상은 아니다. 1,000줄을 넘는 적층 PR이므로 대형 PR 예외 경로를 함께 적용했다.

## 발견한 차단 결함과 메인터너 보정

원 head는 `preconditions.inputSha256`를 `as_str()`으로만 읽었다. 키가 숫자, 불리언,
객체 또는 null이면 문자열 변환 실패가 곧 "전제조건 없음"으로 처리되어 CAS가 fail-open 됐다.
또한 새 CLI 계약 테스트는 컴파일 시점 `CARGO_BIN_EXE_rhwp`를 직접 실행해 nextest archive의
런타임 재매핑 규약과 맞지 않았다.

메인터너 코드 커밋 `aa7ba4e2`는 다음을 보정했다.

- `preconditions`가 명시되면 객체여야 하고, `inputSha256`가 명시되면 문자열이어야 한다.
- 잘못된 JSON 타입 5종을 exit 2와 디스크 무변경으로 고정했다.
- 새 CAS·schema registry 테스트가 런타임 binary 경로를 우선 사용하도록 바꿨다.

contributor 커밋은 rewrite하지 않았고 보정은 원 head의 single-parent 후속 커밋이다.

## 완료한 검증

| 검증 | 결과 |
| --- | --- |
| `cargo test --test run_plan_cas_contract --test schema_registry_contract` | 통과, 12/12 |
| malformed string·non-string·non-object CAS 회귀 | 모두 exit 2, 산출물 없음 |
| `git diff --check` | 통과 |
| 시각 검증 | 생략. CLI 계약·schema·테스트만 변경 |

## 리스크와 권고

- PR #4330을 먼저 merge하면 이 PR의 적층 schema 변경 범위가 명확해진다.
- 로컬 보정 head의 GitHub full CI는 push 뒤 새로 확인해야 한다.
- 원격 push는 작업지시자의 메인터너 보정 승인 범위에서 수행하되 merge는 별도 승인 전까지 하지 않는다.

**보정 head의 full CI 통과와 #4330 선행 정리 후 조건부 merge 권고.**
