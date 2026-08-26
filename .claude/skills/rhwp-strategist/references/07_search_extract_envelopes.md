# 07 search / extract-data 봉투

엔진이 읽는 키만 적는다. CLI 전체를 재정의하지 않는다.

## search --json

호출:

```
rhwp search <doc.hwp> --json -- <keyword>
rhwp search <doc.hwp> --json --limit N -- <keyword>
```

에이전트가 직접 칠 때도 `--` 뒤에 키워드를 둔다. 키워드가 옵션처럼
보이면 파서가 먹는다.

엔진이 쓰는 필드:

| 키 | 쓰임 |
| --- | --- |
| `matches[]` | 각 원소가 EV 하나 |
| `matches[].text` | `quote` |
| `matches[].context` | `context` |
| `matches[].section` 등 | `copy_coords` |
| `truncated` | true 면 `truncatedSearches` 행 |
| `totalMatchCount` | 절단 기록 |
| `omittedCount` | 절단 기록 |

`matches` 가 없거나 빈 배열이면 그 키워드×문서는 0건. 실패가 아니다.

## extract-data --json

```
rhwp extract-data <doc> --kind date --json
rhwp extract-data <doc> --kind amount --json
```

엔진은 `date` 와 `amount` 만 돈다. `--kind quantity` 를 이 스킬이
요구하지 않는다(광고되어도 엔게이지먼트 파이프라인이 호출하지 않음).

| 키 | 쓰임 |
| --- | --- |
| `items[]` | 각 원소가 EV 하나 (kind=data) |
| `items[].kind` | `dataKind` |
| `items[].raw` | `quote` |
| `items[].normalized` | 숫자/ISO 정규화 |
| `items[].currency` / `unit` | 있으면 복사 |
| 좌표 키 | `copy_coords` |

금액 정규화를 에이전트가 다시 계산하지 않는다. 대장의 `normalized` 를
쓴다. "3,180백만원 ≈ 3.2조" 같은 환산은 출처 없는 전망과 같은 급으로
취급한다 — 문서가 그 환산을 갖고 있지 않으면 쓰지 않는다.

## capabilities

`info` 와 `search` 가 광고되지 않으면 엔진은 exit 1.
`extract-data`/`explain`/`scaffold` 는 광고될 때만.

```
rhwp capabilities
rhwp capabilities --json
```

두 형태를 엔진이 순서대로 시도한다. 둘 다 실패하면 명령을 추측하지
않고 중단한다.

## 픽스처 봉투

`fixtures/envelopes/` — 실측 형태를 단순화한 표본. 키가 발명되지 않았는지
계약 시험이 검사한다.

다음: [08_validate_exit.md](08_validate_exit.md).
