# `bindings/node/tools` — 타입 생성기

`gen-types.ts` 하나가 들어 있다. **rhwp 자신이 보고하는 두 스키마**에서
`src/ir.ts` 와 `src/envelopes.ts` 를 만든다.

```bash
npm run gen:types     # 재생성
npm run gen:check     # 디스크와 다르면 exit 1 (CI 의 `생성 타입 최신 검사` 잡)
npx tsx tools/gen-types.ts --help
```

---

## 왜 생성하는가

`bindings_foundation.md` §3 이 못박은 규약이다: **필드명은 봉투 키를 기계 변환한다 —
수기 개명 금지.** 사람이 이름을 다시 붙이기 시작하면 rhwp 가 필드를 하나 더할 때마다
바인딩이 뒤처지고, **뒤처졌다는 사실조차 드러나지 않는다**. 그냥 값이 안 보일 뿐이다.

수기 인터페이스의 진짜 대가는 틀린 타입이 아니라 *조용히* 틀린 타입이다. 컴파일은
되는데 런타임에 필드가 없는 상태 — 가장 찾기 어려운 형태로 드러난다.

---

## 왜 출처가 둘인가

| 출처 | 서술하는 것 | 산출 |
|---|---|---|
| `rhwp export-ir-schema` | **문서 모델**(IR 정의 41개) | `src/ir.ts` |
| `rhwp capabilities` | **명령별 봉투**(`recordFields`) | `src/envelopes.ts` |

파이썬판(M18, `bindings/python/tools/gen_models.py`)은 IR 하나만 생성했다. 동적
언어에서는 `Envelope` 가 봉투에 있는 것을 전부 노출하므로 구조적으로 뒤처질 수
없었기 때문이다. TypeScript 는 다르다 — 사용자가 이 언어를 고른 이유의 상당 부분이
"필드 이름을 컴파일러가 확인해 준다"인데, 봉투를 `Record<string, unknown>` 으로만
주면 바인딩이 그 값어치를 통째로 버린다.

어느 한쪽만으로는 봉투 필드에 정적 타입을 붙일 수 없다. IR 로 봉투를 흉내 내는
순간 수기 매핑이 부활한다.

---

## 산출물 규약

두 파일 모두 **자동 생성물이다. 손으로 고치지 않는다** — 고치면 다음 생성에서
사라진다.

- **LF 줄바꿈**, 인터페이스·필드는 **이름순**. `interface` 는 호이스팅되므로 순서에
  의미가 없고, 이름순이어야 스키마가 조금 바뀔 때 diff 도 조금만 바뀐다.
- **전 필드 `readonly`** — 봉투는 도구가 준 관찰값이지 편집 대상이 아니다.
- **인덱스 시그니처 `readonly [key: string]: unknown`** — IR·봉투 모두 추가-전용
  진화 계약(`additionalProperties: true`)이다. rhwp 가 필드를 하나 더할 때마다 모든
  소비자가 타입 오류로 깨지면 계약이 아니라 족쇄가 된다. 동시에 이 시그니처가 각
  봉투 인터페이스를 `Envelope<T extends RawEnvelope>` 의 제약에 맞춘다.
- 주석 폭은 **칸 수 96**. 한글은 두 칸으로 세므로 글자 수 기준이 아니다.

### 타입 표기 (`tsType`)

`examples/06-ir-schema.ts` 가 출력하는 `field.tsType` 은 여기서 쓰는 표기와 같아야
한다. 예제가 보여 준 것이 곧 `src/ir.ts` 에 나온다는 약속이기 때문이다.

| 스키마 | TypeScript |
|---|---|
| `{"$ref": "#/$defs/X"}` | `X` |
| `{"type": "string"}` | `string` |
| `{"type": "integer"}` · `{"type": "number"}` | `number` |
| `{"type": "boolean"}` | `boolean` |
| `{"type": "null"}` | `null` |
| `{"const": "table"}` | `'table'` |
| `{"type": "string", "enum": [...]}` | `'a' \| 'b'` |
| `{"type": "array", "items": T}` | `readonly T[]` (유니온이면 `readonly (A \| B)[]`) |
| `{"oneOf": [...]}` | `A \| B` |
| 이름 없는 중첩 객체 | 인라인 타입 리터럴 |
| 그 밖 | `unknown` |

`oneOf` 태그 유니온(`Control`)은 판별 유니온이 된다. 변형마다 `kind` 가 **required**
이고 `const` 라서 `control.kind === 'table'` 로 좁혀진다.

이름 없는 중첩 객체를 최상위 인터페이스로 끌어올리지 않는 이유: 그 이름은 **스키마에
없는 것**이고, 생성기가 지어낸 이름은 스키마가 바뀔 때 근거 없이 흔들린다.

---

## 봉투 필드의 타입은 어디서 오나

`capabilities` 는 명령마다 **어떤 필드가 있는지**(`recordFields`)만 선언하고 타입은
말하지 않는다. 그래서 `src/envelopes.ts` 의 대부분 필드는 `unknown` 이다.

짐작한 타입을 적는 순간 그 짐작이 컴파일러의 보증으로 둔갑하고, 사용자는 검사받았다고
믿은 채 틀린 코드를 쓴다. `unknown` 은 한 번 좁히도록 강제하고, 그 지점이 곧 "여기서
계약을 가정했다"는 표시가 된다.

타입을 주는 필드는 생성기 안의 두 표에만 있다.

- `GLOBAL_FIELD_TYPES` — **여러 명령에 같은 뜻으로** 나오는 이름
  (`schemaVersion`·`source`·`pageCount`·`bytes`·`verify` …).
- `COMMAND_FIELD_TYPES` — 한 명령에만 나오거나 명령마다 뜻이 다른 이름.

두 표를 나눈 이유는 실제로 밟은 함정이다: `sections` 는 `info` 에서 구역
**개수**(`number`)지만 `digest --sections` 에서는 절 **목록**(배열)이다. 이름이 같다고
타입이 같지 않다.

표에 항목을 더할 때는 **실제 봉투에서 값을 확인하고** 넣는다. `rhwp <명령> --json` 을
한 번 돌려 보는 것으로 충분하다.

`verify` 는 `src/envelope.ts` 의 `RawVerifyReport` 를 가져다 쓰고 `| null` 을 붙인다.
`null` 은 "검증 안 함"이지 "검증 실패"가 아니다 — 이 둘을 섞으면 **검증하지 않은
저장을 통과로 읽는다.**

### 중첩 표기

`recordFields` 에 `steps[].confusable` 같은 중첩 표기가 오면 최상위 필드(`steps`)만
선언하고 원문 표기는 주석으로 남긴다. `capabilities` 는 중첩의 *모양*을 서술하지
않으므로, 환산하면 없는 필드를 있다고 선언하게 된다.

---

## `--check`

```bash
npm run gen:check
```

생성 결과가 디스크와 다르면 **exit 1** 이고, 몇 번째 줄이 어떻게 다른지 함께 찍는다.
스키마가 바뀌었는데 타입을 다시 만들지 않은 PR 을 CI 가 여기서 잡는다.

CRLF 는 정규화해서 비교한다. 윈도우 체크아웃(`core.autocrlf=true`)은 LF 산출물을
CRLF 로 펼쳐 놓는데, 그건 생성물이 낡았다는 뜻이 아니라 git 의 작업 트리 규칙이다.
정규화하지 않으면 **CI 는 통과하는데 로컬은 항상 실패하는**, 아무도 믿지 않는 게이트가
된다.

---

## 실패하면 멈춘다

끊어진 `$ref`(존재하지 않는 정의를 가리키는 필드)를 만나면 **생성을 중단한다.**
절반쯤 만들다 죽은 파일은 컴파일은 되면서 뜻은 틀린, 최악의 산출물이 된다.

rhwp 를 못 찾으면 `RHWP_BIN` 사용법까지 적어 준다. `export-ir-schema` 가 없는 옛
빌드면 그 사실을 그대로 말한다 — "스키마를 읽지 못했습니다" 로 뭉개면 사용자는
바이너리를 의심하지 않는다.

---

## `--ir-schema` / `--capabilities` (재현·이행기 전용)

두 옵션은 rhwp 를 부르는 대신 파일에서 스키마 JSON 을 읽는다.

```bash
npx tsx tools/gen-types.ts \
  --ir-schema /tmp/ir_schema.json \
  --capabilities /tmp/capabilities.json
```

평상시 경로는 **언제나 rhwp 자신**이다. 이 옵션은 두 경우를 위한 것이다.

1. 명령이 아직 머지되지 않은 브랜치에서 생성해야 할 때 (`export-ir-schema` 는 M18
   에서 들어왔다).
2. "무엇으로 생성했는지"를 손에 쥐고 확인해야 할 때.

옵션이 있다고 스키마가 둘이 되는 것은 아니다 — 파일의 내용도 결국 rhwp 가 낸 것이며,
CI 는 항상 바이너리에서 직접 뽑는다.
