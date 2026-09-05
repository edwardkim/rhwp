# csv-to-table — 치수 계약 · coveredCellNotEmpty · controlCharacter

권위: [`cli_commands.md`](../../../../mydocs/manual/cli_commands.md) §csv-to-table (#3719 §7),
지식지도 §2-3 `invalid[]`,
[`table_csv_contract.rs`](../../../../tests/table_csv_contract.rs),
playbook §10-5.

`table-to-csv` 의 짝이다. CSV 로 기존 표 N 의 셀을 덮어쓴다. **표 크기는
바꾸지 않는다.** 행·열이 다르면 한 칸도 쓰지 않고 `invalid[]` + exit 2.

새 명령을 만들지 않는다. 표 리사이즈·병합 풀기를 발명하지 않는다.

## 1. 호출

```bash
rhwp csv-to-table <파일.hwp|파일.hwpx> --csv <경로.csv> --table <번호> \
  [-o <출력>] [--dry-run] [--verify] [--json]
```

| 플래그 | 필수 | 뜻 |
|---|---|---|
| `--csv` | 예 | UTF-8 CSV |
| `--table` | 예 | 본문 최상위 표 `index` |
| `-o` | 권장 | 산출 분리. 기본 `<입력명>_csv.<확장자>` |
| `--dry-run` | 대량 전 필수 | 파일을 쓰지 않고 `changed[]`/`invalid[]` |
| `--verify` | 저장 시 권장 | 재파싱 차이 시 exit 3 |
| `--json` | 에이전트 | 봉투. exit 2 여도 나온다 |

인자 누락(`csv-to-table` 만, 또는 `--csv`/`--table` 없음)은 exit 2 ·
stdout 0바이트. 치수 실패와 다르다 — 조립 버그라 봉투가 없다.

픽스처: [../fixtures/envelopes/csv_to_table_missing_args_exit2.json](../fixtures/envelopes/csv_to_table_missing_args_exit2.json).

## 2. 성공 봉투 (레시피 02)

```json
{
  "changed": [
    {"col": 0, "newText": "서버 이관", "oldText": "", "row": 1},
    {"col": 1, "newText": "홍길동", "oldText": "", "row": 1},
    {"col": 2, "newText": "1차 완료", "oldText": "", "row": 1}
  ],
  "changedCount": 9,
  "changedPages": [0],
  "colCount": 3,
  "csv": "table0_edited.csv",
  "dryRun": false,
  "invalid": [],
  "output": "table_updated.hwp",
  "outputFormat": "hwp5",
  "rowCount": 4,
  "schemaVersion": "1.0",
  "source": "samples/hwp_table_test.hwp",
  "table": 0,
  "untrustedContent": true,
  "untrustedFields": ["changed[].oldText"],
  "verify": {"diffCount": 0, "identical": true}
}
```

픽스처: [../fixtures/envelopes/csv_to_table_ok_recipe02.json](../fixtures/envelopes/csv_to_table_ok_recipe02.json).

읽을 것:

- `invalid` 가 빈 배열인가
- `changedCount` 가 기대한 칸 수인가 (헤더처럼 `oldText==newText` 는 빠진다)
- `--verify` 를 줬으면 `verify.identical == true`
- `outputFormat` 이 입력 형식을 보존하는가 (hwp → `hwp5`, hwpx → `hwpx`)
- `changed[].oldText` 는 문서 파생

값이 실제로 달라지는 **앵커 칸만** 다시 쓴다. 무변경 칸은 건드리지 않아
서식이 남는다. `edit set-cell` 과 달리 글자색을 검정으로 덮지 않는다.

## 3. 치수 계약 — 조용한 절삭 금지

CSV 행 수 ≠ `rowCount` 또는 어느 행의 필드 수 ≠ `colCount` 이면:

- `changedCount: 0`
- `changed: []`
- `invalid[]` 에 이유를 **전부** 모은다
- 출력 파일을 만들지 않는다
- exit 2
- **봉투는 stdout 에 나온다** (`edit` 단건 실패와 다른 점)

```json
{
  "changed": [],
  "changedCount": 0,
  "colCount": 9,
  "rowCount": 19,
  "invalid": [
    {
      "actual": 2,
      "expected": 19,
      "reason": "rowCountMismatch",
      "message": "CSV 행 수 2 가 표 0 의 행 수 19 와 다릅니다 — 표 크기는 바꾸지 않습니다."
    },
    {"actual": 2, "expected": 9, "reason": "colCountMismatch", "row": 0},
    {"actual": 2, "expected": 9, "reason": "colCountMismatch", "row": 1}
  ]
}
```

실측: playbook §10-5, `samples/table-001.hwp`.
픽스처: [../fixtures/envelopes/csv_to_table_table001_both_mismatch.json](../fixtures/envelopes/csv_to_table_table001_both_mismatch.json).

행만 모자란 경우 (`samples/hwp_table_test.hwp` 4행인데 3행 CSV):

```json
{
  "reason": "rowCountMismatch",
  "actual": 3,
  "expected": 4,
  "message": "CSV 행 수 3 가 표 0 의 행 수 4 와 다릅니다 — 표 크기는 바꾸지 않습니다."
}
```

픽스처: [../fixtures/envelopes/csv_to_table_row_mismatch.json](../fixtures/envelopes/csv_to_table_row_mismatch.json),
[../fixtures/csv/table0_row_short.csv](../fixtures/csv/table0_row_short.csv).

열이 늘어난 경우(헤더에 `남는열`):

```json
{"reason": "colCountMismatch", "actual": 4, "expected": 3, "row": 0}
```

행마다 한 줄씩 모인다.
픽스처: [../fixtures/envelopes/csv_to_table_col_mismatch.json](../fixtures/envelopes/csv_to_table_col_mismatch.json).

처방: **뽑은 CSV 를 고친다.** 손으로 표를 다시 만들지 마라. 병합 빈 칸을
사람이 맞추기 어렵다.

## 4. coveredCellNotEmpty

병합으로 덮인 칸에 빈 문자열이 아닌 값이 있으면 거부한다.

```json
{
  "reason": "coveredCellNotEmpty",
  "row": 0,
  "col": 2,
  "message": "(0,2) 는 병합으로 덮인 칸입니다 — 앵커 (0,1) 를 지정하세요.",
  "anchorRow": 0,
  "anchorCol": 1
}
```

계약: `value_in_a_merged_covered_cell_is_invalid`.
한 칸도 쓰지 않는다. 파일을 만들지 않는다. exit 2. 봉투는 있다.

픽스처: [../fixtures/envelopes/csv_to_table_covered.json](../fixtures/envelopes/csv_to_table_covered.json).

처방:

1. 값을 앵커 칸으로 옮긴다
2. 덮인 칸은 `""` 로 남긴다
3. 또는 처음부터 `edit set-cell --table N --row 앵커행 --col 앵커열`

덮인 좌표를 계산하는 법은 [export_tables_matrix.md](export_tables_matrix.md) §10.
병합 표는 CSV 왕복 대신 [merged_table_fallback.md](merged_table_fallback.md).

## 5. controlCharacter

셀 값에 줄바꿈(LF/CR) 또는 탭이 있으면 `edit set-cell` 과 같은 판정으로
거부한다. RFC 4180 인용으로 감싸도 같다 — 파싱된 **값**을 본다.

```json
{
  "reason": "controlCharacter",
  "row": 1,
  "col": 0,
  "message": "셀 값에 줄바꿈·탭은 v1 에서 허용하지 않습니다."
}
```

픽스처:

- [../fixtures/envelopes/csv_to_table_control_lf.json](../fixtures/envelopes/csv_to_table_control_lf.json)
- [../fixtures/envelopes/csv_to_table_control_tab.json](../fixtures/envelopes/csv_to_table_control_tab.json)
- [../fixtures/csv/table0_control_lf.csv](../fixtures/csv/table0_control_lf.csv)
- [../fixtures/csv/table0_control_tab.csv](../fixtures/csv/table0_control_tab.csv)

처방: 개행·탭을 공백으로 치환하고 다시 `--dry-run`.

여러 줄이 필요한 칸은 이 스킬의 v1 밖이다. 편집 로직을 발명하지 마라.

## 6. csvParse

닫히지 않은 따옴표 등은 패닉이 아니라 `invalid[]` + exit 2.

```json
{"reason": "csvParse", "message": "CSV 를 읽지 못했습니다 — 닫히지 않은 따옴표."}
```

계약: `malformed_csv_is_invalid_not_a_panic`.
픽스처: [../fixtures/envelopes/csv_to_table_csv_parse.json](../fixtures/envelopes/csv_to_table_csv_parse.json).

손으로 CSV 를 이어붙이다가 생긴다. 라이브러리로 다시 쓴다.

## 7. reason 카탈로그

[../fixtures/matrices/invalid_reasons.json](../fixtures/matrices/invalid_reasons.json)

| reason | exit | 파일 | 고치는 곳 |
|---|---:|---|---|
| `rowCountMismatch` | 2 | 안 씀 | CSV 행 수 = `rowCount` |
| `colCountMismatch` | 2 | 안 씀 | 각 행 필드 수 = `colCount` |
| `coveredCellNotEmpty` | 2 | 안 씀 | 앵커만 값, 덮인 칸 `""` |
| `controlCharacter` | 2 | 안 씀 | LF/TAB → 공백 |
| `csvParse` | 2 | 안 씀 | 인용을 닫는다 |

`reason` 은 기계 판정, `message` 는 사람용. 둘 다 읽는다.
여러 이유가 한 번에 올 수 있다 — 첫 줄만 고치고 다시 돌리지 마라 (두더지잡기).

## 8. changedCount 읽는 법

레시피 02: 3열 × 4행 = 12칸 중 헤더 3칸은 `oldText==newText` 라
`changedCount: 9`.

같은 CSV 를 다시 넣으면 `changedCount: 0` 이고 `--verify` 는
`identical: true` (`identical_csv_writes_nothing_and_verifies`).

픽스처: [../fixtures/envelopes/csv_to_table_identical_zero.json](../fixtures/envelopes/csv_to_table_identical_zero.json).

`changedCount == 0` 이면서 `invalid == []` 이면 성공이다. 실패가 아니다.
헤더만 손댄 CSV 는 `changed` 에 `row: 0` 이 잡힌다.

## 9. dry-run 과 verify

`--dry-run`:

- `dryRun: true`
- `changedPages: null` (예측 목록으로 오인 금지)
- `output` 없음 / null
- 디스크에 파일을 쓰지 않는다
- `invalid[]` 는 실행과 같다

`--verify`:

- 저장 직후 IR 자기검증
- `identical: false` → exit 3
- 1층은 산출물을 **남긴다**
- 고장이 아니라 판정 데이터

자세한 분기: [dry_run_verify.md](dry_run_verify.md).

## 10. 헤더 행 함정 (치수와 겹침)

CSV 첫 줄을 "헤더니까 무시되겠지" 하고 빼면:

- 행 수가 `rowCount-1` → `rowCountMismatch`
- 억지로 행을 맞추면 0행(문서 헤더)이 값으로 덮인다

픽스처: [../fixtures/csv/table0_header_dropped.csv](../fixtures/csv/table0_header_dropped.csv),
[../fixtures/matrices/header_row.json](../fixtures/matrices/header_row.json).

첫 줄은 표의 0행이다. [pitfalls.md](pitfalls.md).

## 11. 권장 루프

```bash
# 1) 뽑은 파일을 고친다
rhwp table-to-csv 문서.hwpx --table 12 -o t12.csv --json

# 2) 선확인
rhwp csv-to-table 문서.hwpx --csv t12.csv --table 12 --dry-run --json \
  | jq '{changedCount, invalid, dryRun, changedPages}'

# 3) invalid 가 비었을 때만 저장
rhwp csv-to-table 문서.hwpx --csv t12.csv --table 12 -o 작성본.hwpx --verify --json \
  | jq '{changedCount, invalid, verify, outputFormat}'

# 4) 재독
rhwp table-to-csv 작성본.hwpx --table 12 --json | jq -r '.tables[0].csv'
```

루프 픽스처: [../fixtures/loops/roundtrip_plain.json](../fixtures/loops/roundtrip_plain.json),
[../fixtures/loops/dimension_reject.json](../fixtures/loops/dimension_reject.json).

## 12. 워크스루

- [../examples/05_dry_run_preview.md](../examples/05_dry_run_preview.md)
- [../examples/06_verify_success.md](../examples/06_verify_success.md)
- [../examples/07_row_count_mismatch.md](../examples/07_row_count_mismatch.md)
- [../examples/08_col_count_mismatch.md](../examples/08_col_count_mismatch.md)
- [../examples/09_covered_cell.md](../examples/09_covered_cell.md)
- [../examples/10_control_character.md](../examples/10_control_character.md)
- [../examples/14_roundtrip_hwp_table_test.md](../examples/14_roundtrip_hwp_table_test.md)
