# 09 — 셀 교정 dry-run

갈래: **편집**. 장: `30_편집과_계획.md`. gym 아님. 새 CLI 아님.

권위는 생성 장이 아니라, 생성 장을 **읽는 순서**다. 표본 JSON 을 손대지 않는다.

## 요청

`표 1 행0 열0 을 코덱스 로.`

## 명령

```bash
rhwp edit set-cell samples/basic/issue2007_nested_cell_pagination_42065.hwp --table 1 --row 0 --col 0 --text 코덱스 --dry-run --json
```

`oldText` 는 문서 파생(C3). `--keep-style` 없으면 기본 스타일로 들어간다.
