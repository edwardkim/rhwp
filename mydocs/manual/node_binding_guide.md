---
kind: canonical
status: active
canonical: mydocs/manual/node_binding_guide.md
last_verified: 2026-08-02
---

# Node/TypeScript 바인딩 가이드 — `bindings/node`

M19(#3776)의 산출물인 npm 패키지 `@rhwp/node` 의 설계·규약·유지보수 문서다.
설계 근거의 권위는 [`bindings_foundation.md`](../tech/bindings_foundation.md) 이고,
본 문서는 그 결정을 **실제로 구현한 방식**과 앞으로 지켜야 할 계약을 담는다.
파이썬 1호의 대응 문서는 [`python_binding_guide.md`](python_binding_guide.md) 다 —
두 문서가 어긋나면 계약이 언어마다 갈린 것이므로 어긋난 쪽을 고친다.

## 1. 대원칙 — 바인딩은 새 표면이 아니다

> **바인딩은 기존 계약의 재포장이다.**

TypeScript 쪽에서 판정 로직을 새로 만들면 rhwp 본체와 바인딩이 서로 다른 답을 내는
순간이 온다. 그래서 바인딩이 하는 일은 셋뿐이다.

1. rhwp 프로세스를 띄우고 인자를 조립한다.
2. stdout 봉투를 파싱해 타입이 붙은 객체로 감싼다.
3. 종료 코드를 TypeScript 예외 체계로 옮긴다.

**하지 않는 일**: 문서 파싱, 좌표 계산, 유효성 판정, 재시도 정책.
전부 rhwp 본체가 이미 하고 있고, 두 번 하면 어긋난다.

## 2. 표면 판단 — napi vs WASM (M19 착수 조건)

`bindings_foundation.md` §4 가 M19 착수 조건으로 정한 "napi vs WASM 비교표 갱신"의 결론이다.

| 표면 | 성능 | 배포 | 새로 지는 책임 | 판단 |
|---|---|---|---|---|
| CLI 서브프로세스 | 호출당 기동(수십 ms) + 재파싱 | 바이너리 하나 | 봉투 파서뿐 | **채택(1층)** |
| mcp-serve 세션 | 재파싱 회피 | 동일 | 세션 도구 계약 재사용 | **병용(2층)** |
| napi 네이티브 애드온 | 최고(인프로세스) | Node ABI × OS × arch 프리빌드 | ABI 안정성·메모리 규약·릴리스 매트릭스 | 수요 실증 후 승격 |
| WASM(`@rhwp/editor`) | 중간 | JS 생태계 한정, 파일 시스템 없음 | 번들 로딩 | **브라우저 전용 병용** |

**서브프로세스를 고른 근거**: M18(#3762)이 같은 경로로 **CI 에서 251건을 통과**시켜
"얇은 재포장"이 성립함을 실증했다. 언어가 하나 늘었다고 표면 종류를 바꿀 이유가 없다.

**napi 를 미룬 근거**: Node ABI × OS × arch 매트릭스를 새로 떠안는 비용은, 인프로세스
성능이 **실제 병목으로 실증된 뒤**에 치르는 것이 맞다. 먼저 치르면 병목이 아니었다는
사실을 알게 됐을 때 되돌릴 수 없는 배포 표면만 남는다.

**WASM 을 병용하는 근거**: 브라우저에는 서브프로세스가 없다. 그래서 WASM 어댑터를 두되,
Node 경로와 **같은 `RhwpClient` 인터페이스**를 구현하게 해 소비자 코드가 환경 독립적이
되게 했다. 인터페이스는 같지만 **가능 범위는 다르다**(브라우저는 파일 쓰기 불가) —
그 차이를 문서에 표로 못 박는다.

## 3. 3층 구조 — 표면이 그대로 API 가 된다

| 층 | TypeScript | 대응 rhwp 표면 |
|---|---|---|
| 1층 무상태 | `await info(path)` 등 | CLI `--json` 명령 |
| 2층 세션 | `await openDocument(path)` | `mcp-serve` 세션 도구 |
| 3층 계획 | `new Plan(...).check()/.run()` | `rhwp run` 계획 실행기 |

층 이름을 새로 짓지 않은 것도 의도적이다 — rhwp 문서를 읽은 사람이 API 를 바로 이해하고,
반대도 성립한다. 파이썬의 `rhwp.open` 이 여기서 `openDocument` 인 것만 예외인데,
`open` 은 Node·DOM 양쪽에서 이미 쓰이는 어휘라 named import 로 충돌하기 때문이다.

## 4. 판정 vs 고장 — 이 바인딩의 핵심 규약

| 상황 | exit | TypeScript |
|---|---|---|
| 성공 | 0 | 정상 반환 |
| 읽기·파싱·렌더·쓰기 실패 | 1 | `RhwpRuntimeError` |
| 인자가 틀림 (**호출자 버그**) | 2 | `UsageError` |
| 검증 단언 실패 | 3 | **반환값의 판정 필드** |
| 페이지 수 불일치 | 4 | **반환값의 판정 필드** |

exit 3/4 를 기본으로 예외로 만들지 **않는** 이유:

`--verify` 가 불일치를 보고하거나 `render-diff` 가 회귀를 검출한 것은 **도구가 정상
동작한 결과**다. 예외로 올리면 호출자가 `try/catch` 로 "고장"처럼 다루게 되고, 정작
봉투에 담긴 판정 근거(`diffCount`·`status`·`pages`)를 읽지 않는다.

TypeScript 고유의 근거가 하나 더 있다: **`Promise` 거절은 타입에 나타나지 않는다.**
예외로 만들면 타입 시스템이 판정의 존재를 잊지만, 반환값에 두면 `result.verify` 가
시그니처에 남는다.

```ts
const result = await exportHwpx('원본.hwp', { out: '변환본.hwpx', verify: true });
if (!result.verify?.identical) console.log(`차이 ${result.verify?.diffCount}건`);
```

예외가 필요하면 `throwOnVerdict: true` 로 **명시**한다. 기본값을 뒤집지 않는다.

### 4-1. `renderDiff` — 시각 회귀도 같은 규약

`render-diff` 가 [#3719](https://github.com/edwardkim/rhwp/issues/3719) §6-2 로 `--json`
을 갖게 되어 바인딩이 `renderDiff(path, pathB?, {via, page, maxDisp, throwOnVerdict})`
로 감쌌다. `pathB` 를 주면 두 파일 비교(`mode: "pair"`), 없으면 자기 라운드트립
(`mode: "roundtrip"`)이다.

```ts
const geom = await renderDiff('보고서.hwp', undefined, { via: 'hwpx' });
if (geom.get('regression')) console.log(geom.get('status'), geom.get('maxDisp'));
```

**회귀는 예외가 아니라 봉투의 `status`·`regression` 필드다.** `irDiff` 가 내용(IR)을
본다면 이쪽은 배치(기하)를 본다 — `verify.identical` 이 참이어도 렌더 결과는 밀렸을 수
있으므로 둘은 서로를 대신하지 못한다.

종료 코드는 **모드마다 다르다.** 이 차이는 바인딩이 만든 것이 아니라 본체의 계약이다.

| 모드 | 회귀 검출 시 | 왜 |
|---|---|---|
| `--json` | **exit 3**(판정) | CI 가 "렌더가 깨졌다"와 "파일을 못 읽었다"를 같은 신호로 받으면 안 된다 |
| 사람용 출력 | **exit 1**(종전 그대로) | 이미 1 을 실패로 읽는 CI 스크립트를 깨지 않기 위해서다. 새 의미론은 `--json` 소비자에게만 준다 |

CLI 의 폴더 일괄(`--batch`)은 **감싸지 않는다** — 한 줄 봉투가 아니라 NDJSON 스트림이라
반환 타입이 다르고, 한 함수가 둘 다 돌려주면 호출자가 받은 값을 타입으로 알 수 없다.
폴더 일괄은 `runNdjson` 으로 직접 부른다.

## 5. 타입이 계약을 강제한다 — 생성기의 두 출처

파이썬은 동적 `Envelope` 하나로 충분했다. **TypeScript 는 다르다** — 사용자가 이 언어를
고른 이유의 상당 부분이 "필드 이름을 컴파일러가 확인해 준다"인데, 봉투를
`Record<string, unknown>` 으로만 주면 바인딩이 그 값어치를 통째로 버린다.

그렇다고 손으로 쓰면 rhwp 가 필드를 더할 때마다 뒤처지고, 뒤처졌다는 사실조차 드러나지
않는다. 그래서 **생성**한다.

| 출처 | 무엇을 서술하나 | 산출 |
|---|---|---|
| `rhwp export-ir-schema` | 문서 모델 — 공개 IR 정의 41개 | `src/ir.ts` |
| `rhwp capabilities` | 명령별 봉투 — `recordFields` | `src/envelopes.ts` |

```bash
npm run gen:types     # 재생성 (tools/gen-types.ts)
npm run gen:check     # 디스크와 다르면 exit 1 — CI 가 뒤처짐을 잡는다
```

**출처가 둘인 이유**: 어느 한쪽만으로는 봉투 필드에 정적 타입을 붙일 수 없다. IR 로
봉투를 흉내 내는 순간 수기 매핑이 부활한다. **수기 목록은 두지 않는다.**

`src/ir.ts`·`src/envelopes.ts` 는 생성물이다. 손으로 고치면 다음 재생성에서 사라진다.

### 스키마가 공개 표면을 좁히는 이유

serde 파생에서 자동 추출하면 "직렬화 표현"이 새어 나온다 — 라운드트립 보존용 원본 바이트
(`raw_stream`·`extra_streams`)나 내부 shim(`is_hwp3_variant`)처럼 **바인딩이 알 필요도
없고 알아서도 안 되는** 필드까지 공개 계약이 된다. `src/ir_schema.rs` 에 명시적으로 쓴
목록이 곧 "우리가 외부에 약속하는 IR"이다. (M18 승계.)

## 6. 이름 규약 — 수기 개명 금지

봉투 키는 camelCase 이므로 TypeScript 에서는 변환 없이 그대로 쓰는 것이 기본이다.
`src/naming.ts` 의 기계 변환기(`toSnake`·`toCamel`·`snakeKeys`·`camelKeys`)는
snake_case 를 쓰는 외부 시스템(파이썬 바인딩과 같은 데이터 형태를 주고받을 때 등)과
붙일 때 쓴다.

```
pageCount ↔ page_count · sourceA ↔ source_a · irSchemaVersion ↔ ir_schema_version
```

사람이 이름을 다시 붙이기 시작하면 매핑 표를 유지해야 하고, 그 표는 반드시 뒤처진다.
**규칙만 두고 개명하지 않는다.**

## 7. "모름"과 "없음"을 섞지 않는다

```ts
result.changedPages;   // null = 확정 불가 / [] = 바뀐 쪽 없음 / [0,2] = 그 쪽들
result.verify;         // null = 검증 안 함 (실패가 아님)
```

부분 목록은 침묵보다 나쁘다 — 빠뜨린 항목이 있는 목록은 거짓 통과를 만든다.
rhwp 가 확정할 수 없을 때 `null` 을 내는 규약을 바인딩이 그대로 전한다.
**`if (!result.changedPages)` 로 둘을 뭉뚱그리지 말 것.**

JavaScript 는 `??` 와 `?.` 가 관용구인 언어라 이 오독이 특히 쉽다. `result.verify ?? {}`
한 줄이면 "검증 안 함"과 "검증 실패"가 영원히 구분되지 않는다.

## 8. 오타는 조용히 넘어가지 않는다

```ts
meta.get('pageConut');   // 예외 — 있는 필드를 함께 알려준다
```

없는 필드가 `undefined` 가 되면, 이름을 잘못 쓴 코드가 "값이 없네"로 흘러가 가장 찾기
어려운 버그가 된다. 단, `verify`·`changedPages` 처럼 **계약상 null 이 유효한** 필드는
프로퍼티로 감싸 `null` 을 돌려준다. "없어도 되는 것"과 "이름이 틀린 것"은 다르다.

## 9. 바이너리 탐색 — 순서가 계약이다

1. 환경변수 `RHWP_BIN`
2. 패키지 동봉 (`dist/_bin/`)
3. `PATH`

순서를 뒤집으면 개발자가 로컬 빌드를 가리켜도 동봉본이 실행돼 "왜 수정이 반영 안 되지"라는
진단 불가 상황이 생긴다.

**환경변수를 줬는데 못 쓰면 조용히 다음으로 넘어가지 않고 즉시 실패한다.**
사용자는 그 바이너리를 쓰고 있다고 믿는데 다른 게 실행되면 디버깅이 불가능하다.

윈도우는 실행 비트가 없으므로 확장자(`.exe`/`.bat`/`.cmd`)로 판단한다. 탐색 결과는
프로세스 수명 동안 캐시하고, `clearBinaryCache()`·`findBinary({ refresh: true })` 로 비운다.

**동봉은 아직 비어 있다.** 경로는 계약으로 열어 뒀지만 플랫폼별 릴리스 매트릭스는 미정이다.

## 10. 패키징 — ESM + CJS 듀얼, 런타임 의존성 0

Node 생태계가 아직 둘로 갈려 있다. 한쪽만 내면 다른 쪽 사용자가 **바인딩을 쓰려고 앱
구조를 바꿔야 한다** — 재포장 도구가 요구할 일이 아니다.

| 진입점 | 내용 |
|---|---|
| `@rhwp/node` | 1·2·3층 전부. Node 전용 |
| `@rhwp/node/browser` | `createBrowserClient` — WASM 어댑터 |

진입점을 쪼갠 것은 번들러 때문이다. 하나의 진입점이 런타임 감지로 갈라지면 브라우저
번들에 `node:child_process` 가 딸려 온다. **정적으로 갈라 두면 번들러가 추론할 필요가 없다.**

`dependencies` 는 비어 있다. npm 에서 의존성 하나는 전이 의존성 수십 개이고, 그만큼
버전 충돌·감사 경고·공급망 위험 지점이 는다.

## 11. 자원 정리

세션은 자식 프로세스를 띄운다. **남으면 다음 작업이 파일을 못 연다.**

- `close()` 가 계약이고 **멱등**이다. `try/finally` 는 어디서나 통한다.
- `Symbol.asyncDispose` 를 구현해 `await using` 도 되지만, **런타임에 심볼이 있어야**
  한다(Node 20+). 폴리필은 넣지 않는다 — `sideEffects: false` 패키지가 전역을 오염시키면
  번들러의 가정이 깨진다.
- stdin 을 닫아도 안 죽으면 강제 종료한다.
- 정리 경로에서는 **새 예외를 만들지 않는다** — 원인 예외를 가리면 진단이 어려워진다.

## 12. 계약 패리티 가드 — 뒤처짐을 CI 가 잡는다

`capabilities` 선언과 TypeScript API 를 대조하는 테스트가 CI 에 있다. rhwp 에 `--json`
명령이 늘었는데 바인딩이 따라가지 않으면 **CI 에서 실패**한다. 생성 타입은 `gen:check` 가
따로 감시한다.

바인딩이 뒤처지는 것은 **조용히** 일어난다 — 명령이 늘어도 TypeScript 는 아무 오류 없이
잘 돈다, 그 명령을 못 쓸 뿐이다. M18 에서 이 가드가 개발 중 실제 결함 5건을 잡았다.

M19 에서는 같은 가드가 **양방향으로** 작동했다. `render-diff` 는 처음엔 `--json` 이 없는데
바인딩이 감싸고 있었고(없는 계약을 감쌈 — 모든 호출이 exit 2), `--json` 이 들어온 뒤에는
바인딩에 함수가 없었다(있는 계약을 빠뜨림). **정반대인 두 결함이 같은 테스트 하나에
걸린다** — 노출 기준을 손으로 고른 목록이 아니라 자기서술에 둔 설계의 값어치다.

새 명령을 추가할 때:

1. `src/commands.ts` 에 래퍼 함수
2. `src/index.ts` 의 재수출 — **빠뜨리기 쉬운 자리다.** M19 에서 `renderDiff` 가
   `commands.ts` 에는 있는데 `index.ts` 재수출이 빠져 `index.test.ts` 가 잡았다
3. `test/commands.test.ts` 에 인자 조립 테스트
4. 통합 테스트의 패리티 집합
5. `npm run gen:types` — 봉투 타입 재생성

### 12-1. 노출 기준의 각주 — 명령과 플래그는 다르다

**어떤 명령을 감쌀지는 자기서술이 정하지만, 그 명령의 플래그를 전부 여는 것은 아니다.**
`capabilities` 의 `flags` 는 "이 명령이 이 플래그를 파싱한다"는 사실만 말하고, **그
플래그가 `--json` 모드에서 무엇을 하는지는 말하지 않는다.** 그 간극에서 세 자리가 나왔고,
셋 다 실물 실행으로만 드러났다.

| 닫은 것 | 실측 | 이유 |
|---|---|---|
| `exportText` · `exportStructure` 의 `-o` | `--json` 모드에서 **조용히 무시**(디렉터리조차 안 생긴다) | 받아 주면 "저장했다"는 거짓말이 된다 |
| `exportTables` 의 `-o` | stdout 이 사람용 문장("표 추출 완료: N개 → 경로")으로 **바뀐다** | 봉투 계약이 깨져 `runJson` 이 `ProtocolError` 를 던진다. **본체 쪽 별도 업스트림 이슈감** — 바인딩이 옵션을 닫은 것은 회피일 뿐 수정이 아니다 |
| `renderDiff` 의 `--batch` | NDJSON 스트림 | 한 함수의 반환 타입은 하나여야 한다 |

가른 기준은 **봉투 계약을 지키는가**다. 같은 `-o` 라도 `exportCapabilitiesSchema` 는
저장 후에도 stdout 이 봉투를 유지하므로(`output`·`bytes`) 열어 뒀다.

### 12-2. `Plan.check()` 는 미지원 rhwp 에서 던진다

`check()` 는 계획서를 보내기 **전에** `capabilities` 로 `run` 이 `--dry-run` 을 선언하는지
확인하고, 아니면 **실행하지 않고 `RhwpError` 를 던진다.** 조용히 실제 실행으로 내려가지
않는다.

미지원 rhwp 는 계획서의 `dryRun` 필드를 **그냥 무시하고 편집을 수행하고 저장한다.**
실측으로 재현했다 — `run --plan-json '{…"dryRun":true…}' --json` 이 exit 0, 경고 없음,
봉투에 `dryRun` 필드 없음으로 끝나고 **473,600바이트짜리 편집본이 생겼다.** 호출자는
"검사만 했다"고 믿는다. 실패도 예외도 없이 문서가 바뀌므로 사람이 열어보기 전까지 아무도
모른다 — **조용한 데이터 사고**다.

판정자를 두 곳에 두지 않는다는 원칙(§1)과 충돌하지 않는다. 여기서 바인딩이 판정하는 것은
*문서*가 아니라 **도구의 능력**이고, 그 능력은 `capabilities` 자기서술이라는 단일 출처가
이미 답한다. 바인딩은 새 판정을 만드는 것이 아니라 **계약이 성립하는지만 확인**한다.

`run --dry-run`(#3759) 머지 전에는 항상 던진다. 실행으로 대체할지는 사람이 `run()` 을
명시적으로 불러 정한다.

## 13. 테스트 전략

| 종류 | 바이너리 필요 | 무엇을 지키나 |
|---|---|---|
| 단위 | 없음 | 탐색·이름 변환·예외 매핑·계획 직렬화 (순수 로직) |
| 프로세스 | 가짜 스크립트 | 종료 코드별 동작·봉투 계약 위반 감지 |
| 세션 | 가짜 JSON-RPC 서버 | 프로토콜 취급 (id 대조·알림 무시·정리 보장) |
| 통합 | **실물 rhwp** | 계약 재포장 정합·패리티 가드 |

단위 테스트가 바이너리 없이 도는 것이 중요하다 — CI 대부분을 Rust 빌드 없이 수 초 만에
돌릴 수 있고, 그래야 바인딩 기여의 문턱이 낮아진다. 기여 문턱이 곧 뒤처짐 속도다.

### 가짜 픽스처의 인코딩 함정

실물 rhwp(Rust)는 콘솔 코드페이지와 무관하게 **항상 UTF-8** 을 주고받는다.
가짜 픽스처는 플랫폼 기본 인코딩을 따르므로 stdout·stdin 에 UTF-8 을 **명시**해야 한다.
안 그러면 윈도우(cp949)에서만 깨져 "바인딩 버그"로 오인된다.

```ts
process.stdout.write(Buffer.from(JSON.stringify(envelope), 'utf8'));
```

M18 에서 같은 원인으로 테스트가 두 번 실패했고, 원인은 바인딩이 아니라 픽스처였다.

## 14. 알려진 한계

- npm 미배포. 스코프·버전 정책이 미정이다.
- 패키지에 바이너리 미동봉. `RHWP_BIN`·`PATH` 경로만 검증한다.
- 브라우저는 `@rhwp/editor` WASM 이 별도로 필요하고 파일 쓰기 명령을 지원하지 않는다.
- 인프로세스(napi) 경로 없음 — §2 의 승격 조건 참조.
- `Plan.check()` 는 `run --dry-run`(#3759) 머지 전까지 항상 던진다(§12-2). 그때까지
  3층은 `run()` 만 쓸 수 있다.
- `exportIrSchema()`·`irSchema()`·`npm run gen:check` 는 M18(#3762)의
  `export-ir-schema` 를 요구한다. 그 전 바이너리에서는 exit 2 다.
- `renderDiff` 는 단건만 감싼다. 폴더 일괄(`--batch`)은 `runNdjson` 으로 부른다(§4-1).
- `exportTables -o` 가 `--json` 봉투를 깨는 것은 바인딩이 옵션을 닫아 회피했을 뿐
  **본체 쪽 미해결 사항**이다(§12-1).

## 15. 앞으로 (M20)

`bindings_foundation.md` §4 의 착수 조건을 따른다.

- **M20 (C#/Swift)**: 공공 SI 수요 실증 1건.

세 계열 모두 같은 원리(얇은 재포장)를 따르면, 언어가 늘어도 계약은 rhwp 본체 한 곳에만
있다. 이것이 배수 확장의 구조적 근거다. M19 가 추가한 것은 **생성 타입**이라는 한 겹으로,
정적 타입 언어 계열(M20 의 C#·Swift)이 그대로 승계할 수 있는 패턴이다.

## 관련 문서

- [`bindings_foundation.md`](../tech/bindings_foundation.md) — 설계 결정의 권위
- [`python_binding_guide.md`](python_binding_guide.md) — 파이썬 1호(M18) 대응 문서
- [`agent_surface_playbook.md`](agent_surface_playbook.md) — 표면 추가 절차
- [`cli_json_pipeline_guide.md`](cli_json_pipeline_guide.md) — 봉투 계약
- [`mcp_integration_guide.md`](mcp_integration_guide.md) — 세션 도구 계약
- [`bindings/node/docs/DESIGN.md`](../../bindings/node/docs/DESIGN.md) — 결정 기록(버린 대안 포함)
