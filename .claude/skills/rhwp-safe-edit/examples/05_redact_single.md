# 05 — 개인정보 마스킹 (`edit redact`)

층: 1. 되돌릴 수 없다. 기본 산출 이름도 만들지 않는다.

권위: [single_edit.md](../references/single_edit.md) §7.

## 0. 하지 않는 것

- `-o` 와 `--in-place` 없이 실행 (exit 2).
- `-o` 가 원본 자신 (exit 2).
- `--dry-run` 없이 바로 저장.
- `findings[].raw` 를 이슈·로그·채팅에 붙이기.
- `run` steps 에 `redact` 를 넣기 (없는 action).

## 1. 선확인 (필수)

```bash
rhwp edit redact 계약서.hwp --dry-run --json | jq '.findings[] | {kind, page, masked}'
```

이 호출의 `raw` 는 원문 개인정보다. 터미널에만 두고 복사하지 않는다.

첨부 가능한 사본:

```bash
rhwp edit redact 계약서.hwp --dry-run --no-raw --json > 검토용.json
```

`raw` 키가 빠진다 (`null` 이 아님). `kind`/`masked`/`page`/`charOffset` 은 남는다.

## 2. 경로 거부 표본

```
$ rhwp edit redact samples/복학원서.hwp --json
오류: 마스킹은 되돌릴 수 없습니다. 산출 경로를 -o <출력> 으로 지정하거나,
      원본을 덮어쓸 의도라면 --in-place 를 명시하세요
      (먼저 --dry-run 으로 무엇이 지워질지 확인하기를 권합니다).
exit=2
```

픽스처 [../fixtures/envelopes/redact_missing_output.json](../fixtures/envelopes/redact_missing_output.json)
은 이 stderr 요지와 exit 2 를 기록한다. stdout 은 0바이트라 JSON 봉투가 아니다.

## 3. 실행

```bash
rhwp edit redact 계약서.hwp -o 공개본.hwp --verify --no-raw --json \
  | jq '{redactedCount, findingCount, changedPages, noRaw}'
```

탐지 0건이면 출력 파일을 만들지 않는다.

`--kind ssn,card` 로 좁힐 수 있다. dry-run 과 실행의 kind 가 같아야 선확인이다.

## 4. 재독

```bash
rhwp edit redact 공개본.hwp --dry-run --no-raw --json | jq .findingCount
```

같은 kind 로 0. 원문 재스윕에 `--no-raw` 없이 돌리지 마라.

## 5. `--in-place`

사용자가 "원본을 덮어써" 라고 **명시**했을 때만.
쓰기는 원자적이라 도중 실패해도 원본이 잘리지 않는다.
에이전트 기본은 그래도 `-o`.

## 6. 탐지 범위 (v1)

ssn / card / phone(하이픈 필수, 02 또는 01x) / email.
02 외 지역번호, 여권, 계좌, 13·14·19자리 카드는 범위 밖.
형태가 맞아도 체크섬·생년월일 검증을 통과하지 못하면 탐지하지 않는다.

## 7. 체크리스트

- [ ] dry-run 을 먼저 했다
- [ ] `-o` 가 원본과 다르다
- [ ] 남길 봉투는 `--no-raw`
- [ ] 재독도 `--no-raw`
