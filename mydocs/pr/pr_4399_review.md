---
kind: pr_review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-10
---

# PR #4399 검토 — 작업 캡슐과 audit

## 라우팅

base route: `maintainer_general.md`

modifiers: `intake_and_review.md`, `local_validation.md`,
`multi_pr_update_branch.md`

## 메타데이터와 범위

| 항목 | 값 |
| --- | --- |
| 원 PR / 작성자 | [#4399](https://github.com/edwardkim/rhwp/pull/4399) / @kevin9327 |
| base | `devel` |
| 원 PR head | `a102ee062667bcc139baff32223f5570da69b77b` (2026-08-10 접수 시점) |
| 기준 devel | `e48fe86947fbf9a44b1b98c7037150751af541ab` |
| 가시성 브랜치 | `review/kevin9327-20260810-pr4399` |
| 원 변경 규모 | 12파일, `+883/-3`, 커밋 4개 |
| 적층 관계 | PR #4392 replay 커밋을 포함 |

원 변경은 replay 영수증을 plan과 묶은 작업 캡슐로 저장하고 폴더 단위 재현율을 audit한다.
renderer, fixture, baseline은 바꾸지 않아 visual sweep 대상은 아니다.

## 발견한 차단 결함과 메인터너 보정

적층된 replay는 입력 해시 계산 뒤 엔진이 원 경로를 다시 여는 TOCTOU를 포함했다. audit도
캡슐의 `receipt.inputSha256`를 확인하지 않아 입력이 달라져도 출력 해시만 맞으면 작업에 재현
credit을 줄 수 있었다. 새 replay·audit 테스트 일부는 nextest runtime binary 경로를 우선하지 않았다.

메인터너 코드 커밋 `e73bef6f`는 다음을 보정했다.

- replay·audit 공용 실행 코어가 입력을 한 번 읽고 그 바이트 스냅샷만 엔진에 전달한다.
- 공용 코어가 실제 입력 해시를 반환하고 audit가 receipt 입력 해시와 먼저 대조한다.
- 입력 receipt 변조를 실패로 고정하는 audit 회귀와 해시 뒤 원 파일 교체 unit 회귀를 추가했다.
- replay·audit CLI 테스트는 nextest 런타임 binary 경로를 우선한다.

contributor history는 rewrite하지 않았고 보정은 원 head의 single-parent 후속 커밋이다.

## 완료한 검증

| 검증 | 결과 |
| --- | --- |
| `cargo test --test replay_contract --test audit_contract` | 통과, 7/7 |
| `cargo test --bin rhwp replay_engine_receives_the_hashed_input_snapshot` | 통과, 1/1 |
| 입력 receipt 변조 회귀 | exit 3, 재현 credit 0 |
| `git diff --check` | 통과 |
| 시각 검증 | 생략. 영수증·감사 실행과 계약 테스트만 변경 |

## 리스크와 권고

- PR #4392를 먼저 merge한 뒤 이 PR의 적층 범위를 다시 확인한다.
- 최신 보정 head의 full CI와 다중 러너 결과는 push 뒤 확인해야 한다.
- 후속 PR #4406에도 같은 스냅샷 결함이 있으므로 별도 head에 독립 보정한다.

**#4392 선행 정리와 최신 head full CI 통과 후 조건부 merge 권고. merge는 별도 승인 대상이다.**

