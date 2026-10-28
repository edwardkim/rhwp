# task_m100_3346 처리결과 보고서 — `batch` 에 search·export-tables·fields 축 추가

- **이슈**: [#3346](https://github.com/edwardkim/rhwp/issues/3346)
- **브랜치**: `pr/task-batch-axes` (upstream/devel `8bb8f277d` 직분기)
- **범위**: `src/main.rs`, `tests/batch_axes_contract.rs`(신규), `mydocs/manual/cli_commands.md`
- **분류**: 기능 추가 (로드맵 #2659 Stage 2 완결편)

## 1. 문제

Stage 2 조회 축이 devel 에 전부 반영되어 `export-tables`·`fields`·`search` 가 `--json`
계약을 갖췄는데, `batch` 는 여전히 3축만 지원했다. `capabilities` 실측으로 확인한 공백:

```
json 계약 명령: info, export-text, export-structure, capabilities, export-svg,
               export-tables, search, fields, batch, ir-diff   (10개)
batch 지원 축: export-text, info, export-structure              (3개)
```

에이전트에게 코퍼스 규모 작업이 승부처인데, 문서 한 건씩 프로세스를 띄우면 1만 건
아카이브에서 파싱 비용이 1만 번 반복된다. `batch` 는 그 문제를 이미 풀어 뒀지만
(한 프로세스·파일 간 병렬·입력 순서 보존·panic 격리·부분 실패 exit 1) 새 축들이
그 혜택을 못 받고 있었다.

## 2. 설계 결정

- **동시성 골격은 무변경.** `BatchMode` 에 축만 추가했다. 스레드 풀·순서 보존 재정렬
  버퍼·`catch_unwind` 격리·부분 실패 종료 코드는 한 줄도 건드리지 않았다.
- **봉투 빌더를 추출해 단건/배치가 스키마를 공유한다.** 기존 `info`/`export-structure` 가
  `info_json_value`/`structure_json_value` 를 공유하던 규약을 그대로 따라
  `tables_json_value`·`fields_json_value`·`search_json_value` 를 뽑았다. 단건 명령들이
  인라인으로 봉투를 만들던 것을 이 빌더 호출로 바꿔, **스키마가 갈라질 수 없게** 했다.
- **`collect_field_records()` 추출** — `fields` 의 레코드 조립(중첩 경로 포함)을 한 곳에
  두어 단건/배치가 같은 필드 집합을 낸다.
- **`--query` 는 search 축 전용** — `--mode` 가 `export-structure` 전용인 것과 같은 규약.
  타 축에 쓰면 사용법 오류 2, `search` 인데 없으면 사용법 오류 2.
- **파일당 매치 상한 1,000건** — 대량 코퍼스에서 한 문서가 매치를 수만 건 쏟아내면
  NDJSON 스트림이 부푼다. 단건 `search --limit` 과 같은 취지의 방어다.
- **자기서술 동시 갱신**: `capabilities` 의 `batch.subcommands`·`flags`, MCP `hwp_batch`
  도구의 `inputSchema.enum`, `--help` 를 함께 고쳤다. 드리프트 가드가 CI 에서 잡는다.

## 3. 변경

- `BatchMode` 3축 추가(`Tables`/`Fields`/`Search{query}`), `batch_record` 분기 확장
- 레코드 빌더 3종(`batch_tables_record_inner`·`batch_fields_record_inner`·
  `batch_search_record_inner`)
- 봉투 빌더 3종 + `collect_field_records` 추출 (단건 명령들이 이를 호출하도록 변경)
- `run_batch` 서브커맨드 목록·`--query` 파싱, `capabilities`/MCP/help/문서 갱신

## 4. 검증

- **계약 테스트 8종 red→green**: 3축 각각의 레코드가 **단건 봉투 스키마와 일치** /
  배치 경로에서도 **표 병합 정보 보존**(같은 추출기를 쓴다는 증거) / 입력 순서 보존 +
  부분 실패 exit 1 / `--query` 누락 exit 2 / `--query` 타 축 사용 exit 2 /
  **기존 3축 무회귀** / **capabilities 드리프트 가드**
- 상세 검증 결과(clippy·전체 lib·연관 통합 스위트·fmt·실측 스모크·문서 검사)는
  PR 본문에 기록.

## 5. 남긴 것

- `batch ir-diff`(쌍 목록 배치 검증)는 stdin 입력 형식(쌍 구분) 설계가 필요해 제외했다.
- `batch export-svg`(렌더 매니페스트 배치)는 산출물 경로 충돌 규칙이 필요해 제외했다.
- `batch search` 는 대소문자 구분 고정이다 — `--ignore-case` 를 배치에도 노출할지는
  수요 확인 후 별도로 다루는 것이 맞다고 본다.
