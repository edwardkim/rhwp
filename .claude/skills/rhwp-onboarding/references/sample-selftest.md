# 샘플 자가검증 계약

자가검증은 "이 바이너리가 평범한 문서를 읽고 JSON 봉투를 내는가"를 본다.
렌더 픽셀·한컴 일치·gym 점수는 범위 밖이다.

## 후보 순서

닥터 `SAMPLE_CANDIDATES`:

1. `samples/basic/english.hwp`
2. `samples/basic/KTX.hwp`
3. `samples/basic/BookReview.hwp`
4. `samples/2022년 국립국어원 업무계획.hwp`
5. `samples/2022년 국립국어원 업무계획.hwpx`

`--sample` 이 있으면 그 파일이 존재할 때만 이긴다. 없으면 None.

## 매직 분류 (rhwp 실행 전)

| kind | 조건 | ok |
|---|---|---|
| `missing` | 경로 없음 | false |
| `empty` | 0바이트 | false |
| `too_small` | 64바이트 미만 | false |
| `not_document` | OLE/ZIP/HWP3 시그니처 없음 | false |
| `avoid` | `samples/broken/`, `gym/`, `output/` 등 | false |
| `hwp5` | `D0 CF 11 E0 A1 B1 1A E1` | true |
| `hwpx` | `PK` | true |
| `hwp3` | `HWP Document File` | true |

`ok==false` 면 `info`/`export-text` 를 돌리지 않고 `bad_sample` FAIL.
쓰레기 입력에 파서를 매달리지 않기 위해서다.

## 임계 검사

### `selftest-info`

```bash
rhwp info <샘플> --json
```

통과: exit 0, JSON object, `format` 과 `pageCount` 존재.

### `selftest-export-text`

```bash
rhwp export-text <샘플> --json --max-chars 2000
```

통과: exit 0, JSON object, `pages` 가 길이 1 이상 배열.

## 비임계 검사 (`--skip-extra` 로 생략)

| id | 명령 | 통과 | 없으면 |
|---|---|---|---|
| `selftest-explain` | `explain --json` | `format`/`pageCount`/`summary` | SKIP (구버전) |
| `selftest-digest` | `digest --json --max-chars 500` | `schemaVersion`/`source` | SKIP |
| `selftest-inspect-injection` | `inspect injection --json` | `clean`/`signalCount` | SKIP |

비임계 FAIL 은 건강 판정(`ok`)을 뒤집지 않는다.

## 픽스처 (고의 불량)

`tools/agent_onboarding/fixtures/samples/` 는 **실패 경로용**이다.
자가검증 성공 시연에 쓰지 않는다.

| 파일 | 기대 kind |
|---|---|
| `empty.hwp` | `empty` |
| `tiny.hwp` | `too_small` |
| `not_hwp.txt` | `too_small` 또는 `not_document` |
| `text_named_hwp.hwp` | `not_document` |
| `truncated_ole.hwp` | `too_small` (매직만 8바이트) |
| `zeros.hwp` | `too_small` 또는 `not_document` |

```bash
python tools/agent_onboarding/rhwp_doctor.py --sample tools/agent_onboarding/fixtures/samples/text_named_hwp.hwp --json
# exit=1, exceptions.kind==bad_sample  (바이너리가 있을 때)
```

## 봉투 키 픽스처

`tools/agent_onboarding/fixtures/envelopes/*.json` 의 `required` 배열은
닥터 상수와 같아야 한다. 테스트가 대조한다.

| 명령 | required | 메모 |
|---|---|---|
| `info` | `format, pageCount` | 쪽수·형식. 본문 없음. |
| `export-text` | `pages` | pages[].text 는 untrusted. |
| `explain` | `format, pageCount, summary` | 셀 텍스트 없음. |
| `digest` | `schemaVersion, source` | 발췌는 앞쪽만. |
| `injection` | `clean, signalCount` | 신호 발견 ≠ 실패. |
| `fields` | `fieldCount, fields` | 읽기 전용 조사. |
| `export-tables` | `tableCount, tables` | 좌표·병합. |

## 타임아웃

- 버전: 20s
- 자가검증: 45s

타임아웃은 `selftest_timeout`. 병리 문서에 파서를 물리지 말고 샘플을 바꾼다.

## 성공 판정

1. 분류 `ok==true`.
2. `selftest-info` PASS, `selftest-export-text` PASS.
3. stdout 이 JSON 하나 (`--json` 모드).

실패 다음: [exception-bad-sample.md](exception-bad-sample.md).
