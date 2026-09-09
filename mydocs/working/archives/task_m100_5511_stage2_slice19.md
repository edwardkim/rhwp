# #5511 Stage 2 열아홉 번째 수직 절편 — injection CLI 계약 선행 보강

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 통합 기준선: `1a6ce79fd56e3cdf5813c7938338fcb5b7d0a859`
- 테스트 커밋: `70f3796a2`
- 수행일: 2026-08-19
- 상태: 완료 — injection handler 이동 절편 승인 대기

## 1. 절편 선정과 중단 판단

열아홉 번째 후보는 root의 `inspect_injection` handler를
`src/cli/queries/security_inspection.rs`로 옮기는 move-only 절편이었다. 기존
`tests/injection_scan_contract.rs`의 14개 테스트는 탐지 6종, 정상 HWP 오탐, 읽기 전용,
JSON 봉투, 신뢰도 필터, 검사 범위, 오류, help, capabilities와 실제 MCP 도구 이름 등록부를
폭넓게 보호한다.

그러나 공개 CLI 이동 경계를 조사하자 세 실행 경로가 직접 고정되어 있지 않았다.

1. 양성 payload가 HWP에서 실제 HWPX로 변환된 뒤에도 종류와 근거 발췌가 유지되는가
2. 사람용 출력이 경고·근거·읽기 전용 고지를 유지하고 제어문자를 안전하게 표시하는가
3. MCP가 선언한 전역 암호 stdin 배선까지 실제 `inspect injection` handler가 소비하는가

코어 탐지 단위 테스트와 JSON 중심 계약만으로는 handler 이동 중 이 세 경로의 회귀를 잡지
못한다. 계획서의 중단 조건에 따라 이번 절편에서는 제품 source 이동을 하지 않고, 이 공백만
characterization contract로 보강했다.

## 2. 구현 경계와 공유 seam 판정

현재 `inspect_injection`은 root의 `load_document`, `mcp_tool_name_registry`,
`injection_scan_scopes`, `display_safe`를 소비한다.

- `mcp_tool_name_registry`는 무상태·세션 MCP 등록부를 함께 읽으며 기존 live-registry 계약이
  하드코딩 회귀를 잡는다.
- `injection_scan_scopes`는 `armor`도 소비하므로 injection 전용 helper가 아니다.
- `display_safe`도 threat scan과 armor의 사람용 출력에서 함께 쓰는 터미널 경계 helper다.
- `load_document`는 전역 암호 상태를 적용하는 공용 문서 로더다.

따라서 다음 이동 절편에서는 handler만 security inspection 모듈로 옮기고, 이 네 공유 helper의
소유권 재편은 섞지 않는다. child 모듈은 현재 root seam을 명시적으로 import해 사용한다. 공유
helper를 한 기능 모듈로 끌어들이거나 복제하면 armor·threat scan·암호 명령의 결합 방향과 단일
원천 불변식이 바뀌므로 별도 설계 없이는 수행하지 않는다.

## 3. 추가한 보호 계약

새 `tests/cases/injection_inspection_contract.rs`의 4개 테스트가 기존 14개 계약과 중복되지 않는
경로를 고정한다.

1. 정상 HWP3와 정상 HWPX가 완전한 빈 JSON 봉투를 내는지 확인한다.
2. 정상 HWP3에 지시 무효화 문장을 실행 중 합성하고 HWPX로 실제 변환한 뒤, 두 포맷 모두
   `instruction_override`와 high 신뢰도 및 근거 발췌를 유지하는지 확인한다.
3. 사람용 출력이 탐지 종류·발췌·문서 내용 불신 경고·읽기 전용 고지를 유지하며, 문서의 탭을
   원시 제어문자가 아니라 `⇥`로 표시하는지 확인한다.
4. 실제 암호 HWPX에서 암호 없음은 exit 2, 잘못된 암호는 exit 1과 빈 stdout, 올바른
   `--password-stdin`은 한 줄 JSON 성공이라는 계약을 확인한다.

합성 문서는 임시 디렉터리에만 만들고 테스트 종료 시 제거한다. 저장소에는 공격 문서나 임시
산출물을 추가하지 않았다.

## 4. 지표 변화

| 항목 | Stage 2 절편 18 | Stage 2 절편 19 | 변화 |
|---|---:|---:|---:|
| `src/main.rs` 줄 수 | 38,962 | 38,962 | source 이동 없음 |
| `src/cli/queries/security_inspection.rs` | 659 | 659 | source 이동 없음 |
| injection 전용 integration source | 1 | 2 | +1 |
| injection CLI 계약 | 14 | 18 | +4 |
| Rust test source | 751 | 752 | +1 |
| static test attribute | 3,708 | 3,712 | +4 |
| 전체 release-test 실행 | 7,757 | 7,761 | +4 |

제품 source를 바꾸지 않았으므로 `main.rs`와 security inspection 모듈의 복잡도 및 계층 참조
수치는 절편 18과 동일하다.

## 5. 검증 기록

| 검증 | 결과 |
|---|---|
| 신규 `injection_inspection_contract` focused nextest | 4/4 통과 |
| 기존+신규 injection CLI 계약 | 18/18 통과 |
| release-test 전체 nextest | 7,761/7,761 통과, 4 slow, 38 skipped, 164.918초 |
| `cargo check --all-targets` | 통과 |
| `cargo fmt --all -- --check` | 통과 |
| `cargo clippy --all-targets --target-dir target/pr-review -- -D warnings` | 통과 |
| `cargo test --doc --target-dir target/pr-review` | 8/8 통과, 3 ignored |
| integration manifest 정책 자체 계약 | 16/16 통과 |
| `rust-test-suite-manifest --check --base-ref upstream/devel` | 통과, 752 sources / 3,712 static test attrs / 43 integration targets |
| unit-tier 정책 자체 계약 | 12/12 통과 |
| `rust-unit-test-tiers --check --base-ref upstream/devel` | 통과, 4,225 tests / 298 modules |
| CI impact Node 계약 | classifier+policy 62/62 통과 |
| `scripts/tests/test_ci_impact_workflow.py` | 30/30 통과 |
| `git diff --check` | 통과 |
| `check_markdown_links.py --changed-from upstream/devel` | 기존 capability 등록부 무결성 오류 16건, 절편 19 신규 오류 없음 |

호스트 nextest는 `0.9.137`이고 저장소 권장은 `0.9.140`이라 경고가 있었지만 실행 결과에는
영향이 없었다. Cargo 명령이 인접한 verifier package의 lockfile 순서와 검증용 test target을
작업트리에 다시 생성했으나 둘 다 추적 변경에서 복원했다. 테스트만 추가했으므로 시각 검증과
WASM 빌드는 추가하지 않았다.

Markdown 검사는 기준선에도 존재하는 `agent_capability_registry.md`의 중복 ID·진입점 링크
무결성 오류 16건을 보고했다. 이번 절편이 추가한 테스트 source에는 Markdown 링크가 없고,
이 보고서의 기존 issue 링크 외 신규 내부 링크도 없다.

## 6. 원격 병합 위험 재검증

절편 시작과 종료 시 모두 `origin/devel`과 `upstream/devel`은 `1a6ce79fd`로 동일했다. 테스트
커밋 기준 작업 브랜치는 45커밋 앞서고 0커밋 뒤이며, 최신 `upstream/devel`과의 merge-tree는
충돌 없이 생성됐다.

열린 PR #5544·#5545·#5546·#5548·#5550·#5552·#5556·#5559·#5560·#5562의 최신 head를
다시 조회했다. task branch 전체 변경 경로와 각 PR 변경 경로의 교집합은 모두 0개였고, 시작 시
확인한 실제 head 10개의 가상 병합도 모두 충돌이 없었다. 원격 `devel`과 열린 PR 목록이 시작
점검 뒤 바뀌지 않았으므로 불필요한 merge commit이나 rebase를 만들지 않았다.

이 판정은 시점 증거다. remote push 또는 PR 생성 직전에는 최신 `devel`을 다시 fetch하고 정확한
SHA와 현재 PR head로 다시 검증한다.

## 7. 다음 절편 관문

다음 절편은 보호된 `inspect_injection` handler만
`src/cli/queries/security_inspection.rs`로 물리 이동한다. `inspect_command`의 dispatch 한 줄만
새 모듈 경로로 바꾸고, `load_document`, `mcp_tool_name_registry`, `injection_scan_scopes`,
`display_safe`는 root에 유지한다.

이동 전후에는 정상 HWP3/HWPX, 합성 양성 HWP/HWPX, 사람용 출력, 암호 경로, 기존 6종 탐지와
live MCP 등록부 결과를 비교한다. 탐지 알고리즘·신뢰도·scope 목록·암호 정책·MCP schema를
바꾸지 않는다. 다음 절편은 메인테이너 승인 전 시작하지 않으며 remote push도 별도 승인 전
수행하지 않는다.
