---
kind: pr_review
status: maintainer-review
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-10
---

# PR #4330 검토 — 스키마 버전 레지스트리와 정책 단일 출처

## 결론

**메인터너 보정 뒤 조건부 수용 권고.** contributor 변경은 네 스키마 축의 버전 선언을
`src/schema_registry.rs`로 모으고, capabilities·Node 타입·정책 문서·계약 테스트를 함께 정렬한다.
검토에서 신규 CLI 통합 테스트가 nextest archive의 런타임 실행 파일 재매핑을 무시하는 차단점과,
소스 스캐너가 `Map::insert` 형태의 봉투 버전 리터럴을 놓치는 단일 출처 사각지대를 확인했다.
원 contributor history를 유지한 채 두 차단점을 별도 보정 commit으로 해소했다.

로컬 candidate는 검증을 통과했지만 아직 원격에 push되지 않았다. code·test 보정이 포함됐으므로
review-only fast-pass 대상이 아니다. 최신 candidate의 GitHub Full CI, mergeability 재확인과
작업지시자의 push·review·merge 승인을 모두 충족해야 최종 수용할 수 있다.

## 메타데이터

| 항목 | 2026-08-10 검토 시점 참고값 |
| --- | --- |
| PR | [#4330](https://github.com/edwardkim/rhwp/pull/4330) |
| 관련 이슈 | [#4329](https://github.com/edwardkim/rhwp/issues/4329) |
| 작성자 | `kevin9327` |
| base / draft | `devel` / 아님 |
| contributor source head | `b54615026b97187050851c4b00c127e48911be64` |
| source 규모 | 18 files, +735 / -82, 2 commits |
| source merge 상태 | `MERGEABLE`, `CLEAN`; merge 전 최신 상태 재확인 필요 |
| source checks | source head 기준 required checks 성공; local correction에는 원격 CI 없음 |
| 가시성 branch | `review/kevin9327-20260810-pr4330` |
| local code candidate | `e95fa010688a346f9299b757083a37fd2f9e7294` |
| 기준 devel | `e48fe86947fbf9a44b1b98c7037150751af541ab` |

기준 devel은 contributor source head의 조상이고, 가시성 branch는 source head 뒤에 메인터너
commit만 선형으로 추가했다. contributor commit을 amend·rebase하지 않았다.

## contributor 변경 범위

- `src/schema_registry.rs`를 네 스키마 축의 버전 단일 출처로 추가하고 기존 schema·capabilities·
  provenance·MCP·agent 경로가 이를 참조하도록 정렬했다.
- Node envelope 타입과 `capabilities.schemaRegistry` 소비자 표면을 함께 갱신했다.
- 버전 정책 canonical, 지식 지도, 작업 기록과 증적 이미지를 추가했다.
- `tests/schema_registry_contract.rs`로 버전 리터럴 산개, 실행 봉투, 공개 schema 계약을 검증했다.

변경은 schema·CLI 계약과 문서에 한정된다. renderer, layout, paint, WASM 출력, sample·golden은
바뀌지 않으므로 시각 sweep 대상이 아니다.

## 발견한 차단점과 메인터너 보정

신규 `tests/schema_registry_contract.rs`가 `Command::new(env!("CARGO_BIN_EXE_rhwp"))`를 직접
사용했다. 이 값은 compile 시점 target의 절대 경로이므로 nextest archive가 실행 시점에 주입하는
`CARGO_BIN_EXE_rhwp` 재매핑을 우선하지 못한다. archive를 이동해 실행하면 존재하지 않는 원래
target 경로를 열 수 있어 저장소의 신규 CLI 테스트 계약을 위반한다.

메인터너 보정 `3f60b28f685c66366d855f849e8d67e689b06387`은 같은 테스트 파일에
`rhwp_bin()`을 추가했다. 런타임 환경변수를 먼저 읽고 없을 때만 compile 시점 값을 fallback으로
사용한다. PR이 새로 추가한 CLI 통합 테스트는 이 파일 하나이며, contributor 기능 범위는 바꾸지 않았다.
실행 단계와 rollback 경계는 [구현·검토 계획](pr_4330_review_impl.md)에 기록했다.

통합 후보 재검토에서는 `agent_manifest` 한 곳과 `rhwp-agent scan --jsonl` 두 곳이
`Map::insert("schemaVersion", json!("1.0"))` 형태로 레지스트리를 우회하지만, 기존 계약 테스트가
객체 리터럴 구문만 검사해 이를 통과시키는 점을 추가로 확인했다. 후속 보정
`e95fa010688a346f9299b757083a37fd2f9e7294`는 세 값을 `ENVELOPE_SCHEMA_VERSION`에서 파생하고,
공백·개행을 접은 statement 단위로 객체·insert·인덱스 대입 우회를 탐지하는 회귀와 두 실행 표면의
봉투 assertions를 추가했다. 런타임 생성부 전수 대사에서 남은 리터럴 우회는 없었다.

## 완료한 로컬 검증

| 명령 | 결과 |
| --- | --- |
| `CARGO_INCREMENTAL=0 cargo test --test schema_registry_contract` | 5 / 5 통과 |
| `cargo test --test agent_toolkit_contract scan_jsonl_streams_records_then_summary` | 1 / 1 통과 |
| `cargo test --test plan_schema_contract agent_manifest_carries_the_plan_schema_axis` | 1 / 1 통과 |
| 변경 Rust 파일 대상 `rustfmt --check` | 통과; checkout별 기존 newline style 명시 |
| `git diff --check origin/pr/4330..e95fa010` | 통과 |

첫 cold build는 실행 시간 상한에서 종료됐고, 같은 checkout의 dependency cache를 유지한 재실행이
7분 21초 build 뒤 당시 4개 테스트를 모두 통과했다. 후속 보정 뒤에는 통합 build cache를 재사용해
강화된 5개 registry 계약과 두 실행 회귀를 다시 통과했다. 전체 release-test·clippy는 focused 보정
범위를 검증하는 이번 단계에서 실행하지 않았다.

## 잔여 위험과 최종 조건

- local candidate와 이 review 기록은 원격 CI를 아직 받지 않았다. source·test가 바뀌었으므로 최신
  head의 Full CI와 CodeQL 등 변경 범위별 required check가 새로 성공해야 한다.
- schema registry는 여러 실행 표면을 함께 바꾸므로 focused 4건만으로 전체 CLI·MCP·Node 소비자
  회귀를 대체하지 않는다. 최신 GitHub CI가 이를 보완해야 한다.
- push 직전 원 PR source head와 local branch의 시작 SHA가 여전히 같은지 재확인해야 한다.
- GitHub push, review/comment, merge 권한은 부여되지 않았다.

위 조건과 작업지시자 승인을 충족하면 merge를 권고한다. 하나라도 충족하지 않으면 보류한다.
