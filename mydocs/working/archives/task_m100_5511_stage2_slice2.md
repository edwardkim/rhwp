# #5511 Stage 2 두 번째 수직 절편 — 구조화 개체 query 물리 분리

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 통합 기준선: `c3c35306b1428a2dcd97656d1cbe4a8c74c780a7`
- 구현 커밋: `f1693afdb`
- 수행일: 2026-08-18
- 상태: 완료 — 다음 Stage 2 절편 승인 대기

## 1. 절편 선정

두 번째 이동 대상으로 `form-value`, `header-footer`, `headers-footers`를 선택했다. 세 명령은
양식 개체 또는 머리말·꼬리말을 조회하는 read-only query이고, 같은 문서 load seam과 JSON
envelope 패턴을 공유한다. 편집·저장 seam, renderer와 serializer를 건드리지 않으면서 첫
절편에서 확정한 물리 모듈 경계를 같은 방식으로 확장할 수 있는 단위다.

작업 중 열린 PR #5525가 `src/main.rs`를 변경하므로 파일 단위 충돌 가능성을 별도로
확인했다. 해당 PR의 변경은 `dump_pages`의 `--compat 2022|2024` 인자와 문서 설정에만
한정되고, 이번 절편의 dispatch 및 handler 본문과는 겹치지 않는다. 다만 PR이 먼저 병합되면
push 전 최신 `upstream/devel` 기준으로 다시 충돌과 동작을 검증한다.

## 2. 구현 결과

- `src/cli/queries/structured_objects.rs`가 세 handler 본문을 소유한다.
- `src/main.rs`의 최상위 match는 새 query 모듈 API만 호출한다.
- `src/cli/queries/mod.rs`가 새 read-only adapter 하위 모듈을 선언한다.
- catalog의 명령 metadata, help, capabilities, MCP 정의는 변경하지 않았다.
- `cli_catalog_contract`에 세 handler가 `main.rs`로 되돌아가지 않는 소유권 계약을
  추가했다. dispatch 검사는 rustfmt 줄바꿈과 후행 쉼표에 의존하지 않는다.

새 모듈은 첫 절편과 마찬가지로 기존 `load_document`, `LoadError::report`, exit code 상수를
crate root에서 사용한다. 이는 move-only 동등성을 위한 의도적 임시 seam이며 문서 열기와
typed error의 service 계층 이행은 계획대로 Stage 3에서 처리한다.

## 3. 지표 변화

| 항목 | Stage 2 절편 1 | Stage 2 절편 2 | 변화 |
|---|---:|---:|---:|
| `src/main.rs` 줄 수 | 42,052 | 41,737 | -315 |
| `src/cli/queries/structured_objects.rs` | 없음 | 331 | 신규, 모듈 상한 1,200 이하 |
| `main.rs` 최상위 함수 | 349 | 346 | handler 3개 이동 |
| 누적 이동 read-only handler | 3 | 6 | 3개 추가 |
| CLI CC>25 함수 | 19 | 19 | 변화 없음 |
| CLI 최대 CC | 68 | 68 | `dump_controls`, 변화 없음 |
| `wasm_api::HwpDocument` 직접 참조 | 42 | 42 | Stage 3 대상 |
| `rhwp::service` 참조 | 0 | 0 | Stage 3 대상 |

이번 handler는 모두 CC 25 이하이므로 물리 이동만으로 복잡도 경고 수치는 변하지 않았다.
새 모듈은 331줄로 파일별 1,200줄 상한을 충족한다.

## 4. 외부 동작 동등성

첫 절편 완료 시점의 release-test 바이너리와 이동 후 바이너리에 대해 각 명령의 다음 세
경로를 비교했다.

1. `--json` 성공 출력
2. 사람용 성공 출력
3. 알 수 없는 옵션의 exit code·stdout·stderr

세 명령의 9개 경로 모두 exit code와 stdout/stderr SHA-256이 일치했다. JSON envelope,
provenance mark, 사람용 한글 문구, exit 2와 silent stdout 계약에 byte 차이가 없다.

## 5. 검증 기록

| 검증 | 결과 |
|---|---|
| 이동 명령 focused nextest | 23/23 통과; 연관 set/insert/delete 계약 포함 |
| `cli_catalog_contract` | 6/6 통과 |
| `cli_json_contract` | 31/31 통과 |
| 성공·오류 출력 hash equivalence | 9/9 일치 |
| release-test 전체 nextest | 7,306/7,306 통과, 3 slow, 38 skipped, 158.274초 |
| `cargo fmt --all -- --check` | 통과 |
| `cargo clippy --all-targets --target-dir target/pr-review -- -D warnings` | 통과 |
| `cargo test --doc --target-dir target/pr-review` | 8/8 통과, 2 ignored |
| Rust test suite manifest | 통과, 716 sources / 43 integration targets |
| Rust unit tier | 통과, 4,225 tests / 298 modules |
| `check_markdown_links.py` | 기존 capability 등록부 무결성 오류 16건으로 실패, #5511 신규 오류 없음 |

focused selector는 명령 이름의 공통 부분 때문에 `set-form-value`와 머리말·꼬리말
삽입·삭제 계약까지 포함했다. 더 넓은 같은 기능군 23개가 모두 통과했으며, 전체 nextest로
다시 독립 검증했다.

호스트 nextest는 `0.9.137`이고 저장소 권장은 `0.9.140`이라 경고가 있었지만 실행 결과에는
영향이 없었다. 렌더러·serializer·WASM 동작을 변경하지 않아 계획서의 범위별 게이트에 따라
시각 검증과 WASM 빌드는 이 절편에 추가하지 않았다.

## 6. 다음 절편 관문

다음 Stage 2 후보는 서로 독립적인 diagnostic query 계열이다. 다만 활성 PR #5525가
`dump-pages`를 변경하므로 그 명령은 PR의 통합 여부가 확정될 때까지 다음 절편에서 제외한다.
`dump-note-shape`, `dump-endnote-lines`, `dump-extents`의 helper 의존성과 기존 계약을 먼저
대조한 뒤 충돌 없는 최소 수직 절편을 선정한다.

`search`, `extract-data`, `info`, `structure`, `tables`, `fields`는 `mcp_serve`가
binary-local JSON helper를 역참조한다. 단순 이동으로 순환 의존을 만들 수 있으므로 Stage 2
이동에 섞지 않고 Stage 3 service 경계 설계와 함께 다룬다.

다음 절편은 메인테이너 승인 전 시작하지 않는다. remote push도 별도 승인 전 수행하지
않는다.
