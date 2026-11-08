# #8174 v2-S01 — CLI·공통 자원 정책

- Issue: [#8174](https://github.com/edwardkim/rhwp/issues/8174).
- 승인: 2026-10-24 사용자 「v2 계획 승인·CLI부터 구현 진행」.
- 계획: [v2 구현 계획](../plans/task_m100_8174_impl.md#범위-v2--기본-기능과-환경별-자원-정책).
- 시작 source: `b4f834dbefd805485b5ab29359d5ed1856dc9021`.
- base: fetch로 재확인한 `upstream/devel` `3536e72819e165dbe959ee42cfd4e4cf8db33ce7`.
- 상태: V2-S01의 구현·로컬 검증 완료. v2 전체와 원격 push·PR 생성·게시 완료는 아니다.

## 선행 구현과 이번 변경 경계

| 선행 근거 | 현재 대조와 보존 계약 |
| --- | --- |
| #7911 → PR #7913 → `4662c069e557e928ed7968d061121fde90b1f027` | 공통 IR 직접 가져오기·CLI 쪽 선택·provenance 봉인·원자적 출력과 요청별 예산을 보존한다. 기존 Worker 검증은 Native 격리의 증거로 대체하지 않는다. |
| PR #7970 → `515bd72dcde0eecdeece333666b675354b7fa91d` | 기본 입력 64 MiB·선택 1,024쪽, source traversal과 선택 뒤 예산 축소를 유지한다. 기존 364쪽 시각 결과는 당시 source의 근거이며 새 구현의 결과로 표시하지 않는다. |
| PR #8120 → `80968497c2`, #8190 → `e7a0dec55c` | operand collection의 문서 단위 예산과 코덱 요청 예산·alpha 정확성을 유지한다. |

공통 정책의 생산 → CLI 입력 읽기 → Core의 명시 policy API → parser의 복사 전 검사 →
backend의 요청 예산 → 선택 뒤 축소 → 추출의 텍스트·픽셀 검사 → Core 조판·저장으로
소비 경로를 연결한다. 요청 카운터와 제품용 프로세스 감독의 시간·메모리 지표를 구분한다.
기존 기본 정책은 보존하고 작은 환경용 프로필·검증된 범위 안의 명시적 조정 옵션을 추가한다.

## 구현 결과

- `PdfImportProfile`의 `standard`는 기존 요청 예산을 유지한다. `constrained`와 strict JSON
  override는 한도를 낮출 수 있으며 입력을 열기 전에 객체 형태·중복·미지 키·0·음수·overflow를
  거부한다. 이전 고정 텍스트·그림 픽셀 한도도 같은 유효 policy에서 소비한다.
- `DocumentCore::from_pdf_bytes_with_policy`와 `parse_pdf_with_metadata_and_policy`로
  Native CLI·Rust 소비자가 같은 정책과 provenance 봉인을 사용한다. HWP5/HWPX의 기존
  읽기 한도는 변경하지 않으며 새 CLI 조정 표면은 PDF 전용이다.
- CLI는 요청마다 새 실행 프로그램을 만들고 입력 읽기·IR 생성·조판·HWP/HWPX 직렬화를
  그 수명 안에서 수행한다. 부모의 입력은 64 KiB 이하의 설정 IPC이며 PDF 바이트를 먼저
  읽거나 복제하지 않는다. 최종 파일은 부모가 응답 크기와 성공 상태를 확인한 뒤 교체한다.
- Linux에서는 AS/CPU 제한과 부모 종료 신호·`no_new_privs`를 적용하고 Unix에서는 private
  process group을 정리한다. nonblocking pipe와 종료 표지가 읽지 않은 stdin·상속된 출력 pipe의
  대기를 끝낸다. escaped session까지 종료하는 sandbox로 확대하지 않는다.
- 환경은 font·플랫폼용 목록만 전달하며 auth token·loader hook 등은 승계하지 않는다.
  이 목록 제한은 파일·네트워크 접근 차단이 아니다. 성공 봉투의 `execution`은 실행 한도·
  wall·관측 RSS·OS별 적용 여부·`filesystemNetworkSandbox:false`를 구분한다.
- 도움말·capabilities·MCP 선택 인자·capabilities schema·에이전트 필드 사전과
  [사용 가이드](../manual/pdf_resource_policies.md)를 함께 연결했다.

## 검증 경계

새 검사는 유효한 xref를 가진 4×2 RGB PDF의 입력·픽셀·정책 계약과 독립 Python 프로그램의
실제 OS 동작을 검사한다. 조판 좌표나 render golden을 추가하지 않으며, 정상 문서의 생성
결과는 아래의 변경 전후 HWPX 바이트 비교로 따로 확인한다. PDF 내용·배치·색 변환 규칙은
변경하지 않았다. 이 근거를 F01–F08의 새 시각 검증이나 fresh WASM 실행으로 바꾸어 보고하지 않는다.
현재 실행 OS는 Linux/WSL2다. 전체 parser 할당 증명과 비신뢰 업로드 지원 완료는 선언하지 않는다.
실행 감독은 `import-pdf`에 연결했다. `info`·`export-svg`·`convert` 등의 기존 공통 로더는
직접 Core를 사용하며, 이 절편의 프로세스 감독을 공유하지 않는다. MD05의 공식 지원 판정에서
해당 진입점과 직접 Rust API의 경계를 계속 추적한다.

## #7911의 남은 할당 경계 대조

| 항목 | 현재 호출 경로에서 확인한 사실 | 이번 감독과의 관계·남은 필수 작업 |
| --- | --- | --- |
| B01 | `FontInfo::from_dict`가 Type1/CFF encoding을 호출한다. Type1은 최대 64 KiB를 검색하지만 CFF INDEX·charset과 mapping의 `Vec`·`HashMap` 생성은 별도 요청 계상이 남아 있다. | 자원 한도가 있는 별도 CLI 프로세스로 메인 프로세스와 분리한다. 이 분리를 직접 Rust/WASM 경로의 할당 증명으로 대체하지 않으며 Native/WASM의 실제 예약·복구 검증을 v2에서 계속한다. |
| B02 | `embedded_glyph_name`의 문자열 배열과 `char_to_unicode`의 memo hit clone·miss insert가 남아 있다. memo는 FontInfo 사이에서 Arc로 공유한다. | CLI의 요청 수명을 넘겨 같은 작업자 메모리를 재사용하지 않는다. 문자열·memo 생성 전 계상과 sticky 실패·정상 매핑 검증은 여전히 필요하다. |
| B03 | 기존 제한 경로의 전역 cache 우회·TrueType map 공유는 유지된다. `load_fonts_impl`의 이름·fingerprint 정렬 Vec와 cache index·이름 복사는 별도 경계다. | 이미 해결한 전역 cache 격리를 다시 구현하지 않는다. 남은 deep copy와 공유를 분리해 감사하며 프로세스 AS 제한을 카운터 계상 완료로 기록하지 않는다. |
| B04 | frontend의 추출·인식·lowering·JSON 진단 수집은 기존 구조·텍스트·그림 예산과 별도로 allocator 비용이 남는다. | 이번에는 입력·픽셀·텍스트·backend 한도의 실제 전달과 직렬화/응답 크기를 검사한다. 전체 수집·cache의 할당 감사가 완료됐다는 주장은 하지 않는다. |
| B05 | zune worker의 수평 upsample `allocation.resize`는 요청 allocator를 호출한다. | 실제 capacity 증가 분기의 기존 미관측 상태는 유지한다. 독립적인 입력과 Native/WASM 실행 증거를 확보하기 전에는 완료로 바꾸지 않는다. |

이 표는 정적 경계 조사이며 각 항목에서 취약점을 새로 실행·재현했다는 뜻이 아니다.
MD05-B의 공식 지원과 v2 전체 완료를 위한 필수 추적을 유지한다.

## 검증 준비 중 확인한 문제

- 오래된 generated suite가 삭제된 source를 가리켜 첫 `cargo fmt --all`이 실패했다.
  source·기대값은 복원하지 않고 현재 source의 manifest를 다시 생성했다.
- 감독 코드의 private source를 `#[path]`로 포함한 첫 검사는 exception target 수가 49/48이 돼
  자동 수집 정책에 걸렸다. Native host가 실제 소비할 공통 `resource_supervisor` 진입점으로
  옮기고 공개 동작을 검사해 48/48을 유지했다. target 예산·CI 호출을 늘리지 않았다.
- 포맷 전 생성한 배정과 포맷 후 동적 runner의 배정이 달라 첫 집중 실행은 0건이었다.
  통과로 세지 않으며 최종 포맷 뒤 manifest 생성 → 실제 집중 실행 순서로 바로잡았다.
- 첫 workspace Clippy는 새 검사 한 곳의 `err().expect()`를 지적했다. `expect_err()`로
  보정했고 검사 조건이나 기존 baseline은 변경하지 않았다. 최종 결과는 아래 실행 로그로 확인한다.
- 실제 집중 실행의 첫 결과는 5/6 통과였다. Serde가 `[]`를 기본값의 struct로 읽어 잘못된
  JSON 설정이 수용됐다. 루트·policy·backend·execution의 객체 형태를 검사한 뒤 원문을 다시
  typed parser에 넘겨 배열을 거부하면서 중복 키 거부도 유지했다. 기대값을 바꾸지 않고
  같은 반례를 재실행해 정책 6개·OS 감독 7개가 통과했다.
- 후속 코드 검토에서 부모의 `Value` 재구성이 진단 JSON을 감독 경계 밖에서 확장하는 경로를
  확인했다. 진단은 검증한 `RawValue`를 빌려 전달하고, 작은 봉투 필드와 출처 표지만 부모가
  구성하도록 보강했다. 큰 AST 재생성을 없애며 전체 부모 RSS 보증으로 확대하지 않는다.
- 첫 전체 회귀는 11,386개 중 11,384개가 통과했다. 새 `resourceProfiles`의 capabilities
  schema 등록과 `resourceProfile`·`execution`의 필드 사전 등록을 빠뜨려 계약 검사 2개가
  실패했다. 검사·기대값을 유지하고 schema와 사전을 보완한 뒤 최종 전체 검증을 재실행했다.

최종 검증의 runtime/test 파일 해시는 ignored
`output/pr-review/issue8174/resume-20261024/source-state-final.json`에 고정했다.
컴파일에 포함하는 `agent_knowledge_map.md`도 같은 목록에 넣었다.
해당 source의 집중 검사·전체 게이트·기준 비교 결과를 이어서 기록한다.

## 최종 실행 결과

공통 target은 `/home/edward/mygithub/rhwp/target/pr-review`, Cargo profile은
`release-test`다. `CARGO_BUILD_JOBS=4`·테스트 동시성 4를 사용했으며 Cargo 작업은 순차 실행했다.
정확한 인자·종료 코드·소요 시간은 같은 증적 경로의 `validation-status-final.json`에 보존한다.

| 검사 | 명령 요약 | 결과 |
| --- | --- | --- |
| 집중 계약 | `cargo nextest run --locked --cargo-profile release-test --tests -E 'test(capabilities_schema_contract::) \| test(knowledge_map_field_dictionary_contract::) \| test(issue_8174_pdf_resource_policy::) \| test(issue_8174_pdf_process_supervisor::)'` | 32/32 PASS. 신규 정책·CLI 6개, OS 감독 7개와 schema·사전 19개 |
| 기존 PDF CLI | `node scripts/run-rust-test.mjs issue_7911_pdf_cli -- --cargo-profile release-test` | 5/5 PASS. HWP/HWPX·쪽 선택·오류/출력 계약 |
| 포맷 | `cargo fmt --all -- --check` | PASS |
| Native lint | `cargo clippy --locked --profile release-test -- -D warnings` | PASS |
| WASM32 library lint | `cargo clippy --locked --profile release-test -p rhwp --lib --target wasm32-unknown-unknown -- -D warnings` | PASS |
| workspace | `cargo build --locked --profile release-test --workspace` | PASS |
| 전체 target lint | `cargo clippy --locked --profile release-test --workspace --all-targets -- -D warnings` | PASS |
| 자동 수집 정책 | `node scripts/rust-test-suite-manifest.mjs --check --base-ref 3536e72819e165dbe959ee42cfd4e4cf8db33ce7` | PASS. 기존 suite·exception 예산 유지 |
| 전체 회귀 | `cargo nextest run --locked --cargo-profile release-test --tests --test-threads 4 --no-fail-fast` | 11,386/11,386 PASS, 기존 skip 45개. 실행 1,057.402초, 빌드 포함 1,573.174초 |
| 문서·변경 범위 | 변경 문서의 metadata·상대 링크·`git diff --check` | metadata 대상 6개·링크 대상 9개 PASS. source freeze 21개 파일·최종 일반 CLI SHA 일치 |

신규 OS 검사는 AS 64 MiB에서 128 MiB 할당 실패와 다음 정상 실행, CPU 1초 busy loop 종료,
stdout 초과, 읽지 않은 1 MiB stdin의 deadline, 종료한 leader의 상속 pipe, escaped session의
pipe 대기 종료, timeout의 process group 종료를 실제 실행했다. escaped 자손은 검사에서 직접
정리했으며 이를 감독이 전체 프로세스 트리를 종료한 증거로 삼지 않는다.
CLI에서는 입력·그림·출력·진단·wall 한도 실패마다 기존 `KEEP` 파일·빈 stdout을 확인하고
새 constrained 요청의 저장·재열기로 복구를 확인했다.

새 Native/CLI 검사는 기존 `tests/cases/` 자동 수집 경로로 실행했으며 CI 호출·suite 예산·skip을
추가하지 않았다. manifest 배정 규칙의 Node 계약 검사도 23/23 통과했다. nextest의 권고 버전
차이와 기존 `junit.report-skipped` 설정 경고는 남아 있으나 실행 누락이나 실패로 이어지지 않았다.

## 이전 구현 대비 정상 출력·성능

승인 직전 source `b4f834dbefd805485b5ab29359d5ed1856dc9021`을 `git archive`로 ignored
`base-source/`에 준비하고 동일 compiler·`release-test`·공용 target으로 빌드했다. 별도 branch나
worktree를 만들지 않았다. 현재 코드의 전체 검사가 끝난 뒤에만 기준 빌드와 측정을 수행했다.
입력마다 base·standard·constrained의 순서를 바꿔 3회씩, 총 36회를 실행했다.
최종 표는 base·후보 모두 `cargo build --locked --profile release-test --bin rhwp`의
일반 CLI graph로 맞춘 결과다.

| 공개 입력 | 출력 쪽 | HWPX 바이트 | base → standard 중앙값(ms) | base → standard 관측 합산 RSS(MiB) | standard 출력 |
| --- | ---: | ---: | ---: | ---: | --- |
| [scan-jpeg-control-200.pdf](../../pdf/issue8183/scan-jpeg-control-200.pdf) | 1 | 211,949 | 73.923 → 86.143 | 30.16 → 37.92 | 3/3 바이트 동일 |
| [image-placement-controls.pdf](../../pdf/issue7966/image-placement-controls.pdf) | 3 | 34,076 | 21.630 → 27.059 | 21.41 → 28.70 | 3/3 바이트 동일 |
| [scan-jpeg-buffer-controls.pdf](../../pdf/issue7971/scan-jpeg-buffer-controls.pdf) | 5 | 766,935 | 184.789 → 214.405 | 53.53 → 61.36 | 3/3 바이트 동일 |
| [nested_table_text_caption-2020.pdf](../../pdf/issue5875/nested_table_text_caption-2020.pdf) | 12 | 1,422,018 | 300.087 → 338.576 | 41.34 → 50.67 | 3/3 바이트 동일 |

`standard`의 추가 시간은 이 표에서 약 5.4–38.5ms, 관측 합산 RSS 증가는 약 7.3–9.3 MiB다.
작은 입력은 상대 증가율이 약 25%로 커진다. 프로세스 시작·IPC·감독 비용이 있으므로 성능 영향이
없다고 판정하지 않는다. 3회 측정과 같은 호스트의 관측 결과이며 통계적 보증이나 전체 입력의
상한이 아니다. RSS는 5ms 간격의 부모·자손 합산 관측 최댓값으로 공유 페이지를 중복 계산할 수
있으며 물리 메모리 peak·작업자 AS 제한과 구분한다.

`constrained`는 200 DPI·placement·nested 입력에서 각각 3/3 성공·base와 바이트 동일이었다. 중앙값은 이 순서대로
85.938ms·27.122ms·331.815ms다. `scan-jpeg-buffer-controls.pdf`는 세 번 모두
`PDF_RESOURCE_LIMIT`로 거부했다. 첫 그림부터 1,928×2,725 = 5,253,800픽셀로
개별 4,000,000픽셀 한도를 넘는다. 누적 한도 때문이라는 초기 판단을 정정했으며 부분 변환을
성공으로 처리하지 않았다. 이 입력은 기존 한도를 유지한 `standard`에서 정상 처리한다.

증적은 같은 `output/pr-review/issue8174/resume-20261024/` 아래에 보존했다.

- `benchmark-inputs.json`: 공개 입력의 바이트 수·SHA-256.
- `benchmark.json`, `bench-*.json`·`bench-*.stderr`: 36회 결과·출력 SHA-256·실행 지표.
- `source-state-final.json`: 검증한 runtime/test/내장 문서 21개 파일의 SHA-256.
- `candidate-binary-final.json`: 일반 CLI 후보 `1718d4fc0d247b7962bc8eeccce6a120a97ce8b8534fa700ec0e76d2c04c16eb`.
- `candidate-binary-tests.json`: 전체 회귀의 CLI `e03c8d7568b12abf1607e08ae724090b207332dc104981bfc5493cd1cecb7db5`.
- 기준 binary: `1b326a56b72f91499f16ca5c63ccc0218c6cd26e0bfd9457b344482745662cd4`.
- `validation-status-final.json`·`*-final.log`: 최종 gate의 명령·결과와 로그.
- `comparison-status.json`·`base-build.log`·`restore-current-build.log`: 순차 기준 빌드·비교와
  현재 작업공간의 일반 CLI 복원. 아래의 빌드 graph 차이와 후속 확인을 함께 따른다.

### 일반 빌드와 테스트 빌드의 구분

처음 복원한 일반 CLI의 SHA-256이 전체 회귀의 CLI와 달라 자동 일치 검사가 실패했다.
source 21개 파일의 해시는 그대로였고 두 바이너리의 capabilities 출력은 바이트 동일이었다.
Cargo fingerprint를 대조하니 본체 feature와 profile은 같으나 테스트 graph에서
`serde_json/alloc` 등이 합산돼 의존성 fingerprint가 달랐다. 소스 불일치로 단정하거나
이전 파일을 target에 덮어쓰지 않았다.

첫 비교는 테스트 graph 후보와 일반 graph base를 섞었으므로 최종 성능 근거에서 제외했다.
원본은 `benchmark-devgraph/`와 `bin/rhwp-candidate-tests`에 보존했다. 일반 graph끼리
36회를 다시 실행한 결과가 위 표다. 복원된 일반 CLI를 바꾸지 않은 채 기존 회귀 harness의
PDF 정책 6개·기존 PDF CLI 5개·schema 17개·필드 사전 2개를 직접 실행해 **30/30 PASS**를
확인했다. 이는 전체 회귀 11,386건을 다시 실행했다는 뜻이 아니다.
명령·harness/CLI SHA-256·종료 코드는 `normal-cli-contracts.json`·`normal-cli-contract-*.log`,
graph 차이는 `restore-fingerprint-comparison.json`에 보존했다. 일반 CLI의 최종 후보 SHA와
공용 target의 실행 파일 SHA는 일치한다. source·기능·기대값은 이 확인 과정에서 변경하지 않았다.

## 남은 승인 범위

S01의 완료를 F01–F08 전체 기본 탑재나 MD05-B의 공식 지원 완료로 확대하지 않는다.
승인된 [S02–S05](../plans/task_m100_8174_impl.md#범위-v2--기본-기능과-환경별-자원-정책)를
계속 추적하며 별도 이슈로 넘기거나 범위에서 제거하지 않는다.

- Native 기본 코덱·ICC·xref 복구·SVG 글리프·링크와 B01–B05의 할당 경계 검증.
- 직접 Rust·기존 CLI 공통 로더·다른 OS와 파일/네트워크 접근의 실제 지원 경계.
- WASM의 같은 정책 전달·Worker 중단/재생성, Studio 실제 열기·편집·저장과 F07·F08.
- 변경 뒤 전체 graph의 포장·독립 소비와 최종 배포 정책. `publish_ready=false`를 유지한다.
