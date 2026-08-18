# 10 — exit 3 두 갈래

같은 숫자, 다른 디스크. 예외로 throw 하지 않고 봉투를 읽는다.

권위: [verify_loops.md](../references/verify_loops.md) §3.3,
[failure_envelopes.md](../references/failure_envelopes.md) §4,
`tests/edit_verify_contract.rs`.

## 1. 1층 — 산출물이 남는다

```bash
rhwp edit fill-fields samples/field-01.hwp \
  --data '{"회사명":"검증사"}' -o /tmp/fill.hwp --verify --json
```

계약:

- `verify.identical == true` ⇒ exit 0
- `verify.identical == false` ⇒ exit 3
- 어느 쪽이든 `/tmp/fill.hwp` 가 **존재**한다

픽스처 [../fixtures/envelopes/edit_verify_diff.json](../fixtures/envelopes/edit_verify_diff.json),
[../fixtures/envelopes/edit_verify_ok.json](../fixtures/envelopes/edit_verify_ok.json).

다음: 산출물을 지우기 전에 `fields` 재독. 값이 맞으면 사용자에게
"자기검증이 차이를 봤지만 값은 반영됐다"고 보고한다.
`--verify` 는 원본↔산출 비교가 아니다.

## 2. 3층 단언 — 산출물이 없다

```json
"assertions": {"verify": true}
```

차이가 있으면 exit 3, `output` 경로에 이번 실행의 파일이 없다.
픽스처 [../fixtures/envelopes/run_verify_fail_no_output.json](../fixtures/envelopes/run_verify_fail_no_output.json).

다음: 원본은 그대로. 재계획. "깨진 파일을 복구"하지 않는다.

## 3. `--verify` 를 안 붙인 1층

```json
"verify": null
```

exit 0. 통과가 아니다. 픽스처 [../fixtures/envelopes/verify_null.json](../fixtures/envelopes/verify_null.json).

## 4. `ir-diff` 는 세 번째 갈래

```bash
rhwp ir-diff 원본.hwp 산출.hwp --json
```

차이 시 exit 3, `categories`. 서식 채우기 전후는 차이가 정상이다.
무손실 변환 계약에만 `diffCount == 0` 을 건다.
`--json` 없이 돌리면 차이가 있어도 exit 0 일 수 있다.

## 5. 체크리스트

- [ ] exit 3 을 파싱했다
- [ ] 1층이면 산출물 존재를 확인했다
- [ ] 3층이면 산출물 부재를 확인했다
- [ ] `verify: null` 을 통과로 말하지 않았다
