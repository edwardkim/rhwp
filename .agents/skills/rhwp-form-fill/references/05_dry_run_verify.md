# 05 — --dry-run 과 --verify

두 플래그는 새 명령이 아니다. 이미 있는 쓰기 명령에 붙는 판정이다.

원칙: **선검증은 실행과 같은 명령줄에서 `--dry-run` 하나만 빼면 실행**이
되도록 쓴다. 인자를 다시 조립하면 실수한다.

## --dry-run

파일을 쓰지 않고 변경 예정만 보고한다.

| 적용 | 효과 |
| --- | --- |
| `edit fill-fields --dry-run` | 출력 파일 없음. `output` 키 없음. `dryRun: true` |
| `batch fill --dry-run` | 행별 판정만. `--out-dir` 는 여전히 필수 |
| `edit insert-image --dry-run` | 그림 배치 예정 + overflow. 파일 없음 |

계약 (`edit_fill_fields_contract`):

- dry-run 은 지정한 `-o` 경로에 파일을 만들지 않는다
- 없는 필드 이름은 `notFound` 로 보고하고 filledCount 는 있는 키만

레시피 01 실측 (오타):

```json
{"ambiguous":[],"changedPages":null,"confusable":[],"dryRun":true,"filled":[],"filledCount":0,"notFound":["noSuchField"],"schemaVersion":"1.0","source":"…/form-01.hwp"}
```

여기서 잡고 진행한다. `notFound` 를 본 채 실행하지 않는다.

`batch fill --dry-run` 에도 `--out-dir` 가 필요한 이유: 선검증과 실행의
명령줄이 토큰 하나 차이어야 한다. 폴더 인자를 빼면 두 명령줄이 달라져
"미리 본 것과 다른 실행" 이 된다.

## --verify

저장 **직후** 산출물을 다시 읽어 IR 을 비교한다 (#3702).

```bash
rhwp edit fill-fields 신청서.hwp --data @row.json -o out.hwp --verify --json
```

통과: `verify.identical: true` 이고 `verify.diffCount: 0`.

실패: `identical: false`, **exit 3**, 산출물은 남는다. 판정은 데이터다.
봉투를 버린 채 종료 코드만 보면 무엇이 달랐는지 모른다.

플래그가 없으면 `verify` 는 `null` 이고 정상 저장은 exit 0
(`edit_verify_contract`).

`batch fill --verify` 는 행마다 같은 자기검증을 한다. 차이가 있으면
최종 종료 코드에 반영된다. 채움·저장 자체는 성공이고 stderr 에
`batch fill: … 검증 판정` 요약이 붙는다.

## 3단 루프 (공식)

```
① dry-run     파일을 만들지 않고 무엇이 바뀔지
② 실행        -o 로 산출 분리
③ 재독/verify 보고를 믿지 않고 다시 읽는다
```

02장의 최소 습관과 같다. 한 단을 건너뛰고 제출하지 않는다.

## 조합

| dry-run | verify | 파일을 쓰는가 | 차이 시 exit |
| --- | --- | --- | --- |
| 예 | 아니오 | 아니오 | — |
| 예 | 예 | 아니오 | verify 대상 파일이 없음. dry-run 이 이김 |
| 아니오 | 아니오 | 예 | 0 (verify 없음) |
| 아니오 | 예 | 예 | identical false 면 3 |

에이전트 기본값은 단건·batch 모두 **먼저 dry-run, 통과 후 verify 실행**.

## identical false 처방

1. 봉투의 `verify` 를 그대로 보고
2. `fields` 로 요청 값이 있는지 재독
3. 해당 쪽만 `export-svg` 또는 레시피 06 `render-diff`
4. 구조를 고치려고 fill 이외의 edit 를 이 스킬에서 발명하지 않음

`identical: false` 는 문서 구조 특이 케이스일 수 있다. 값이 들어 있는데
IR 비교가 다른 노드를 세는 경우가 있다. 그때도 exit 3 을 성공으로
바꾸지 않는다. 사람에게 산출 경로와 차이를 넘긴다.

## jq 게이트

```bash
rhwp edit fill-fields 신청서.hwp --data @row.json -o out.hwp --verify --json \
  | jq -e '.verify.identical and (.notFound|length==0) and (.ambiguous|length==0)' \
  > /dev/null || { echo "채움 실패 — --json 없이 재실행해 상세 확인"; exit 1; }
```

`--json` 없이 재실행하면 사람용 설명이 나온다. 자동화는 항상 `--json`.
