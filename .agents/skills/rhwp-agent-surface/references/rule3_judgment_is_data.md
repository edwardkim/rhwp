# 규칙 3 — 판정은 데이터다

플레이북 §1 규칙 3, §7. 표본: [`../fixtures/envelopes/`](../fixtures/envelopes/).

`isError` 하나만 보면 두 방향으로 틀린다.

- 성공을 실패로 읽는다 — CLI exit 3 (`identical:false`)
- 실패를 성공으로 읽는다 — `notFound` 가 찬 exit 0

## 데이터인 필드

| 필드 | 대표 명령 | CLI exit | MCP isError |
|---|---|---|---|
| `identical:false` | `ir-diff` | 3 | false |
| `replacedCount:0` | `edit replace-text` | 0 | false |
| `notFound` / `ambiguous` | `edit fill-fields` | 0 | false |
| `matchCount:0` | `search` | 0 | false |
| `invalid[]` (CSV 치수) | `csv-to-table` | 2 | true |
| `invalid[]` (계획 선검증) | `run` | 2 | **false** (MCP) |
| `status:"OVER"` / `regression` | `render-diff` | 3 | false |
| `verifyPages` 불일치 | `convert --verify-pages` | 4 | false |

`run` 과 `csv-to-table` 은 exit 2 여도 **봉투를 낸다**. 빈 stdout 이라고
단정하지 마라. 비어 있지 않으면 읽는다.

## isError 인 것만

- 없는 파일 (CLI exit 1, stdout 0바이트)
- 닫힌/모르는 `docId`
- 필수 인자 누락
- 알 수 없는 도구 (`didYouMean` + `nextCall`)
- 알 수 없는 프로필
- 병합 덮인 칸 같은 **조립** 오류 (exit 2, 같은 인자 재시도 금지)

## 치환 0건의 함정

```
rhwp edit replace-text samples/hwp3-sample.hwp --find 존재하지않는문자열ZZZ --replace X -o out/rep0.hwp --json
# replacedCount:0, exit=0, 파일 없음, 봉투에 output 키 없음
```

후속이 `output` 을 무조건 열면 여기서 깨진다. **`replacedCount > 0` 을
먼저** 본다.

## 채움 완료 조건

`filledCount` 는 완료가 아니다.

```
notFound == [] && ambiguous == []
```

반복 필드는 `이름[N]` (0 기준). `--dry-run` 으로 이 두 배열이 빈 것을
확인한 뒤에 저장한다.

## CLI exit 3 = MCP isError false

같은 `ir-diff` 가 CLI 에서는 exit 3, MCP 에서는 `isError:false` +
`identical:false` 다. MCP 소비자가 isError 만 보면 **차이가 있는 변환을
통과**시킨다. 반드시 `identical`/`diffCount`/`categories` 를 읽는다.
