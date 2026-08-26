# #5511 Stage 2 열 번째 수직 절편 — 행정 데이터 추출 query 물리 분리

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 통합 기준선: `1a6ce79fd56e3cdf5813c7938338fcb5b7d0a859`
- 구현 커밋: `8ac67dd9669ad33941d605b72daf9f88320fae03`
- 수행일: 2026-08-19
- 상태: 완료 — 다음 Stage 2 절편 승인 대기

## 1. 절편 선정과 경계 판단

열 번째 이동 대상으로 `extract-data`를 선택했다. 이 명령은 행정문서의 날짜·금액·수량을
원문 주소와 함께 읽는 read-only query다. 단건·batch의 JSON schema, 종류 필터, 문서별
출력 상한, 사람용 출력, MCP 선언과 오류 exit code를 기존 계약이 보호하므로 move-only
절편으로 적합했다.

절편 시작 전에 `upstream/devel`이 16커밋 전진한 사실을 확인했다. #5511 변경과 직접 겹친
파일은 `Cargo.toml`뿐이었고, 원격은 verifier workspace member를, 이 브랜치는 기존 CLI
계약 target을 다뤄 hunk가 겹치지 않았다. 별도 브랜치나 worktree를 만들지 않고 현재 작업
브랜치를 최신 기준선에 직접 재배치했으며 충돌은 없었다. 절편 시작·종료 시 활성 PR 중
`src/main.rs`, `src/cli/queries/data_extraction.rs`, `src/cli/queries/mod.rs`, 단건·batch 계약,
CLI catalog 계약과 이 보고서에 겹치는 변경은 없었다.

## 2. 구현 결과와 보호 불변식

- 새 `src/cli/queries/data_extraction.rs`가 `extract_data_command` 전체를 소유한다.
- `src/main.rs`의 최상위 match는 data extraction query 모듈 API만 호출한다.
- 공개 함수 표식만 정규화한 기계 비교에서 handler 본문이 이동 전과 일치했다.
- `cli_catalog_contract`가 구현 위치, root 재유입 금지, dispatch와 인자 전달을 고정한다.
- 옵션 파싱, 출력 순서, JSON schema, `--kind`, `--limit`, exit code와 stdout/stderr 분리를
  바꾸지 않았다.

`extract_data_json_value`는 단건 CLI뿐 아니라 `batch_extract_data_record_inner`도 소비한다.
이를 CLI 하위로 옮기면 batch가 CLI adapter에 역의존하므로 crate root에 보존했다. 공유
추출 service와 envelope를 application/service 경계로 내리는 일은 Stage 3에서 다룬다.

## 3. 지표 변화

| 항목 | Stage 2 절편 9 | Stage 2 절편 10 | 변화 |
|---|---:|---:|---:|
| `src/main.rs` 줄 수 | 40,468 | 40,319 | -149 |
| `src/cli/queries/data_extraction.rs` | 없음 | 156 | 신규, 모듈 상한 이하 |
| `main.rs` 최상위 함수 | 329 | 328 | handler 1개 이동 |
| 누적 이동 read-only handler | 13 | 14 | 1개 추가 |
| `main.rs` CC>25 함수 | 19 | 19 | 변화 없음 |
| `main.rs` 최대 CC | 68 | 68 | 변화 없음 |
| data extraction 모듈 CC>25 함수 | 없음 | 0 | 새 경계 내 초과 없음 |
| `wasm_api::HwpDocument` 직접 참조 | 42 | 42 | 변화 없음 |
| `rhwp::model` 직접 참조 | 63 | 63 | 변화 없음 |
| `rhwp::parser` 직접 참조 | 89 | 89 | 변화 없음 |
| `rhwp::renderer` 직접 참조 | 24 | 24 | 변화 없음 |
| `rhwp::service` 참조 | 0 | 0 | Stage 3 대상 |

`extract_data_command`는 CC 25 이하라 복잡도 경고 수치가 변하지 않았다. 이번 절편도
공용 helper를 root에 둔 binary-local seam을 명시적으로 유지하며 service 이행과 동작
변경을 섞지 않았다.

## 4. 외부 동작 동등성

아홉 번째 절편 통합 바이너리와 이동 후 바이너리에 대해 다음 열두 경로의 exit code와
stdout/stderr SHA-256을 비교했다.

1. HWPX 사람용 출력
2. HWPX JSON 출력
3. 날짜 종류 필터
4. HWP 금액 종류 필터
5. 문서별 1건 출력 상한
6. HWP3 입력
7. 존재하지 않는 파일
8. 필수 경로 누락
9. 잘못된 종류
10. 위치 인자 초과
11. 0인 출력 상한
12. 알 수 없는 옵션

열두 경로 모두 byte 단위로 일치했다. 대표적으로 사람용 출력은 245 bytes,
SHA-256 `71f08686b65524389972396b4b2d0ca1e11147ae00a2cb6ca7179e5c9ea3fd76`, JSON
출력은 541 bytes,
SHA-256 `9e67cf0dff7edfdbe28ee935c60b96ae2983e01dca4aa8dbdc7e33374be8de4b`다. 금액
필터의 10,943-byte JSON과 `--limit 1`의 386-byte JSON도 각각
`38bd820276fe0808598469cb8d7ab5b977e5430fb0ddc089e2013dc5abb56894`,
`c712cc2ef1330d4046d7210fe86423fd4879715abd144faf9b02208419a13241`로 일치했다.
오류 경로는 stdout이 비어 있으며 기존 exit 1/2와 stderr hash를 그대로 유지했다.

## 5. 검증 기록

| 검증 | 결과 |
|---|---|
| 이동 전·후 단건·batch focused nextest | 각각 26/26 통과 |
| `cli_catalog_contract` | 13/13 통과 |
| 정상·필터·상한·오류 출력 hash equivalence | 12/12 일치 |
| release-test 전체 nextest | 7,743/7,743 통과, 3 slow, 38 skipped, 163.051초 |
| `cargo check --all-targets` | 통과 |
| `cargo fmt --all -- --check` | 통과 |
| `cargo clippy --all-targets --target-dir target/pr-review -- -D warnings` | 통과 |
| `cargo test --doc --target-dir target/pr-review` | 8/8 통과, 3 ignored |
| Rust test suite manifest | 통과, 750 sources / 3,694 static test attrs / 43 integration targets |
| Rust unit tier | 통과, 4,225 tests / 298 modules |
| test policy 자체 계약 | integration 16/16, unit tier 12/12 통과 |
| `scripts/tests/test_ci_impact_workflow.py` | 30/30 통과 |
| `git diff --check` | 통과 |
| `check_markdown_links.py --changed-from upstream/devel` | 기존 capability 등록부 무결성 오류 16건, #5511 신규 오류 없음 |

호스트 nextest는 `0.9.137`이고 저장소 권장은 `0.9.140`이라 경고가 있었지만 실행 결과에는
영향이 없었다. Cargo가 최신 원격에서 이름이 인접한 두 verifier package의 lockfile 순서만
재정렬했으나 #5511과 무관한 파생 변화이므로 추적 변경에서 복원했다. parser·추출 규칙·조판
로직은 바꾸지 않고 adapter 위치만 옮겼으므로 시각 검증과 WASM 빌드는 추가하지 않았다.

Markdown 검사는 기준선에도 존재하는 `agent_capability_registry.md`의 중복 ID·진입점 링크
무결성 오류 16건을 보고했다. 이번 절편이 추가한 보고서 링크 오류는 없다.

## 6. 다음 절편 관문

다음 절편은 최신 `upstream/devel`에서 남은 read-only query를 다시 조사해 공용 helper 소비
방향과 기존 계약 밀도가 명확한 최소 단위를 선정한다. `dump-pages`는 #5525에서 확인된 계약
드리프트를 먼저 해소해야 하며 `dump_controls`는 CC 68이라 move-only 대상이 아니다.

다음 절편은 메인테이너 승인 전 시작하지 않으며 remote push도 별도 승인 전 수행하지 않는다.
