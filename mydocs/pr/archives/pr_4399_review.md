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

후속 독립 검토에서는 세 가지 보안 차단점을 추가로 확인했다.

- 공용 입력 스냅샷은 전역 temp의 예측 가능한 이름과 Unix 기본 `0644` 가능성, 정상 경로
  수동 삭제에 의존해 민감 입력 노출·잔존 위험이 있었다.
- 캡슐은 parsed `plan`만 저장하지만 `receipt.planSha256`은 raw plan text의 해시였다.
  audit가 plan hash와 `receipt.steps`를 대조하지 않아 `output`처럼 산출에 중립적인 plan
  변조도 같은 output hash로 재현 credit을 받을 수 있었다.
- `read_dir(...).flatten()`은 항목별 열거 오류를 조용히 버렸고 비 UTF-8 접두사의 capsule
  이름은 필터 또는 실패 보고에서 누락될 수 있어 전수 감사가 fail-open이었다.

후속 메인터너 코드 커밋 `76d6580d405a9b998ac0e1e73a8716eb3e547829`은 `src/main.rs`와
`tests/audit_contract.rs`를 보정했다. 입력·산출은 Unix `0700` 전용 scratch 안에 두고 입력을
`0600`으로 만들며 RAII로 정리한다. 캡슐은 raw `planText`를 보존하고 audit는
`planText` 해시↔`receipt.planSha256`, parsed `plan`↔`planText`, receipt step 수↔plan step 수,
실제 실행 step 수를 모두 대조한다. 디렉터리 항목 오류는 runtime failure이고 capsule 이름은
lossy 변환으로 비 UTF-8 접두사도 빈 이름 없이 포함·보고한다.

## 완료한 검증

| 검증 | 결과 |
| --- | --- |
| `cargo test --test replay_contract` | 통과, 4/4 |
| `cargo test --test audit_contract` | 통과, 4/4 (Windows에서 Unix 전용 1건 제외) |
| `cargo test --bin rhwp replay_engine_receives_the_hashed_input_snapshot` | 통과, 1/1 |
| `cargo test --bin rhwp audit_directory_entry_errors_are_not_silently_dropped` | 통과, 1/1 |
| 입력 receipt 변조 회귀 | exit 3, 재현 credit 0 |
| output-neutral plan/planText/step 변조 회귀 | 3건 모두 exit 3, 재현 credit 0 |
| `git diff --check` | 통과 |
| 시각 검증 | 생략. 영수증·감사 실행과 계약 테스트만 변경 |

## 리스크와 권고

- PR #4392를 먼저 merge한 뒤 이 PR의 적층 범위를 다시 확인한다.
- Windows focused 실행에서 RAII 정리는 확인했다. Unix `0700`/`0600` 권한 단언과 비 UTF-8
  capsule 이름 회귀는 Unix hosted runner에서 확인해야 한다.
- 최신 보정 head의 full CI와 다중 러너 결과는 push 뒤 확인해야 한다.
- 후속 PR #4406에도 같은 스냅샷 결함이 있으므로 별도 head에 독립 보정한다.

**#4392 선행 정리와 최신 head full CI 통과 후 조건부 merge 권고. merge는 별도 승인 대상이다.**
