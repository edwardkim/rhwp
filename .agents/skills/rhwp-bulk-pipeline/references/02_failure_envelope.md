# 02 — 실패도 봉투다

한 파일이 깨져도 프로세스는 죽지 않는다. 그 파일의 오류 레코드를 내고
다음 파일로 간다. 성공 4 + 실패 1 이면 stdout 5줄, exit 1.

## 공통 실패 스키마

```json
{"error": "문서를 열 수 없습니다: 지정된 파일을 찾을 수 없습니다. (os error 2)", "exitClass": "runtime", "schemaVersion": "1.0", "source": "samples/없는파일.hwp", "untrustedContent": false, "untrustedFields": []}
```

필수 키:

| 키 | 값 |
| --- | --- |
| `schemaVersion` | `"1.0"` |
| `source` | stdin 에 적힌 경로 그대로 |
| `error` | 사람/기계가 읽는 메시지 |
| `exitClass` | `"runtime"` (행 단위 실패) |

실측 원형(레시피 9)은 `untrustedContent` / `untrustedFields` 도 싣는다.
출처 표지 소비는 `rhwp-provenance` 스킬. 이 스킬은 필드를 지우지 않는다.

fill 축의 실패 레코드는 같은 스키마에 `row`(0 기준)가 붙는다.

## 실패가 아닌 것

다음 값은 **성공 레코드**다. `error` 키가 없다. exit 0.

- `tableCount: 0` — 표가 없는 문서
- `fieldCount: 0` — 누름틀이 없는 문서
- `matchCount: 0` — 검색어가 없는 문서
- `itemCount: 0` — 날짜/금액/수량이 없는 문서 (`extract-data` 명문: 0건은 오류 아님)
- `truncated: true` — 한도에 걸린 것. 실패가 아니라 절단 신호

이것들을 `select(.error)` 에 넣으면 재시도 목록이 비고, 반대로 성공으로만
세면 맞다.

## 스트림을 끊지 않는 이유

무인 CI 에서 271건 중 3건이 손상돼도 268건의 본문은 남아야 한다.
첫 실패에서 프로세스가 죽으면 에이전트는 손상 파일만 보고 코퍼스를 잃는다.

단건 명령은 실패 시 stdout 0바이트다. 배치는 반대다. 같은 `error` 키를
단건 소비 코드에 넣으면 단건은 그 키를 성공 시 갖지 않으므로
`if "error" in rec` 로 분기하면 단건/배치를 같은 함수로 읽을 수 있다.

## panic · 추출 예외

capabilities 명문: 건별 실패(읽기·파싱·추출·panic)는 레코드로 격리하고
스트림을 계속한다. panic 도 한 파일을 죽일 뿐 프로세스를 죽이지 않는 것이
배치 계약이다. 재시도 부류는 `R-PARSE` (재시도 금지)로 분류한다.

## 사용법 오류는 봉투가 아니다

`--password`, `--query` 누락, convert 이름 충돌, 빈 fill CSV 는
**레코드를 한 줄도 내지 않고** exit 2 다. stdin 도 소비하지 않는 경우가
있다 (`batch_axes_contract` 의 BrokenPipe 주석). 에이전트는 stdout 이
비고 `$LASTEXITCODE -eq 2` 이면 플래그부터 고친다. 재시도 루프에 넣지 않는다.

## 권위

- `mydocs/manual/cli_commands.md` §batch
- `mydocs/manual/cli_json_pipeline_guide.md`
- `mydocs/manual/recipes/09_bulk_extract_convert.md`
- `mydocs/manual/recipes/05_mail_merge_batch_fill.md`
- `rhwp capabilities` 의 batch 항목
- 픽스처: `fixtures/` · 이 장: `02_failure_envelope.md`
- 이슈 #5311. gym 아님. 새 CLI 아님.
