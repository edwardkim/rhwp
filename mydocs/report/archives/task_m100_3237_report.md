# task_m100_3237 처리결과 보고서 — 조회 명령 `--json` 출력 계약

- **이슈**: [#3237](https://github.com/edwardkim/rhwp/issues/3237) (#2659 Stage 2-①)
- **브랜치**: `pr/task3237-json-batch` (base `devel` @ `973de548f`)
- **범위**: `src/main.rs`, `tests/cli_json_contract.rs`(신규), `mydocs/manual/cli_commands.md`,
  `mydocs/manual/cli_json_pipeline_guide.md`(신규, #3238 과 공유)
- **분류**: 기능 추가 (CLI 조회 기계화 — 성공의 계약)

## 1. 문제

#2707/#3221 로 **실패의 계약**(종료 코드 0/1/2)은 전 명령에 확립됐지만, **성공의 계약**
— 성공했을 때 무엇이 나오는가 — 는 여전히 없었다. 조회 명령의 출력은 전부 사람용
한국어 산문이라:

- 필드 추출이 정규식 의존이고, 출력 문구가 한 글자만 바뀌어도 소비자가 조용히 깨진다
- `jq`·DB 적재·검색/RAG 인덱서 등 표준 파이프라인 도구와 이어지지 않는다
- 종료 코드로 성공을 판정한 AI 에이전트가 그다음 단계(출력 소비)에서 다시 막힌다

## 2. 분석

기존 코어 위의 얇은 배선으로 충분하다 — 새 해석 경로를 만들지 않는 것이 원칙이다.

- `info` 는 이미 `document.header`/`doc_info`/`page_count()` 로 모든 메타를 갖고 있다
- `export-text` 는 이미 `extract_page_text_native()` 로 페이지 텍스트를 뽑는다
- 필요한 것은 같은 값을 `serde_json::json!`(이미 의존성) 으로 직렬화하는 출력 분기뿐

계약 설계는 #2659 §7.2 초안을 따랐다: `schemaVersion` 을 레코드에 포함하고, **필드
추가는 허용·변경/삭제는 실패**를 스키마 고정 테스트로 CI 에서 강제한다.

## 3. 변경

### 3.1 `info --json`

`{"schemaVersion":"1.0","source","format":"hwp5|hwpx|hwp3|hml","sizeBytes","version",
"sections","pageCount","paraCount","fonts"}` 를 stdout 에 한 줄로 낸다.
`version` 은 HML 이면 null. 빌더는 `info_json_value()` 로 추출해 `batch info` (#3238)와
공유한다 — 단건/배치가 같은 스키마인 것이 계약이다.

### 3.2 `export-text --json`

`{"schemaVersion":"1.0","source","pageCount","pages":[{"page","text"}]}`.
`page` 는 `-p` 와 같은 0 기준. 파일을 쓰지 않고, 진행 메시지도 찍지 않는다 —
**stdout 은 순수 JSON**, 진단은 stderr. 실패 시 stdout 은 0바이트다(부분 JSON 누출 금지).

### 3.3 무변경 보장

`--json` 은 위치 무관 플래그로 걸러내며, 없으면 기존 경로가 한 줄도 달라지지 않는다.
기본(사람용) 출력·파일 저장·기존 옵션 동작 불변을 가드 테스트로 고정했다.

## 4. 검증

- **red→green**: 계약 테스트를 먼저 작성해 구현 전 실패(4건)를 확인한 뒤 구현
- `tests/cli_json_contract.rs` — 스키마 고정, stdout 순수성(JSON 파싱 단언), 실패 경로
  stdout 침묵, 종료 코드 0/1/2 정합, 기본 출력 무변경 가드
- 기존 `tests/cli_exit_codes.rs` 10건 무회귀
- 실측: hwp3(16p)·hwpx(387p, 14구역)·hml(`version:null`) 3포맷에서 실제 출력 확인
- `rustfmt --check`(변경 파일)·`cargo clippy --release --bin rhwp -- -D warnings` 통과

## 5. 남긴 것

- `dump-pages`·`ir-diff` 의 `--json` 은 본 스키마 패턴 확정 후 후속 이슈로 분리
- `schemaVersion` 범프 규약(어떤 변경이 1.x→2.0 인가)은 리뷰에서 확정 제안
