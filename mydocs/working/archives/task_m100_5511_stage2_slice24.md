# #5511 Stage 2 스물네 번째 수직 절편 — armor CLI handler 이동

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 통합 기준선: `1a6ce79fd56e3cdf5813c7938338fcb5b7d0a859`
- 구현 커밋: `baba50461`
- 수행일: 2026-08-19
- 상태: 완료 — 다음 Stage 2 절편 승인 대기

## 1. 절편 범위

스물세 번째 절편에서 실제 HWP5·HWPX, 암호 HWP3·HWP5·HWPX, 사람용 격벽과 실패 경로까지
보강한 12개 계약을 기준으로 root의 `armor_command` handler만
`src/cli/queries/security_inspection.rs`로 물리 이동했다.

- top-level `armor` dispatch를 새 모듈 경로로 바꿨다.
- handler는 모듈 경계에서 호출되므로 `pub(crate)` 가시성만 부여했다.
- 인자 파싱, 문서 로딩, 페이지별 텍스트 추출, nonce 생성, 주입 scan, JSON·사람용 출력과
  exit code는 바꾸지 않았다.
- armor 알고리즘, provenance, MCP schema와 암호 전달 정책은 바꾸지 않았다.

구현 diff는 170줄 추가와 170줄 삭제다. handler 이름과 본문은 위치만 바뀌었고, 제품 동작을
추가하거나 제거하지 않았다.

## 2. 공유 seam 유지

`load_document`, `mcp_tool_name_registry`, `injection_scan_scopes`, `display_safe`와 전역 암호 상태는
계획대로 root에 유지했다. 대상 모듈은 이미 가진 `fs`, `provenance`, 종료 코드 상수와 root의
공유 helper를 사용한다.

새 wrapper, context 구조, helper 복제나 service 계층의 역방향 의존은 만들지 않았다. 따라서
이번 절편은 security query adapter 소유권만 이동하며 파서·layout·직렬화 계층의 책임을
바꾸지 않는다.

## 3. 동작 동등성과 지표 변화

이동 전 보호한 기존 `armor_contract` 8건과 신규 `armor_cli_contract` 4건을 새 위치에서 함께
실행해 12/12가 통과했다.

- 실제 HWP5·HWPX의 격벽·scope·provenance 봉투와 입력 바이트 불변성이 유지된다.
- 암호 HWP3·HWP5·HWPX의 `--password-stdin` 성공과 누락·오입력 종료 코드가 유지된다.
- 호출별 nonce, 격벽 위조 방지와 주입 신호 탐지가 유지된다.
- 사람용 경고·읽기 전용 고지와 제어문자 안전 표시가 유지된다.
- JSON 성공·실패 경로의 stdout/stderr 분리가 유지된다.

| 항목 | Stage 2 절편 23 | Stage 2 절편 24 | 변화 |
|---|---:|---:|---:|
| `src/main.rs` 줄 수 | 38,730 | 38,561 | -169 |
| `src/cli/queries/security_inspection.rs` | 893 | 1,062 | +169, 1,200줄 상한 이하 |
| Rust test source | 754 | 754 | 변화 없음 |
| static test attribute | 3,721 | 3,721 | 변화 없음 |
| armor CLI 계약 | 12 | 12 | 변화 없음 |

## 4. 검증 기록

| 검증 | 결과 |
|---|---|
| 기존+신규 armor focused nextest | 12/12 통과, 1.459초 |
| release-test 전체 nextest | 7,770/7,770 통과, 3 slow, 38 skipped, 162.912초 |
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
| `check_markdown_links.py --changed-from upstream/devel` | 기존 capability 등록부 무결성 오류 16건, 절편 24 신규 오류 없음 |

호스트 nextest는 `0.9.137`이고 저장소 권장은 `0.9.140`이라 경고가 있었지만 실행 결과에는
영향이 없었다. 준비 스크립트가 로컬 검증용 Cargo test target 두 개를 생성하고 Cargo가
인접 verifier package의 lockfile 순서를 바꿨으나, 둘 다 추적 변경에서 복원했다. 임시 검증
파일도 남기지 않았다.

renderer·serializer·WASM 경계를 건드리지 않은 move-only 변경이므로 시각 검증과 WASM
빌드는 추가하지 않았다. Markdown 검사는 기준선에도 존재하는
`agent_capability_registry.md`의 중복 ID·진입점 링크 무결성 오류 16건만 보고했다.

## 5. 기존 불일치의 비확대

스물세 번째 절편에서 확인한 두 현상은 이번 move-only 범위에 섞지 않았다.

- CLI `armor`에는 전역 password 입력이 있지만 MCP `hwp_armor` schema와
  `supports_password_stdin`에는 대응 표면이 없다.
- 실제 HWP5·HWPX의 성공 실행에서도 기존 `LAYOUT_OVERFLOW` 진단이 stderr에 나온다.

이동 후 계약은 현재 동작이 변하지 않았음을 확인했을 뿐, 두 현상을 바람직한 규약으로
고정하지 않는다. 각각 별도 후속 이슈에서 기능·진단 소유권을 조사해야 한다.

## 6. 원격 병합 위험 재검증

절편 시작과 구현 검증 종료 시 `origin/devel`과 `upstream/devel`은 모두 `1a6ce79fd`로
동일했다. 구현 커밋 기준 작업 브랜치는 55커밋 앞서고 0커밋 뒤이며, 최신
`upstream/devel`과의 merge-tree는 충돌 없이 생성됐다.

종료 시 열린 PR은 11개이며 task branch 전체 변경 경로와 모든 열린 PR 변경 경로의 교집합은
0개다. 절편 중 #5562 head가 `8eb39c0e`로, #5559 head가 `afb92020`으로 바뀌었다. 두 head를
실제로 fetch해 현재 구현 HEAD와 각각 가상 병합했고 둘 다 충돌 없이 tree를 생성했다. 따라서
merge나 rebase를 만들지 않았다.

이 판정은 시점 증거다. remote push 또는 PR 생성 직전에는 최신 `devel`과 PR head를 다시
fetch하고 exact SHA 기반 merge-tree를 다시 검증한다.

## 7. 다음 절편 관문

root에서 다음으로 이어지는 독립 handler는 약 150줄의 `extract_thumbnail`이다. 기존
`issue_3366_thumbnail_contract`와 `genpreview_json_contract`가 인자 파싱, 종료 코드, 파일·base64·
data URI·JSON 모드와 실패 시 stdout 경계를 일부 보호한다.

그러나 `thumbnail`은 내장 미리보기를 읽는 조회 성격과 기본 모드에서 새 이미지 파일을 쓰는
명령 성격을 함께 가진다. 이를 곧바로 `queries` 아래로 이동하면 CQRS 소유권을 흐릴 수 있고,
현재 security inspection 모듈도 1,062줄이라 다른 책임을 더 넣지 않아야 한다.

따라서 다음 절편은 제품 코드를 이동하기 전에 다음만 조사한다.

- 성공한 파일 출력의 바이트·format·MIME·크기·경로와 입력 불변성 계약이 충분한지 확인한다.
- base64와 data URI의 decoding 결과가 실제 내장 미리보기와 같은지 확인한다.
- 기본 출력 경로 생성과 파일 저장 실패의 부작용·종료 코드 계약을 확인한다.
- `thumbnail`을 query adapter, export/output adapter 또는 별도 preview 모듈 중 어디에 둘지
  CQRS·SOLID 기준으로 결정한다.
- 계약이 부족하면 characterization test만 추가하고 이동하지 않는다.

다음 절편은 메인테이너 승인 전 시작하지 않으며 remote push도 별도 승인 전 수행하지 않는다.
