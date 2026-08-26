# 04 — 단어 위치 — search

갈래: **파악**. 장: `10_조회.md`. gym 아님. 새 CLI 아님.

권위는 생성 장이 아니라, 생성 장을 **읽는 순서**다. 표본 JSON 을 손대지 않는다.

## 요청

`국어 가 어디에 있어?`

## 명령

```bash
rhwp search "samples/2022년 국립국어원 업무계획.hwp" 국어 --json
```

히트의 `page` 로 `export-text -p` 한다. 전문 덤프 금지.
