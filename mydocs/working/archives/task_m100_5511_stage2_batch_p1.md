# #5511 Stage 2 기능군 배치 P1 — agent protocol 경계

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 구현·최종 통합 기준: `upstream/devel` `b914bdf4bf1a8f922f03ea6b141f0d9c2b10a98f`
- 최종 코드 HEAD: `3f033f916d4fe050c8f5225f7c95df061e00cd04`
- 수행일: 2026-08-20
- 상태: 완료 — Wave P 종료, C0 진입 승인 대기

## 1. 결과

`src/main.rs`의 replay·audit·lineage·signing·anchor·gate·governance·bundle·disclosure·
settlement·harness·plan 구현을 `src/cli/protocol/` 아래로 물리 분리했다. 명령별 평면 구조가
아니라 capsule, trust, exchange, harness, plan의 보안 경계와 수명주기를 기준으로 소유권을
배치했다. root wrapper는 남기지 않았으며 최상위 dispatch가 새 소유 경로를 직접 호출한다.

| 책임 | 파일 수 | 최종 줄 수 범위 |
|---|---:|---:|
| capsule replay·signing·lineage·audit | 5 | 155~376 |
| trust anchor·gate·governance | 4 | 7~850 |
| exchange bundle·disclosure·settlement | 4 | 7~538 |
| harness | 1 | 542 |
| plan adapter·execution·condition | 3 | 110~671 |
| protocol root 조립 | 1 | 35 |

18개 protocol 파일은 모두 1,200줄 상한 이하다. `src/main.rs`는 P1 시작의 20,741줄에서
15,621줄로 5,120줄 줄었다. inventory에서 고정한 실제 protocol 구현 5,124줄은 전부 새 소유
경로로 이동했고, 모듈 선언과 dispatch 정리의 순증감을 포함한 root 순감소가 5,120줄이다.

## 2. 소유권과 보호 불변식

`sha256_hex_of`, `CasPathLock`, CAS 동시성 test hook, `check_expect_sha256`으로 구성된 145줄
범용 CAS seam은 root에 유지했다. 이 규약은 P1 plan 실행과 이후 C0 edit runtime이 함께
사용한다. P1 아래로 옮기면 C0가 protocol adapter를 역참조하고, 복제하면 동일한 사전 해시와
경로 잠금 불변식이 둘로 갈라진다. 최종 소유 위치는 C0 inventory에서 다시 판정한다.

공개 JSON·capsule·journal·bundle·disclosure·settlement 형식, hash·signature·Merkle·CAS
알고리즘, 오류 순서와 exit code는 바꾸지 않았다. sibling 공유 helper는 가장 가까운 parent
모듈에 두었고 root와 protocol 사이의 양방향 참조나 기능군 간 helper 복제를 만들지 않았다.

## 3. 복잡도 정정

이동 전 CC 25를 넘던 6개 함수는 기존 동작 순서를 유지하는 private helper로 분해했다.

| 함수 | 이동 전 CC | 분해 결과 |
|---|---:|---|
| `cmd_audit_report` | 31 | option 해석과 reproduction·attribution·anchoring 집계 분리 |
| `cmd_bundle_verify` | 26 | option 해석과 검증 실행 분리 |
| `cmd_gate` | 30 | option·policy signature·target·policy load 분리 |
| `cmd_harness_status` | 27 | option 해석과 상태 검증 분리 |
| `cmd_lineage` | 27 | option·keyring/anchor·trace·출력 분리 |
| `run_plan_engine` | 57 | condition·정적 검증·atomic 실행·journal 조립 분리 |

최종 `src/cli/protocol/` 경로에서 `clippy::cognitive_complexity` 25 초과 경고는 0건이다.
parser·serializer·암호·hash 규약에는 손대지 않았다.

## 4. 커밋 계보

| 커밋 | 역할 |
|---|---|
| `07d557092` | P1 범위·97개 보호 계약·CAS seam·복잡도 inventory |
| `3f033f916` | agent protocol adapter 18개 파일 이동과 6개 고복잡도 책임 분해 |

## 5. 최종 검증

| 검증 | 결과 |
|---|---|
| P1 직접 focused 계약 | 이동 전·최종 97/97 통과 |
| CAS debug-only 동시성 계약 | 정확히 한 실행만 commit, 1/1 통과 |
| 최종 release-test 전체 nextest | 8,005/8,005 통과, 3 slow, 38 skipped, 158.267초 |
| protocol 모듈 CC 25 상한 | 초과 경고 0건 |
| `cargo fmt --all -- --check`·`git diff --check` | 통과 |
| `cargo check --all-targets` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과 |
| `cargo test --doc` | 8/8 통과, 3 ignored |
| integration manifest 정책·현재 상태 | 18/18, 803 sources / 3,956 attrs / 41/48 targets 통과 |
| unit-tier 정책·현재 상태 | 12/12, 4,225 tests / 299 modules 통과 |
| CI impact Node·Python workflow 계약 | 62/62, 163/163 통과 |

전체 nextest의 release all-target 준비와 링크에는 27분 11초가 걸렸고, 실제 8,005개 실행은
158.267초였다. 앞선 focused nextest 시도는 filter와 무관하게 모든 target을 먼저 빌드하는
특성 때문에 준비 비용이 커져 중단했고, 생성된 release test binary로 직접 97개 계약을
검증한 뒤 최종 전체 nextest를 정상 완주했다. 이는 테스트 실패가 아니라 로컬 실행 경로의
비효율이며 최종 검증 모집단에는 누락이 없다.

`rust-test-suite-manifest --prepare`와 Cargo가 만든 검증 파생 변경은 추적 변경에 포함하지
않았다. 로컬 nextest 0.9.137이 저장소 권고 0.9.140보다 낮다는 경고가 있었지만 전체 모집단은
정상 실행되어 전건 통과했다. P1은 move-only CLI protocol 변경이므로 renderer·layout·WASM·
native-skia·시각 검증 발생 조건에 해당하지 않는다.

## 6. 최신 devel과 열린 PR

완료 시점 fetch에서 `origin/devel`과 `upstream/devel`은 모두 `b914bdf4b`이며 최종 코드 HEAD의
조상이다. 브랜치는 이 기준선보다 30개 커밋 앞서고 뒤처진 커밋은 없다. 열린 devel 대상 PR은
#5647, #5689, #5691, #5693, #5695, #5707, #5709, #5710, #5718이다. 각 최신 head의 변경
경로를 다시 조회했으며 P1의 `src/main.rs`, `src/cli/mod.rs`, `src/cli/protocol/`, inventory·
계획·완료 보고서 경로와 겹치지 않는다.

이 판정은 시점 증거다. 향후 통합·push 직전에 exact base SHA, 열린 PR head와 merge 가능성을
다시 확인한다. 이 보고서 작성 시점에는 remote push를 수행하지 않았다.

## 7. 다음 승인 단위

P1 완료로 agent protocol adapter 분리를 마쳤다. 다음 기능군은 C0 `run_edit`와 serialize·
verify·write 공통 seam이다. P1에 남긴 범용 CAS seam을 포함해 command module이 공유할 최소
runtime API와 `EditContext` 또는 동등한 명시적 의존 묶음의 필요성을 먼저 inventory한다.
service 계층 전환은 Stage 3 범위로 남기며, C0는 메인테이너의 P1 완료 승인과 별도 진입 승인
전 시작하지 않는다.
