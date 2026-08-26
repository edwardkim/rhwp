# 10 — 누름틀 dry-run

갈래: **편집**. 장: `30_편집과_계획.md`. gym 아님. 새 CLI 아님.

권위는 생성 장이 아니라, 생성 장을 **읽는 순서**다. 표본 JSON 을 손대지 않는다.

## 요청

`회사명 칸에 코덱스. 미리보기만.`

## 명령

```bash
rhwp edit fill-fields samples/field-01.hwp --data '{"회사명": "코덱스"}' --dry-run --json
```

`notFound` / `ambiguous` 를 성공으로 읽지 않는다. 반복 필드는 `이름[N]`.
