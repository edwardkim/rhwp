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
런타임 재매핑 규약과 맞지 않았다. 독립 재검토에서는 빈/오타 precondition 객체도 CAS 없이
실행되고, 해시를 시작 시 한 번만 확인한 뒤 잠금 없이 저장해 두 동시 writer가 모두 검사를
통과하고 마지막 저장이 앞 편집을 덮을 수 있음을 추가 확인했다.

메인터너 코드 커밋 `aa7ba4e2`는 다음을 보정했다.

- `preconditions`가 명시되면 객체여야 하고, `inputSha256`가 명시되면 문자열이어야 한다.
- 잘못된 JSON 타입 5종을 exit 2와 디스크 무변경으로 고정했다.
- 새 CAS·schema registry 테스트가 런타임 binary 경로를 우선 사용하도록 바꿨다.

후속 코드 커밋 `7c1c0c13`과 회귀 커밋 `7152c849`는 다음을 보정했다.

- preconditions 객체는 `inputSha256` 하나를 반드시 가져야 하며 빈 객체·오타·미지 키를 거부한다.
- CAS가 명시된 run/edit만 canonical source 경로 기반의 안정적인 cross-process lock을 잡는다.
- 잠금 안에서 read-check-edit-write를 직렬화하고 저장 직전 입력 해시도 다시 대조한다.
- debug test의 두 child를 잠금 직전 barrier에 모으고 최초 해시 통과 marker가 하나뿐임을
  확인해, 잠금 제거 mutation이 결정적으로 실패하는 in-place 경합 회귀를 추가했다. 이 내부
  barrier 환경변수 경로는 release binary에서 컴파일하지 않는다(`44af0727`).

contributor 커밋은 rewrite하지 않았고 보정은 원 head의 single-parent 후속 커밋이다.

## 완료한 검증

| 검증 | 결과 |
| --- | --- |
| `cargo test --test run_plan_cas_contract --test schema_registry_contract` | 통과, 14/14 |
| malformed·빈·오타·미지 precondition 회귀 | 모두 exit 2, 산출물 없음 |
| 동시 in-place CAS 회귀 | 정확히 한 실행 exit 0, 다른 실행 exit 2, hash-pass marker 1개 |
| `git diff --check` | 통과 |
| 시각 검증 | 생략. CLI 계약·schema·테스트만 변경 |

## 리스크와 권고

- PR #4330을 먼저 merge하면 이 PR의 적층 schema 변경 범위가 명확해진다.
- 로컬 보정 head의 GitHub full CI는 push 뒤 새로 확인해야 한다.
- 잠금은 같은 canonical 경로를 쓰는 협력 rhwp CAS writer 사이의 advisory 경계다. hardlink alias와
  잠금에 참여하지 않는 외부 writer는 저장 직전 재검사 뒤의 짧은 경합 창까지 완전히 막지 못한다.
- 원격 push는 작업지시자의 메인터너 보정 승인 범위에서 수행하되 merge는 별도 승인 전까지 하지 않는다.

**보정 head의 full CI 통과와 #4330 선행 정리 후 조건부 merge 권고.**
