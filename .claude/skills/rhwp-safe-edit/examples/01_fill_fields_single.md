# 01 — 누름틀 한 건 (`edit fill-fields`)

층: 1. 목표: `samples/field-01.hwp` 의 회사명·작성자만 채운 산출물을
원본과 다른 경로에 남긴다. 원본 바이트는 그대로다.

권위: [single_edit.md](../references/single_edit.md) §3,
[verify_loops.md](../references/verify_loops.md) §1–5.
픽스처: [../fixtures/plans](../fixtures/catalog.json) 이 아니라 1층 봉투
[../fixtures/envelopes/fill_ok.json](../fixtures/envelopes/fill_ok.json).

## 0. 이 편에서 하지 않는 것

- `--in-place` 로 원본을 덮어쓰지 않는다.
- `fields` 없이 필드 이름을 지어내지 않는다.
- `filledCount` 만 보고 완료라고 하지 않는다.
- 다음 치환을 같은 산출물에 이어 붙이지 않는다. 이어야 하면 07 편(`run`)으로 간다.

## 1. 발견

```bash
rhwp fields samples/field-01.hwp --json
```

기대 키: `schemaVersion`, `fields[].name`, `fields[].value`.
이 샘플은 테스트가 11개 누름틀(회사명/작성자/부서명/전화번호/이메일/제목/목차1×5)로
소개한다. 에이전트는 그 숫자를 외우지 않고 이 호출의 배열을 읽는다.

동명이 있으면 배열 순서가 순번이다. `목차1` 이 다섯 번이면
`목차1[0]` … `목차1[4]` 만 지목할 수 있다.

## 2. 선확인

```bash
rhwp edit fill-fields samples/field-01.hwp \
  --data '{"회사명":"페타플로","작성자":"홍길동"}' \
  --dry-run --json
```

읽는 필드:

| 키 | 기대 |
|----|------|
| `dryRun` | `true` |
| `filledCount` | 2 |
| `filled[].name` | 회사명, 작성자 |
| `notFound` | `[]` |
| `ambiguous` | `[]` |
| `output` | **부재** |
| `changedPages` | `null` |

`out/field-filled.hwp` 를 아직 만들지 않았다. `-o` 를 붙였더라도 dry-run 은 touch 하지 않는다.

`notFound` 가 비지 않으면 data 키를 고치고 이 절을 반복한다.
`ambiguous` 가 비지 않으면 `이름[N]` 으로 고친다. 12 편.

## 3. 실행

```bash
rhwp edit fill-fields samples/field-01.hwp \
  --data '{"회사명":"페타플로","작성자":"홍길동"}' \
  -o out/field-filled.hwp --verify --json
```

기대 봉투 골격은 `fixtures/envelopes/fill_ok.json` 과 같다.

- exit 0 이고 `verify.identical == true` 이거나
- exit 3 이고 `verify.identical == false` — 산출물은 **있다**. 10 편.

`verify` 가 `null` 이면 `--verify` 를 빼먹은 것이다. 통과가 아니다.

## 4. 재독

```bash
rhwp fields out/field-filled.hwp --json \
  | jq -c '[.fields[]|select(.name=="회사명" or .name=="작성자")|{name,value}]'
```

기대: `[{"name":"작성자","value":"홍길동"},{"name":"회사명","value":"페타플로"}]`
(순서는 문서 순). 다른 필드 `value` 는 입력과 같다.

## 5. 눈검증

`changedPages` 가 배열이면 그 쪽만:

```bash
rhwp export-svg out/field-filled.hwp -o out/svg -p 0 --json
```

`null` 이면 이 절을 건너뛴다. dry-run 잔재를 렌더하지 않는다.

## 6. 원본 대조

```bash
# POSIX
cmp samples/field-01.hwp samples/field-01.hwp
# 실행 전에 떠 둔 해시와 비교하는 것이 15 편.
```

원본 경로에 `-o` 를 두지 않았으므로 원본은 그대로여야 한다.

## 7. 실패를 이 편에서 보면

| 관찰 | 편/문서 |
|------|---------|
| `notFound` 비지 않음, exit 0 | [failure_envelopes.md](../references/failure_envelopes.md) §3.1 |
| `ambiguous` | 12 편 |
| `--data @row.json` 이 CP949 | exit 1, UTF-8 로 저장 |
| 다음 편집이 남음 | 07 편 |

## 8. 명령 체크리스트

- [ ] `rhwp fields` 를 먼저 돌렸다
- [ ] `--dry-run --json` 과 실행이 같은 data / 같은 입력이다
- [ ] `-o` 가 `samples/field-01.hwp` 가 아니다
- [ ] `--verify` 를 붙였거나, 안 붙였으면 `verify: null` 을 통과로 말하지 않는다
- [ ] `notFound == []` 그리고 `ambiguous == []`
- [ ] `rhwp fields` 재독이 data 와 같다
