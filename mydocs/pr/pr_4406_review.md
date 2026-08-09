---
kind: pr_review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-10
---

# PR #4406 검토 — 작업 캡슐 계보와 deep replay

## 라우팅

base route: `maintainer_general.md`

modifiers: `intake_and_review.md`, `local_validation.md`,
`multi_pr_update_branch.md`, `large_pr_rework.md`

## 메타데이터와 범위

| 항목 | 값 |
| --- | --- |
| 원 PR / 작성자 | [#4406](https://github.com/edwardkim/rhwp/pull/4406) / @kevin9327 |
| base | `devel` |
| 원 PR head | `d69abf374fffe40f5ac99c66171fc4876075b263` (2026-08-10 접수 시점) |
| 기준 devel | `e48fe86947fbf9a44b1b98c7037150751af541ab` |
| 가시성 브랜치 | `review/kevin9327-20260810-pr4406` |
| 원 변경 규모 | 14파일, `+1433/-3`, 커밋 7개 |
| 적층 관계 | PR #4392 replay와 #4399 capsule/audit 커밋을 포함 |

원 변경은 작업 캡슐에 부모 링크를 추가하고 계보 무결성과 deep replay 재현성을 검사한다.
renderer, fixture, baseline은 바꾸지 않아 visual sweep 대상은 아니다.

## 발견한 차단 결함과 메인터너 보정

계보 검사가 누락되거나 형식이 잘못된 receipt·parent 해시를 빈 값으로 받아 검증을 건너뛸 수
있었다. 자식 캡슐과 같은 폴더의 부모 경로를 원 문자열로 저장한 뒤 다시 자식 폴더에 결합해
상대 경로가 중복될 수 있었고, 적층된 replay·audit에는 해시한 입력과 엔진이 실제로 읽는 입력이
달라질 수 있는 TOCTOU 및 nextest binary 경로 문제가 남아 있었다.

메인터너 코드 커밋 `7584408f`는 다음을 보정했다.

- receipt 입력·출력 해시와 parent 해시를 64자리 SHA-256 형식으로 fail-closed 검증한다.
- 부모 캡슐 경로를 정규화하고 자식 캡슐 폴더 안의 부모는 그 폴더 기준 상대 경로로 저장한다.
- replay·audit·deep lineage가 한 번 읽어 해시한 입력 스냅샷만 엔진에 전달하고 실제 입력 해시도 대조한다.
- parent 해시 누락, 상대 부모 경로, audit 입력 receipt 변조, 해시 뒤 원본 교체 회귀를 추가한다.
- replay·audit·lineage CLI 테스트는 nextest 런타임 binary 경로를 우선한다.

contributor history는 rewrite하지 않았고 보정은 원 head의 single-parent 후속 커밋이다.

## 완료한 검증

| 검증 | 결과 |
| --- | --- |
| `cargo test --test replay_contract --test audit_contract --test lineage_contract` | 통과, 9/9 |
| `cargo test --bin rhwp replay_engine_receives_the_hashed_input_snapshot` | 통과, 1/1 |
| 누락 parent SHA 회귀 | exit 3, `valid: false` |
| 상대 부모 경로 회귀 | `a.capsule.json` 저장 및 정상 추적 |
| `git diff --check` | 통과 |
| 시각 검증 | 생략. 계보·영수증 실행과 계약 테스트만 변경 |

## 리스크와 권고

- PR #4392, #4399를 먼저 정리한 뒤 이 PR의 적층 범위를 다시 확인한다.
- 최신 보정 head의 full CI와 다중 러너 결과는 push 뒤 확인해야 한다.
- deep replay는 외부 엔진·파일 시스템 환경에 의존하므로 CI 외 실제 배포 환경에서도 후속 관찰한다.

**#4392와 #4399 선행 정리, 최신 head full CI 통과 후 조건부 merge 권고. merge는 별도 승인 대상이다.**
