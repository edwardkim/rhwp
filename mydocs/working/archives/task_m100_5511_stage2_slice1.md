# #5511 Stage 2 첫 수직 절편 — 문서 인벤토리 query 물리 분리

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 통합 기준선: `c3c35306b1428a2dcd97656d1cbe4a8c74c780a7`
- 구현 커밋: `3c14a5bc1`
- 수행일: 2026-08-18
- 상태: 완료 — 다음 Stage 2 절편 승인 대기

## 1. 절편 선정

첫 이동 대상으로 `word-count`, `bookmarks`, `charts`를 선택했다. 세 명령은 문서 전체의
수치 또는 포함 개체를 열거하는 read-only query이고, 서로 인접한 작은 handler이며 각각
독립 CLI 계약 테스트가 있다. 편집·저장 seam, MCP의 binary helper 역참조, renderer와
serializer를 건드리지 않으므로 Stage 2의 물리 모듈 경계를 검증하기에 가장 작은 단위다.

작업 전 열린 `devel` 대상 PR 53건의 파일을 확인했다. `src/main.rs`, `src/cli/`,
`tests/cli_catalog_contract.rs`와 겹치는 열린 PR은 없었다.

## 2. 구현 결과

- `src/cli/queries/document_inventory.rs`가 세 handler 본문을 소유한다.
- `src/main.rs`의 최상위 match는 query 모듈 API만 호출한다.
- `src/cli/queries/mod.rs`가 read-only CLI adapter 경계를 선언한다.
- catalog의 명령 metadata, help, capabilities, MCP 정의는 변경하지 않았다.
- `cli_catalog_contract`에 세 handler가 `main.rs`로 되돌아가지 않고 query 모듈에
  존재하는지 확인하는 소유권 계약을 추가했다.

새 모듈은 기존 `load_document`, `LoadError::report`, exit code 상수를 crate root에서
사용한다. 이것은 move-only 동등성을 위한 의도적 임시 seam이다. 이 절편에서 별도 shared
God module을 만들거나 service 계층 이행을 섞지 않았으며, 문서 열기와 typed error의
의존성 역전은 계획대로 Stage 3에서 처리한다.

## 3. 지표 변화

| 항목 | Stage 1 | Stage 2 절편 1 | 변화 |
|---|---:|---:|---:|
| `src/main.rs` 줄 수 | 42,260 | 42,052 | -208 |
| `src/cli/queries/document_inventory.rs` | 없음 | 220 | 신규, 모듈 상한 1,200 이하 |
| `main.rs` 최상위 함수 | 352 | 349 | handler 3개 이동 |
| 이동된 read-only handler | 0 | 3 | `word-count`, `bookmarks`, `charts` |
| CLI CC>25 함수 | 19 | 19 | 변화 없음 |
| CLI 최대 CC | 68 | 68 | `dump_controls`, 변화 없음 |
| `wasm_api::HwpDocument` 직접 참조 | 42 | 42 | Stage 3 대상 |
| `rhwp::service` 참조 | 0 | 0 | Stage 3 대상 |

Stage 1 보고서 시점의 `main.rs` 최상위 함수 수는 catalog helper 정리 결과를 현재 source와
같은 방식으로 다시 세어 352로 산정했다. 이번 절편에서 정확히 세 함수가 이동했다.

## 4. 외부 동작 동등성

Stage 1 release-test 바이너리와 이동 후 바이너리에 대해 각 명령의 다음 세 경로를 비교했다.

1. `--json` 성공 출력
2. 사람용 성공 출력
3. 알 수 없는 옵션의 exit code·stdout·stderr

세 명령의 9개 경로 모두 exit code와 stdout/stderr SHA-256이 일치했다. JSON envelope,
provenance mark, 사람용 한글 문구, exit 2와 silent stdout 계약에 byte 차이가 없다.

## 5. 검증 기록

| 검증 | 결과 |
|---|---|
| 이동 명령 focused nextest | 9/9 통과 |
| `cli_catalog_contract` | 5/5 통과 |
| `cli_json_contract` | 31/31 통과 |
| 성공·오류 출력 hash equivalence | 9/9 일치 |
| release-test 전체 nextest | 7,305/7,305 통과, 3 slow, 38 skipped, 167.635초 |
| `cargo fmt --all -- --check` | 통과 |
| `cargo clippy --all-targets --target-dir target/pr-review -- -D warnings` | 통과 |
| `cargo test --doc --target-dir target/pr-review` | 8/8 통과, 2 ignored |
| Rust test suite manifest | 통과, 716 sources / 43 integration targets |
| Rust unit tier | 통과, 4,225 tests / 298 modules |
| `check_markdown_links.py` | 기존 capability 등록부 무결성 오류 16건으로 실패, #5511 신규 오류 없음 |

호스트 nextest는 `0.9.137`이고 저장소 권장은 `0.9.140`이라 경고가 있었지만 실행 결과에는
영향이 없었다. 렌더러·serializer·WASM 동작을 변경하지 않아 계획서의 범위별 게이트에 따라
시각 검증과 WASM 빌드는 이 절편에 추가하지 않았다.

## 6. 다음 절편 관문

다음 Stage 2 후보는 `form-value`, `header-footer`, `headers-footers`의 구조화 개체 조회
계열이다. 세 명령도 read-only이며 현재 root load seam을 공유하므로 첫 절편에서 확정한
모듈 API와 검증 절차를 반복 적용할 수 있다.

`search`, `extract-data`, `info`, `structure`, `tables`, `fields`는 `mcp_serve`가
binary-local JSON helper를 역참조한다. 이 계열은 단순 이동으로 순환 의존을 만들 수 있으므로
다음 절편에 섞지 않고 Stage 3 service 경계 설계와 함께 다룬다.

다음 절편은 메인테이너 승인 전 시작하지 않는다. remote push도 별도 승인 전 수행하지
않는다.
