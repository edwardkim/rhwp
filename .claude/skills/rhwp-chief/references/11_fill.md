# 11. goal=fill

```
rhwp edit fill-fields <doc> --data @<params.data> -o out/filled<suffix> --json
```

`params.data` 는 요청 폴더 안 값 JSON. `@` 접두는 CLI 가 파일을 읽게 하는
기존 문법이다.

## 게이트

1. `params.data` 없음 → C08 `needs-agent` (값 파일을 발명하지 않는다)
2. 경로 탈출·파일 없음 → C01/C02 `failed`
3. fill-fields exit ≠ 0 또는 산출 없음 → `failed`
4. 봉투의 `notFound` · `ambiguous` · `confusable` 중 하나라도 비어 있지
   않으면 산출 파일을 **지우고** `failed` (C09)

`confusable` 은 form-fill 스킬의 이웃 계약과 같다. 침묵 성공 금지.

## 산출

- `out/filled.hwpx` 또는 `out/filled.hwp` (원본 접미 보존)
- `summary`: `필드 N건 채움 (봉투 게이트 통과)`

## 하지 않는 것

- `batch fill` — 명단 N행은 표에 아직 없다. 반복되면 §16 으로 행을 더한다.
- `이름[N]` 추론. 값 JSON 이 키를 가지고 있어야 한다.
- `sanitize` 자동 연쇄. 제출 정리는 표 밖 — needs-agent 또는 후속 요청.
- 새 `edit mail-merge` 명령.
