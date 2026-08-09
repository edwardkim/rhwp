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
달라질 수 있는 TOCTOU 및 nextest binary 경로 문제가 남아 있었다. 독립 재검토에서는 bare
capsule 경로의 빈 parent 디렉터리, `parent` 필드 누락의 합법 root 오인, plan·step 영수증을
대조하지 않는 audit/deep lineage, 전역 임시 폴더의 민감 입력 노출 위험도 확인했다.

메인터너 코드 커밋 `7584408f`는 다음을 보정했다.

- receipt 입력·출력 해시와 parent 해시를 64자리 SHA-256 형식으로 fail-closed 검증한다.
- 부모 캡슐 경로를 정규화하고 자식 캡슐 폴더 안의 부모는 그 폴더 기준 상대 경로로 저장한다.
- replay·audit·deep lineage가 한 번 읽어 해시한 입력 스냅샷만 엔진에 전달하고 실제 입력 해시도 대조한다.
- parent 해시 누락, 상대 부모 경로, audit 입력 receipt 변조, 해시 뒤 원본 교체 회귀를 추가한다.
- replay·audit·lineage CLI 테스트는 nextest 런타임 binary 경로를 우선한다.

후속 코드 커밋 `37d866ed`는 다음을 추가 보정했다.

- bare capsule 파일명은 현재 디렉터리를 기준으로 부모 경로를 정규화하고 `parent` 필드 누락은 실패한다.
- 원 plan text를 캡슐에 보존해 `planSha256`, parsed plan, 실제 step 수를 audit와 lineage가 대조한다.
- replay 입력·산출은 Unix mode 0700 전용 scratch 폴더와 0600 입력 파일에 두고 RAII로 정리한다.
- 누락 plan/parent, plan·step 변조, bare 경로, scratch 정리 회귀를 추가한다.

두 번째 후속 코드 커밋 `2d593563`은 shallow lineage도 plan step 수와 receipt를 즉시
대조하고, audit 폴더 항목 열거 오류를 조용히 제외하지 않고 fail-closed 처리한다. 비 UTF-8
prefix의 capsule 이름도 lossy 표시 이름으로 감사 분모와 실패 보고에서 보존한다.

최종 독립 검토에서는 lineage가 capsule bytes를 `String::from_utf8_lossy`로 치환한 뒤 JSON을
파싱해, 원본이 유효한 UTF-8이 아니어도 replacement character가 JSON 문자열 안에서 허용되면
검증을 계속할 수 있음을 확인했다. 또한 replay에서 `--capsule`과 `--parent`가 같은 기존 파일
또는 그 symlink alias를 가리키면 부모를 해시한 뒤 capsule 쓰기가 원 부모를 덮어쓸 수 있었다.

세 번째 후속 코드 커밋 `5012f519e7367aa276463a6d30a216950efb159d`은 `src/main.rs`와
`tests/lineage_contract.rs`를 보정했다. lineage는 raw bytes를 `serde_json::from_slice`로 strict
파싱한다. replay는 parent를 읽거나 capsule을 쓰기 전에 기존 실파일 identity/canonical path를
대조해 동일 대상이면 usage error로 거절한다. invalid UTF-8이 lossy 변환 후에는 유효한 JSON이
되는 음성 회귀, 동일 경로·Unix symlink alias에서 원 부모 바이트가 불변인 회귀를 추가했다.

contributor history는 rewrite하지 않았고 보정은 원 head의 single-parent 후속 커밋이다.

## 완료한 검증

| 검증 | 결과 |
| --- | --- |
| `cargo test --test replay_contract --test audit_contract --test lineage_contract` | 통과, 13/13 (Windows에서 Unix symlink 1건 제외) |
| `cargo test --bin rhwp replay_engine_receives_the_hashed_input_snapshot` | 통과, 1/1 |
| 누락 parent·plan SHA 회귀 | exit 3, `valid: false` |
| bare capsule 상대 부모 경로 회귀 | `a.capsule.json` 저장 및 정상 추적 |
| plan·step 영수증 변조 회귀 | audit exit 3, 재현 credit 0 |
| replay scratch 수명 회귀 | 실행 후 전용 폴더 제거 확인 |
| audit 항목 열거 | per-entry 오류를 무시하지 않는 명시적 실패 경로 확인 |
| invalid UTF-8 capsule 회귀 | strict JSON 파싱 실패, lineage exit 3 |
| capsule=parent 회귀 | usage exit 2, stdout 0바이트, 원 부모 바이트 불변 |
| `git diff --check` | 통과 |
| 시각 검증 | 생략. 계보·영수증 실행과 계약 테스트만 변경 |

## 리스크와 권고

- PR #4392, #4399를 먼저 정리한 뒤 이 PR의 적층 범위를 다시 확인한다.
- 동일 경로와 canonical symlink alias는 차단하고 Unix에서는 device/inode도 대조한다. Windows
  hardlink alias는 안정적인 cross-platform file-id 대조가 없어 여전히 같은 파일로 식별하지
  못할 수 있다.
- Unix symlink alias 원본 불변 회귀는 Windows 로컬 환경에서 제외됐으므로 hosted Unix CI에서
  실행 결과를 확인해야 한다.
- 최신 보정 head의 full CI와 다중 러너 결과는 push 뒤 확인해야 한다.
- deep replay는 외부 엔진·파일 시스템 환경에 의존하므로 CI 외 실제 배포 환경에서도 후속 관찰한다.

**#4392와 #4399 선행 정리, 최신 head full CI 통과 후 조건부 merge 권고. merge는 별도 승인 대상이다.**
