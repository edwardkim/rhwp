# #5511 Stage 2 일곱 번째 수직 절편 — HWP5 raw record 진단 query 물리 분리

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 통합 기준선: `9d352d56d37a1dbd305b209ff660a0f25557e14b`
- characterization 커밋: `67b530973`
- 구현 커밋: `a05b88771`
- 수행일: 2026-08-19
- 상태: 완료 — 다음 Stage 2 절편 승인 대기

## 1. 절편 선정과 선행 계약 고정

일곱 번째 이동 대상으로 `dump-records`를 선택했다. 이 명령은 HWP5 CFB의 첫 BodyText
section을 제한된 크기로 읽고 raw record 목록과 일부 payload hex를 출력하는 read-only
diagnostic이다. 공개 catalog 명령이지만 이동 전 테스트는 인자 누락, 읽기 실패와 일반 HWP5
성공만 직접 보호했다.

이동 전에 다음 characterization을 별도 커밋으로 추가했다.

- HWP3처럼 CFB가 아닌 입력은 stdout 없이 runtime failure로 끝난다.
- 실제 EncryptVersion 4 HWP5 fixture는 비밀번호 없음 2, 불일치 1, 일치 0의 exit contract와
  stdout/stderr 분리를 유지한다.
- 첫 위치 인자 뒤의 여분 값과 `--json`을 무시하는 현행 동작을 일반 출력과 직접 비교한다.

마지막 항목은 현행 UX를 승인한 것이 아니다. move-only 동등성을 증명하고 엄격한 인자 검증을
별도 동작 변경으로 분리하기 위한 기준선이다. characterization 추가 후 해당 모듈의 focused
test는 14개에서 17개로 늘었고 모두 통과했다.

절편 시작·종료 시 활성 PR 중 `src/main.rs`, diagnostics 모듈, 해당 exit-code 및 catalog
계약 파일과 겹치는 변경은 없었다.

## 2. 구현 결과

- `src/cli/queries/diagnostics.rs`가 stream 상한 상수와 `dump_raw_records` 전체를 소유한다.
- `src/main.rs`의 최상위 match는 diagnostics 모듈 API만 호출한다.
- 공개 함수 표식을 정규화한 기계 비교에서 상수·주석·handler 본문이 이동 전과 일치했다.
- CFB 열기, FileHeader 해석, 압축·암호 분기, record 읽기와 출력 형식은 변경하지 않았다.
- 전역 인증 pre-scan이 설정하는 기존 `cli_password` seam을 그대로 사용한다.
- `cli_catalog_contract`가 handler 구현 및 dispatch 소유권을 고정한다.

parser와 암호 구현을 service로 숨기지는 않았다. 이번 절편은 command/query의 물리적 소유권만
복원하며 의존 방향 전환은 Stage 3에 남겼다.

## 3. 지표 변화

| 항목 | Stage 2 절편 6 통합 후 | Stage 2 절편 7 | 변화 |
|---|---:|---:|---:|
| `src/main.rs` 줄 수 | 40,828 | 40,676 | -152 |
| `src/cli/queries/diagnostics.rs` | 944 | 1,096 | +152, 모듈 상한 1,200 이하 |
| `main.rs` 최상위 함수 | 332 | 331 | handler 1개 이동 |
| 누적 이동 read-only handler | 10 | 11 | 1개 추가 |
| CLI CC>25 함수 | 19 | 19 | 변화 없음 |
| CLI 최대 CC | 68 | 68 | `dump_controls`, 변화 없음 |
| `wasm_api::HwpDocument` 직접 참조 | 42 | 42 | 변화 없음 |
| `rhwp::model` 직접 참조 | 63 | 63 | 변화 없음 |
| `rhwp::parser` 직접 참조 | 95 | 89 | parser·crypto 참조 6개 이동 |
| `rhwp::renderer` 직접 참조 | 24 | 24 | 변화 없음 |
| `rhwp::service` 참조 | 0 | 0 | Stage 3 대상 |

`dump_raw_records`는 CC 25 이하라 복잡도 경고 수치는 변하지 않았다. diagnostics 모듈은
1,096줄로 상한을 충족하지만 잔여 여유가 104줄뿐이다. 이후 handler를 편의상 이 파일에 더
쌓지 않고 새 응집 경계를 먼저 정해야 한다.

## 4. 외부 동작 동등성

여섯 번째 절편의 최신 `devel` 통합 바이너리와 이동 후 바이너리에 대해 다음 열 경로의 exit
code와 stdout/stderr SHA-256을 비교했다.

1. 일반 HWP5 성공
2. 여분 위치 인자
3. 현행 미선언 `--json` 무시
4. 필수 인자 누락
5. 존재하지 않는 파일
6. HWP3 비CFB 입력
7. 0-byte 입력
8. 암호 HWP5의 비밀번호 누락
9. 암호 HWP5의 잘못된 비밀번호
10. 암호 HWP5의 올바른 비밀번호

열 경로 모두 byte 단위로 일치했다. 일반 출력은 199,759 bytes,
SHA-256 `b8e899fb6b5535ad50d2e5b8704fab6b26c5c5a7ee2403ac4f0ad765e6b97701`, 암호 문서
정상 출력은 1,432,206 bytes,
SHA-256 `b764aefa8d8f3818d441e9e374c80549847c0a89155df704bc6c3795507a0e21`다. 오류 경로는
stdout이 비어 있고 한글 문구·exit 1/2·stderr hash가 모두 동일했다.

## 5. 검증 기록

| 검증 | 결과 |
|---|---|
| 이동 전 characterization focused nextest | 17/17 통과 |
| 이동 후 diagnostic focused nextest | 17/17 통과 |
| `cli_catalog_contract` | 11/11 통과 |
| 성공·호환·파싱·암호 출력 hash equivalence | 10/10 일치 |
| release-test 전체 nextest | 7,316/7,316 통과, 3 slow, 38 skipped, 161.135초 |
| `cargo fmt --all -- --check` | 통과 |
| `cargo clippy --all-targets --target-dir target/pr-review -- -D warnings` | 통과 |
| `cargo test --doc --target-dir target/pr-review` | 8/8 통과, 2 ignored |
| Rust test suite manifest | 통과, 717 sources / 3,274 static test attrs / 43 integration targets |
| Rust unit tier | 통과, 4,225 tests / 298 modules |
| `git diff --check` | 통과 |
| `check_markdown_links.py` | 기존 capability 등록부 무결성 오류 16건으로 실패, #5511 신규 오류 없음 |

characterization 추가로 generated harness 배정 weight가 달라져 `--prepare`로 로컬 harness를
재생성했다. 추적 파일 변경은 없었고 최종 manifest `--check`가 통과했다. 전체 nextest는 같은
717개 source를 모두 실행했으며 새 테스트 4건을 포함한다.

호스트 nextest는 `0.9.137`이고 저장소 권장은 `0.9.140`이라 경고가 있었지만 실행 결과에는
영향이 없었다. parser·crypto 로직은 바꾸지 않고 adapter 위치만 옮겼으므로 시각 검증과 WASM
빌드는 추가하지 않았다.

## 6. 다음 절편 관문

diagnostics 파일의 잔여 상한은 104줄뿐이므로 다음 절편은 기존 파일에 handler를 추가하는
방식으로 진행하지 않는다. `dump-pages`는 #5525 self-review가 지적한 help·capabilities·JSON·
사용자 문서 계약 드리프트가 해소되기 전 move-only 대상으로 선택하지 않는다.

다음 절편은 남은 query inventory에서 공유 helper와 service 의존을 다시 그린 뒤, 독립된 새
query 모듈을 만들 수 있는 기능 계열을 선택한다. `search_document`는 약 140줄이지만
`search_json_value`를 batch와 공유하므로 root로의 역참조를 만들지 않는 seam을 먼저 검토해야
한다. `dump_controls`는 약 1,269줄이고 CC 68이라 단순 파일 이동 대상이 아니다.

다음 절편은 메인테이너 승인 전 시작하지 않으며 remote push도 별도 승인 전 수행하지 않는다.
