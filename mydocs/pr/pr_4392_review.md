---
kind: pr_review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-10
---

# PR #4392 검토 — replay 작업 영수증

## 라우팅

base route: `maintainer_general.md`

modifiers: `intake_and_review.md`, `local_validation.md`,
`multi_pr_update_branch.md`

## 메타데이터와 범위

| 항목 | 값 |
| --- | --- |
| 원 PR / 작성자 | [#4392](https://github.com/edwardkim/rhwp/pull/4392) / @kevin9327 |
| base | `devel` |
| 원 PR head | `3d2a18cbae00c873ab1508bfaf5b3c0270793f15` (2026-08-10 접수 시점) |
| 기준 devel | `e48fe86947fbf9a44b1b98c7037150751af541ab` |
| 가시성 브랜치 | `review/kevin9327-20260810-pr4392` |
| 원 변경 규모 | 10파일, `+466/-3`, 커밋 2개 |

원 변경은 plan을 임시 산출로 재실행하고 입력·계획·출력 SHA-256 영수증을 발급한다.
renderer, fixture, baseline은 바꾸지 않아 visual sweep 대상은 아니다.

## 발견한 차단 결함과 메인터너 보정

원 head는 입력을 한 번 읽어 `inputSha256`를 계산한 뒤 `run_plan_engine()`이 같은 경로를
다시 열었다. 두 읽기 사이에 다른 프로세스가 입력을 교체하면 영수증은 첫 바이트를 서명하지만
산출물은 두 번째 바이트에서 만들어지는 TOCTOU가 발생한다.

메인터너 코드 커밋 `f3f12e65`는 해시한 입력 바이트를 exclusive 임시 파일에 고정하고 엔진이
그 스냅샷만 읽도록 했다. 실행 뒤 원 plan input을 복원하고 임시 파일을 제거한다. 원 파일을
해시 뒤 교체해도 엔진 closure가 해시된 스냅샷을 받는 unit 회귀를 추가했으며, 새 replay CLI
테스트도 nextest 런타임 binary 경로를 우선 사용한다.

## 완료한 검증

| 검증 | 결과 |
| --- | --- |
| `cargo test --test replay_contract` | 통과, 4/4 |
| `cargo test --bin rhwp replay_engine_receives_the_hashed_input_snapshot` | 통과, 1/1 |
| `git diff --check` | 통과 |
| 시각 검증 | 생략. replay 실행·영수증·테스트만 변경 |

## 리스크와 권고

- 임시 입력은 같은 확장자를 유지하며 create-new 이름 충돌을 제한된 재시도로 처리한다.
- 보정 head의 Windows 외 러너와 full CI는 push 뒤 확인해야 한다.
- 후속 적층 PR #4399와 #4406도 각각 같은 결함을 독립 보정해야 한다.

**최신 head full CI 통과 후 조건부 merge 권고. merge는 별도 승인 대상이다.**

