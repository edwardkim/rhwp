# 병합 표 폴백 — 기존 `edit set-cell` 만

권위: [`cli_commands.md`](../../../../mydocs/manual/cli_commands.md) §edit set-cell (#3381),
레시피 02 "병합 표는 이 레시피 대신",
playbook §10-5, `edit_set_cell_contract.rs`.

CSV 에는 병합 개념이 없다. `table-to-csv` 는 덮인 칸을 빈 문자열로 채워
뽑을 수 있지만, 그 자리에 값을 넣고 `csv-to-table` 하면
`coveredCellNotEmpty` 로 거절된다.

이 스킬은 **병합을 풀거나 다시 합치는 편집 로직을 발명하지 않는다.**
devel 에 있는 `edit set-cell` 만 쓴다.

## 1. 언제 갈아타나

`export-tables` 이후:

```
any(rowSpan>1 or colSpan>1) 이고 값을 되돌릴 계획
    → csv-to-table 을 호출하지 않는다
    → edit set-cell --table index --row R --col C --text 값
```

뽑기만 하면 되면 `table-to-csv` 는 허용 (`extract-only`).
되돌리기가 필요한 순간부터 set-cell 축이다.

행렬: [../fixtures/matrices/merge_decision.json](../fixtures/matrices/merge_decision.json).
루프: [../fixtures/loops/merge_fallback.json](../fixtures/loops/merge_fallback.json).

## 2. 기존 명령 — 조립만

```bash
rhwp edit set-cell <파일> --table N --row R --col C --text <값> \
  [-o <출력>] [--dry-run] [--keep-style] [--json]
```

| 플래그 | 뜻 |
|---|---|
| `--table`/`--row`/`--col` | `export-tables` 와 같은 0 기준 격자. **앵커 좌표** |
| `--text` | 넣을 값. 빈 문자열은 비우기. 줄바꿈·탭 불가 |
| `--keep-style` | 안내문 글자모양 유지. 기본은 검정·비이탤릭·비진하게 (#3391) |
| `-o` | 산출 분리. 기본 `<입력명>_cell.<확장자>` |
| `--dry-run` | 파일 없이 `oldText`→`newText` |
| `--json` | 봉투 |

이 스킬이 `insert-row` / `merge-cells` / `split-cell` 을 안내하지 않는다.
그 명령이 저장소에 있어도 **표↔CSV 왕복의 폴백이 아니다.**

## 3. 성공 봉투

```json
{
  "schemaVersion": "1.0",
  "source": "samples/table-001.hwp",
  "table": 0,
  "row": 0,
  "col": 1,
  "oldText": "5월",
  "newText": "5월(수정)",
  "dryRun": false,
  "keepStyle": false,
  "overflow": [],
  "output": "작성본.hwp",
  "outputFormat": "hwp5"
}
```

픽스처: [../fixtures/envelopes/set_cell_anchor_ok.json](../fixtures/envelopes/set_cell_anchor_ok.json).

재독은 같은 좌표:

```bash
rhwp export-tables 작성본.hwp --json \
  | jq '.tables[] | select(.index==0) | .cells[] | select(.row==0 and .col==1).text'
```

발견 → 기록 → 재독이 **한 주소**로 닫힌다. 새 검증 명령을 만들지 않는다.

## 4. 덮인 칸 — exit 2 + 앵커 안내 + stdout 0바이트

```
$ rhwp edit set-cell samples/table-001.hwp --table 0 --row 0 --col 2 --text 잘못 --json
(0,2) 는 병합으로 덮인 칸입니다 — 앵커 (0,1) 를 지정하세요.
exit=2
```

`csv-to-table` 과 다른 점:

| | `csv-to-table` 덮인 칸 | `edit set-cell` 덮인 칸 |
|---|---|---|
| stdout | `invalid[]` 봉투 | **0바이트** |
| stderr | (봉투의 message) | 앵커 좌표 문장 |
| exit | 2 | 2 |
| 파일 | 안 만듦 | 안 만듦 |

픽스처: [../fixtures/envelopes/set_cell_covered_exit2.json](../fixtures/envelopes/set_cell_covered_exit2.json).

에이전트는 stderr 에서 `(r,c) … 앵커 (R,C)` 를 읽고 **그 앵커로 다시
호출**한다. 덮인 칸에 쓰는 우회 경로를 발명하지 마라.

격자 밖 좌표도 exit 2 · stdout 0바이트.
픽스처: [../fixtures/envelopes/set_cell_oob_exit2.json](../fixtures/envelopes/set_cell_oob_exit2.json).

## 5. 앵커를 찾는 법

```bash
rhwp export-tables samples/table-001.hwp --json \
  | jq '.tables[0].cells[] | select(.rowSpan>1 or .colSpan>1) | {row,col,rowSpan,colSpan,text}'
```

`colSpan: 3` 인 `(0,1)` `5월` 이면 덮인 칸은 `(0,2)`, `(0,3)` 이다.
값은 `(0,1)` 에만 있다.

세로 병합 `rowSpan: 3` 인 `(0,7)` 이면 덮인 칸은 `(1,7)`, `(2,7)`.

덮인 좌표 집합 = 격자 − 앵커 집합. 그 좌표로 `set-cell` 하지 않는다.

## 6. `--keep-style` 과 기본 검정

기본은 제출용 양식의 파란 안내문 스타일을 실값이 상속하지 않게 검정으로
기록한다 (#3391). 안내문 모양을 유지해야 하면 `--keep-style`.

`csv-to-table` 은 글자색을 덮지 않는다. 이미 서식이 잡힌 보고서의
**값만** 갱신하는 축이다. 병합 때문에 set-cell 로 갈아타면 스타일 계약이
달라진다. 안내문을 살릴지 사용자가 밝히지 않았으면 `--keep-style` 을
기본으로 켜지 말고, 기본(검정)을 쓰되 스타일 변화를 보고한다.

픽스처: [../fixtures/envelopes/set_cell_keep_style.json](../fixtures/envelopes/set_cell_keep_style.json).

## 7. overflow 는 막지 않는다

```json
"overflow": [
  {
    "target": "table0[2,3]",
    "text": "아주 긴 안내문…",
    "cellWidthPx": 214.63,
    "textWidthPx": 440.0,
    "lines": 3
  }
]
```

exit 0. 채우기는 된다. `--dry-run` 에서도 검사한다.
신호를 무시하면 표 밖으로 넘친 문서를 완성본으로 오판한다.

픽스처: [../fixtures/envelopes/set_cell_overflow.json](../fixtures/envelopes/set_cell_overflow.json).

이 스킬은 overflow 를 고치는 조판 명령을 발명하지 않는다. 값을 짧게
쓰거나 사용자에게 넘친다고 알린다.

## 8. 셀 하나 vs 표 전체

| 요청 | 병합 없음 | 병합 있음 |
|---|---|---|
| 칸 하나 | `set-cell` 또는 CSV 왕복 | `set-cell` |
| 표 전체 값 | `csv-to-table` | **칸마다 `set-cell`** |
| 스프레드시트에서 보고만 | `table-to-csv` | `table-to-csv` (되돌리기 금지) |

여러 칸을 `set-cell` 로 이어 붙이면 중간 실패 시 반쯤 채워진 산출이
남을 수 있다. 원자성이 필요하면 `rhwp-safe-edit` 의 `run` + `set_cell`
action 으로 위임한다. **이 스킬은 run 계획서를 발명하지 않는다.**

## 9. dry-run

```bash
rhwp edit set-cell 양식.hwpx --table 0 --row 2 --col 1 --text "1,234" --dry-run --json
```

픽스처: [../fixtures/envelopes/set_cell_dry_run.json](../fixtures/envelopes/set_cell_dry_run.json).

파일을 쓰지 않는다. `oldText`/`newText`/`overflow` 만 본다.
그다음 `-o` 를 붙여 저장하고 `export-tables` 로 재독한다.

## 10. 누름틀 없는 표 칸 서식

`samples/복학원서.hwp` — 표 3개, 누름틀 0.
`edit fill-fields` 축이 아니다. 좌표를 `export-tables` 로 잡고
`set-cell` 로 채운다.

이 스킬이 `rhwp-form-fill` 본문을 바꾸지 않는다. 누름틀이 있으면 그쪽.

## 11. 하지 않는 것 (다시)

- 병합을 풀고 CSV 왕복한 뒤 다시 합치지 않는다
- 덮인 칸에 쓰는 우회 `--force` 를 지어내지 않는다
- `insert-row`/`delete-col`/`merge-cells` 를 폴백으로 안내하지 않는다
- DocumentCore `set_cell` 구현을 이 파동에서 고치지 않는다

워크스루: [../examples/12_merged_fallback_set_cell.md](../examples/12_merged_fallback_set_cell.md),
[../examples/09_covered_cell.md](../examples/09_covered_cell.md).
