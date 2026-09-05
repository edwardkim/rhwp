# 16 — CAS 전제 (`preconditions.inputSha256`)

층: 3. 계획이 세워진 뒤 입력이 바뀌면 실행 0·저장 0·exit 3.
사용법 오류(2)가 아니다. `invalid[]` 는 비어 있다.

권위: [run_plans.md](../references/run_plans.md) §7, #4378 R22, #3905.

## 1. 지문을 얻는 법

실행기가 쓰는 함수는 파일 전체 SHA-256 소문자 64자.
`Get-FileHash` / `sha256sum` 과 같다.

```bash
sha256sum samples/field-01.hwp
```

계획서:

```json
"preconditions": {
  "inputSha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
}
```

픽스처 [../fixtures/plans/valid_preconditions.json](../fixtures/plans/valid_preconditions.json)
은 자리만 맞춘 표본이다. 실제 해시는 실행 시 다시 계산한다.

## 2. 불일치 봉투

픽스처 [../fixtures/envelopes/run_precondition_failed.json](../fixtures/envelopes/run_precondition_failed.json).

- exit **3**
- `invalid: []`
- `preconditionFailed.kind == "inputSha256"`
- `expected` / `actual`
- `nextCall.name == "run"`
- `nextCall.arguments` 에 `--plan-json` (actual 해시로 교체된 계획), `--dry-run`, `--json`

## 3. 다음 호출

`nextCall` 을 그대로 실행한다. dry-run 이라 디스크를 안 건드린다.

- 통과: `--dry-run` 만 빼고 같은 계획을 실행
- `invalid[]` 가 나옴: 문서가 바뀌어 step 이 성립하지 않음. 발견부터 재계획

인자를 고쳐 exit 2 를 없애려 하지 마라. 문법이 틀린 것이 아니다.

## 4. 문법 오류와의 구분

빈 `preconditions: {}` 는 exit 2 (`inputSha256` 하나 필요).
64자가 아닌 문자열도 exit 2.
이것은 4.2 갈래 (FixPlanKeys) 이지 CAS 판정이 아니다.

## 5. 체크리스트

- [ ] 불일치를 exit 2 로 분류하지 않았다
- [ ] `invalid[]` 가 비어 있음을 확인했다
- [ ] `nextCall` 을 dry-run 으로 실행했다
