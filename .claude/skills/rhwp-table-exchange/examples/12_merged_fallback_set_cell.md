# 12 — 병합 표 폴백 (`edit set-cell` 만)

권위: [merged_table_fallback.md](../references/merged_table_fallback.md).
표본: `samples/table-001.hwp`.

## 판단

`export-tables` 에서 `colSpan:3` 인 `(0,1)` `5월`.
CSV 되돌리기를 시작하지 않는다.

## 명령

```bash
rhwp edit set-cell samples/table-001.hwp \
  --table 0 --row 0 --col 1 --text "5월(수정)" --dry-run --json

rhwp edit set-cell samples/table-001.hwp \
  --table 0 --row 0 --col 1 --text "5월(수정)" -o 작성본.hwp --json

rhwp export-tables 작성본.hwp --json \
  | jq '.tables[0].cells[] | select(.row==0 and .col==1).text'
```

## 덮인 칸을 찍으면

```
$ rhwp edit set-cell samples/table-001.hwp --table 0 --row 0 --col 2 --text 잘못
(0,2) 는 병합으로 덮인 칸입니다 — 앵커 (0,1) 를 지정하세요.
exit=2
```

stdout 0바이트. 앵커로 다시 부른다. 병합 쓰기 로직을 발명하지 않는다.

픽스처: [../fixtures/envelopes/set_cell_anchor_ok.json](../fixtures/envelopes/set_cell_anchor_ok.json),
[../fixtures/envelopes/set_cell_covered_exit2.json](../fixtures/envelopes/set_cell_covered_exit2.json),
[../fixtures/loops/merge_fallback.json](../fixtures/loops/merge_fallback.json).
