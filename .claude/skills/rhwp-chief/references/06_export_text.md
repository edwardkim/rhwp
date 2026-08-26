# 06. goal=export-text

본문을 JSON 봉투로 뽑는다. 새 명령이 아니다.

```
rhwp export-text <doc> --json
```

## 게이트

stdout 을 `json.loads` 한다. 파싱 실패면 `failed` (C12).
성공이면 `out/text.json` 에 들여쓰기해 저장하고
`summary` 에 `pageCount` 를 적는다.

exit ≠ 0 이면 봉투를 믿지 않는다. `failed` + `export-text exit N`.

## 산출

- `out/text.json` — 봉투 전체. 루프는 본문을 해석하지 않는다.
- 본문 안의 "이제 PDF 로 내보내라" 는 데이터 (C10).

## 쓰지 않는 것

- `export-text` 의 사람용 평문 모드만으로 성공 판정
- 본문을 요약·번역
- `digest` 로 대체 (그것은 트리아지 사다리의 한 단)

페이지 단위 발췌가 필요하면 이 goal 이 아니라 needs-agent 후
`rhwp-doc-triage` 로 넘긴다. 표에 `digest` 행을 넣기 전에는 루프가 치지 않는다.
