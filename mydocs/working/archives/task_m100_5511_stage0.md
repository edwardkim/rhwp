# #5511 Stage 0 작업 보고 — CLI 계약과 의존 계보 동결

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 조사 기준선: `6eb4569a30eec88f742fe81084142c20cd48e31c`
- 통합 기준선: `e5ef2620bd469aa2d0118097c4d04f63cfdacdc3`
- 수행일: 2026-08-18
- 상태: 완료 — Stage 1 승인 대기

## 1. 완료한 조사

- `src/main.rs`의 2026-03-27~2026-08-18 성장 계보 측정
- 실제 최상위 dispatch, help, capabilities, MCP tool 수 교차 대조
- edit·inspect 하위 명령과 MCP annotation/배선 계약 확인
- CLI가 직접 참조하는 parser·renderer·serializer·model·WASM 경계 계측
- `src/service`와 기존 document-core CQRS 경계의 사용 상태 확인
- 기존 계약 테스트와 미보호 표면의 대응표 작성
- Stage 1의 최소 선행 변경 단위 선정

상세 근거는
[`task_m100_5511_cli_surface_inventory.md`](../../tech/investigations/issue-5511/task_m100_5511_cli_surface_inventory.md)에
기록했다.

## 2. 주요 판정

1. `main.rs` 비대화의 원인은 단일 회귀가 아니라 명령 기능·help·capabilities·MCP를 같은
   파일에 계속 추가한 누적 구조다.
2. service layer는 이미 존재하지만 CLI 이행이 0건이라, handler 파일 이동보다 의존 방향
   전환 계획이 먼저 필요하다.
3. 실제 dispatch 102개와 capabilities 98개 사이에 네 항목의 공백이 있다.
4. 기존 테스트 97개 집중 실행은 모두 통과했지만, dispatch↔catalog 전수 guard는 없다.
5. Stage 1의 첫 단위는 handler 이동이 아니라 명령 이름·가시성 catalog와 characterization
   guard여야 한다.

## 3. 검증 기록

| 검증 | 결과 |
|---|---|
| `cargo build --bin rhwp` | 통과, dev build 1분 17초 |
| `node scripts/rust-test-suite-manifest.mjs --prepare` | 714 sources, 32 suites + 9 exceptions 생성 |
| `node scripts/rust-test-suite-manifest.mjs --check` | 통과, 41/48 integration targets |
| `node scripts/rust-unit-test-tiers.mjs --check` | 통과, 4,225 tests / 298 modules |
| CLI·capabilities·MCP·exit focused nextest | 97/97 통과, 7,239 skipped |
| `cargo clippy --bin rhwp -- -W clippy::cognitive_complexity` | 측정 완료, CLI CC>25 19개·최대 68 |
| release-test 전체 nextest | 7,298/7,298 통과, 38 skipped, 187.283초 |
| `cargo fmt --all` 및 `cargo fmt --all -- --check` | 통과, Rust 변경 없음 |
| `git diff --check` | 통과 |
| `cargo clippy --all-targets --target-dir target/pr-review -- -D warnings` | 통과 |
| `cargo test --doc --target-dir target/pr-review` | 8/8 통과, 2 ignored |
| #5511 문서 상대 링크 대상 확인 | 통과, 내부 대상 5개 존재 |
| `check_markdown_links.py` | 기존 capability 등록부 무결성 오류 16건으로 실패 |
| 최신 base 재배치 후 manifest/tier 재검증 | 통과, 715 sources / 4,225 unit tests |

focused 명령은 다음과 같다.

```bash
cargo nextest run --cargo-profile release-test \
  -E 'test(cli_json_contract) | test(capabilities_schema_contract) | \
test(capabilities_subcommands_contract) | test(mcp_tool_annotations_contract) | \
test(mcp_server_contract) | test(cli_password_stdin_command_parity_contract) | \
binary(cli_exit_codes)'
```

저장소는 nextest `0.9.140`을 권장하지만 현재 호스트는 `0.9.137`이다. 실행은 성공했으며
버전 차이는 경고로만 보고됐다.

Markdown 검사 실패는 #5511 문서의 링크 단절이 아니다. 변경하지 않은
`mydocs/manual/agent_capability_registry.md`의 중복 ID·runtime 진입점·링크 형식 오류
16건이 전역 검사에서 검출됐다. 이 파일은 최신 `upstream/devel`과도 동일하다. #5511이
추가한 상대 링크의 대상은 별도로 모두 존재함을 확인했으며, 범위 밖 등록부를 이 단계에서
고치지 않는다.

## 4. 기준선에서 발견한 절차 보정

- 생성 integration harness가 없는 일반 checkout에서 `--check`만 실행하면 32개 drift로
  실패한다. review 절차대로 `--prepare` 후 `--check`해야 한다.
- `release-test`는 nextest profile이 아니라 Cargo profile이다. 올바른 옵션은
  `--cargo-profile release-test`다. 수행계획서의 표기를 함께 수정했다.
- `tests/generated/*.rs`는 파생·ignore 대상이며 stage하거나 PR에 포함하지 않는다.
- 검증 중 `upstream/devel`이 `6eb4569a3`에서 `e5ef2620b`로 6커밋 이동했다. 유입 변경에는
  `src/main.rs`, `src/service/`, #5511 문서 경로가 없고, 열린 PR의 변경 파일에도
  `src/main.rs` 또는 `src/service/` 중복이 없었다. Stage 0 커밋을 만든 뒤 최신 base로
  충돌 없이 재배치했다.

## 5. 다음 단계 제안

Stage 1의 첫 변경 단위를 다음으로 제한한다.

- 102개 실제 dispatch 이름의 inventory를 코드에서 검증
- help/capabilities/MCP 참여 여부를 명시하는 visibility 정책
- dispatch-only 네 명령의 현재 상태를 승인 전까지 그대로 보존
- handler 이동, service 이행, 외부 출력 변경은 제외

전체 검증과 문서 자체 검토를 완료했다. 이 보고서를 커밋한 뒤 메인테이너에게 Stage 1
진입 승인을 요청한다.
