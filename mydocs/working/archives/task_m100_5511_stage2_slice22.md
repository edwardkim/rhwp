# #5511 Stage 2 스물두 번째 수직 절편 — threat-scan CLI handler 이동

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 통합 기준선: `1a6ce79fd56e3cdf5813c7938338fcb5b7d0a859`
- 구현 커밋: `dce639653`
- 수행일: 2026-08-19
- 상태: 완료 — 다음 Stage 2 절편 승인 대기

## 1. 절편 범위

스물한 번째 절편에서 보강한 실제 HWP5·HWPX·HWP3, 사람용 출력, 종료 코드와 truncation
계약을 기준으로 root의 `cmd_threat_scan` handler만
`src/cli/queries/security_inspection.rs`로 물리 이동했다.

- top-level `threat-scan` dispatch를 새 모듈 경로로 바꿨다.
- handler는 모듈 경계에서 호출되므로 `pub(crate)` 가시성만 부여했다.
- 인자 파싱, 파일 읽기, scan 호출, JSON 봉투, 사람용 출력과 exit code는 바꾸지 않았다.
- 탐지 휴리스틱, threat model, finding 상한과 정직한 HWP3 note는 바꾸지 않았다.

구현 diff는 103줄 추가와 101줄 삭제다. 두 줄 차이는 모듈 경로를 명시한 top-level dispatch
포맷에서 생겼고, handler 본문과 주석은 위치만 바뀌었다.

## 2. 공유 seam 유지

`threat_scan` handler가 쓰는 `fs`, `provenance`와 종료 코드 상수는 대상 모듈의 기존 import를
그대로 사용한다. injection·threat scan·armor가 함께 쓰는 `display_safe`는 계획대로 root에
유지하고 대상 모듈이 import한다.

새 wrapper, context 구조, helper 복제나 역방향 service 의존은 만들지 않았다. 따라서 이번
절편은 query adapter 소유권만 이동하며 탐지·직렬화 계층의 책임을 바꾸지 않는다.

## 3. 동작 동등성과 지표 변화

이동 전 보호한 기존 `threat_scan_contract` 9건과 신규
`threat_scan_cli_contract` 5건을 새 위치에서 함께 실행했다.

- HWP5·HWPX의 정상·PE·손상 record·script·외부 참조 판정이 유지된다.
- JSON provenance와 문서 파생 untrusted detail이 유지된다.
- 실제 HWP5·HWPX와 HWP3의 형식·scope·note 봉투가 유지된다.
- 사람용 경고와 제어문자 안전 표시가 유지된다.
- usage·runtime·help의 exit code와 stdout/stderr 분리가 유지된다.
- 2,001개 script entry에서 findings 2,000개와 `truncated: true`가 유지된다.
- 스캔 전후 입력 문서 바이트가 동일하다.

| 항목 | Stage 2 절편 21 | Stage 2 절편 22 | 변화 |
|---|---:|---:|---:|
| `src/main.rs` 줄 수 | 38,828 | 38,730 | -98 |
| `src/cli/queries/security_inspection.rs` | 793 | 893 | +100, 1,200줄 상한 이하 |
| `main.rs` CC>25 함수 | 19 | 19 | 변화 없음 |
| security inspection 모듈 CC>25 함수 | 0 | 0 | 변화 없음 |
| Rust test source | 753 | 753 | 변화 없음 |
| static test attribute | 3,717 | 3,717 | 변화 없음 |
| threat-scan CLI 계약 | 14 | 14 | 변화 없음 |

함수 내부 분기와 호출은 바꾸지 않았고 이동 대상은 CC 25 이하이므로 복잡도를 다른 파일로
옮겨 숨기지 않았다.

## 4. 검증 기록

| 검증 | 결과 |
|---|---|
| 기존+신규 threat-scan focused nextest | 14/14 통과 |
| release-test 전체 nextest | 7,766/7,766 통과, 3 slow, 38 skipped, 167.232초 |
| `cargo check --all-targets` | 통과 |
| `cargo fmt --all -- --check` | 통과 |
| `cargo clippy --all-targets --target-dir target/pr-review -- -D warnings` | 통과 |
| `cargo test --doc --target-dir target/pr-review` | 8/8 통과, 3 ignored |
| integration manifest 정책 자체 계약 | 16/16 통과 |
| `rust-test-suite-manifest --check --base-ref upstream/devel` | 통과, 753 sources / 3,717 static test attrs / 43 integration targets |
| unit-tier 정책 자체 계약 | 12/12 통과 |
| `rust-unit-test-tiers --check --base-ref upstream/devel` | 통과, 4,225 tests / 298 modules |
| CI impact Node 계약 | classifier+policy 62/62 통과 |
| `scripts/tests/test_ci_impact_workflow.py` | 30/30 통과 |
| `git diff --check` | 통과 |
| `check_markdown_links.py --changed-from upstream/devel` | 기존 capability 등록부 무결성 오류 16건, 절편 22 신규 오류 없음 |

호스트 nextest는 `0.9.137`이고 저장소 권장은 `0.9.140`이라 경고가 있었지만 실행 결과에는
영향이 없었다. 준비 스크립트가 로컬 검증용 Cargo test target 두 개를 생성하고 Cargo가
인접 verifier package의 lockfile 순서를 바꿨으나, 둘 다 추적 변경에서 복원했다. 임시 검증
출력 파일도 남기지 않았다.

renderer·serializer·WASM 경계를 건드리지 않은 move-only 변경이므로 시각 검증과 WASM
빌드는 추가하지 않았다. Markdown 검사는 기준선에도 존재하는
`agent_capability_registry.md`의 중복 ID·진입점 링크 무결성 오류 16건만 보고했다.

## 5. 원격 병합 위험 재검증

절편 시작과 구현 검증 종료 시 `origin/devel`과 `upstream/devel`은 모두 `1a6ce79fd`로
동일했다. 구현 커밋 기준 작업 브랜치는 51커밋 앞서고 0커밋 뒤이며, 최신
`upstream/devel`과의 merge-tree는 충돌 없이 생성됐다.

열린 PR #5544·#5545·#5546·#5548·#5550·#5552·#5556·#5559·#5560·#5562의 head는 절편
시작과 종료 사이 바뀌지 않았다. task branch 전체 변경 경로와 각 PR 변경 경로의 교집합은
모두 0개다. 따라서 merge나 rebase를 만들지 않았다.

이 판정은 시점 증거다. remote push 또는 PR 생성 직전에는 최신 `devel`과 PR head를 다시
fetch하고 exact SHA 기반 merge-tree를 다시 검증한다.

## 6. 다음 절편 관문

다음 security query 후보는 root의 `armor_command`다. 약 168줄이고 이동하면 security
inspection 모듈은 약 1,061줄로 예상되어 1,200줄 상한 아래지만, nonce 생성·페이지별 텍스트
추출·프롬프트 주입 탐지·사람용 격벽 출력과 여러 공용 seam을 한 handler가 함께 사용한다.

다음 절편에서는 기존 `armor_contract`가 실제 HWP3·HWPX, 암호 문서, 사람용 제어문자 출력,
페이지 추출 실패와 nonce 재생성 경계를 충분히 보호하는지 먼저 조사한다. 부족하면 이동하지
않고 characterization contract만 추가한다. 충분하면 `armor_command`만 이동하며
`load_document`, `mcp_tool_name_registry`, `injection_scan_scopes`, `display_safe`와 전역 암호
상태는 root에 유지한다.

다음 절편은 메인테이너 승인 전 시작하지 않으며 remote push도 별도 승인 전 수행하지 않는다.
