# 12 — `filledCount` 성공 ≠ 완료

층: 1. 순번 없는 동명 키는 첫 칸만 채우고 `ambiguous` 를 남긴다. exit 0.

권위: [failure_envelopes.md](../references/failure_envelopes.md) §3.2,
form_filling_guide, #3476.

## 1. 표본

`samples/field-01.hwp` 의 `목차1` ×5.

```bash
rhwp edit fill-fields samples/field-01.hwp --data '{"목차1":"개요"}' --dry-run --json
```

기대 골격 [../fixtures/envelopes/fill_ambiguous_exit0.json](../fixtures/envelopes/fill_ambiguous_exit0.json):

```json
{
  "filledCount": 1,
  "filled": [{"name": "목차1", "occurrence": 0, "value": "개요"}],
  "notFound": [],
  "ambiguous": [{"name": "목차1", "matched": 1, "total": 5}]
}
```

5칸 중 1칸. 이것을 완성본으로 넘기지 않는다.

## 2. 고치는 법

```bash
rhwp fields samples/field-01.hwp --json | jq -r '.fields[] | select(.name=="목차1") | .name'
# 다섯 줄 → 순번 0..4
```

```json
{"목차1[0]":"개요","목차1[1]":"범위","목차1[2]":"일정","목차1[3]":"예산","목차1[4]":"기타"}
```

다시 dry-run. `ambiguous` 가 비고 `filledCount` 가 5 인지 본다.

## 3. `notFound` 도 같은 함정

```json
{"filledCount": 1, "notFound": ["없는필드"], "ambiguous": []}
```

exit 0. 픽스처 [../fixtures/envelopes/fill_notfound_exit0.json](../fixtures/envelopes/fill_notfound_exit0.json).

3층은 이 키를 선검증에서 거부한다. 1층이 더 관대하므로 봉투를 더 엄격히 읽는다.

## 4. 완료 식

```
exit ∈ {0,3} AND notFound == [] AND ambiguous == [] AND filledCount == 지목한 키 수
```

## 5. 체크리스트

- [ ] `fields` 로 동명 개수를 셌다
- [ ] 동명이면 `이름[N]`
- [ ] dry-run 의 ambiguous/notFound 를 읽었다
