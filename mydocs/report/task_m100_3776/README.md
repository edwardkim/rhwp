---
kind: report
status: active
canonical: mydocs/report/task_m100_3776/README.md
last_verified: 2026-08-02
---

# #3776 처리 기록 — M19 rhwp-node TypeScript 바인딩 + export-capabilities-schema

실행 원문은 [`evidence.txt`](evidence.txt)에 있다. 아래 수치는 전부 그 파일과 대조된다.

## 문제

`bindings_foundation.md` §4 가 M19 착수 조건으로 **"M18 의 봉투 매핑 규약 재사용 판정
(napi vs WASM 비교표 갱신)"** 을 못박았다. M18(#3762)이 PR #3775 로 제출되면서 그 조건이
열렸다 — 판단의 근거가 될 실측이 생겼다는 뜻이다.

지금 하는 이유는 둘이다. 첫째, 바인딩은 **본체 계약이 굳어 있을 때 얇다.** 명령 표면이
흔들리는 동안 언어를 늘리면 뒤처짐을 언어 수만큼 반복해서 갚는다. `capabilities` 자기서술과
`--json` 봉투 규약이 계약 테스트로 고정된 지금이 가장 싼 시점이다. 둘째, TypeScript 는
파이썬과 **다른 것을 요구한다** — 사용자가 이 언어를 고른 이유의 상당 부분이 "필드 이름을
컴파일러가 확인해 준다"인데, 봉투를 `Record<string, unknown>` 으로만 주면 그 값어치를
통째로 버린다. 그래서 M19 는 M18 의 재탕이 아니라 **타입 생성 축이 하나 더 붙는다.**

## 표면 판정 — napi vs WASM, 그리고 서브프로세스

`bindings_foundation.md` §4 가 요구한 비교표 갱신의 결론이다.

| 표면 | 성능 | 배포 | 새로 지는 책임 | 판정 |
|---|---|---|---|---|
| CLI 서브프로세스 | 호출당 기동(수십 ms) + 재파싱 | 바이너리 하나 | 봉투 파서뿐 | **채택 (1층)** |
| `mcp-serve` 세션 | 재파싱 회피 | 동일 | 세션 도구 계약 재사용 | **병용 (2층)** |
| napi 네이티브 애드온 | 최고(인프로세스) | Node ABI × OS × arch 프리빌드 | ABI 안정성·메모리 규약·릴리스 매트릭스 | 수요 실증 후 승격 |
| WASM(`@rhwp/editor`) | 중간 | JS 생태계 한정, 파일 시스템 없음 | 번들 로딩 | **브라우저 전용 병용** |

**서브프로세스를 고른 근거**는 추측이 아니라 실측이다. M18 이 같은 경로로 **CI 에서
251건을 통과**시켜 "얇은 재포장"이 실제로 성립함을 증명했다. 언어가 하나 늘었다고 표면
종류를 바꿀 이유가 없고, 바꾸면 M18 에서 얻은 증거를 버리고 처음부터 다시 쌓아야 한다.

**napi 를 미룬 근거**는 비용의 성격이다. napi 는 Node ABI × OS × arch 매트릭스를 **새로
떠안는다** — 한 번 배포하면 되돌릴 수 없는 종류의 부채다. 그 비용은 인프로세스 성능이
**실제 병목으로 실증된 뒤에** 치르는 것이 맞다. 먼저 치르면, 병목이 아니었다는 사실을
알게 됐을 때 남는 것은 유지해야 할 배포 표면뿐이다. 지금 우리에겐 "느려서 못 쓰겠다"는
사용 보고가 **한 건도 없다.** 없는 문제를 되돌릴 수 없는 방법으로 푸는 것이 최악이다.

**WASM 을 병용하는 근거**는 물리적 제약이다. 브라우저에는 서브프로세스가 없다. 그래서
WASM 어댑터를 두되 Node 경로와 **같은 `RhwpClient` 인터페이스**를 구현하게 했다 — 환경마다
전혀 다른 API 를 주면 같은 업무 로직을 두 벌 유지하게 된다. 인터페이스는 같지만 **가능
범위는 다르다**(브라우저는 파일 쓰기 불가). 그 차이는 숨기지 않고 문서에 표로 못 박았다.

## 구현

### 1. `export-capabilities-schema` (Rust)

`capabilities` **산출물 자체의** JSON Schema(2020-12)를 기계 산출한다.
`export-ir-schema`(M18)가 **문서 모델**의 자기서술이라면 이쪽은 **명령 표면**의 자기서술이다.

- `src/capabilities_schema.rs` — 정의 **18개**(`schema` 9 + `mcpSchema` 9), 루트 `Capabilities`.
- 봉투: `{schemaVersion, capabilitiesSchemaVersion, dialect, definitionCount, schema, mcpSchema}`.
  `--bare` 는 봉투 없이 본문만, `-o` 는 파일 산출 (`export-ir-schema` 와 같은 규약).
- `tests/capabilities_schema_contract.rs` — `#[test]` **17건**이 스키마 건전성을 고정한다.

둘을 합치지 않은 이유: 문서 모델이 바뀔 때 명령 표면 타입까지 재생성되면 무엇이 왜
바뀌었는지 추적할 수 없다. 두 축은 서로 다른 속도로 움직인다.

### 2. 3층 — 표면이 그대로 API 가 된다

| 층 | TypeScript | 대응 rhwp 표면 |
|---|---|---|
| 1층 무상태 | `await info(path)` 등 25개 명령 | CLI `--json` |
| 2층 세션 | `await openDocument(path)` | `mcp-serve` 세션 도구 |
| 3층 계획 | `new Plan(...).check()/.run()` | `rhwp run` |

층 이름을 새로 짓지 않은 것은 M18 과 같은 판단이다 — rhwp 문서를 읽은 사람이 API 를 바로
이해하고, 반대도 성립한다.

노출 기준은 **손으로 고른 목록이 아니라 `capabilities` 의 `json` 선언**이다. 최초 측정
시점에 명령 55개 중 `json` 선언은 24개였고, 작업 중 `render-diff --json` 이 들어와
**25개**가 됐다. 바인딩이 감싸는 것은 정확히 그 25개다. 나머지 30개(`dump-*`·`hwp5-*`
진단 계열, `mcp-serve`)는 함수로 없다. 이 기준을 코드가 아니라 자기서술에 두었기 때문에
본체에 명령이 늘면 계약 패리티 가드가 뒤처짐을 잡는다.

**단, 명령이 열리는 것과 그 명령의 플래그가 전부 열리는 것은 다르다.** `capabilities` 의
`flags` 는 "이 명령이 이 플래그를 파싱한다"까지만 말하고 **`--json` 모드에서 그 플래그가
무엇을 하는지는 말하지 않는다.** 실물을 돌려 보니 세 자리가 갈렸다.

| 닫은 것 | 실측 | 이유 |
|---|---|---|
| `export-text` · `export-structure` 의 `-o` | `--json` 모드에서 **조용히 무시**(디렉터리조차 안 생긴다) | 받아 주면 "저장했다"는 거짓말이 된다 |
| `export-tables` 의 `-o` | stdout 이 사람용 문장으로 **바뀐다** | 봉투 계약이 깨져 `runJson` 이 `ProtocolError` 를 던진다. **본체 쪽 별도 이슈감** |
| `render-diff --batch` | NDJSON 스트림 | 한 함수의 반환 타입은 하나여야 한다 |

가른 기준은 플래그 이름이 아니라 **봉투 계약을 지키는가**다. 같은 `-o` 라도
`export-capabilities-schema` 는 저장 후에도 stdout 이 봉투를 유지하므로(`output`·`bytes`)
`out` 을 열어 뒀다.

### 3. 타입 생성기 — 두 출처

파이썬판(`tools/gen_models.py`)은 IR 하나만 생성했다. 동적 언어에서는 `Envelope` 하나가
봉투에 있는 것을 전부 노출하므로 **구조적으로 뒤처질 수 없었기** 때문이다. TypeScript 는
다르다. 그래서 `tools/gen-types.ts` 는 두 곳을 읽는다.

| 출처 | 서술하는 것 | 산출 | 정의 수 |
|---|---|---|---|
| `rhwp export-ir-schema` | 문서 모델(IR) | `src/ir.ts` | 41 (interface 39 + alias 2) |
| `rhwp capabilities` | 명령별 봉투 `recordFields` | `src/envelopes.ts` | 26 (interface 25 + alias 1) |

어느 한쪽만으로는 봉투 필드에 정적 타입을 붙일 수 없다. IR 로 봉투를 흉내 내는 순간
수기 매핑이 부활하고, 수기 매핑은 반드시 뒤처진다. `npm run gen:check` 는 생성 결과가
디스크와 다르면 exit 1 이다 — 스키마가 바뀌었는데 타입을 다시 만들지 않은 PR 을 CI 가
잡는다. 낡은 생성 타입은 "컴파일은 되는데 런타임에 필드가 없는" 가장 나쁜 형태로 드러난다.

### 4. 브라우저 어댑터

`@rhwp/node/browser` 의 `createBrowserClient(wasm)` 가 `@rhwp/editor` WASM 을 **같은 봉투
타입**으로 감싼다. WASM 모듈은 호출자가 로드해 넘긴다 — 번들 크기를 바인딩이 강제하지
않기 위해서다.

## 개발 중 가드·테스트가 잡은 실제 결함

이 절이 이 기록에서 가장 값진 부분이다. **아홉 건 모두 사람이 눈으로 읽어서는 못 잡았고,
전부 "조용히 잘못 도는" 종류였다.**

| 결함 | 잡은 것 |
|---|---|
| `errors.ts` 인덱스 접근 2건이 `noUncheckedIndexedAccess` 에서 컴파일 불가 | 타입 생성기 에이전트의 tsc 검사 |
| `envelope.ts` `children()` 이 배열 항목을 안 걸러 `TypeError` | 단위 테스트 설계 |
| `extractPages` 가 존재하지 않는 `--pages` 를 보내 **항상** exit 2 | 통합 테스트 실물 실행 |
| `renderDiff` 가 `--json` 미지원 명령을 감쌈 | 통합 테스트 실물 실행 |
| `exportHwpx`/`convert` 의 `-o` 가 **항상** exit 2 (출력은 위치 인자) | 표면 확장 에이전트 실물 실행 |
| `extractPages` JSDoc 이 0 기준이라 했으나 실제는 **1 기준** | 표면 확장 에이전트 실물 실행 |
| `Plan.check()` 가 미지원 rhwp 에서 **실제 편집·저장을 수행** | 단위 테스트 에이전트의 rhwp 소스 대조 |
| `src/index.ts` 가 `renderDiff` 재수출 누락 | `index.test.ts` 재수출 대조 |
| `exactOptionalPropertyTypes` 가 `errors.ts` 를 깨뜨림 | 패키징 에이전트 |

세 건(`extractPages --pages`, `exportHwpx`/`convert` 의 `-o`, `extractPages` 의 쪽 기준)은
**같은 뿌리**다: `capabilities` 의 `flags` 선언을 읽고 "이 명령은 이 플래그를 받는다"까지는
맞게 옮겼는데, **플래그가 실제로 무엇을 하는지는 선언에 없다.** 그 간극은 실물을 돌려야만
드러난다. 자기서술을 노출 기준으로 삼는 설계(§D11)의 한계선이 정확히 여기다.

### `extractPages` — 모든 호출이 실패하고 있었다

초기 시그니처는 `extractPages(path, pages: string, {out})` 로 `--pages "2-4"` 를 조립했다.
그런데 rhwp 의 `extract-pages` 는 `--from`/`--to` 만 받는다 — `capabilities` 가
`flags: ["--from","--to","--json"]` 으로 그렇게 선언한다. `--pages` 는 `digest` 쪽 어휘다.

```
$ rhwp extract-pages samples/2010-01-06.hwp --pages "1-2" -o out.hwp --json
알 수 없는 옵션: --pages
exit=2
```

**이 함수의 모든 호출이 예외 없이 실패하고 있었다.** 시그니처를
`extractPages(path, from: number, to: number, {out})` 로 고쳐 `--from N --to M` 을 보낸다.
문자열을 받아 바인딩이 내부에서 쪼개는 설계도 가능했지만, 그러면 `"2-4"`·`"2..4"`·`"2,4"`
중 무엇이 맞는지를 **바인딩이 새로 정하게 된다** — 재포장이 아니라 새 표면이다.

같은 실행에서 두 가지가 더 드러났다. `out` 은 타입상 선택이지만 rhwp 는 출력 경로 없이는
exit 2 이고(원본을 덮어쓰지 않는 것이 이 명령의 안전 규약이다), `pagesBefore: 6 →
pagesAfter: 2` 처럼 **결과 쪽수가 요청 범위와 다를 수 있다**(문단 단위로 지우고 레이아웃이
다시 흐른다). 둘 다 문서에 반영했다.

### `renderDiff` — 같은 가드에 두 번, 정반대 방향으로 걸렸다

`capabilities` 의 `render-diff` 항목에는 `json` 키가 **아예 없다**(`category: "diagnostic"`,
키는 `category`·`name`·`summary` 셋뿐). 실제로도 `--json` 을 주면 exit 2 다.

```
$ rhwp render-diff samples/2010-01-06.hwp --json
오류: 알 수 없는 옵션: --json
exit=2
```

제거했다. 없는 계약을 감싸면 "호출은 있는데 항상 실패하는 API" 가 되고, 그것은 없는 것보다
나쁘다 — 사용자가 자기 코드를 의심하는 데 시간을 쓴다.

**그리고 이 문서 작업 도중 상황이 뒤집혔다.** 같은 브랜치에서 `render-diff --json` 이
[#3719](https://github.com/edwardkim/rhwp/issues/3719) §6-2 로 구현됐고, 재측정 결과
`capabilities` 가 이제 `json: true` 와 `recordFields` **18개**를 선언한다
(`json` 선언 명령 24 → **25**개). 실물 실행도 봉투를 낸다.

```
$ rhwp render-diff samples/2010-01-06.hwp --json | head -c 120
{"hardStructPages":0,"maxDisp":6.0,"mode":"roundtrip","overPages":6,"pageCountA":6,…
exit=3        # 회귀 검출은 판정이므로 exit 3
```

즉 한동안 **반대 방향의 결함**이었다 — 있는 계약을 바인딩이 빠뜨렸다. 정반대인 두 결함이
같은 가드 하나에 걸린다는 점이 노출 기준을 자기서술에 둔 설계의 값어치다.

**지금은 래퍼가 있다.** `commands.ts` 에
`renderDiff(path, pathB?, {via, page, maxDisp, throwOnVerdict})` 를 두고 `index.ts` 가
재수출한다. `pathB` 를 주면 두 파일 비교(`mode: "pair"`), 없으면 자기 라운드트립
(`mode: "roundtrip"`)이다.

옮기면서 그대로 보존한 계약 두 가지를 적어 둔다.

**① 회귀는 예외가 아니라 `status`·`regression` 필드다.** 도구는 정상 동작했고 문서에
대한 판정이 실패한 것이므로, `--verify` 와 같은 규약으로 값으로 전한다
(`throwOnVerdict` 를 준 경우에만 `VerdictFailed`).

**② 종료 코드가 모드마다 다르다.**

| 모드 | 회귀 검출 시 | 근거 |
|---|---|---|
| `--json` | **exit 3** | CI 가 "렌더가 깨졌다"와 "파일을 못 읽었다"를 같은 신호로 받으면 안 된다 |
| 사람용 출력 | **exit 1**(종전 그대로) | 이미 1 을 실패로 읽는 CI 스크립트를 깨지 않기 위해서다 |

새 의미론(3)을 `--json` 소비자에게만 준 것은 본체의 판단이고 바인딩은 그것을 옮길 뿐이다.
바인딩이 두 모드를 하나로 "정리"했다면 그 순간 재포장이 아니라 새 표면이 된다.

**하지 않기로 한 것 — `--batch`.** CLI 의 폴더 일괄 축은 감싸지 않았다. 계약이 깨져서가
아니라 **반환 타입이 달라서**다: 단건은 한 줄 봉투이고 배치는 NDJSON 스트림이다. 한
함수가 둘 다 처리하면 호출자가 받은 값이 봉투인지 배열인지 **타입으로 알 수 없게 되고**,
그러면 TypeScript 를 고른 이유가 사라진다. 폴더 일괄은 `runNdjson` 이라는 이미 있는 문을
쓴다.

### `exportHwpx`·`convert` — `-o` 를 모르는 두 명령

산출 명령 대부분이 `-o <경로>` 를 받으므로 변환 두 명령도 그럴 것이라 옮겼는데, 실물은
그렇지 않았다.

```
$ rhwp export-hwpx samples/2010-01-06.hwp -o out.hwpx --json
오류: 알 수 없는 옵션: -o
exit=2
```

**두 명령의 산출 경로는 위치 인자다.** `exportHwpx` 는 생략하면 `<입력 stem>.hwpx` 로
가지만 `convert` 는 기본 경로가 없어 **필수**다. 바인딩은 `convert` 에서 `out` 이 없으면
프로세스를 띄우기 전에 `UsageError` 를 던진다 — CLI 사용법 덤프는 "인자가 틀렸다"까지만
말하지만, 여기서 던지면 **무엇이 빠졌는지 이름으로** 알릴 수 있다.

이 결함도 `extractPages --pages` 와 같은 자리에서 났다. `capabilities` 는 플래그 목록을
주지만 **위치 인자를 서술하지 않는다.** 선언만 읽으면 절대 드러나지 않는다.

### `extractPages` — JSDoc 이 0 기준이라 말하고 있었다

`--pages` → `--from`/`--to` 로 고친 뒤에도 주석은 "0 기준"이라 적혀 있었다. 실물은 다르다.

```
$ rhwp extract-pages samples/2010-01-06.hwp --from 0 --to 1 -o out.hwp --json
오류: 쪽 범위가 잘못됐습니다 … (1 기준)
exit=1
```

**rhwp 전체에서 유일하게 1 기준인 축이다.** 다른 명령의 `page` 인자와 `digest` 의 `pages`
창은 전부 0 기준이다. 이것이 위험한 이유는 **틀려도 실패하지 않기 때문**이다 — `search`
결과의 `page`(0 기준)를 그대로 넘기면 오류 없이 **한 쪽 밀린 문서**가 나온다. 바인딩이
임의로 통일하지 않은 것은 재포장이 계약을 바꾸지 않는다는 원칙 때문이고, 대신 시그니처
JSDoc·API 레퍼런스·요리책의 "자주 하는 실수"에 세 번 적었다.

### `src/index.ts` — 함수는 있는데 내보내지 않았다

`renderDiff` 를 `commands.ts` 에 추가하고 `index.ts` 재수출을 빠뜨렸다. `index.test.ts`
가 `commands.ts` 의 공개 심볼과 `index.ts` 재수출 목록을 대조해 잡았다.

이것이 조용한 종류인 이유: `commands.ts` 를 읽는 사람에게는 함수가 **있다.** 내부 테스트도
`commands.ts` 를 직접 import 하면 통과한다. 실패하는 것은 `import { renderDiff } from
'@rhwp/node'` 를 쓰는 **패키지 사용자뿐**이고, 그 경로는 저장소 안에서 아무도 밟지 않는다.
재수출 대조 테스트가 없으면 배포 후에야 드러났을 결함이다.

### `Plan.check()` — 조용한 데이터 사고를 막는 가드

가장 위험했던 건이다. `check()` 는 계획서에 `dryRun: true` 를 실어 보내는데, **`--dry-run`
을 모르는 rhwp 는 그 필드를 그냥 무시하고 편집·저장한다.** 실측으로 재현했다.

```
$ ls scratchpad/plan_out.hwp
ls: cannot access '…/plan_out.hwp': No such file or directory

$ rhwp run --plan-json '{…,"dryRun":true,"steps":[{"action":"replace_text",…}]}' --json
{"assertions":{…},"changedPages":[0],"input":"samples/field-01.hwp",
 "output":"…/plan_out.hwp","outputFormat":"hwp5",…,
 "steps":[{"action":"replace_text","find":"전략 기획서","replacedCount":1,"step":0}],"verify":null}
exit=0

$ ls -l scratchpad/plan_out.hwp
-rw-r--r-- 1 swsz9 197609 473600 Aug  2 10:50 plan_out.hwp
```

exit 0, 경고 없음, 봉투에 `dryRun` 필드 없음 — 그리고 **473,600 바이트짜리 편집본이
생겼다.** 호출자는 "검사만 했다"고 믿는다. 실패도 예외도 없이 문서가 바뀌므로, 눈으로
열어보기 전까지 아무도 모른다. 이것이 조용한 데이터 사고의 교과서적 형태다.

그래서 `check()` 는 계획서를 보내기 **전에** `capabilities` 로 `run` 이 `--dry-run` 을
선언하는지 확인하고, 아니면 실행하지 않고 예외를 던진다. 이 브랜치의 rhwp 는
`run` 의 `flags` 가 `["--json","--plan-json"]` 이므로 **현재는 항상 던진다** — #3759 가
머지돼야 풀린다. 실행으로 대체할지는 사람이 `run()` 을 명시적으로 불러 정한다.

"미지원이면 그냥 실행" 이 편의처럼 보이지만, 그 편의의 대가는 **호출자가 안전하다고
믿는 경로에서 파일이 바뀌는 것**이다. 바인딩이 지불해도 되는 대가가 아니다.

### `Envelope.children()` — 배열 안의 배열

`children(key)` 는 배열 필드를 봉투 배열로 감싼다. `Envelope` 생성자는 배열을 거부하므로,
항목 중 다시 배열인 것(`[[1,2]]`)을 걸러 내지 않으면 감싸는 자리에서 `TypeError` 가 난다 —
**조회하려던 필드와 아무 상관 없는 곳에서** 터지므로 원인 추적이 가장 어려운 형태다.
객체가 아닌 항목은 봉투가 될 수 없으니 조용히 제외한다. `get()` 이 오타에 예외를 던지는
것과 규칙이 다른 이유: 배열에 이질적 항목이 섞이는 것은 **호출자의 실수가 아니라 봉투의
모양**이고, 호출자가 할 수 있는 일이 없다.

### 타입 검사 설정 2건

`noUncheckedIndexedAccess` 와 `exactOptionalPropertyTypes` 는 켜면 아프지만, 켜지 않으면
"`arr[0]` 이 항상 있다고 가정한 코드"와 "`undefined` 를 넣어도 되는 선택 속성"이 조용히
통과한다. 둘 다 `errors.ts` 를 깨뜨렸고, 둘 다 고쳤다. 바인딩이 사용자에게 타입 안전을
팔면서 자기 컴파일은 느슨하게 하는 것은 앞뒤가 맞지 않는다.

## 검증

숫자는 전부 [`evidence.txt`](evidence.txt) 의 실행 원문과 일치한다.

| 항목 | 실측 |
|---|---|
| rhwp 버전 | `0.8.2`, capabilities `schemaVersion: 1.0` |
| 명령 표면 | 55개 중 `json` 선언 **24개** / 미선언 31개 (작업 중 `render-diff --json` 이 들어와 **25 / 30** 으로 바뀜 — 증적 §11). 바인딩이 감싸는 것은 그 25개 전부 |
| `export-capabilities-schema` | 정의 **18개**(`schema` 9 + `mcpSchema` 9), 루트 `#/$defs/Capabilities` |
| Rust 계약 테스트 | `capabilities_schema_contract.rs` `#[test]` **17건** |
| 생성 타입 | `src/ir.ts` 정의 **41**, `src/envelopes.ts` 정의 **26** |
| 패키지 규모 | `src/` **13파일 4,936줄**, `test/` **19파일 5,597줄** (최종 측정) |
| 문서 | `docs/` 5종 **2,937줄** + README **341** + CHANGELOG **146** |
| 예제 | **12편 1,524줄** |
| 테스트 | 선언 **323개**(단순 310 + `it.each` 13) — 단위 11파일 / 통합 5파일 (최종 측정) |
| 타입 검사 | `tsc --noEmit -p tsconfig.json` (TypeScript **7.0.2**) **오류 0건** |
| TS 구문 검사 | 대상 `.ts` **전 파일 통과** |
| 링크 검사 | `agent_knowledge_map.md` 이상 없음 |

타입 검사는 느슨한 설정에서의 0건이 아니다. `tsconfig.json` 이 `strict` 에 더해
`noUncheckedIndexedAccess`·`exactOptionalPropertyTypes`·`noImplicitOverride`·
`noUnusedLocals`·`noUnusedParameters`·`noImplicitReturns`·`useUnknownInCatchVariables` 를
전부 켜고, `include` 가 `src/`·`test/`·`tools/`·`examples/` 를 모두 덮는다.
**예제까지 같은 게이트를 통과한다** — 예제가 소비자와 똑같이 `@rhwp/node` 로 import 하도록
`paths` 를 자기 소스로 매핑했기 때문이다. 다른 import 경로를 쓰는 예제는 실제로 동작하는지
증명하지 못한다.

엄격 옵션 두 개는 실제로 결함을 냈다(위 표의 `errors.ts` 2건). 켜지 않았다면
"`arr[0]` 이 항상 있다고 가정한 코드"와 "`undefined` 를 넣어도 되는 선택 속성"이 조용히
통과했을 것이다.

### 이 PC 에서 돌리지 못한 것 — 그대로 적는다

- **vitest 미실행.** 의존성은 로컬에 설치돼 있어(`node_modules`, 잠금 파일은 커밋하지
  않는다) `tsc` 게이트는 위 표대로 **실제로 돌렸다.** 다만 테스트 스위트 전체 실행은
  이 기록의 범위에서 돌리지 않았다. 위 "선언 181개" 는 **정적 계수**이며, `it.each` 가
  실행 시 표 행 수만큼 늘어나므로 **실제 실행 건수는 그보다 많다.** 실행 결과의 권위는
  `.github/workflows/node-binding.yml` 의 4개 잡(단위 매트릭스 18/20/22 · 통합 ·
  패키지 빌드 · 생성 타입 최신 검사)이다.
- **Rust 계약 테스트 미실행.** 이 PC 의 Windows SDK `dbghelp.lib` 손상으로
  `release-test` 프로파일 링크가 실패한다(`CVT1107` → `LNK1123`). `release` 프로파일
  바이너리는 이미 빌드돼 있어 위의 실물 실행에는 썼다. 계약 테스트 검증도 CI 가 권위다.

"통과했다"고 쓸 수 있는 것만 통과했다고 썼다. 돌리지 못한 것을 돌린 것처럼 적으면 이
기록 전체의 신뢰가 사라진다.

### 인코딩 함정 (윈도우, M18 과 같은 자리)

증적 수집 중 파이썬 집계 스크립트의 출력을 파일로 리다이렉트하자 한글이 깨졌다. 원인은
바인딩도 rhwp 도 아니고 **파이썬이 플랫폼 기본 인코딩(cp949)으로 stdout 을 연 것**이다.
`PYTHONIOENCODING=utf-8` 로 해결했고, `evidence.txt` 는 그 설정으로 다시 수집했다.

M18 은 같은 뿌리(윈도우 파이썬의 기본 인코딩)에 **가짜 픽스처 쪽에서** 두 번 걸렸다.
이번엔 픽스처가 아니라 측정 스크립트였다 — 즉 이 함정은 특정 코드가 아니라 **윈도우에서
파이썬으로 rhwp 출력을 다루는 모든 자리**에 있다. rhwp(Rust)는 콘솔 코드페이지와 무관하게
항상 UTF-8 이므로 본체는 무고하고, 값이 깨지면 먼저 의심할 곳은 도구 쪽이다.

## 등재

- `bindings/README.md` — Native/csharp/swift 만 있던 목록에 **CLI 서브프로세스 바인딩**
  절을 만들고 `python/`(M18, 제출·머지 대기)과 `node/`(M19, 진행)을 추가했다.
  napi 를 미룬 근거도 한 문단으로 남겼다.
- `mydocs/manual/agent_knowledge_map.md` — §1-4 "다른 언어에서 쓰려는가 — 바인딩 가이드"
  를 신설하고, §6 에 `capabilities_schema_contract.rs` 행을 추가했다.

## 남은 것

- **M18(#3762, PR #3775) 머지가 선행 조건이다.** 이 브랜치의 rhwp 에는
  `export-ir-schema` 가 없어 `exportIrSchema()`·`irSchema()` 가 exit 2 이고,
  `npm run gen:check` 도 M18 머지 후에야 이 저장소 CI 에서 의미가 있다.
  `bindings/python` 은 이 브랜치에 존재하지 않는다(`git ls-files` 0건).
- **`Plan.check()` 는 #3759 머지 전까지 항상 던진다.** 설계대로의 동작이지만, 계획 층을
  쓰려는 사용자에게는 그때까지 3층이 `run()` 만 남는다는 뜻이다.
- **npm 미배포.** 이름(`@rhwp` 스코프) 선점 확인이 필요하다.
- **바이너리 미동봉.** 파이썬 1호와 같다 — `RHWP_BIN` 또는 `PATH` 로 제공해야 한다.
  플랫폼별 바이너리 배포(선택적 `optionalDependencies`)는 이번 범위 밖이다.
- **`mydocs/manual/node_binding_guide.md` 의 링크 2건(13행·266행)이 깨져 있다** —
  `python_binding_guide.md` 가 M18 산출물이라 이 브랜치에 없다. M18 이 먼저 머지되면
  자연 해소되고, M19 를 먼저 머지하려면 링크를 풀어야 한다.
- **`exportTables -o` 는 별도 업스트림 이슈감이다.** `--json` 을 준 호출에 `-o` 를 더하면
  stdout 이 사람용 문장("표 추출 완료: N개 → 경로")으로 바뀌어 봉투 계약이 깨진다
  (`runJson` 이 `ProtocolError`). 바인딩은 옵션을 열지 않는 것으로 **회피**했을 뿐
  수정하지 않았다 — `--json` 을 준 호출에서 출력 형식이 바뀌는 것은 명령 하나의 사정이
  아니라 봉투 계약 전체의 예외이므로, 본체 쪽에서 다뤄야 한다.
- **`renderDiff --batch` 는 감싸지 않았다 — 결함이 아니라 결정이다.** 단건은 한 줄 봉투,
  배치는 NDJSON 스트림이라 반환 타입이 다르다. 한 함수가 둘 다 처리하면 호출자가 받은
  값을 타입으로 알 수 없다. 폴더 일괄은 `runNdjson` 으로 부른다. 근거는
  `docs/DESIGN.md` D14 에 남겼다.
- **napi 승격**은 인프로세스 성능이 실제 병목이라는 사용 보고가 나온 뒤다. 지금은 없다.

## 관련 문서

- [`node_binding_guide.md`](../../manual/node_binding_guide.md) — 설계·규약 canonical
- [`bindings_foundation.md`](../../tech/bindings_foundation.md) — 표면 판단 매트릭스의 권위
- [`agent_knowledge_map.md`](../../manual/agent_knowledge_map.md) §1-4 — 언어별 진입점
