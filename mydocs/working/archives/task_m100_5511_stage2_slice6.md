# #5511 Stage 2 여섯 번째 수직 절편 — 문서 진단 query 물리 분리

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 구현 기준선: `c3c35306b1428a2dcd97656d1cbe4a8c74c780a7`
- 구현 커밋: `cb4b057ac`
- 수행일: 2026-08-19
- 상태: 완료 — 최신 `upstream/devel` 동기화·재검증 완료, 다음 Stage 2 절편 승인 대기

## 1. 절편 선정

여섯 번째 이동 대상으로 `diag`를 선택했다. 이 명령은 DocInfo의 번호·글머리표 수,
ParaShape의 head type 분포, 구역별 개요 번호와 해당 문단을 stdout에 요약하는 read-only
diagnostic이다. 전용 helper나 상태 변경이 없고, 기존 exit-code 및 알 수 없는 플래그 계약이
독립 테스트로 보호되어 전체 handler를 하나의 응집된 절편으로 옮길 수 있었다.

절편 시작 시 활성 PR #5525의 `src/main.rs` 변경은 인접한 `dump_pages` 내부의
`--compat 2022|2024` 처리에만 한정되어 `diag` 본문·dispatch와 겹치지 않았다. 전체 검증이
끝난 뒤 PR #5525가 `upstream/devel`에 병합되어 원격은 3커밋 전진했다. 최신 원격과 현재
branch의 3-way 통합 시뮬레이션은 충돌 없이 성공했으며 `diag` 소유권 변경과 겹치는 hunk는 없다.

## 2. 구현 결과

- `src/cli/queries/diagnostics.rs`가 `diag_document` 전체 본문을 소유한다.
- `src/main.rs`의 최상위 match는 diagnostics 모듈 API만 호출한다.
- 공개 함수 표식과 마지막 빈 줄을 정규화한 기계 비교에서 이동 전후 본문이 일치했다.
- 명령 이름, 옵션 해석, 출력 순서·문구, exit code와 stdout/stderr 배치는 변경하지 않았다.
- `cli_catalog_contract`가 handler 구현 및 dispatch 소유권을 고정한다.
- 기존 `cli_exit_codes_dump_diag`와 `diagnostics_flag_contract`가 성공·인자 누락·읽기 실패·
  파싱 실패·알 수 없는 플래그 경로를 보호한다.

새 모듈은 기존 `load_document`, `LoadError::report`, exit code를 crate root에서 계속 사용한다.
`HeadType` model 직접 참조는 handler와 함께 diagnostics 모듈로 이동했으며 service 경계 이행은
Stage 3의 범위로 보존했다.

## 3. 지표 변화

| 항목 | Stage 2 절편 5 | Stage 2 절편 6 | 변화 |
|---|---:|---:|---:|
| `src/main.rs` 줄 수 | 40,936 | 40,807 | -129 |
| `src/cli/queries/diagnostics.rs` | 815 | 944 | +129, 모듈 상한 1,200 이하 |
| `main.rs` 최상위 함수 | 333 | 332 | handler 1개 이동 |
| 누적 이동 read-only handler | 9 | 10 | 1개 추가 |
| CLI CC>25 함수 | 19 | 19 | 변화 없음 |
| CLI 최대 CC | 68 | 68 | `dump_controls`, 변화 없음 |
| `wasm_api::HwpDocument` 직접 참조 | 42 | 42 | Stage 3 대상 |
| `rhwp::model` 직접 참조 | 64 | 63 | `HeadType` 참조 이동 |
| `rhwp::renderer` 직접 참조 | 24 | 24 | 변화 없음 |
| `rhwp::service` 참조 | 0 | 0 | Stage 3 대상 |

`diag_document`는 CC 25 이하라 복잡도 경고 수치는 변하지 않았다. diagnostics 모듈은
944줄로 파일별 1,200줄 상한을 충족한다. 이 표는 구현 커밋의 기준선에서 측정한 값이며,
검증 중 전진한 원격 3커밋은 동기화 뒤 다음 절편의 새 기준으로 다시 계측한다.

## 4. 외부 동작 동등성

다섯 번째 절편 완료 시점의 release-test 바이너리와 이동 후 바이너리에 대해 다음 여섯 경로의
exit code와 stdout/stderr SHA-256을 비교했다.

1. `samples/hwp3-sample.hwp` 성공 출력
2. 기존 호환 동작인 여분 위치 인자 무시
3. 필수 인자 누락
4. 알 수 없는 `--json` 플래그
5. 존재하지 않는 파일
6. 파싱할 수 없는 `/dev/null`

여섯 경로 모두 byte 단위로 일치했다. 성공 출력, 한글 오류 문구, exit 0/1/2와
stdout/stderr 배치에 변화가 없다. 여분 위치 인자를 무시하는 현행 동작도 이번 move-only
절편에서는 의도적으로 보존했으며, 그 UX가 바람직하다고 새로 결정한 것은 아니다.

## 5. 검증 기록

| 검증 | 결과 |
|---|---|
| diagnostic focused nextest | 19/19 통과 |
| `cli_catalog_contract` | 10/10 통과 |
| 성공·호환·오류 출력 hash equivalence | 6/6 일치 |
| release-test 전체 nextest | 7,311/7,311 통과, 3 slow, 38 skipped, 157.885초 |
| `cargo fmt --all -- --check` | 통과 |
| `cargo clippy --all-targets --target-dir target/pr-review -- -D warnings` | 통과 |
| `cargo test --doc --target-dir target/pr-review` | 8/8 통과, 2 ignored |
| Rust test suite manifest | 통과, 716 sources / 3,269 static test attrs / 43 integration targets |
| Rust unit tier | 통과, 4,225 tests / 298 modules |
| `git diff --check` | 통과 |
| `check_markdown_links.py` | 기존 capability 등록부 무결성 오류 16건으로 실패, #5511 신규 오류 없음 |

호스트 nextest는 `0.9.137`이고 저장소 권장은 `0.9.140`이라 경고가 있었지만 실행 결과에는
영향이 없었다. parser·serializer·renderer 계산 로직은 바꾸지 않고 diagnostic adapter의
위치만 옮겼으므로 시각 검증과 WASM 빌드는 추가하지 않았다.

## 6. 최신 `devel` 동기화와 재검증

메인테이너 승인 뒤 `upstream/devel@9d352d56d37a1dbd305b209ff660a0f25557e14b`을 기존
절편 SHA 계보를 보존하는 merge 방식으로 반영했다. 통합 커밋은 `7a6b4902d`이며 충돌은 없었다.
통합 직후 branch는 0 behind / 19 ahead다.

통합 head에서 다음 게이트를 다시 실행했다.

| 통합 재검증 | 결과 |
|---|---|
| `diag` focused nextest | 19/19 통과 |
| `cli_catalog_contract` | 10/10 통과 |
| #5525 한글 2024 호환 focused test | 1/1 통과 |
| release-test 전체 nextest | 7,312/7,312 통과, 3 slow, 38 skipped, 165.905초 |
| clippy `-D warnings` | 통과 |
| doc-test | 8/8 통과, 2 ignored |
| fmt / diff check | 통과 |
| Rust test suite manifest | 717 sources / 3,270 static test attrs / 43 integration targets |
| Rust unit tier | 4,225 tests / 298 modules |

#5525의 새 case는 `rust-test-suite-manifest --prepare`로 로컬 생성 harness에 편입한 뒤 실행했다.
생성 harness는 추적 파일을 바꾸지 않았고 최종 `--check`가 통과했다. 통합 후 `main.rs`는
40,828줄이며 diagnostics 944줄, 최상위 함수 332개, `wasm_api::HwpDocument` 42회,
`rhwp::model` 63회, `rhwp::renderer` 24회, `rhwp::service` 0회다.

## 7. 다음 절편 관문

병합된 PR #5525는 `dump-pages --compat 2022|2024`를 추가했지만 해당 PR의 self-review에
help·capabilities·JSON·사용자 문서 계약 보정이 후속 과제로 기록되어 있다. 따라서
`dump-pages`는 이 드리프트를 먼저 해소하기 전 move-only 후보로 선택하지 않는다. 다음
Stage 2 후보는 상한 상수와 handler를 합쳐 약 154줄인 `dump-records`다. 이를 옮겨도
diagnostics 모듈은 약 1,100줄로 1,200줄 상한 이내다. 활성 PR 중 `src/main.rs`, diagnostics,
해당 exit-code·catalog 계약 파일과 겹치는 변경은 없다.

다만 현재 테스트는 인자 누락, 읽기 실패와 일반 HWP5 성공만 직접 보호한다. CFB 열기,
FileHeader 읽기·파싱, 지원하지 않는 암호 버전, 비밀번호 누락·불일치, record 파싱과 기존 여분
인자 처리의 출력·exit-code 기준선은 이동 전에 추가로 고정해야 한다. 따라서 일곱 번째 절편은
characterization과 hash 기준선 확보를 먼저 수행한 뒤 move-only를 진행하는 조건부 후보로 둔다.

다음 절편은 메인테이너 승인 전 시작하지 않으며 remote push도 별도 승인 전 수행하지 않는다.
