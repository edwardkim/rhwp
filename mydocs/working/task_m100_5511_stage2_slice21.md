# #5511 Stage 2 스물한 번째 수직 절편 — threat-scan CLI 이동 전 계약 보강

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 통합 기준선: `1a6ce79fd56e3cdf5813c7938338fcb5b7d0a859`
- 계약 커밋: `444ee88f2`
- 수행일: 2026-08-19
- 상태: 완료 — 다음 Stage 2 절편 승인 대기

## 1. 절편 범위와 중단 판단

스무 번째 절편에서 다음 후보로 지정한 root의 `cmd_threat_scan`을 바로 이동하지 않고, 먼저
handler의 관찰 가능 계약을 조사했다. 기존 `tests/threat_scan_contract.rs` 9건은 HWP5·HWPX
구조 탐지, 탐지 종류와 결정성, JSON provenance를 보호했지만 다음 경계는 직접 보호하지
않았다.

- 실제 공개 HWP5·HWPX와 HWP3에 대한 CLI 봉투
- 사람용 정상·경고·지원하지 않는 형식 출력
- usage·runtime·help의 exit code와 stdout/stderr 분리
- 문서에서 파생된 제어문자의 터미널 안전 표시
- 최대 finding 수와 `truncated` 봉투의 일치
- 스캔 전후 입력 바이트 불변성

따라서 이번 절편은 제품 코드를 이동하지 않고 characterization contract만 추가하는 지점에서
멈췄다. handler 이동과 누락 계약 발견을 한 커밋에 섞지 않아, 다음 절편의 회귀가 물리 이동
때문인지 기존 동작 변경 때문인지 분리해서 판정할 수 있게 했다.

## 2. 추가한 보호 계약

`tests/cases/threat_scan_cli_contract.rs`에 다섯 개의 black-box CLI 계약을 추가했다.

| 계약 | 보호하는 동작 |
|---|---|
| 실제 HWP5·HWPX 정상 스캔 | 형식·scope·빈 findings·notes·truncated와 입력 바이트 불변성 |
| HWP3 정직한 보고 | `unknown`, 빈 scope와 "스캔하지 않았습니다" note를 JSON·사람 출력에 모두 유지 |
| 외부 참조 사람용 경고 | 위협 종류·안전 고지와 문서 파생 탭의 `⇥` 표시, 원시 탭 출력 금지 |
| usage·runtime·help | 사용 오류 2, 실행 오류 1, help 0과 stdout/stderr 분리 |
| finding 상한 | 2,001개 script entry에서 findings 2,000개와 `truncated: true` 유지 |

합성 HWPX는 테스트별 임시 디렉터리에서 생성하고 `Drop`에서 파일과 디렉터리를 제거한다.
저장소 sample이나 사설 코퍼스를 만들거나 변경하지 않는다.

첫 focused 실행에서는 XML 속성의 `&#x9;`가 parser 단계에서 실제 탭으로 해석될 것이라는 테스트
가정 때문에 1건이 실패했다. 제품 동작을 바꾸지 않고 fixture에 실제 탭 바이트를 넣어
`display_safe` 경계를 직접 통과하도록 정정했다. 이후 신규 5건과 기존 9건이 모두 통과했다.

## 3. 동작과 구조 영향

이번 절편의 추적 변경은 integration test source 한 파일뿐이다. `src/main.rs`,
`src/cli/queries/security_inspection.rs`, 탐지 휴리스틱, JSON schema, CLI dispatch와
`display_safe`는 바꾸지 않았다.

| 항목 | Stage 2 절편 20 | Stage 2 절편 21 | 변화 |
|---|---:|---:|---:|
| `src/main.rs` 줄 수 | 38,828 | 38,828 | 변화 없음 |
| `src/cli/queries/security_inspection.rs` | 793 | 793 | 변화 없음 |
| Rust test source | 752 | 753 | +1 |
| static test attribute | 3,712 | 3,717 | +5 |
| threat-scan CLI 계약 | 9 | 14 | +5 |

제품 함수가 바뀌지 않았으므로 root와 security inspection 모듈의 복잡도 수치, 계층 참조와
공개 CLI surface도 동일하다.

## 4. 검증 기록

| 검증 | 결과 |
|---|---|
| 신규 threat-scan focused nextest | 5/5 통과 |
| 기존+신규 threat-scan focused nextest | 14/14 통과 |
| release-test 전체 nextest | 7,766/7,766 통과, 3 slow, 38 skipped, 165.932초 |
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
| `check_markdown_links.py --changed-from upstream/devel` | 기존 capability 등록부 무결성 오류 16건, 절편 21 신규 오류 없음 |

호스트 nextest는 `0.9.137`이고 저장소 권장은 `0.9.140`이라 경고가 있었지만 실행 결과에는
영향이 없었다. 준비 스크립트가 로컬 검증용 Cargo test target 두 개를 생성하고 Cargo가
인접 verifier package의 lockfile 순서를 바꿨으나, 둘 다 추적 변경에서 복원했다.
renderer·serializer·WASM 경계를 건드리지 않은 테스트 전용 변경이므로 시각 검증과 WASM
빌드는 추가하지 않았다.

Markdown 검사는 기준선에도 존재하는 `agent_capability_registry.md`의 중복 ID·진입점 링크
무결성 오류 16건만 보고했다. 이번 절편에서 새로 생긴 Markdown 오류는 없다.

## 5. 원격 병합 위험 재검증

절편 시작과 종료 시 `origin/devel`과 `upstream/devel`은 모두 `1a6ce79fd`로 동일했다. 계약
커밋 기준 작업 브랜치는 49커밋 앞서고 0커밋 뒤이며, 최신 `upstream/devel`과의 merge-tree는
충돌 없이 생성됐다.

열린 PR #5544·#5545·#5546·#5548·#5550·#5552·#5556·#5559·#5560·#5562의 head는 절편
시작과 종료 사이 바뀌지 않았다. task branch 전체 변경 경로와 각 PR 변경 경로의 교집합은
모두 0개다. 따라서 merge나 rebase를 만들지 않았다.

이 판정은 시점 증거다. remote push 또는 PR 생성 직전에는 최신 `devel`과 PR head를 다시
fetch하고 exact SHA 기반 merge-tree를 다시 검증한다.

## 6. 다음 절편 관문

다음 절편에서는 보호된 `cmd_threat_scan` handler만
`src/cli/queries/security_inspection.rs`로 물리 이동한다.

- top-level dispatch를 새 모듈 경로로 바꾼다.
- handler의 인자 파싱, scan 호출, JSON·사람 출력과 exit code는 바꾸지 않는다.
- 탐지 휴리스틱과 threat model을 바꾸지 않는다.
- injection·threat scan·armor가 공유하는 `display_safe`는 root에 유지한다.
- 이동 후 신규+기존 14건과 전체 검증을 다시 실행한다.
- security inspection 모듈은 약 900줄로 예상되며 1,200줄 상한 아래에 둔다.

다음 절편은 메인테이너 승인 전 시작하지 않으며 remote push도 별도 승인 전 수행하지 않는다.
