# #5511 Stage 2 열세 번째 수직 절편 — 문서 어포던스 explore query 물리 분리

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 통합 기준선: `1a6ce79fd56e3cdf5813c7938338fcb5b7d0a859`
- 구현 커밋: `c1069058a`
- 수행일: 2026-08-19
- 상태: 완료 — 다음 Stage 2 절편 승인 대기

## 1. 절편 선정과 경계 판단

열세 번째 이동 대상으로 `explore`를 선택했다. 이 명령은 문서에서 이미 계산된 표·필드·차트·
구조·각주·미주·주입 신호·은닉 텍스트 facts를 모아 사용 가능한 행동 메뉴를 내는 read-only
query다. 메뉴의 순위와 정직성 문구는 `document_core::queries::explore`, 탐지 결과는 각
공개 query가 소유하므로 CLI handler는 판정 로직을 재구현하지 않는다.

`explore_document`는 cognitive complexity 25 이하이고 이동 후 모듈은 158줄이라 CC>25=0과
1,200줄 상한을 지킨다. menu facts와 security 우선순위는 `explore_menu_contract`, JSON
provenance는 `provenance_contract`가 보호한다. HWP5·HWPX·HWP3, 사람용·JSON, 표·필드·
각주·미주·암호 및 오류 경로도 이동 전 출력 hash로 추가 고정했다.

절편 시작·종료 시 `upstream/devel`은 `1a6ce79fd`로 동일했다. 활성 PR 중 `src/main.rs`,
`src/cli/queries/explore.rs`, query module index, explore·provenance·CLI catalog 계약과 이
보고서에 겹치는 변경은 없었다.

## 2. 구현 결과와 보호 불변식

- 새 `src/cli/queries/explore.rs`가 `explore_document` CLI adapter를 소유한다.
- `src/main.rs`의 최상위 match는 explore query 모듈 API만 호출한다.
- 공개 함수 표식만 정규화한 기계 비교에서 이동 블록이 byte 단위로 일치했다.
- `cli_catalog_contract`가 handler 소유권, root 재유입 금지와 dispatch를 고정한다.
- 사람용 메뉴, JSON schema·provenance, facts 집계, security 우선순위, format label, exit code와
  stdout/stderr 분리를 바꾸지 않았다.

`mcp_tool_name_registry`는 explore 외에 injection scan과 MCP surface도 함께 소비한다. 이를
explore CLI 하위로 옮기면 다른 adapter와 MCP가 explore에 역의존하므로 crate root에 보존했다.
새 모듈은 이 공유 seam을 읽기만 하며, registry와 security query를 application/service 경계로
내리는 일은 Stage 3에서 다룬다.

## 3. 지표 변화

| 항목 | Stage 2 절편 12 | Stage 2 절편 13 | 변화 |
|---|---:|---:|---:|
| `src/main.rs` 줄 수 | 39,763 | 39,613 | -150 |
| `src/cli/queries/explore.rs` | 없음 | 158 | 신규, 모듈 상한 이하 |
| `main.rs` plain 최상위 함수 | 321 | 320 | handler 1개 이동 |
| 누적 이동 read-only handler | 16 | 17 | 1개 추가 |
| `main.rs` CC>25 함수 | 19 | 19 | 변화 없음 |
| `main.rs` 최대 CC | 68 | 68 | 변화 없음 |
| explore 모듈 CC>25 함수 | 없음 | 0 | 새 경계 내 초과 없음 |
| `wasm_api::HwpDocument` 직접 참조 | 42 | 42 | 변화 없음 |
| `rhwp::model` 직접 참조 | 63 | 63 | 변화 없음 |
| `rhwp::parser` 직접 참조 | 80 | 72 | detect call·FileFormat variant 참조 이동 |
| `rhwp::renderer` 직접 참조 | 24 | 24 | 변화 없음 |
| `rhwp::service` 참조 | 0 | 0 | Stage 3 대상 |

이번 절편은 explore의 format detection과 facts 조립을 책임 경계 안에 두되 공유 MCP registry는
root에 남겼다. 따라서 복잡도를 다른 파일로 숨기지 않았고 service 이행과 관찰 가능한 동작 변경도
섞지 않았다.

## 4. 외부 동작 동등성

열두 번째 절편 통합 바이너리와 이동 후 바이너리에 대해 다음 열다섯 경로의 exit code와
stdout/stderr byte 수·SHA-256을 비교했다.

1. 일반 문서 사람용 메뉴
2. 일반 문서 JSON 봉투
3. 필드 문서 JSON
4. 병합 셀이 있는 표 문서 JSON
5. 각주 문서 JSON
6. 미주 문서 JSON
7. HWPX format label 경로
8. HWP3 format label 경로
9. 존재하지 않는 파일
10. 필수 경로 누락
11. 위치 인자 초과
12. 알 수 없는 옵션
13. 암호 문서에 비밀번호 없음
14. 암호 문서에 잘못된 비밀번호
15. 암호 문서에 올바른 비밀번호

열다섯 경로 모두 byte 단위로 일치했다. 사람용 메뉴 663 bytes의 SHA-256은
`dedbed6e1b02cba805f8375b0f0499d24429cd8b57361c9c0d180c7012830aa5`, 일반 JSON
786 bytes는 `b88dad7a89173a7eea7eb8e406ff6858e18610dad4eb8c4a3a3b702702267dd4`, HWP3
JSON 1,432 bytes는 `bb53cafe3ec41b4ab41bf073573b256f3d32a4c8ecf2e81c3a01928f2a1980af`다.
올바른 비밀번호 경로 1,511 bytes의 hash는
`9b82447e81df20acdccd9bc940f350d7cb44b7ea48a5b7531a8722f93a65cd6c`로 일치했다.
오류 경로는 stdout이 비어 있으며 기존 exit 1/2와 stderr hash를 그대로 유지했다.

## 5. 검증 기록

| 검증 | 결과 |
|---|---|
| 이동 전·후 explore menu·provenance focused nextest | 각각 16/16 통과 |
| `cli_catalog_contract` | 이동 전 15/15, 이동 후 16/16 통과 |
| 사람용·JSON·format·암호·오류 출력 hash equivalence | 15/15 일치 |
| release-test 전체 nextest | 7,746/7,746 통과, 3 slow, 38 skipped, 176.278초 |
| `cargo check --all-targets` | 통과 |
| `cargo fmt --all -- --check` | 통과 |
| `cargo clippy --all-targets --target-dir target/pr-review -- -D warnings` | 통과 |
| `cargo test --doc --target-dir target/pr-review` | 8/8 통과, 3 ignored |
| Rust test suite manifest | 통과, 750 sources / 3,697 static test attrs / 43 integration targets |
| Rust unit tier | 통과, 4,225 tests / 298 modules |
| test policy 자체 계약 | integration 16/16, unit tier 12/12 통과 |
| `scripts/tests/test_ci_impact_workflow.py` | 30/30 통과 |
| `git diff --check` | 통과 |
| `check_markdown_links.py --changed-from upstream/devel` | 기존 capability 등록부 무결성 오류 16건, #5511 신규 오류 없음 |

호스트 nextest는 `0.9.137`이고 저장소 권장은 `0.9.140`이라 경고가 있었지만 실행 결과에는
영향이 없었다. Cargo가 최신 원격에서 이름이 인접한 두 verifier package의 lockfile 순서만
재정렬했으나 #5511과 무관한 파생 변화이므로 추적 변경에서 복원했다. parser·보안 탐지·구조
추론·조판 로직은 바꾸지 않고 adapter 위치만 옮겼으므로 시각 검증과 WASM 빌드는 추가하지 않았다.

Markdown 검사는 기준선에도 존재하는 `agent_capability_registry.md`의 중복 ID·진입점 링크
무결성 오류 16건을 보고했다. 이번 절편이 추가한 보고서 링크 오류는 없다.

## 6. 다음 절편 관문

다음 절편은 남은 read-only inventory에서 `inspect hidden-text`·`inspect unicode` 같은 보안
조회 계열의 전용 helper, module 크기와 CC를 다시 조사한다. 보안 query를 한 파일에 무리하게
묶거나 CC>25 handler를 단순 이동하지 않고, 계약이 충분한 최소 경계를 최신 `upstream/devel`에서
다시 선정한다.

다음 절편은 메인테이너 승인 전 시작하지 않으며 remote push도 별도 승인 전 수행하지 않는다.
