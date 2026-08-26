# 05 — 표 전량 — export-tables

갈래: **수확**. 장: `20_표와_데이터.md`. gym 아님. 새 CLI 아님.

권위는 생성 장이 아니라, 생성 장을 **읽는 순서**다. 표본 JSON 을 손대지 않는다.

## 요청

`표 구조랑 좌표 먼저.`

## 명령

```bash
rhwp export-tables samples/basic/issue2007_nested_cell_pagination_42065.hwp --json
```

눈에 보이는 격자 표가 표 0이 아닐 수 있다. 제목을 감싼 무테두리 표가 흔하다.
셀 텍스트는 출처 표지다.
