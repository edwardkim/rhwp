# 16 — 재현 트레이스

기계 JSON 은 `fixtures/traces/T01.json` … . 이 장은 사람이 따라 칠
명령과 기대 봉투다. 값은 레시피 01·05 와 계약 테스트에서 온 것만
실측로 적고, 나머지는 같은 계약의 재현 절차다.

새 명령을 쓰지 않는다.

## T01 form-01 조사

```bash
rhwp fields samples/form-01.hwp --json
```

기대: `fieldCount: 1`, `fields[0].name == "myMsg01"`,
`textSecurity.status == "clean"`. 정지 F04.

## T02 dry-run 오타

```bash
rhwp edit fill-fields samples/form-01.hwp \
  --data '{"noSuchField":"x"}' --dry-run --json
```

기대: `dryRun: true`, `notFound: ["noSuchField"]`, `filledCount: 0`,
파일 없음. 레시피 01 실측. 정지 F06.

## T03 채움 실측

```bash
rhwp edit fill-fields samples/form-01.hwp \
  --data '{"myMsg01":"홍길동 귀하"}' \
  -o form-01_filled.hwp --json
```

기대: `filledCount: 1`, `notFound: []`, `ambiguous: []`,
`outputFormat: "hwp5"`. 레시피 01.

재독:

```bash
rhwp fields form-01_filled.hwp --json \
  | jq -e '[.fields[]|select(.name=="myMsg01")][0].value=="홍길동 귀하"'
```

## T04 verify

같은 인자에 `--verify`. 기대 `verify.identical: true`,
`diffCount: 0`. 정지 F07.

## T05 batch 2행

```
{"myMsg01":"김철수 귀하"}
{"myMsg01":"이영희 귀하"}
```

```bash
rhwp batch fill --form samples/form-01.hwp --data row1.jsonl \
  --out-dir batch_out --json
```

기대: NDJSON 2줄, `row` 0 과 1, `0001.hwp` / `0002.hwp`.
레시피 05 실측.

## T06 field-01 회사명 dry-run

```bash
rhwp edit fill-fields samples/field-01.hwp \
  --data '{"회사명":"주식회사 A"}' -o out.hwp --dry-run --json
```

기대: `dryRun: true`, `filledCount: 1`, `out.hwp` 없음.
`edit_fill_fields_contract`.

## T07 없는 필드 보고

`--data '{"회사명":"A","존재하지않는필드":"B"}' --dry-run`

기대: `notFound` 에 `존재하지않는필드`, `filledCount: 1`.

## T08 목차1 순번

```json
{"목차1[0]":"가","목차1[1]":"나","목차1[2]":"다","목차1[3]":"라","목차1[4]":"마"}
```

기대: `ambiguous: []`, `filledCount: 5`.

순번 없이 `"목차1":"가"` 만 주면 filledCount 1 + ambiguous.

## T09 fieldCount 0

```bash
rhwp fields samples/hwp3-sample.hwp --json
```

기대: `fieldCount: 0`, exit 0. 오류가 아니다. 정지 F02.

## T10 sanitize 멱등

```bash
rhwp edit sanitize filled.hwp -o 배포본.hwp --json
rhwp edit sanitize 배포본.hwp -o 재확인.hwp --json | jq .removedCount
```

두 번째 0.

## T11 HWPX 형식 보존

HWPX 입력에 `-o out.hwpx`. 기대 `outputFormat: "hwpx"`.
`-o out.hwp` 는 변환 경고. 이 스킬은 확장자를 입력과 맞춘다.

## T12 기본 출력명

`-o` 생략 시 `<입력>_filled.hwp` 가 입력 옆에 생긴다. 습관은
`output/` 로 분리.

## T13 batch 순번 이름

`--name-field` 없음. 1행 `0001`, 12행도 4자리, 10000행이면 5자리.

## T14 금지 문자

`--name-field` 값이 `홍/길` → 파일명 `홍_길`. 하위 폴더를 만들지 않음.

## T15 동명

같은 성명 두 행 → `홍길동.hwp`, `홍길동_2.hwp`.

## T16 threads 와 순서

`--threads 4` 여도 NDJSON 은 입력 행 순서. `row` 가 섞이면 계약 위반.

## T17 실패 행 잔류

한 행의 JSON 이 깨져도 그 줄에 `error` 가 남고 다음 행을 계속한다.
줄 수 = 데이터 행 수.

## T18 서식 못 염

없는 `--form` 은 시작 전 한 번만 실패. 행마다 같은 오류를 N번 내지
않는다.

## T19 insert-image overflow

`--x 90000` (A4 폭 59528 밖). 기대: `overflow` 길이 ≥ 1, 삽입은 됨.

## T20 insert-image 첫 쪽

`--page 0`. `--page 1` 은 두 번째 쪽 (P08).

## T21 100mm 환산

`--x 28346 --y 28346 --width 8504 --height 8504`. 레시피 01 주석.

## T22 verify 없음

플래그 없으면 `verify: null`, exit 0 (`edit_verify_contract`).

## T23 깨진 JSON

`--data '{이건 JSON 이 아님' --dry-run` → exit 2.

## T24 없는 파일

`없는파일-edit.hwp` → exit 1, stdout 빈, `-o` 미생성.

## T25 batch stdin 무시

`echo form.hwp | rhwp batch fill --form … --data …` 는 stdin 을 쓰지
않는다. 서식은 `--form`.

## T26 헤더만 CSV

`fixtures/data/empty_header_only.csv` → exit 2.

## T27 fields 기본 출력

`--json` 없으면 JSON 파싱이 실패해야 한다 (사람용 요약).

## T28 memo 표본

`samples/field-01-memo.hwp`. guide 또는 memo 가 비어 있지 않다.

## T29 nested 배열

모든 `fields[].location.nested` 는 배열 (`fields_json_contract`).

## T30 로고 셀 스킵

기관명 필드의 nested 가 tableCell 이고 그 셀에 그림이 있으면 `--data`
에서 그 키를 뺀다. 새 명령 없음.

## T31 보안 인계

`textSecurity.status != "clean"` 이면 fill 을 호출하지 않는다.

## T32 표 칸 인계

fieldCount 0 + 빈 셀 → set-cell. 이 스킬 종료.

## T33 폴더 선별

`find forms | rhwp batch fields --json`. 이건 fill 축이 아님.

## T34 재독 대조

fill 후 `fields` 로 value 비교. 보고만 믿지 않음.

## T35 jq 단건 게이트

19장 스크립트. identical ∧ 빈 배열.

## T36 jq batch 게이트

name-field 제외 후 실패 행 0건.

## T37 dry-run 원본 불변

입력 mtime/hash 불변. `-o` 경로 없음.

## T38 실패 원본 불변

exit 1 경로에서 입력 불변, 출력 미생성.

## T39 keep-preview

`--keep-preview` 는 이미지. 텍스트 미리보기는 대상.

## T40 본문 export-text 동일

sanitize 전후 `export-text` 의 text 가 같다.

## 재현 방법

```bash
cargo build --release
./target/release/rhwp fields samples/form-01.hwp --json
```

픽스처 JSON 의 `argv` 를 그대로 쓴다. 트레이스가 `usesExistingCommand:
true` 이면 구현을 추가하지 말고 위 가족 중 하나로 환원한다.
