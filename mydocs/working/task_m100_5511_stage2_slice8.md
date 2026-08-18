# #5511 Stage 2 여덟 번째 수직 절편 — 주소 기반 문서 검색 query 물리 분리

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 통합 기준선: `9d352d56d37a1dbd305b209ff660a0f25557e14b`
- 구현 커밋: `e5a349bbb`
- 수행일: 2026-08-19
- 상태: 완료 — 다음 Stage 2 절편 승인 대기

## 1. 절편 선정과 경계 판단

여덟 번째 이동 대상으로 `search`를 선택했다. 이 명령은 문서의 텍스트를 찾되 평문만 반환하지
않고 구역·문단·문자 offset·조판 페이지 주소를 보존하는 read-only query다. JSON·사람용
출력, 표와 글상자 내부 검색, 0건, 대소문자, 결과 상한, 문단 context와 `-`로 시작하는
검색어까지 기존 계약 테스트가 보호하고 있어 Stage 2의 move-only 절편으로 적합했다.

이전 절편에서 `src/cli/queries/diagnostics.rs`가 1,096줄에 도달해 1,200줄 상한의 여유가
104줄뿐이므로, 검색을 diagnostics에 더하지 않고 응집된 새 `queries/search.rs`를 만들었다.
절편 시작·종료 시 활성 PR 중 `src/main.rs`, `src/cli/queries/`,
`tests/cli_catalog_contract.rs`와 이 작업 보고서에 겹치는 변경은 없었다.

## 2. 구현 결과와 보호 불변식

- `src/cli/queries/search.rs`가 `search_document` 전체를 소유한다.
- `src/main.rs`의 최상위 match는 search query 모듈 API만 호출한다.
- 공개 함수 표식만 정규화한 기계 비교에서 주석과 handler 본문이 이동 전과 일치했다.
- 옵션 파싱, exit code, stdout/stderr 분리, 검색 결과 순서와 출력 schema를 바꾸지 않았다.
- `cli_catalog_contract`가 handler 구현 및 dispatch 소유권을 고정한다.

`search_json_value`는 CLI search뿐 아니라 batch와 `src/mcp_serve.rs`도 소비한다. 이를 search
모듈로 옮기면 MCP가 CLI에 역의존하므로 Stage 2에서는 crate root에 보존했다. 공유 envelope를
application/service 경계로 내리는 작업은 Stage 3의 의존 방향 전환에서 다룬다.

## 3. 지표 변화

| 항목 | Stage 2 절편 7 | Stage 2 절편 8 | 변화 |
|---|---:|---:|---:|
| `src/main.rs` 줄 수 | 40,676 | 40,535 | -141 |
| `src/cli/queries/search.rs` | 없음 | 149 | 새 query 경계 |
| `src/cli/queries/diagnostics.rs` | 1,096 | 1,096 | 변화 없음 |
| `main.rs` 최상위 함수 | 331 | 330 | handler 1개 이동 |
| 누적 이동 read-only handler | 11 | 12 | 1개 추가 |
| `main.rs` CC>25 함수 | 19 | 19 | 변화 없음 |
| `main.rs` 최대 CC | 68 | 68 | `dump_controls`, 변화 없음 |
| search 모듈 CC>25 함수 | 없음 | 0 | 새 경계 내 초과 없음 |
| `wasm_api::HwpDocument` 직접 참조 | 42 | 42 | 변화 없음 |
| `rhwp::model` 직접 참조 | 63 | 63 | 변화 없음 |
| `rhwp::parser` 직접 참조 | 89 | 89 | 변화 없음 |
| `rhwp::renderer` 직접 참조 | 24 | 24 | 변화 없음 |
| `rhwp::service` 참조 | 0 | 0 | Stage 3 대상 |

`search_document`는 CC 25 이하라 복잡도 경고 수치도 변하지 않았다. 이번 절편은 root의 공유
helper를 새 모듈이 호출하는 binary-local seam을 명시적으로 남겼으며 service 분해를 가장한
추가 동작 변경은 하지 않았다.

## 4. 외부 동작 동등성

일곱 번째 절편 통합 바이너리와 이동 후 바이너리에 대해 다음 열 경로의 exit code와
stdout/stderr SHA-256을 비교했다.

1. 일반 사람용 검색
2. JSON 검색
3. 대소문자 무시 JSON 검색
4. `--limit 1`
5. `--context 1`
6. 검색 결과 0건
7. 알 수 없는 옵션
8. 필수 검색어 누락
9. 존재하지 않는 파일
10. `--` 뒤의 하이픈 시작 검색어

열 경로 모두 byte 단위로 일치했다. 대표적으로 일반 출력은 53,354 bytes,
SHA-256 `baecfd4f23a8f081f6945ae002fd2dd8e6466f4803261aa83366e5c92490af6b`, JSON
출력은 299,328 bytes,
SHA-256 `1fb9e76c7a711fa8fc5ffcb3b3014ef27a7cf1264e8ca90519ca587f5d1831b0`다. 검색
0건도 exit 0과 253-byte JSON을 유지했고, 오류 경로는 stdout이 비어 있으며 기존 exit
1/2와 stderr hash가 일치했다.

## 5. 검증 기록

| 검증 | 결과 |
|---|---|
| 이동 전·후 search focused nextest | 각각 17/17 통과 |
| `cli_catalog_contract` | 12/12 통과 |
| 정상·옵션·0건·오류 출력 hash equivalence | 10/10 일치 |
| release-test 전체 nextest | 7,317/7,317 통과, 3 slow, 38 skipped, 167.186초 |
| `cargo fmt --all -- --check` | 통과 |
| `cargo clippy --all-targets --target-dir target/pr-review -- -D warnings` | 통과 |
| `cargo test --doc --target-dir target/pr-review` | 8/8 통과, 2 ignored |
| Rust test suite manifest | 통과, 717 sources / 3,275 static test attrs / 43 integration targets |
| Rust unit tier | 통과, 4,225 tests / 298 modules |
| `git diff --check` | 통과 |
| `check_markdown_links.py --changed-from upstream/devel` | 기존 capability 등록부 무결성 오류 16건, #5511 신규 오류 없음 |

호스트 nextest는 `0.9.137`이고 저장소 권장은 `0.9.140`이라 경고가 있었지만 실행 결과에는
영향이 없었다. parser·검색·조판 로직은 바꾸지 않고 adapter 위치만 옮겼으므로 시각 검증과
WASM 빌드는 추가하지 않았다.

Markdown 검사는 기준선에도 존재하는 `agent_capability_registry.md`의 중복 ID·진입점 링크
무결성 오류 16건 때문에 전체 exit는 실패했다. 이번 절편이 추가한 보고서 링크 오류는 없다.

## 6. 다음 절편 관문

다음 절편은 남은 query inventory와 공유 helper 소비자를 최신 `upstream/devel`에서 다시
계측한 뒤 독립 모듈로 옮길 수 있는 가장 작은 read-only 계열을 선정한다. `dump-pages`는
#5525에서 확인된 help·capabilities·JSON·사용자 문서 계약 드리프트가 먼저 해소되어야 하며,
`dump_controls`는 약 1,269줄과 CC 68을 가진 분해 대상이라 move-only 절편으로 취급하지 않는다.

Stage 2의 다음 이동에서도 공유 helper를 CLI 하위로 끌어내려 MCP·batch의 역의존을 만드는
선택은 피한다. 다음 절편은 메인테이너 승인 전 시작하지 않으며 remote push도 별도 승인 전
수행하지 않는다.
