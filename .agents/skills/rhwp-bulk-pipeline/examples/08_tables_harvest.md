# 예제 08 — 표 수확

```bash
rhwp batch export-tables --json < examples/lists/tables_one.txt \
  | jq -c '{source, tableCount, merges: [.tables[].cells[] | select((.colSpan//1)>1 or (.rowSpan//1)>1)]}'
```

병합이 빠지면 단건과 다른 추출기를 쓴 것이다. 계약 테스트가 막는다.
`tableCount: 0` 은 실패 목록에 넣지 않는다.

이슈 #5311. gym 아님. 새 CLI 아님.
