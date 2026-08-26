# 03 — 표 한 칸 (`edit set-cell`)

층: 1. 목표: `export-tables` 가 준 `index/row/col` 에 값을 쓰고
같은 좌표로 다시 읽는다.

권위: [single_edit.md](../references/single_edit.md) §5,
playbook §14, `tests/edit_set_cell_contract.rs`.

## 0. 하지 않는 것

- 배열 순번을 `--table` 에 넣기. `tables[].index` 다. 0부터가 아닐 수 있다.
- 병합으로 덮인 칸에 쓰기. 앵커를 안내받으며 exit 2, stdout 0바이트.
- 중첩 표 (`cells[].nested`). v1 범위 밖.
- `--text` 에 줄바꿈·탭.
- `overflow` 를 무시하고 제출본 만들기.

## 1. 발견

```bash
rhwp export-tables 양식.hwpx --json | jq '.tables[] | {index, rows, cols, cellCount}'
rhwp export-tables 양식.hwpx --json \
  | jq '.tables[] | select(.index==12) | .cells[:6] | .[] | {row,col,text}'
```

공개 샘플 `samples/table-001.hwp` 는 테스트가 좌표를 하드코딩하지 않고
최상위 표의 첫 셀을 다시 고른다 (`edit_verify_contract.rs`). 에이전트도 같다.

## 2. 선확인

```bash
rhwp edit set-cell 양식.hwpx --table 12 --row 1 --col 1 --text "1,234" --dry-run --json
```

기대 키: `oldText`, `newText`, `overflow`, `dryRun: true`, `keepStyle: false`.
`overflow` 가 비지 않으면 값을 줄이거나 사용자에게 알린다. 11 편.

기본은 검정·비이탤릭·비진하게 (#3391). 안내문 파란 글자를 유지하려면
dry-run 에도 `--keep-style` 을 같이 붙인다.

## 3. 실행

```bash
rhwp edit set-cell 양식.hwpx --table 12 --row 1 --col 1 --text "1,234" \
  -o 작성본.hwpx --verify --json
```

실측 골격:

```json
{"changedPages":[6,7],"col":1,"newText":"1,234","oldText":"",
 "outputFormat":"hwpx","overflow":[],"row":1,"table":12}
```

HWPX 입력은 HWPX 산출 (#3383). `outputFormat` 을 `info --json` 의 `format` 과 대조한다.

## 4. 재독

```bash
rhwp export-tables 작성본.hwpx --json \
  | jq -r '.tables[]|select(.index==12)|.cells[]|select(.row==1 and .col==1).text'
```

기대 `1,234`. 이웃 칸은 그대로.

## 5. 병합 칸을 치면

```
(0,2) 는 병합으로 덮인 칸입니다 — 앵커 (0,1) 를 지정하세요.
exit=2  stdout_bytes=0
```

같은 좌표로 재시도하지 않는다. 앵커 `(0,1)` 로 dry-run 부터.

## 6. 체크리스트

- [ ] `index` 를 `--table` 에 넣었다
- [ ] dry-run 에서 `overflow` 를 읽었다
- [ ] 재독 좌표가 쓰기 좌표와 같다
- [ ] 여러 칸이면 03 을 반복하지 않고 07 (`run` `set_cell` 여러 step) 또는
      13 (`csv-to-table`) 으로 간다
