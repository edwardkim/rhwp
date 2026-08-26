# #5511 Stage 2 열여덟 번째 수직 절편 — PR 제출 구조 최소 정정

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 통합 기준선: `1a6ce79fd56e3cdf5813c7938338fcb5b7d0a859`
- 정정 커밋: `4684856e7`
- 수행일: 2026-08-19
- 상태: 완료 — 다음 Stage 2 기능 절편 승인 대기

## 1. 정정 사유

열일곱 번째 절편에서 처음으로 integration manifest를 `--base-ref upstream/devel` 제출 모드로
검사하자 Stage 1에서 누적된 두 문제가 확인됐다.

1. 신규 `cli_catalog_contract` 원본이 허용 경로 `tests/cases/`가 아니라 최상위 `tests/`에 있었다.
2. review worktree와 CI에서만 생성해야 하는 `Cargo.toml` test target 블록 두 개가 task branch의
   추적 변경으로 커밋돼 있었다.

기존 절편 검증은 준비된 현재 상태의 manifest 일치와 정책 자체 테스트를 통과했지만, PR base와
HEAD의 제출 diff를 판정하지 않아 이 누적 문제를 찾지 못했다. 따라서 기능 이동을 멈추고 PR 제출
구조만 고치는 최소 정정 절편으로 처리했다.

## 2. 원격 병합 위험 선행 점검

메인테이너가 우려한 최근 원격 변경과의 병합 위험을 구현보다 먼저 확인했다.

- `origin/devel`과 `upstream/devel`은 모두 `1a6ce79fd`로 동일했다.
- 절편 시작 시 작업 브랜치는 기준선보다 42커밋 앞서고 0커밋 뒤였다.
- 현재 HEAD와 `upstream/devel`의 merge-tree는 충돌 없이 생성됐다.
- 열린 PR 10개의 변경 경로와 task branch 변경 경로의 교집합은 0개였다.
- 각 PR의 실제 `refs/pull/<N>/head`를 fetch해 task branch와 개별 merge-tree를 만들었고,
  #5544·#5545·#5546·#5548·#5550·#5552·#5556·#5559·#5560·#5562 모두 충돌 없이 생성됐다.
- 전체 검증 후 두 원격 `devel`을 다시 fetch했으며 SHA는 여전히 `1a6ce79fd`, ahead/behind는
  43/0, merge-tree도 다시 충돌 0건이었다.

따라서 이 시점에는 불필요한 merge commit이나 rebase를 만들지 않았다. 이 판정은 시점 증거이므로
remote push 또는 PR 생성 직전 최신 `devel`을 다시 fetch하고 정확한 SHA로 재검증해야 한다.

## 3. 정정 내용과 불변식

- `tests/cli_catalog_contract.rs`를 `tests/cases/cli_catalog_contract.rs`로 이동했다.
- 한 단계 깊어진 경로에 맞춰 `#[path]`와 `include_str!` 11곳만 `../src`에서 `../../src`로
  바꿨다.
- 경로 접두사를 정규화한 전후 source 비교는 byte 단위로 일치했다.
- `Cargo.toml`에서 파생 `cli_catalog_contract`와 `cli_json_contract` target 블록 8줄을 제거했다.
- `--prepare`는 로컬 검증 중 두 target을 다시 생성하지만, 정정 커밋에는 포함하지 않았다.
- 제품 source, catalog 내용, 명령 수, 테스트 assertion과 공개 CLI 동작은 바꾸지 않았다.

정정 커밋은 이전 커밋 대비 95% rename으로 인식된다. PR base 관점에서는 허용된
`tests/cases/cli_catalog_contract.rs` 원본만 새 파일이고 `Cargo.toml`에는 최종 차이가 없다.

## 4. 검증 기록

| 검증 | 결과 |
|---|---|
| 경로 접두사 정규화 후 source 비교 | byte 단위 일치 |
| 이동 후 `cli_catalog_contract` focused nextest | 19/19 통과 |
| integration manifest 정책 자체 계약 | 16/16 통과 |
| `rust-test-suite-manifest --check --base-ref upstream/devel` | 통과 |
| unit-tier 정책 자체 계약 | 12/12 통과 |
| `rust-unit-test-tiers --check --base-ref upstream/devel` | 통과, 4,225 tests / 298 modules |
| Rust test suite manifest | 통과, 751 sources / 3,708 static test attrs / 43 integration targets |
| release-test 전체 nextest | 7,757/7,757 통과, 4 slow, 38 skipped, 166.676초 |
| `cargo check --all-targets` | 통과 |
| `cargo fmt --all -- --check` | 통과 |
| `cargo clippy --all-targets --target-dir target/pr-review -- -D warnings` | 통과 |
| `cargo test --doc --target-dir target/pr-review` | 8/8 통과, 3 ignored |
| CI impact Node 계약 | classifier 31/31, policy 31/31 통과 |
| `scripts/tests/test_ci_impact_workflow.py` | 30/30 통과 |
| 시작·종료 remote merge-tree | 각각 충돌 0건 |
| 열린 PR 실제 head 10개 가상 병합 | 10/10 충돌 없음 |
| `git diff --check` | 통과 |
| `check_markdown_links.py --changed-from upstream/devel` | 기존 capability 등록부 무결성 오류 16건, #5511 신규 오류 없음 |

첫 정적 검사에서 이동으로 길어진 `SECURITY_INSPECTION_SOURCE`의 `include_str!` 한 줄에 rustfmt
차이가 발견됐다. 제품·계약 실패는 아니며 표준 줄바꿈을 적용해 아직 push되지 않은 정정 커밋에
포함한 뒤 fmt·check·clippy·doc gate를 다시 통과했다.

호스트 nextest는 `0.9.137`이고 저장소 권장은 `0.9.140`이라 경고가 있었지만 실행 결과에는
영향이 없었다. Cargo 명령이 인접한 verifier package의 lockfile 순서와 검증용 test target을
작업트리에 다시 생성했으나 둘 다 추적 변경에서 복원했다. 테스트 source 위치만 바꿨으므로 시각
검증과 WASM 빌드는 추가하지 않았다.

Markdown 검사는 기준선에도 존재하는 `agent_capability_registry.md`의 중복 ID·진입점 링크
무결성 오류 16건을 보고했다. 이번 정정 절편이 추가한 링크 오류는 없다.

## 5. 다음 절편 관문

branch-wide PR 제출 구조가 base-aware 정책까지 통과했으므로 다음 기능 후보는
`inspect injection`이다. 다만 handler가 root의 공유 `mcp_tool_name_registry`와
`injection_scan_scopes`를 사용하고, 후자는 `armor`도 함께 소비한다. 다음 절편에서는 먼저
`injection_scan_contract`의 CLI·MCP·암호·사람용/JSON 출력 보호 범위와 공유 seam 소유권을
조사한다. 보호가 부족하면 characterization contract만 먼저 추가하고 기능 이동과 섞지 않는다.

다음 절편 시작 전에도 `upstream/devel`을 다시 fetch한다. 원격이 전진했다면 exact SHA 기반
merge-tree와 변경 경로 중첩을 먼저 판정하고, 실제 merge 또는 rebase는 메인테이너 승인 뒤 별도
통합 절차로 수행한다. remote push는 별도 승인 전 수행하지 않는다.
