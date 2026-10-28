# #5511 Stage 2 기능군 배치 P1 inventory — agent protocol 경계

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 조사 HEAD: `1d7bef90b59b532cdaec83a06f51f9f9959473f3`
- 통합 기준선: `upstream/devel` `b914bdf4bf1a8f922f03ea6b141f0d9c2b10a98f`
- 작성일: 2026-08-20
- 상태: P1 진입 승인 — 이동 전 계약·소유권 고정

## 1. 범위 판정

P1의 실제 agent protocol 구현은 `src/main.rs`의 `replay_sha256_hex`부터
`evaluate_step_condition`까지 5,124줄이다. 함수 선언은 cfg별 CAS test hook을 포함해 41개이고,
`ReplayScratchDir`의 `Drop` 구현이 하나 더 있다. 계획서의 5,686줄·54함수는 이전 재계측 SHA의
좌표였으며, Q·M 배치에서 공유 helper와 인접 metadata가 먼저 이동하면서 현재 경계가 줄었다.

바로 앞의 `agent_manifest_value`와 `cmd_export_agent_manifest`는 M1의 metadata projection과
protocol 실행 경계 사이에 있는 별도 export adapter이므로 P1에 넣지 않는다. 바로 뒤의
`edit_serialize_snapshot`부터는 C0 command runtime 경계이므로 역시 제외한다.

`sha256_hex_of`, `CasPathLock`, CAS 동시성 test hook, `check_expect_sha256` 145줄은 P1과 이후 C0
edit runtime이 함께 사용하는 범용 무결성 seam이다. 이를 protocol 아래로 옮기면 C0가 P1을
역참조하고, 복제하면 동일한 CAS 검증 규약이 둘로 갈라진다. 따라서 P1에서는 root 공유 seam으로
유지하며 protocol 모듈은 이를 하위 의존으로 사용한다. C0에서 command runtime을 설계할 때 최종
소유 위치를 다시 판정한다.

## 2. 보호 계약 기준선

이동 전 아래 15개 직접 계약 모듈을 release profile로 실행해 97/97 통과했다.

| 계약 축 | 모듈 | 건수 |
|---|---|---:|
| capsule replay·plan | `replay_contract`, `run_plan_contract`, `plan_schema_contract` | 38 |
| CAS·dry-run·journal | `run_plan_cas_contract`, `run_plan_dry_run_contract`, `run_plan_journal_hash_chain_contract` | 25 |
| audit·lineage | `audit_contract`, `audit_standard_contract`, `lineage_contract` | 14 |
| signing·anchor·gate | `signing_contract`, `anchor_contract`, `gate_contract` | 12 |
| exchange·harness | `bundle_contract`, `disclose_contract`, `settle_contract`, `harness_contract` | 8 |

이 계약들은 plan 결정성, 사용자 입력 비변경, CAS와 journal hash chain, 조건·action 실행,
signature, lineage, anchor checkpoint·Merkle, gate deny-default, bundle 공격 입력, disclosure와
settlement round-trip·방어, harness 수명주기·tamper를 보호한다. P1은 공개 형식이나 알고리즘을
바꾸지 않는 물리 이동이므로 신규 characterization을 선행하지 않는다. 구현 절편마다 해당 focused
집합을 재실행하고 최종 HEAD에서 97개와 전체 회귀를 다시 실행한다.

## 3. 복잡도 중단 조건과 처리

이동 전 all-target clippy의 cognitive-complexity 계측에서 다음 6개 함수가 상한 25를 넘었다.

| 함수 | CC | 분해 기준 |
|---|---:|---|
| `cmd_audit_report` | 31 | 입력 수집, 축별 계산, report 출력 |
| `cmd_bundle_verify` | 26 | bundle 검증, material 복원·대조 |
| `cmd_gate` | 30 | 입력 해석, policy 평가, 결과 출력 |
| `cmd_harness_status` | 27 | 상태 수집, capsule 재검증, 출력 |
| `cmd_lineage` | 27 | chain 수집·검증, envelope 출력 |
| `run_plan_engine` | 57 | plan 검증, step 실행, journal·summary 조립 |

계획서의 중단 조건에 따라 이 함수들을 그대로 다른 파일에 숨기지 않는다. 관찰 가능한 출력과
오류 순서를 보존하는 private helper로 분해하고, 새 protocol 경로의 CC>25 경고가 0건인지
확인한다. parser·serializer·암호·hash 알고리즘 변경이 필요해지면 즉시 P1을 중단한다.

## 4. 목표 소유권

명령 이름별 평면 디렉터리가 아니라 보안 경계와 수명주기를 기준으로 다음 책임 트리를 사용한다.

```text
src/cli/protocol/
├── mod.rs
├── capsule/
│   ├── mod.rs          # capsule 공통 replay·snapshot seam
│   ├── replay.rs       # replay adapter
│   ├── lineage.rs      # parent chain 검증·출력
│   ├── audit.rs        # capsule 집합 감사
│   └── signing.rs      # key 생성·서명 검증
├── trust/
│   ├── mod.rs
│   ├── anchor.rs       # append/checkpoint/verify
│   ├── gate.rs         # policy gate
│   └── governance.rs   # Y10 report·recall·conformance
├── exchange/
│   ├── mod.rs
│   ├── bundle.rs       # transport bundle
│   ├── disclosure.rs   # selective disclosure
│   └── settlement.rs   # proposal·verification·record
├── harness.rs          # harness init·wrap·status
└── plan/
    ├── mod.rs          # CLI adapter·plan projection
    ├── execution.rs    # step 실행·journal·summary
    └── condition.rs    # step 조건 평가
```

각 파일은 1,200줄 이하를 유지한다. sibling 공유는 가장 가까운 parent의 `pub(super)` API로만
노출하고 root wrapper나 기능군 간 helper 복제를 만들지 않는다. `src/main.rs`는 최상위 dispatch가
새 소유 경로를 직접 호출하고, 앞서 판정한 범용 CAS seam만 제공한다.

## 5. 구현·커밋 순서

1. 이 inventory를 독립 커밋으로 고정한다.
2. capsule 공통부·replay·signing·lineage·audit를 이동하고 lineage 고복잡도를 분해한다.
3. anchor·gate·governance를 이동하고 gate·audit-report 고복잡도를 분해한다.
4. bundle·disclosure·settlement를 이동하고 bundle verify 고복잡도를 분해한다.
5. harness를 이동하고 status 고복잡도를 분해한다.
6. plan adapter·실행·조건을 이동하고 `run_plan_engine`을 책임 분해한다.
7. P1 전체 검증, 지표, 최신 devel·열린 PR 위험을 완료 보고서에 기록하고 커밋한다.

각 구현 커밋은 이동과 동작 변경을 섞지 않으며, `cargo fmt --all -- --check`, 해당 focused 계약,
`git diff --check`를 통과해야 한다. 최종 배치에는 release-test 전체, all-target check·clippy,
doc-test, integration manifest, unit-tier, CI 정책 계약을 적용한다.

## 6. 원격 위험과 중단 기준

조사 시점 `origin/devel`과 `upstream/devel`은 모두 `b914bdf4b`이고 현재 HEAD의 조상이다. 열린
devel 대상 PR #5647, #5689, #5691, #5693, #5695, #5707, #5709, #5710의 최신 변경 경로에는
`src/main.rs`, 계획한 `src/cli/protocol/`, `src/cli/mod.rs`, P1 직접 계약, #5511 문서가 없다.
이 판정은 시점 증거이므로 P1 완료와 push 전에 다시 조회한다.

다음 경우에는 같은 승인 배치 안에서도 이동을 멈추고 메인테이너에게 보고한다.

- 97개 기준 계약의 출력·exit·부작용 또는 보안 방어가 달라지는 경우
- 범용 CAS seam과 protocol 사이에 양방향 의존이 생기는 경우
- 새 파일 1,200줄 또는 CC 25 상한을 지킬 수 없는 경우
- 최신 devel·열린 PR이 같은 함수, 테스트, 모듈 경계를 변경한 경우
- move-only 범위를 넘어 공개 schema·암호·hash·저장 형식 변경이 필요한 경우
