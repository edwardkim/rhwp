# --dry-run / --verify / exit 2·3 은 데이터다

권위: [`cli_commands.md`](../../../../mydocs/manual/cli_commands.md) §csv-to-table · §종료 코드 #2707,
레시피 02, `table_csv_contract.rs` (`dry_run_writes_no_file`,
`identical_csv_writes_nothing_and_verifies`).

에이전트가 자주 하는 실수: exit 2 를 예외로 올리고 stdout 을 버린다.
`csv-to-table` 은 **exit 2 에서도 `invalid[]` 봉투를 낸다.** exit 3 은
`--verify` 판정이다. 고장이 아니다.

새 명령을 만들지 않는다.

## 1. 세 층

```
① --dry-run --json     디스크 무변경. changed/invalid 만
② -o out --json        저장. verify 키는 null (안 줌)
③ -o out --verify --json  저장 + 재파싱. 차이 시 exit 3
```

① 없이 ③ 로 건너뛰지 마라. 치수 오류면 ②도 파일을 안 만들지만,
습관이 아니라 계약에 기대는 편이 안전하다.

## 2. dry-run 봉투

레시피 02 + 계약 실측:

```bash
rhwp csv-to-table samples/hwp_table_test.hwp \
  --csv table0_edited.csv --table 0 --dry-run --json
```

```json
{
  "changedCount": 9,
  "changed": [{"row": 1, "col": 0, "oldText": "", "newText": "서버 이관"}],
  "invalid": [],
  "dryRun": true,
  "changedPages": null,
  "output": null,
  "rowCount": 4,
  "colCount": 3,
  "table": 0,
  "schemaVersion": "1.0"
}
```

픽스처: [../fixtures/envelopes/csv_to_table_dry_run.json](../fixtures/envelopes/csv_to_table_dry_run.json).

고정할 것:

| 필드 | dry-run | 의미 |
|---|---|---|
| `dryRun` | `true` | 선확인이다 |
| `changedPages` | `null` | 확정 불가. 빈 배열(바뀐 쪽 없음)이 아니다 |
| `output` | 없음/null | 경로를 열어보지 마라 |
| 디스크 | 안 씀 | `-o` 를 줘도 파일이 생기면 계약 위반 |
| `invalid` | 실행과 같음 | 여기서 고친다 |

`changedPages: null` 을 "바뀐 쪽 없음"으로 읽지 마라. 눈검증은 실제 저장
후에 한다.

## 3. verify 성공

```bash
rhwp csv-to-table samples/hwp_table_test.hwp \
  --csv table0_edited.csv --table 0 \
  -o table_updated.hwp --verify --json
```

```json
{
  "changedCount": 9,
  "invalid": [],
  "dryRun": false,
  "output": "table_updated.hwp",
  "outputFormat": "hwp5",
  "changedPages": [0],
  "verify": {"diffCount": 0, "identical": true}
}
```

exit 0. 픽스처: [../fixtures/envelopes/csv_to_table_verify_ok.json](../fixtures/envelopes/csv_to_table_verify_ok.json).

`--verify` 를 안 주면 `verify` 는 **`null`** 이다. `null` 을 통과로 읽지 마라.
확인하지 않은 것이다.

같은 CSV 를 다시 넣으면 `changedCount: 0` + `identical: true` 도 정상이다.

## 4. verify 실패 — exit 3 은 판정

```json
{
  "changedCount": 9,
  "invalid": [],
  "output": "table_updated.hwp",
  "verify": {"diffCount": 2, "identical": false}
}
```

- exit **3**
- `invalid[]` 는 비어 있다 (사용법 오류가 아님)
- 1층 `csv-to-table` 은 산출물을 **남긴다**
- stdout 봉투를 버린 채 예외로 올리면 `diffCount` 를 못 읽는다

픽스처: [../fixtures/envelopes/csv_to_table_verify_fail.json](../fixtures/envelopes/csv_to_table_verify_fail.json).
루프: [../fixtures/loops/verify_exit3.json](../fixtures/loops/verify_exit3.json).

처방:

1. `verify.diffCount` 를 읽는다
2. `export-tables` 로 저장본을 재독한다
3. CSV 와 `cells[].text` 를 diff 한다
4. 병합·중첩 혼재를 다시 본다
5. 필요하면 `edit set-cell` 축으로 갈아탄다

`--verify` 통과는 자기 재파싱 게이트일 뿐이다. 무손실 계약이면 별도로
`ir-diff <원본> <산출> --json` 을 돌린다 (이 스킬의 필수 단계는 아님).

## 5. exit 2 — 치수는 봉투, 조립은 침묵

같은 숫자 2 가 두 갈래다.

| 갈래 | stdout | 읽는 것 | 예 |
|---|---|---|---|
| 계약 거부 | `invalid[]` 봉투 | `reason` 전부 | 치수·덮인칸·제어문자·csvParse |
| 사용법 | 0바이트 | stderr | `--csv` 누락, 파일 positional 없음 |

에이전트 의사코드:

```
code = wait(cmd)
if code == 0:
    read envelope; still check invalid, changedCount, verify
elif code == 1:
    runtime; original untouched; stdout empty on single-command
elif code == 2:
    text = stdout
    if text is JSON and "invalid" in obj:
        DATA — fix CSV or coordinates; do not retry same argv
    else:
        assembly bug — fix flags
elif code == 3:
    DATA — verify.identical is false; output exists; reread
```

[failure_envelopes.md](failure_envelopes.md) 가 표 전체다.

## 6. dry-run 과 실행은 같은 명령줄

선확인이 의미를 가지려면:

```
csv-to-table <in> --csv C --table N --dry-run --json
csv-to-table <in> --csv C --table N -o OUT --verify --json
```

`--table` 을 바꾸거나 CSV 경로를 바꾸면 ①의 `changedCount` 는 ③을
보증하지 않는다. 플래그 하나만 빼는 형태라야 한다.

`-o` 를 dry-run 에 붙여도 파일은 안 생긴다. 생겨도 쓰지 마라.

## 7. changedPages

| 값 | 의미 | 눈검증 |
|---|---|---|
| `null` | 확정 불가 (dry-run 항상) | 하지 않음 |
| `[]` | 바뀐 쪽 없음 | 해당 없음 |
| `[0, 2]` | 0 기준 쪽 목록 | `export-svg -p 0` 등 |

`null` 과 `[]` 를 섞어 읽지 마라.

## 8. 입력 형식 보존

HWPX 입력 → `outputFormat: "hwpx"`.
HWP5/HWP3 입력 → `outputFormat: "hwp5"`.

레시피 02 표본은 `.hwp` 라 `hwp5` 다.
지자체 양식 `.hwpx` 왕복은 `hwpx` 여야 한다
(`identical_csv_writes_nothing_and_verifies`).

`-o ….hwp` 를 HWPX 입력에 명시하면 경로를 존중하되 형식 변경 경고가
나간다. 표 왕복에서 형식을 바꾸지 마라.

## 9. 원본 불변

- dry-run: 원본·산출 둘 다 안 건드림
- invalid: 산출 파일을 만들지 않음. 원본 그대로
- exit 1: 원본 그대로
- 성공: 원본은 그대로, `-o` 산출만 생김

`--in-place` 를 이 명령에 붙이지 마라. 그런 플래그가 계약에 없다.

## 10. 체크리스트

- [ ] dry-run 의 `invalid` 가 `[]`
- [ ] `changedCount` 가 기대와 같거나, 0 이면 이유가 설명된다
- [ ] `changedPages` 가 dry-run 에서 `null`
- [ ] `--verify` 를 줬고 `identical: true`
- [ ] exit 3 이면 산출물을 열어 `export-tables` diff
- [ ] exit 2 인데 stdout JSON 이면 `reason` 을 고친다
- [ ] exit 2 인데 stdout 0바이트면 argv 를 고친다

워크스루: [../examples/05_dry_run_preview.md](../examples/05_dry_run_preview.md),
[../examples/06_verify_success.md](../examples/06_verify_success.md),
[../examples/15_verify_exit3.md](../examples/15_verify_exit3.md).
