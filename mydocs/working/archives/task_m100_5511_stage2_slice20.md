# #5511 Stage 2 스무 번째 수직 절편 — injection CLI handler 이동

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 통합 기준선: `1a6ce79fd56e3cdf5813c7938338fcb5b7d0a859`
- 구현 커밋: `42079563a`
- 수행일: 2026-08-19
- 상태: 완료 — 다음 Stage 2 절편 승인 대기

## 1. 절편 범위

열아홉 번째 절편에서 보강한 HWP3·HWPX·사람용 출력·암호 실행 계약을 기준으로 root의
`inspect_injection` handler만 `src/cli/queries/security_inspection.rs`로 물리 이동했다.

- `inspect_command`의 injection dispatch를 새 모듈 경로로 바꿨다.
- handler는 모듈 경계에서 호출되므로 `pub(crate)` 가시성만 부여했다.
- 인자 파싱, 문서 로드, 탐지 옵션, JSON 봉투, 사람용 출력과 exit code는 바꾸지 않았다.
- 탐지 알고리즘, 신뢰도, scope 목록, MCP schema와 암호 정책은 바꾸지 않았다.

구현 diff는 137줄 추가와 137줄 삭제다. import와 dispatch를 제외한 handler 본문은 위치만
바뀌었다.

## 2. 공유 seam 유지

다음 네 helper는 계획대로 root에 유지했다.

| helper | 유지 이유 |
|---|---|
| `load_document` | 전역 암호 상태를 적용하는 여러 명령의 공용 문서 로더 |
| `mcp_tool_name_registry` | injection과 armor가 함께 쓰는 무상태·세션 MCP 이름 단일 원천 |
| `injection_scan_scopes` | injection JSON·사람 출력과 armor가 공유하는 실제 검사 범위 |
| `display_safe` | injection·threat scan·armor가 공유하는 터미널 출력 안전 경계 |

새 모듈은 이 seam을 명시적으로 import해 읽기만 한다. helper 복제, 소유권 변경, 새 context
구조 도입은 이번 move-only 절편에 섞지 않았다.

## 3. 동작 동등성

기존 `injection_scan_contract` 14건과 열아홉 번째 절편의
`injection_inspection_contract` 4건을 새 위치에서 함께 실행했다.

- 정상 HWP 전수와 명시적 HWP3·HWPX는 clean이다.
- 여섯 탐지 종류와 정상 공문 어투의 양성·음성 쌍이 유지된다.
- 합성 HWP를 HWPX로 실제 변환한 뒤에도 `instruction_override`, high 신뢰도와 근거 발췌가
  유지된다.
- JSON 봉투·신뢰도 필터·scope·stdout/stderr·help·capabilities·MCP live registry가 동일하다.
- 사람용 경고와 제어문자 표시, 암호 없음·오류·stdin 성공 경로가 동일하다.
- 스캔은 입력 문서를 변경하거나 payload를 정화하지 않는다.

focused 18건과 전체 release-test가 모두 통과했으므로 관찰 가능한 계약 차이는 없었다.

## 4. 지표 변화

| 항목 | Stage 2 절편 19 | Stage 2 절편 20 | 변화 |
|---|---:|---:|---:|
| `src/main.rs` 줄 수 | 38,962 | 38,828 | -134 |
| `src/cli/queries/security_inspection.rs` | 659 | 793 | +134, 1,200줄 상한 이하 |
| `main.rs` CC>25 함수 | 19 | 19 | 변화 없음 |
| security inspection 모듈 CC>25 함수 | 0 | 0 | 변화 없음 |
| Rust test source | 752 | 752 | 변화 없음 |
| static test attribute | 3,712 | 3,712 | 변화 없음 |
| injection CLI 계약 | 18 | 18 | 변화 없음 |

함수의 내부 분기와 직접 의존은 바꾸지 않았으므로 전체 CLI 평가 단위의 복잡도와 계층 참조
수치는 동일하다. 이 절편은 root에서 query adapter 소유권만 옮겼다.

## 5. 검증 기록

| 검증 | 결과 |
|---|---|
| 기존+신규 injection CLI focused nextest | 18/18 통과 |
| release-test 전체 nextest | 7,761/7,761 통과, 3 slow, 38 skipped, 168.829초 |
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
| `check_markdown_links.py --changed-from upstream/devel` | 기존 capability 등록부 무결성 오류 16건, 절편 20 신규 오류 없음 |

호스트 nextest는 `0.9.137`이고 저장소 권장은 `0.9.140`이라 경고가 있었지만 실행 결과에는
영향이 없었다. Cargo 명령이 인접한 verifier package의 lockfile 순서와 검증용 test target을
작업트리에 다시 생성했으나 둘 다 추적 변경에서 복원했다. renderer·serializer·WASM 경계를
건드리지 않은 move-only 변경이므로 시각 검증과 WASM 빌드는 추가하지 않았다.

Markdown 검사는 기준선에도 존재하는 `agent_capability_registry.md`의 중복 ID·진입점 링크
무결성 오류 16건만 보고했다. 이번 절편의 Rust source 이동으로 생긴 Markdown 오류는 없다.

## 6. 원격 병합 위험 재검증

절편 시작과 종료 시 `origin/devel`과 `upstream/devel`은 모두 `1a6ce79fd`로 동일했다. 구현
커밋 기준 작업 브랜치는 47커밋 앞서고 0커밋 뒤이며, 최신 `upstream/devel`과의 merge-tree는
충돌 없이 생성됐다.

열린 PR #5544·#5545·#5546·#5548·#5550·#5552·#5556·#5559·#5560·#5562의 head도 시작과
종료 사이 바뀌지 않았다. task branch 전체 변경 경로와 각 PR 변경 경로의 교집합은 모두
0개이고, 앞 절편에서 실제 head 10개의 가상 병합도 모두 충돌 없이 생성됐다. 따라서 merge나
rebase를 만들지 않았다.

이 판정은 시점 증거다. remote push 또는 PR 생성 직전에는 최신 `devel`과 PR head를 다시
fetch하고 exact SHA 기반 merge-tree를 다시 검증한다.

## 7. 다음 절편 관문

다음 read-only security query 후보는 root의 `cmd_threat_scan`이다. 약 100줄이고 현재 security
inspection 모듈과 합쳐도 1,200줄 상한 이하지만, 기존 `threat_scan_contract`는 코어 탐지와 JSON
봉투 중심이다. 다음 절편에서는 먼저 사람용 출력, usage/runtime exit, 정상·양성 HWP/HWPX,
truncation·note 및 `display_safe` 공유 seam의 보호 범위를 조사한다.

보호가 부족하면 characterization contract만 먼저 추가하고 이동과 섞지 않는다. 충분하면
`cmd_threat_scan` handler만 이동하며 탐지 휴리스틱과 공유 `display_safe`는 바꾸지 않는다. 다음
절편은 메인테이너 승인 전 시작하지 않으며 remote push도 별도 승인 전 수행하지 않는다.
