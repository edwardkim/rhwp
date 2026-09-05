# 08 — 조건부 step (`if`)

층: 3. 목표: 입력 문서에 필드/문구가 있을 때만 그 step 을 적용하고,
건너뛴 step 을 저널에서 숨기지 않는다.

권위: [run_plans.md](../references/run_plans.md) §5, #3719 §6-8.

## 0. 하지 않는 것

- `if` 에 조건 두 개 (and/or 가 계획서에 없음 → invalid).
- 앞 step 이 쓴 값을 뒤 step `if` 로 읽기. 판정은 **입력 문서 기준 1회**.
- 빈 `if: {}`.

선확인은 다른 계획과 같다.

```bash
rhwp run plan.json --dry-run --json
```

`preview[]` 에서 `skipped: true` 인 항목의 `step` 인덱스가 계획서와 같은지 본다.

## 1. 세 종류 (정확히 하나)

픽스처:

- [valid_conditional_field_exists.json](../fixtures/plans/valid_conditional_field_exists.json)
- [valid_conditional_field_equals.json](../fixtures/plans/valid_conditional_field_equals.json)
- [valid_conditional_text_found.json](../fixtures/plans/valid_conditional_text_found.json)
- [invalid_two_conditions.json](../fixtures/plans/invalid_two_conditions.json)

```json
{"action": "replace_text", "find": "임시", "replace": "확정",
 "if": {"fieldExists": "결재란"}}
```

```json
{"action": "fill_fields", "data": {"비고": "해당없음"},
 "if": {"fieldEquals": {"name": "해당여부", "value": "N"}}}
```

```json
{"action": "set_checkbox", "occurrence": 0,
 "if": {"textFound": "개인정보 수집에 동의"}}
```

## 2. 건너뜀은 자리 유지

dry-run preview:

```json
{"step": 1, "action": "replace_text", "skipped": true,
 "reason": "fieldExists '결재란' 이 문서에 없습니다"}
```

(reason 문장은 실행기 구현의 문자열. 테스트는 `skipped: true` 키를 고정한다.)

실행 저널에도 같은 인덱스로 남는다. 사람 모드:

```
완료: 1 step 적용 · 1 step 건너뜀, 산출 …
  - step 1 건너뜀: …
```

건너뛴 step 은 "실행 가능" 개수에 넣지 않는다. dry-run 예고와 실행 적용 수가 같다.

## 3. 선검증 면제

조건이 거짓이면 그 step 의 대상이 없어도 invalid 가 아니다.
없는 필드를 채우는 step 에 `if.fieldExists` 를 달면, 필드가 없을 때
그 step 만 건너뛰고 나머지는 적용된다.

조건이 참인데 대상이 없으면 (모순) 선검증이 거부한다.
`fieldExists` 가 참인데 같은 이름의 data 키가 범위 밖인 경우 등.

## 4. 입력 기준의 함정

```json
{"action": "fill_fields", "data": {"상태": "완료"}},
{"action": "replace_text", "find": "미완료", "replace": "완료",
 "if": {"fieldEquals": {"name": "상태", "value": "완료"}}}
```

두 번째 `if` 는 입력 문서의 `상태` 다. 첫 step 이 방금 쓴 값이 아니다.
"채운 뒤에 치환"은 조건이 아니라 **순서** 로 쓴다. 둘 다 조건 없이 나열한다.

## 5. 체크리스트

- [ ] `if` 키가 정확히 하나
- [ ] 앞 step 결과에 의존하지 않는다
- [ ] preview/journal 에서 skipped step 의 인덱스가 계획서와 같다
