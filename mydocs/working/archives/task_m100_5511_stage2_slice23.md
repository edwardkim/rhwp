# #5511 Stage 2 스물세 번째 수직 절편 — armor CLI 이동 전 계약 보강

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 통합 기준선: `1a6ce79fd56e3cdf5813c7938338fcb5b7d0a859`
- 계약 커밋: `f748bb40f`
- 수행일: 2026-08-19
- 상태: 완료 — 다음 Stage 2 절편 승인 대기

## 1. 절편 범위와 중단 판단

스물두 번째 절편에서 다음 후보로 지정한 root의 `armor_command`를 바로 이동하지 않고,
handler의 관찰 가능 계약과 공용 seam을 먼저 조사했다. 기존 `tests/armor_contract.rs` 8건은
HWP3 기반 합성 공격에 대해 다음 핵심을 강하게 보호한다.

- 입력 문서 무변경
- 호출별 128비트 nonce와 격벽 위조 방지
- 격벽 안 본문 보존과 주입 신호 탐지
- 정상 문서의 clean 봉투
- JSON provenance와 untrusted field 표지
- 실패 시 stdout 0바이트와 help·capabilities·MCP 표면 배선

그러나 실제 HWP5·HWPX, 세 암호 형식의 성공 경로, 사람용 격벽의 제어문자 안전 출력은
직접 보호하지 않았다. 따라서 이번 절편은 제품 코드를 이동하지 않고 characterization
contract만 추가하는 지점에서 멈췄다.

## 2. 추가한 보호 계약

`tests/cases/armor_cli_contract.rs`에 네 개의 black-box CLI 계약을 추가했다.

| 계약 | 보호하는 동작 |
|---|---|
| 실제 HWP5·HWPX | 완전한 격벽·scope·provenance 봉투와 입력 바이트 불변성 |
| 암호 HWP3·HWP5·HWPX | `--password-stdin` 성공과 각각 24·64·23쪽 실측 |
| 암호 누락·오입력 | 누락 2, 오입력 1, stdout 0바이트와 오입력 비밀번호 비노출 |
| 사람용 격벽 | 주입 경고·격벽·읽기 전용 고지와 문서 파생 탭의 `⇥` 표시, 원시 탭 금지 |

합성 공격 HWP는 테스트별 임시 디렉터리에서 만들고 `Drop`에서 파일과 디렉터리를 제거한다.
저장소 sample이나 사설 코퍼스를 만들거나 변경하지 않는다.

기존 8건과 신규 4건을 함께 실행해 12/12가 통과했다. 이동 전후 비교에 필요한 실제 형식,
암호 상태와 사람용 출력 경계가 확보됐으므로 다음 절편은 handler 본문의 물리 이동만으로
제한할 수 있다.

## 3. 조사 중 확인한 기존 표면 불일치

CLI `armor`는 `load_document`를 사용하므로 전역 `--password`와 `--password-stdin`으로 암호
HWP3·HWP5·HWPX를 모두 연다. 반면 실제 `capabilities --mcp` 출력의 `hwp_armor`는 다음과
같다.

- `inputSchema.properties`는 `path`만 포함한다.
- `cli.passwordStdin`은 `null`이다.
- `supports_password_stdin` allowlist에도 `hwp_armor`가 없다.

따라서 CLI에는 있는 암호 기능을 MCP 호출자는 사용할 수 없다. 이는 이번 이동으로 생긴 문제가
아니며, move-only 커밋에 기능 변경을 섞지 않기 위해 고치거나 실패 계약으로 고정하지 않았다.
후속 이슈에서 `hwp_armor`의 password schema·writeOnly·stdin 전달과 MCP 실제 호출을 함께
복구해야 한다.

실제 HWP5/HWPX armor 성공 실행 중 layout 계층의 `LAYOUT_OVERFLOW` 진단이 stderr에
출력되는 기존 현상도 관찰했다. JSON stdout은 한 줄 순수 봉투이고 종료 코드는 0이지만 성공
경로 stderr가 조용하지 않다. 이 진단 소유권도 handler 이동과 무관하므로 이번 계약은 불필요한
stderr를 정상 규약으로 고정하지 않았고 별도 후속 조사 대상으로 남긴다.

## 4. 구조와 지표 영향

이번 절편의 추적 변경은 integration test source 한 파일뿐이다. `src/main.rs`, security
inspection 모듈, armor 알고리즘, nonce 생성과 CLI·MCP schema는 바꾸지 않았다.

| 항목 | Stage 2 절편 22 | Stage 2 절편 23 | 변화 |
|---|---:|---:|---:|
| `src/main.rs` 줄 수 | 38,730 | 38,730 | 변화 없음 |
| `src/cli/queries/security_inspection.rs` | 893 | 893 | 변화 없음 |
| Rust test source | 753 | 754 | +1 |
| static test attribute | 3,717 | 3,721 | +4 |
| armor CLI 계약 | 8 | 12 | +4 |

제품 함수가 바뀌지 않았으므로 root와 security inspection 모듈의 복잡도, 계층 참조와 공개
CLI surface도 동일하다.

## 5. 검증 기록

| 검증 | 결과 |
|---|---|
| 기존+신규 armor focused nextest | 12/12 통과 |
| release-test 전체 nextest | 7,770/7,770 통과, 3 slow, 38 skipped, 171.549초 |
| `cargo check --all-targets` | 통과 |
| `cargo fmt --all -- --check` | 통과 |
| `cargo clippy --all-targets --target-dir target/pr-review -- -D warnings` | 통과 |
| `cargo test --doc --target-dir target/pr-review` | 8/8 통과, 3 ignored |
| integration manifest 정책 자체 계약 | 16/16 통과 |
| `rust-test-suite-manifest --check --base-ref upstream/devel` | 통과, 754 sources / 3,721 static test attrs / 43 integration targets |
| unit-tier 정책 자체 계약 | 12/12 통과 |
| `rust-unit-test-tiers --check --base-ref upstream/devel` | 통과, 4,225 tests / 298 modules |
| CI impact Node 계약 | classifier+policy 62/62 통과 |
| `scripts/tests/test_ci_impact_workflow.py` | 30/30 통과 |
| `git diff --check` | 통과 |
| `check_markdown_links.py --changed-from upstream/devel` | 기존 capability 등록부 무결성 오류 16건, 절편 23 신규 오류 없음 |

호스트 nextest는 `0.9.137`이고 저장소 권장은 `0.9.140`이라 경고가 있었지만 실행 결과에는
영향이 없었다. 준비 스크립트가 로컬 검증용 Cargo test target 두 개를 생성하고 Cargo가
인접 verifier package의 lockfile 순서를 바꿨으나, 둘 다 추적 변경에서 복원했다.

renderer·serializer·WASM 경계를 건드리지 않은 테스트 전용 변경이므로 시각 검증과 WASM
빌드는 추가하지 않았다. Markdown 검사는 기준선에도 존재하는
`agent_capability_registry.md`의 중복 ID·진입점 링크 무결성 오류 16건만 보고했다.

## 6. 원격 병합 위험 재검증

절편 시작과 계약 검증 종료 시 `origin/devel`과 `upstream/devel`은 모두 `1a6ce79fd`로
동일했다. 계약 커밋 기준 작업 브랜치는 53커밋 앞서고 0커밋 뒤이며, 최신
`upstream/devel`과의 merge-tree는 충돌 없이 생성됐다.

종료 시 열린 PR은 11개다. 절편 중 #5564가 새로 열렸고 #5552의 head가
`7289f361`로 바뀌었지만, task branch 전체 변경 경로와 모든 열린 PR 변경 경로의 교집합은
0개다. 새 #5564 head `08cbd13c`와 바뀐 #5552 head를 실제 fetch해 현재 HEAD와 각각 가상
병합했고 둘 다 충돌 없이 tree를 생성했다. 따라서 merge나 rebase를 만들지 않았다.

이 판정은 시점 증거다. remote push 또는 PR 생성 직전에는 최신 `devel`과 PR head를 다시
fetch하고 exact SHA 기반 merge-tree를 다시 검증한다.

## 7. 다음 절편 관문

다음 절편에서는 보호된 `armor_command` handler만
`src/cli/queries/security_inspection.rs`로 물리 이동한다.

- top-level dispatch를 새 모듈 경로로 바꾼다.
- 인자 파싱, 페이지별 텍스트 추출, nonce 재생성, scan, JSON·사람 출력과 exit code는
  바꾸지 않는다.
- `load_document`, `mcp_tool_name_registry`, `injection_scan_scopes`, `display_safe`와 전역 암호
  상태는 root에 유지한다.
- 기존 MCP password 불일치와 성공 stderr 진단은 이동 커밋에 섞지 않는다.
- 이동 후 기존+신규 12건과 전체 검증을 다시 실행한다.
- security inspection 모듈은 약 1,061줄로 예상되며 1,200줄 상한 아래에 둔다.

MCP password parity와 성공 stderr 진단은 별도 이슈로 등록해야 하며, 외부 GitHub 변경은
메인테이너 승인 전 수행하지 않는다. 다음 절편과 remote push도 각각 별도 승인 전 시작하지
않는다.
