---
kind: working-note
status: completed
issue: 4963
stage: W5-reproduction-canary
last_verified: 2026-08-23
---

# Task M100 #4963 W5 — 공개 Hyper-V 재현 canary

- **이슈**: [#4963](https://github.com/edwardkim/rhwp/issues/4963)
- **계획**: [`task_m100_4963.md`](../../plans/archives/task_m100_4963.md)
- **재현 정본**:
  [`hyperv_reproduction_guide.md`](../../tech/investigations/issue-4963/hyperv_reproduction_guide.md)
- **단계 상태**: tracked host controller three-state 실행·복원·독립 비교 완료

## 1. 목적과 공개 경계

기존 Hyper-V attestation은 원 실행의 checkpoint 계보를 증명했지만 다른 개발자가 같은 환경과 상태
loop를 만드는 절차는 충분히 설명하지 못했다. 이번 canary는 공개 가이드와
`scripts/oracle_stage4_hyperv_canary.ps1`이 실제 disposable Windows guest에서 다음을 끝까지 수행하는지
검증했다.

1. raw VM·Standard checkpoint identity를 local-only 값으로 대사한다.
2. 각 상태 전에 baseline을 복원한다.
3. exact-only, subst-only, none-related managed set을 독립 구성한다.
4. interactive HWP 2020으로 같은 rank 8 fixture를 PDF로 출력한다.
5. 상태 증거를 회수하고 각 상태 뒤 baseline을 다시 복원한다.
6. 새 결과를 기존 acceptance ladder와 독립 비교한다.

font bytes, 자격 증명, VM/checkpoint 이름·GUID·절대 경로, private corpus와 문서 식별 정보는 추적하지
않는다. 공개 JSON에는 path-free environment identity와 결과 hash만 남긴다.

## 2. 기준선과 입력

| 항목 | SHA-256 또는 값 |
| --- | --- |
| baseline font manifest | `3bcd379d1f7fc217aad47a0b44b952d993c86ebbfabf46009386e4b3de768b40` |
| unrelated font projection | `437a36e513cce9d2909d904f3d07d2341051cc017e21be9ec6d35bbb9d87bc78` |
| rank 8 HWPX fixture | `f6edc8fc43dfd3256385e9752979c14a7041e50c06d36be47cef6e3486835084` |
| fixture manifest | `1e18915164b677ed3de23ee8991a6d3f593fa479e840a8a39461482d7c8796b1` |
| fixture semantic projection | `4a72d8cc641e88e9aa0e4cdc7f10eb192b2811759546efc5ac974730944ec4de` |
| HWP build | `11, 0, 0, 9136` |
| HWP executable | `7f00961398802c41620f5ef32fa2d2a26f7ff71f172723be36c660ea86a72bce` |
| culture / UI / system locale | `ko-KR` / `en-US` / `en-US` |

baseline manifest는 checkpoint restore 직후 두 번 실행해 byte-stable임을 확인했다. disconnected
interactive session에서는 `Win32_ComputerSystem.UserName`이 비어 있었지만 `explorer.exe` owner와 session
id가 정확히 하나 존재했다. controller는 이 실제 token을 기능 탐지해 일회성 Scheduled Task를 실행한다.

## 3. 실패 경로와 복구

성공 전에 두 번의 상태 변경 전 실패가 있었다.

| 시도 | 중단 원인 | font 상태 변경 | 복구 |
| --- | --- | --- | --- |
| 1 | WSL UNC source를 `Copy-Item -ToSession`에 직접 전달 | 없음 | `finally`에서 baseline 확인 |
| 2 | guest execution policy가 font-state helper 실행 차단 | 없음 | `finally`에서 baseline 확인 |

Windows local staging을 사용하고 helper에 process-scope execution policy를 적용한 뒤 controller를 다시
실행했다. 실패를 성공 증거와 합치지 않았고, 실패별 recovered manifest를 owner-only local evidence에
남겼다. 이 과정은 실행 실패보다 복구 실패를 우선하는 보호 불변식이 실제 오류 경로에서도 작동함을
확인했다.

## 4. three-state 결과

| 상태 | PDF font | page | visual line | glyph | typesetting projection |
| --- | --- | ---: | ---: | ---: | --- |
| exact-only | `KoPubWorldBatangLight` | 1 | 30 | 1,556 | `38f83a79…b4c7` |
| subst-only | `HCRBatang-Bold` | 1 | 30 | 1,556 | `59801255…27be` |
| none-related | `HCRBatang-Bold` | 1 | 30 | 1,556 | `59801255…27be` |

exact-only는 none-related와 달랐고, subst-only는 none-related와 같았다. 세 projection은 기존
[`oracle_stage5_rank8_acceptance_ladder.json`](../../tech/investigations/issue-4963/oracle_stage5_rank8_acceptance_ladder.json)의
동일 상태 projection과 모두 정확히 일치했다. raw PDF hash는 생성 metadata 때문에 달랐지만 font,
glyph, advance, position과 line을 정규화한 조판 결과는 재현됐다.

공개 결과
[`oracle_stage4_hyperv_reproduction_canary.json`](../../tech/investigations/issue-4963/oracle_stage4_hyperv_reproduction_canary.json)의
file SHA-256은 `f411d55f28a4d9f319b4b8676c216d8363facd2895a855d26d4c24d8c841b811`, 내부 canonical
SHA-256은 `b31d0e07437fceb80efb6e10fcda5a4834eb5b7cf7c9496ba3d95600a9466c17`이다.

## 5. 종료 상태

- 세 상태 모두 실행 전후 baseline과 unrelated projection을 복구했다.
- 최종 restore 뒤 managed font와 HWP process가 남지 않았다.
- 자동 checkpoint는 비활성화하고 Standard checkpoint 계약을 유지했다.
- DPAPI credential과 Windows staging·중복 output을 삭제했다.
- 원시 PDF·manifest·실패 증적은 owner-only local evidence에만 보관했다.
- 공개 파일에는 raw identity, credential, absolute path, font bytes, private corpus identity가 없다.

이 canary로 제3자 Hyper-V 재현 경로의 마지막 기술 게이트를 닫았다. 다음 절차 게이트는
[`task_m100_4963_report.md`](../../report/archives/task_m100_4963_report.md)에 대한 메인테이너 승인이다.

## 6. 로컬 검증

| 검증 | 결과 |
| --- | --- |
| Stage 2·3·4·4 profile·5 queue Python tests | 42/42 통과 |
| Oracle Node contract tests | 13/13 통과 |
| Node executable contract·Stage 4 contract check | 각각 통과 |
| Hyper-V PowerShell AST parse | 5/5 통과 |
| 변경 파일 Markdown 내부 상대 링크 | 9개, 이상 없음 |
| `cargo fmt --all`·`cargo fmt --all -- --check` | 통과 |
| `git diff --check` | 통과 |
