# Task M100 #5511 최종 보고서 — `src/main.rs` CQRS·SOLID 경계 복원

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 작업 브랜치: `task_m100_5511`
- 최초 기준선: `upstream/devel` `6eb4569a30eec88f742fe81084142c20cd48e31c`
- 최종 통합 기준선: `upstream/devel` `c1acbf97c803537322966cb3bb8c41f2db76fc40`
- 작업 head: `011f19436527f6b1342d2de0a66fcd52788d1f14`
- 작업 merge commit: `0a674165499bdb2e3150c3b9a3e6abb3c62b8046`
- 최신 원격 결합 merge commit: `6a80dd8bb7b13998216e03789e3ce681c1df6d64`
- 완료일: 2026-08-20
- 상태: 완료 — 메인테이너가 Stage 0~2 결과를 현 기준선으로 수용

## 1. 완료 판정

#5511은 `src/main.rs`에 함께 있던 command, query, output, metadata, agent protocol 책임을
계약 보존형 수직 절편과 기능군 배치로 분리했다. 최종적으로 계획한 Stage 2 배치
Q1~Q7·M1·P1·C0~C6를 모두 완료했고, `main.rs`의 편집 handler는 전수 책임 모듈로 이동했다.

메인테이너는 추가 분해 비용과 향후 기능 PR에서의 재증가 가능성을 함께 평가해 2,075줄의
현 기준선을 수용했다. 계획서의 Stage 3 service layer·DIP 전환과 Stage 4의 1,200줄·직접 의존
0 목표는 완료로 간주하지 않고 미착수 상태로 종료한다. 재증가 방지는 후속 이슈
[#5767](https://github.com/edwardkim/rhwp/issues/5767)이 담당한다.

## 2. 결과 지표

| 항목 | 최초 기준선 | 최종 | 변화 |
|---|---:|---:|---:|
| `src/main.rs` 줄 수 | 42,370 | 2,075 | -40,295 (-95.1%) |
| 최상위 함수 | 351 | 45 | -306 |
| `main.rs`의 편집 handler | 92 | 0 | -92 |
| `rhwp::wasm_api::HwpDocument` 직접 참조 | 42 | 9 | -33 |
| `rhwp::model` 직접 참조 | 71 | 1 | -70 |
| `rhwp::parser` 직접 참조 | 95 | 10 | -85 |
| `rhwp::renderer` 직접 참조 | 28 | 0 | -28 |
| `rhwp::serializer` 직접 참조 | 16 | 0 | -16 |

남은 45개 최상위 함수와 `HwpDocument`·parser 직접 참조는 미완료 Stage 3·4 입력에 해당한다.
따라서 이 보고서는 service layer 또는 DIP 전환의 완료를 주장하지 않는다.

## 3. 완료 계보

### Stage 0~1

- CLI 명령, dispatch, help, capabilities, MCP, JSON·exit-code 계약과 의존 방향을 조사 문서로
  동결했다.
- 명령 catalog의 단일 원천과 surface 참여 검사를 추가하고, capabilities metadata를 catalog에서
  파생하도록 했다.

### Stage 2 초기 수직 절편

- document·structured-object query와 note·endnote·extent·raw-record·search·field·data·digest·
  explain·explore·hidden-text·unicode·watermark·injection·threat·armor 경계를 순차 분리했다.
- 24개 절편 뒤 검증 반복 비용을 계측해 기능군 배치로 재기준화했으며, 보호 불변식과 중단
  조건은 유지했다.

### Stage 2 기능군 배치

- Q1~Q7: preview, render output, data exchange, scan·batch, diagnostics, conversion, internal
  verification을 query·output·command 책임으로 분리했다.
- M1: MCP definitions, capabilities, help projection을 catalog 정본에서 파생하는 metadata
  모듈로 분리했다.
- P1: replay, audit, lineage, anchor, gate, bundle 등 agent protocol을 capsule, trust,
  exchange, harness, plan 책임으로 나눴다.
- C0~C6: edit runtime과 field·text·privacy·object·media·table·equation·document structure·
  formatting·header/footer·note story command를 책임별 모듈로 이동했다.
- 이동 전에 column definition, Q5 사람용 출력, Q7 diagnostics, cell style, header/footer picture
  등 미보호 계약을 characterization test로 고정했다.

세부 단위와 각 배치의 focused 결과는
[`task_m100_5511_stage2_batch_plan.md`](../plans/archives/task_m100_5511_stage2_batch_plan.md)와
[`issue-5511` 조사 정본](../tech/investigations/issue-5511/README.md),
`mydocs/working/task_m100_5511_*` 보고서에 남겼다.

## 4. 보호한 외부 계약

- CLI 명령·옵션·help 문구와 순서, 기본값, exit code, stdout/stderr 분리
- JSON·NDJSON 필드와 순서, MCP tool 이름·schema·annotation
- HWP/HWPX parser·serializer·renderer 결과와 round-trip 의미
- query와 상태 변경 command의 CQRS 경계, 파일을 쓰는 output adapter의 부작용 경계
- catalog와 help·capabilities·MCP surface의 참여 동형성

이 작업은 parser·serializer·renderer 알고리즘, WASM API, 시각 출력 또는 신규 배포 채널을
변경하지 않았다. 따라서 시각·WASM 검증은 적용하지 않았다.

## 5. 최종 통합 HEAD 검증

작업 branch를 `0a674165499bdb2e3150c3b9a3e6abb3c62b8046`으로 정상 merge한 뒤 push 직전
원격이 `c1acbf97c803537322966cb3bb8c41f2db76fc40`로 전진한 것을 발견했다. #5768의 renderer·
Studio 통합과 소스 경로 중첩은 없었고 `mydocs/orders/20260820.md`의 독립된 두 기록을 모두
보존했다. 최신 원격을 다시 정상 merge한 `6a80dd8bb7b13998216e03789e3ce681c1df6d64`에서
다음 관문을 전부 재검증했다.

| 관문 | 결과 |
|---|---|
| suite manifest prepare/check | 817 sources, 3,995 attributes, 32 suites + 9 exceptions |
| suite manifest policy test | 18/18 passed |
| unit-tier check | 4,225 tests, 299 modules, ready 0 |
| unit-tier policy test | 12/12 passed |
| `cargo fmt --all -- --check` | passed |
| `git diff --check upstream/devel..HEAD` | passed |
| `cargo check --locked --all-targets` | passed |
| `cargo clippy --locked --all-targets --target-dir target/pr-review -- -D warnings` | passed |
| `cargo test --locked --doc --target-dir target/pr-review` | 8 passed, 3 ignored |
| 전체 release-test nextest | 8,227/8,227 passed, 3 slow, 39 skipped |

최종 nextest 실행 시간은 202.063초였다. 로컬 nextest 0.9.137은 저장소 권장 0.9.140보다 낮다는
경고를 냈지만 실행과 결과 판정은 성공했다.

## 6. 범위 조정과 후속 책임

- Stage 3의 application/service layer, typed error, 전역 상태 제거와 DIP 전환은 미착수다.
- Stage 4의 `main.rs` 1,200줄 이하, 직접 `HwpDocument` 참조 0, CLI 평가 단위의 추가 복잡도
  수렴도 미완료다.
- 위 미완료 항목을 완료한 것처럼 닫지 않는다. 이번 종료는 Stage 0~2가 만든 2,075줄 기준선을
  메인테이너가 수용한 범위 조정이다.
- 기여자 코드 복잡도 가이드라인과 base/head 증분 CI는 #5767에서 구현한다.
- 이후 service/DIP 전환이 필요하면 현재 수치와 남은 직접 의존을 새 계획의 기준선으로 삼는다.

## 7. 통합 방식

장기 작업 중 원격 `devel` 변경을 각 배치 경계에서 정상 merge하고 결합 HEAD를 다시 검증했다.
최종 이행은 메인테이너가 승인한 예외 절차에 따라 PR을 만들지 않고, 최신 원격과 동기화한 로컬
`devel`에 작업 브랜치를 `--no-ff` 정상 merge한 뒤 보호 브랜치에 직접 push한다. 원격 반영과
GitHub Actions 성공을 확인한 뒤 #5511에 이 merge SHA와 검증 결과를 남기고 이슈를 닫는다.

## 8. 원격 Lint 후속 보정

첫 원격 이행 head `5f5ce5e65cc133bee5dd89909fb9da7ec228c85e`의
[CI run 32362191294](https://github.com/edwardkim/rhwp/actions/runs/32362191294)에서
`Validate LLM verifier tool contracts`가 실패했다. `shadow_agree`의 실존 명령·출력 필드 검사가
모든 dispatch와 envelope producer가 `src/main.rs` 한 파일에 있다고 가정해, Q7에서
`ir-diff`의 `identical` 생산자를 `src/cli/queries/ir_comparison.rs`로 옮긴 뒤의 구조를 따라가지
못한 것이 원인이었다.

`tools/llm_verifier/shadow_agree/tests/test_checks.py`가 production Rust source 전체에서 top-level·
중첩 dispatch와 출력 필드를 확인하도록 보정했다. CLI 명령, envelope, 생성 코퍼스는 바꾸지
않았다. 실패한 shadow suite 31/31과 CI의 `Validate LLM verifier tool contracts` 7개 명령을
그대로 재실행해 모두 통과했다. 보정 head의 새 Actions 성공을 확인한 뒤 이슈를 닫는다.
