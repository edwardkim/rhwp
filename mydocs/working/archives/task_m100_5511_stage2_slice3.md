# #5511 Stage 2 세 번째 수직 절편 — 각주·미주 모양 진단 query 물리 분리

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 통합 기준선: `c3c35306b1428a2dcd97656d1cbe4a8c74c780a7`
- 구현 커밋: `3b000ec5b`
- 수행일: 2026-08-18
- 상태: 완료 — 다음 Stage 2 절편 승인 대기

## 1. 절편 선정

세 번째 이동 대상으로 `dump-note-shape`를 선택했다. 앞 절편의 다음 후보였던
`dump-note-shape`, `dump-endnote-lines`, `dump-extents`를 다시 조사한 결과 세 명령을 한 번에
옮기면 독립 절편의 범위를 넘는다.

- `dump-note-shape`는 전용 JSON helper 세 개만 소유한다.
- `dump-endnote-lines`는 문단 line trace와 여러 formatter, 공용 `brief_text`·`control_kind`
  seam에 연결된다.
- `dump-extents`는 renderer tree type, 중첩 walker와 gap 분석을 한 handler 안에 포함한다.

따라서 가장 작은 query 하나로 새 diagnostics 모듈의 경계를 먼저 검증했다. 활성 PR #5525의
`src/main.rs` 변경은 계속 `dump_pages`의 `--compat 2022|2024` 처리에만 한정되므로 이번
dispatch와 handler 구간에는 겹치지 않는다.

## 2. 구현 결과

- `src/cli/queries/diagnostics.rs`가 `dump_note_shape`와 전용 JSON helper 세 개를 소유한다.
- `src/main.rs`의 최상위 match는 diagnostics 모듈 API만 호출한다.
- `src/cli/queries/mod.rs`가 read-only diagnostic adapter 경계를 선언한다.
- catalog의 명령 metadata, help, capabilities와 출력 schema는 변경하지 않았다.
- `cli_catalog_contract`가 handler 구현 및 dispatch 소유권을 고정한다.

새 모듈은 기존 `load_document`, `LoadError::report`, exit code 및 `hu_to_mm_i`를 crate root에서
사용한다. `hu_to_mm_i`는 다른 진단 명령 12곳도 사용하는 공용 함수이므로 이번 절편에서
복제하거나 성급히 이동하지 않았다. 이 binary-local seam의 의존성 역전은 Stage 3에서
처리한다.

## 3. 지표 변화

| 항목 | Stage 2 절편 2 | Stage 2 절편 3 | 변화 |
|---|---:|---:|---:|
| `src/main.rs` 줄 수 | 41,737 | 41,648 | -89 |
| `src/cli/queries/diagnostics.rs` | 없음 | 101 | 신규, 모듈 상한 1,200 이하 |
| `main.rs` 최상위 함수 | 346 | 342 | handler 1개·전용 helper 3개 이동 |
| 누적 이동 read-only handler | 6 | 7 | 1개 추가 |
| CLI CC>25 함수 | 19 | 19 | 변화 없음 |
| CLI 최대 CC | 68 | 68 | `dump_controls`, 변화 없음 |
| `wasm_api::HwpDocument` 직접 참조 | 42 | 42 | Stage 3 대상 |
| `rhwp::model` 직접 참조 | 71 | 70 | `FootnoteShape` 참조 이동 |
| `rhwp::service` 참조 | 0 | 0 | Stage 3 대상 |

이 handler와 helper는 모두 CC 25 이하이므로 복잡도 경고 수치는 변하지 않았다. 새 모듈은
101줄로 파일별 1,200줄 상한을 충족한다.

## 4. 외부 동작 동등성

두 번째 절편 완료 시점의 release-test 바이너리와 이동 후 바이너리에 대해 다음 세 경로의
exit code와 stdout/stderr SHA-256을 비교했다.

1. `samples/field-01.hwp` 성공 출력
2. 입력 인자 누락의 사용법 오류
3. 존재하지 않는 파일의 런타임 오류

세 경로 모두 byte 단위로 일치했다. pretty JSON의 필드·공백·단위 변환, exit 2 사용법
문구, exit 1 파일 오류와 silent stdout 계약에 변화가 없다.

## 5. 검증 기록

| 검증 | 결과 |
|---|---|
| diagnostic focused nextest | 17/17 통과 |
| `cli_catalog_contract` | 7/7 통과 |
| 성공·사용법·런타임 출력 hash equivalence | 3/3 일치 |
| release-test 전체 nextest | 7,307/7,307 통과, 3 slow, 38 skipped, 158.245초 |
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

다음 Stage 2 후보는 `dump-endnote-lines`와 그 전용 line-trace helper 묶음이다. 공용
`brief_text`·`control_kind`를 root seam으로 유지할지, 진단 모듈 내부의 더 작은 공용 경계로
옮길지는 다음 절편 시작 시 참조 지점을 다시 계측해 결정한다. `dump-extents`는 renderer tree
의존성과 함수 크기가 더 크므로 이 후보와 섞지 않는다.

활성 PR #5525가 먼저 병합되면 push 전 최신 `upstream/devel` 기준으로 충돌과 전체 동작을
다시 검증한다. 다음 절편은 메인테이너 승인 전 시작하지 않으며 remote push도 별도 승인 전
수행하지 않는다.
