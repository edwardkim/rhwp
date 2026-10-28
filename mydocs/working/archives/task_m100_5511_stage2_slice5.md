# #5511 Stage 2 다섯 번째 수직 절편 — renderer extent 진단 query 물리 분리

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 통합 기준선: `c3c35306b1428a2dcd97656d1cbe4a8c74c780a7`
- 구현 커밋: `f87858bbe`
- 수행일: 2026-08-19
- 상태: 완료 — 다음 Stage 2 절편 승인 대기

## 1. 절편 선정

다섯 번째 이동 대상으로 `dump-extents`를 선택했다. 이 명령은 renderer tree의 실제 bbox를
순회해 쪽 위·아래 경계 이탈과 콘텐츠 사이 빈 구간을 출력하는 read-only diagnostic이다.
`describe`, `walk`, `any_text`는 handler 내부 중첩 함수라 다른 명령과 공유하지 않으며 전체
본문을 하나의 응집된 절편으로 옮길 수 있다.

활성 PR #5525의 `src/main.rs` 변경은 바로 다음 함수인 `dump_pages` 내부의
`--compat 2022|2024` 처리에만 한정된다. `dump-extents` 본문과 dispatch에는 겹치지 않고,
절편 종료 시점에도 PR은 open·mergeable clean이다.

## 2. 구현 결과

- `src/cli/queries/diagnostics.rs`가 `dump_extents` 전체 본문과 중첩 helper를 소유한다.
- `src/main.rs`의 최상위 match는 diagnostics 모듈 API만 호출한다.
- 이동 본문은 공개 함수 표식과 마지막 빈 줄을 제외하고 원본과 기계적으로 일치한다.
- catalog의 exceptional visibility, help, 출력 형식과 옵션 의미는 변경하지 않았다.
- `cli_catalog_contract`가 handler 구현 및 dispatch 소유권을 고정한다.
- 기존 exit-code 계약에 `dump-extents`의 인자 누락, 파일 오류, 범위 초과, 숫자 파싱과
  성공 경로를 편입했다.

새 모듈은 기존 `load_document`, `LoadError::report`, exit code를 crate root에서 계속
사용한다. renderer tree 조회도 기존 `HwpDocument` adapter를 그대로 거치며 service 경계
이행은 Stage 3에서 처리한다.

## 3. 지표 변화

| 항목 | Stage 2 절편 4 | Stage 2 절편 5 | 변화 |
|---|---:|---:|---:|
| `src/main.rs` 줄 수 | 41,230 | 40,936 | -294 |
| `src/cli/queries/diagnostics.rs` | 521 | 815 | +294, 모듈 상한 1,200 이하 |
| `main.rs` 최상위 함수 | 334 | 333 | handler 1개 이동 |
| 누적 이동 read-only handler | 8 | 9 | 1개 추가 |
| CLI CC>25 함수 | 19 | 19 | 변화 없음 |
| CLI 최대 CC | 68 | 68 | `dump_controls`, 변화 없음 |
| `wasm_api::HwpDocument` 직접 참조 | 42 | 42 | Stage 3 대상 |
| `rhwp::model` 직접 참조 | 64 | 64 | 변화 없음 |
| `rhwp::renderer` 직접 참조 | 25 | 24 | render tree type 참조 이동 |
| `rhwp::service` 참조 | 0 | 0 | Stage 3 대상 |

`dump_extents`는 중첩 순회가 있지만 CC 25 이하라 복잡도 경고 수치는 변하지 않았다.
diagnostics 모듈은 815줄로 파일별 1,200줄 상한을 충족한다.

## 4. 외부 동작 동등성

네 번째 절편 완료 시점의 release-test 바이너리와 이동 후 바이너리에 대해 다음 여덟 경로의
exit code와 stdout/stderr SHA-256을 비교했다.

1. `samples/hwp3-sample.hwp` 0쪽의 일반 extent 출력
2. 같은 쪽의 `--gaps` 출력
3. 같은 쪽의 `--outside` 출력
4. 잘못된 페이지 숫자
5. 잘못된 `--min-h` 숫자
6. 범위를 벗어난 페이지
7. 존재하지 않는 파일
8. 필수 인자 누락

여덟 경로 모두 byte 단위로 일치했다. bbox·gap·경계 이탈 출력, 진행 문구, exit 0/1/2,
한글 오류 문구와 stdout/stderr 배치에 변화가 없다.

## 5. 검증 기록

| 검증 | 결과 |
|---|---|
| diagnostic focused nextest | 14/14 통과 |
| `cli_catalog_contract` | 9/9 통과 |
| 일반·gaps·outside·오류 출력 hash equivalence | 8/8 일치 |
| release-test 전체 nextest | 7,310/7,310 통과, 3 slow, 38 skipped, 157.531초 |
| `cargo fmt --all -- --check` | 통과 |
| `cargo clippy --all-targets --target-dir target/pr-review -- -D warnings` | 통과 |
| `cargo test --doc --target-dir target/pr-review` | 8/8 통과, 2 ignored |
| Rust test suite manifest | 통과, 716 sources / 3,268 static test attrs / 43 integration targets |
| Rust unit tier | 통과, 4,225 tests / 298 modules |
| `check_markdown_links.py` | 기존 capability 등록부 무결성 오류 16건으로 실패, #5511 신규 오류 없음 |

case 파일은 독립 Cargo test target이 아니라 generated regression suite에 편입되므로 focused
검증은 `cargo test --test <case>`가 아니라 nextest expression으로 실행했다. manifest
`--prepare`는 추적 파일을 변경하지 않았고 최종 `--check`가 통과했다.

호스트 nextest는 `0.9.137`이고 저장소 권장은 `0.9.140`이라 경고가 있었지만 실행 결과에는
영향이 없었다. renderer 계산 로직은 바꾸지 않고 diagnostic adapter의 위치만 옮겼으므로
시각 검증과 WASM 빌드는 추가하지 않았다.

## 6. 다음 절편 관문

`dump-pages`는 더 작은 query지만 활성 PR #5525가 같은 함수의 계약을 변경하므로 지금
이동하지 않는다. 다음 Stage 2 후보는 self-contained read-only 진단인 `diag`다. DocInfo와
문단 head-type 요약 의존성을 다시 계측하고 출력 기준선을 고정한 뒤 최소 절편으로 판단한다.

PR #5525가 먼저 병합되면 최신 `upstream/devel` 기준으로 기존 절편과 인접 dispatch를 다시
검증한다. 다음 절편은 메인테이너 승인 전 시작하지 않으며 remote push도 별도 승인 전
수행하지 않는다.
