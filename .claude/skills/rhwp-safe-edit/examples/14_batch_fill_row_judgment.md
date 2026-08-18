# 14 — 메일머지 행 판정 (`batch fill`)

층: 배치. 원자 `run` 이 아니다. 서식 1 + 데이터 N행.
행별 `notFound` 가 있어도 **최종 exit 0** 일 수 있다.

권위: [single_edit.md](../references/single_edit.md) §10,
playbook §9-4.

## 1. 전행 dry-run

```bash
rhwp batch fill --form samples/field-01.hwp --data out/rows.jsonl \
  --out-dir out/merge --name-field 회사명 --dry-run --json
```

실측 세 줄 (playbook):

```
{"dryRun":true,"filledCount":3,"notFound":[],"row":0, …}
{"dryRun":true,"filledCount":3,"notFound":[],"row":1, …}
{"dryRun":true,"filledCount":2,"notFound":["없는필드"],"row":2, …}
batch fill: 3행 중 3 성공, 0 실패
exit=0
```

row 2 는 실행 성공이 아니라 레코드 판정 실패다. 데이터를 고치기 전에
`--dry-run` 을 떼지 않는다.

픽스처 [../fixtures/envelopes/batch_row_notfound.json](../fixtures/envelopes/batch_row_notfound.json)
은 그 세 번째 줄의 골격이다.

## 2. 완료 식 (행 단위)

```
각 레코드: error 없음 AND notFound == [] AND (동명이면 ambiguous 없음)
전행이 위 식을 만족한 뒤에만 실행
```

최종 exit 0 을 전행 완료로 읽지 않는다.

## 3. 실행 후

같은 NDJSON 을 다시 훑는다. 표본 한 행만 `fields` 재독.
실패 행은 `source`/`output`/`row` 로 격리해 단건 `edit fill-fields` 또는
데이터 수정 후 그 행만 다시 `batch fill` 하지 말고 — 배치 도구가 행 재시도
입구를 따로 열지 않으면 — 단건으로 처리한다.

## 4. `run` 과의 경계

- N명분 독립 산출 → `batch fill`
- 한 문서에 여러 종류의 편집 → `run`
- 한 문서에 칸 하나 → 1층 `edit`

`run` 을 N번 돌리지 않는다. `batch` 를 원자 다단계 대신 쓰지 않는다.

## 5. 체크리스트

- [ ] 전행 dry-run
- [ ] 행마다 notFound 를 봤다
- [ ] 최종 exit 0 만 믿지 않았다
