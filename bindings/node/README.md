# @rhwp/node — Node/TypeScript 바인딩

HWP·HWPX·HWP3·HML 문서를 읽고 편집·렌더링하는 [rhwp](https://github.com/edwardkim/rhwp) 엔진의
Node/TypeScript 바인딩입니다.

> **바인딩은 새 표면이 아니라 기존 계약의 재포장입니다.**
> CLI `--json` 봉투와 `mcp-serve` 세션 도구가 이미 증명한 계약 위에만 서고,
> TypeScript 쪽에서 판정 로직을 새로 만들지 않습니다. rhwp 본체에 명령이 늘면
> 바인딩은 자동으로 따라옵니다 — 계약 패리티 가드가 뒤처짐을 CI 에서 잡습니다.

파이썬 바인딩(M18, #3762)과 **같은 규약의 TypeScript 대응본**입니다. 층 이름·판정 규약·
"모름과 없음"의 구분이 모두 동일하므로, 한쪽 문서를 읽은 사람은 다른 쪽을 바로 씁니다.

## 설치

```bash
npm install @rhwp/node
```

> **아직 npm 에 배포되지 않았습니다.** 지금은 저장소의 `bindings/node` 를 직접 빌드해
> 쓰거나(`npm run build`), workspace/`file:` 의존으로 참조합니다. 배포는 별도 과제입니다.

`rhwp` 실행 파일이 필요합니다. 탐색 순서는 다음과 같습니다.

| 순서 | 위치 | 쓰임 |
|---|---|---|
| 1 | 환경변수 `RHWP_BIN` | 로컬 빌드를 가리킬 때 |
| 2 | 패키지 동봉 (`dist/_bin/`) | 배포본 |
| 3 | `PATH` | 시스템 설치본 |

```bash
export RHWP_BIN=/path/to/rhwp        # 선택
node -e "import('@rhwp/node').then(m => console.log(m.findBinary()))"
```

환경변수를 **줬는데 쓸 수 없으면** 조용히 다음 경로로 넘어가지 않고 즉시
`BinaryNotFoundError` 입니다. 사용자가 그 바이너리를 쓰고 있다고 믿는데 다른 게 실행되면
디버깅이 불가능하기 때문입니다.

> **바이너리는 아직 패키지에 동봉되지 않습니다.** `dist/_bin/` 경로는 계약으로 열어 뒀고,
> 플랫폼별 CI 매트릭스는 후속 과제입니다. 지금은 1·3 경로만 실제로 동작합니다.

## 3층 구조

rhwp 의 에이전트 표면이 그대로 TypeScript API 가 됩니다.

| 층 | 무엇 | 언제 |
|---|---|---|
| 1층 무상태 | `await info(path)` 등 | 호출 하나 = 작업 하나 |
| 2층 세션 | `const doc = await openDocument(path)` | 같은 문서를 반복해서 만질 때 |
| 3층 계획 | `await new Plan(...).run()` | 여러 편집을 원자적으로, 검증까지 |

### 1층 — 무상태

```ts
import { info, search, exportTables } from '@rhwp/node';

const meta = await info('보고서.hwp');
console.log(meta.get('pageCount'), meta.get('format'));

// 주소가 붙은 검색 — 매치마다 (구역·문단·쪽·문자 오프셋)
const hits = await search('보고서.hwp', '예산');
for (const m of hits.children('matches')) {
  console.log(`${m.get('page')}쪽: ${m.get('context')}`);
}

// 표를 셀 좌표와 함께
const tables = await exportTables('양식.hwpx');
console.log(tables.get('tableCount'));
```

### 2층 — 세션

문서를 한 번 열어 두고 여러 번 만집니다. 호출마다 재파싱하지 않으므로 대형 문서에서 빠릅니다.

```ts
import { openDocument } from '@rhwp/node';

const doc = await openDocument('서식.hwp');
try {
  await doc.fillFields({ 성명: '홍길동', 부서: '기획팀' });
  await doc.replaceText('2025년', '2026년');

  const saved = await doc.save('제출본.hwp', { verify: true });
  if (!saved.verify?.identical) throw new Error('저장본이 의도한 문서가 아닙니다');

  // 바뀐 쪽만 눈으로 확인 — 상수 비용
  for (const page of saved.changedPages ?? []) {
    await doc.renderPage(page, `확인_${page}.svg`);
  }
} finally {
  await doc.close();
}
```

`Symbol.asyncDispose` 가 구현돼 있으므로 TypeScript 5.2+ 와 `Symbol.asyncDispose` 를 가진
런타임(Node 20+)에서는 `await using` 을 쓸 수 있습니다.

```ts
await using doc = await openDocument('서식.hwp');
await doc.fillFields({ 성명: '홍길동' });
```

`try/finally` 는 어디서나 통하고, `await using` 은 런타임 지원이 있을 때의 축약입니다.
**어느 쪽이든 반드시 닫으세요** — 자식 프로세스가 남으면 다음 작업이 파일을 못 엽니다.

### 3층 — 계획

여러 편집을 **의도**로 선언하면 rhwp 가 안전을 보장합니다.
정적 선검증(실행 0) → 원자 실행(인메모리) → 단언 통과 시에만 단 한 번 저장.

```ts
import { Plan } from '@rhwp/node';

const plan = new Plan('서식.hwp', '제출본.hwp')
  .fillFields({ 성명: '홍길동' })
  .replaceText('2025년', '2026년')
  .setCheckbox(1)
  .verify();

const preview = await plan.check();   // 디스크 무변경 — 실행 전 검사
if (!preview.ok) {
  console.error(preview.describeViolations());
} else {
  const journal = await plan.run();
  console.log(journal.verify?.identical);
}
```

중간 step 이 실패해도 **반쪽 편집 문서가 남지 않습니다** — 전 step 이 메모리에서 통과해야 저장합니다.

`check()` 는 계획서를 보내기 전에 `capabilities` 로 rhwp 가 `run --dry-run` 을 지원하는지
확인하고, 지원하지 않으면 **실행으로 내려가지 않고 예외를 던집니다.** 미지원 버전은
계획서의 `dryRun` 을 그냥 무시하고 편집·저장하기 때문입니다 — 호출자는 "검사만 했다"고
믿는데 문서가 바뀌는, 실패도 예외도 없는 조용한 사고가 됩니다. 이 예외는 계획 위반
(`preview.ok === false`)과 다른 사건입니다.

## 판정 vs 고장

이 바인딩의 핵심 규약입니다.

```ts
// 판정 실패는 예외가 아니다 — 도구는 정상 동작했고, 문서에 대한 단언이 실패한 것
const result = await exportHwpx('원본.hwp', { out: '변환본.hwpx', verify: true });
if (!result.verify?.identical) {
  console.log(`차이 ${result.verify?.diffCount}건`);   // 봉투를 읽어 판단
}

// 예외를 원하면 명시
await exportHwpx('원본.hwp', { verify: true, throwOnVerdict: true });   // VerdictFailed
```

| 상황 | 종료 코드 | TypeScript |
|---|---|---|
| 성공 | 0 | 정상 반환 |
| 읽기·파싱·렌더·쓰기 실패 | 1 | `RhwpRuntimeError` |
| 인자가 틀림 (**우리 쪽 버그**) | 2 | `UsageError` |
| 검증 단언 실패 | 3 | **반환값의 판정 필드** |
| 페이지 수 불일치 | 4 | **반환값의 판정 필드** |

exit 3/4 를 기본으로 예외로 만들면 호출자가 `try/catch` 로 "고장"처럼 다루게 되고,
정작 봉투에 담긴 판정 근거(`diffCount`·`status`)를 읽지 않게 됩니다.

시각 회귀도 같은 규약입니다. `renderDiff` 는 렌더 기하를 비교해 회귀를 **판정**합니다.

```ts
// 자기 라운드트립 — HWPX 로 저장했다 다시 읽었을 때 배치가 밀렸는가
const geom = await renderDiff('보고서.hwp', undefined, { via: 'hwpx' });
if (geom.get('regression')) {
  console.log(geom.get('status'), geom.get('maxDisp'), geom.get('overPages'));
}

// 두 파일 직접 비교 (mode: 'pair')
await renderDiff('원본.hwp', '변환본.hwpx');
```

`--json` 모드에서 회귀 검출은 **exit 3**(판정)이지 exit 1(고장)이 아닙니다.
CLI 를 사람이 직접 쓰는 모드는 **종전대로 exit 1** 입니다 — 이미 1 을 실패로 읽는 CI
스크립트를 깨지 않기 위해서이고, 새 의미론은 `--json` 소비자에게만 줍니다.

## "모름"과 "없음"의 구분

```ts
const result = await fillFields('서식.hwp', { 성명: '값' }, { out: '산출.hwp' });

result.changedPages;   // null = 확정 불가 / [] = 바뀐 쪽 없음 / [0,2] = 그 쪽들
result.verify;         // null = 검증 안 함 (실패가 아님)
```

부분 목록은 침묵보다 나쁩니다 — 빠뜨린 항목이 있는 목록은 거짓 통과를 만듭니다.
그래서 rhwp 는 확정할 수 없으면 `null` 을 내고, 바인딩은 그걸 `null` 로 전합니다.
**`if (!result.changedPages)` 로 둘을 뭉뚱그리지 마세요.**

## 오타는 조용히 넘어가지 않습니다

```ts
const meta = await info('보고서.hwp');
meta.get('pageConut');   // 예외 — 있는 필드를 함께 알려준다
```

없는 필드가 `undefined` 가 되면, 필드 이름을 잘못 쓴 코드가 "값이 없네"로 흘러가
가장 찾기 어려운 버그가 됩니다.

## 타입이 계약을 강제합니다

파이썬은 동적 `Envelope` 하나로 충분했지만, TypeScript 에서는 봉투 필드가 **정적 타입으로
노출돼야** 값어치가 있습니다. 그래서 타입을 손으로 쓰지 않고 **두 출처에서 생성**합니다.

| 출처 | 무엇을 만드나 | 산출 |
|---|---|---|
| `rhwp export-ir-schema` | 공개 IR 정의 41개 | `src/ir.ts` |
| `rhwp capabilities` | 명령별 `recordFields` | `src/envelopes.ts` |

```bash
npm run gen:types     # 재생성
npm run gen:check     # CI: 디스크와 다르면 exit 1
```

수기 목록은 없습니다. rhwp 가 필드를 늘리면 생성기가 따라가고, 따라가지 않은 PR 은
`gen:check` 가 잡습니다.

## 브라우저

서브프로세스를 띄울 수 없는 환경에서는 `@rhwp/editor` WASM 을 같은 인터페이스로 감쌉니다.

```ts
import { createBrowserClient } from '@rhwp/node/browser';

const client = createBrowserClient(wasm);   // RhwpClient
```

Node 경로와 브라우저 경로가 **같은 `RhwpClient` 인터페이스**를 구현하므로, 소비자 코드는
환경에 독립적입니다. 다만 브라우저에서는 WASM 모듈을 호출자가 직접 로드해 넘겨야 하고,
파일 시스템에 쓰는 명령(`export-pdf -o` 등)은 성립하지 않습니다.

## 대량 처리

```ts
const records = await batch('export-text', ['a.hwp', 'b.hwp', 'c.hwp']);
for (const r of records) {
  if ('error' in r) console.error(`실패: ${r.source} — ${r.error}`);
  else console.log(`${r.source}: ${r.pageCount}쪽`);
}
```

부분 실패도 실패지만 **성공분은 스트림에 남습니다.** 실패 하나로 전체를 버리지 마세요.
나오는 대로 처리하려면 `iterNdjson` 을 씁니다.

## ESM · CJS

둘 다 지원합니다. `exports` 맵이 조건별로 서로 다른 산출물을 가리킵니다.

```ts
import { info } from '@rhwp/node';              // ESM
const { info } = require('@rhwp/node');          // CJS
```

런타임 의존성은 **0** 입니다 — Node 표준 모듈만 씁니다.

## 개발

```bash
cd bindings/node
npm install

npm test              # 전체 (통합 포함, 실물 rhwp 필요)
npm run test:unit     # 바이너리 없이 단위만
npm run typecheck
npm run lint
npm run build
```

단위 테스트는 rhwp 빌드 없이 돕니다 — 탐색·이름 변환·예외 매핑·계획 직렬화는 순수
로직입니다. 실제 문서를 만지는 통합 테스트만 격리돼 있습니다.

### 계약 패리티 가드

`rhwp capabilities` 선언과 TypeScript API 를 대조하는 테스트가 CI 에 있습니다.
rhwp 에 명령이 늘었는데 바인딩이 뒤처지면 **CI 에서 실패**합니다.
수기 목록을 두지 않는 것이 이 바인딩이 뒤처지지 않는 이유입니다.

노출 기준도 같은 자기서술입니다: `capabilities` 가 `json` 을 선언한 명령만 감쌉니다.
`--json` 이 없는 진단 계열(`dump-*`·`hwp5-*`)은 함수로 **없습니다** — 없는 계약을 감싸면
"호출은 있는데 항상 실패하는 API" 가 되기 때문입니다. 필요하면 `runJson`·`runRaw` 로
직접 부릅니다.

가드는 반대 방향도 봅니다. `render-diff` 가
[#3719](https://github.com/edwardkim/rhwp/issues/3719) §6-2 로 `--json` 을 갖게 됐을 때,
본체는 `json: true` 를 선언하는데 바인딩에는 `renderDiff` 가 없었습니다 — **있는 계약을
빠뜨린 상태**이고, 이것도 같은 가드가 잡았습니다. 지금은 `renderDiff` 가 있습니다.
없는 계약을 감싸는 것과 있는 계약을 빠뜨리는 것은 둘 다 결함이며, 노출 기준을 자기서술에
둔 덕분에 둘이 같은 테스트 하나에 걸립니다.

다만 **자기서술이 노출을 자동으로 결정하지는 않습니다.** 같은 명령이라도 특정 플래그를
받으면 `--json` 봉투 계약이 깨지는 경우가 있어, 그런 플래그는 옵션으로 열지 않았습니다
(아래 [노출하지 않은 것](#노출하지-않은-것)).

### 노출하지 않은 것

`capabilities` 가 `json` 을 선언한 명령은 전부 감싸지만, **그 명령의 플래그를 전부 여는
것은 아닙니다.** 봉투 계약을 깨거나 아무 일도 하지 않는 플래그는 옵션에서 뺐습니다.

| 뺀 것 | 실측 | 왜 |
|---|---|---|
| `exportText` · `exportStructure` 의 `-o` | `--json` 모드에서 **조용히 무시**된다(파일이 생기지 않는다) | 받아 주면 "저장했다"는 거짓말이 된다. 파일이 필요하면 봉투 내용을 직접 쓴다 |
| `exportTables` 의 `-o` | stdout 이 사람용 문장("표 추출 완료: N개 → 경로")으로 **바뀐다** | `--json` 봉투 계약이 깨져 `runJson` 이 `ProtocolError` 로 터진다. 이쪽은 바인딩이 피할 문제가 아니라 **본체 쪽 별도 이슈감**이다 |
| `renderDiff` 의 `--batch` | 한 줄 봉투가 아니라 **NDJSON 스트림**이다 | 한 함수가 봉투와 배열을 모두 돌려주면 호출자가 받은 값이 무엇인지 타입으로 알 수 없다. 폴더 일괄은 `runNdjson` 으로 직접 부른다 |

`--json` 자체가 없는 진단 계열(`dump-*`·`hwp5-*`)과 `mcp-serve` 는 애초에 함수가 없습니다.

## 문서

| 문서 | 내용 |
|---|---|
| [`docs/API.md`](docs/API.md) | 전 API 레퍼런스 |
| [`docs/COOKBOOK.md`](docs/COOKBOOK.md) | 실전 레시피 |
| [`docs/TROUBLESHOOTING.md`](docs/TROUBLESHOOTING.md) | 증상별 원인·처방 |
| [`docs/MIGRATION.md`](docs/MIGRATION.md) | 기존 JS/TS 도구·COM 자동화에서 이주 |
| [`docs/DESIGN.md`](docs/DESIGN.md) | 설계 결정과 버린 대안 |
| [`../../mydocs/manual/node_binding_guide.md`](../../mydocs/manual/node_binding_guide.md) | 저장소 표준 가이드 |

## 알려진 한계

- npm 미배포 — 저장소에서 빌드해 쓴다.
- 바이너리 미동봉 — `RHWP_BIN` 또는 `PATH` 로 실행 파일을 제공해야 한다.
- 브라우저에서는 WASM(`@rhwp/editor`)이 별도로 필요하고, 파일 쓰기 명령은 쓸 수 없다.
- 인프로세스(napi) 경로는 없다 — 성능이 실제 병목으로 실증되면 검토한다([`docs/DESIGN.md`](docs/DESIGN.md) D1).
- `exportIrSchema()` 와 생성 타입(`src/ir.ts`)은 rhwp 의 `export-ir-schema` 를 요구한다.
  그 명령은 M18([#3762](https://github.com/edwardkim/rhwp/issues/3762), PR
  [#3775](https://github.com/edwardkim/rhwp/pull/3775))에서 들어오므로, 그 PR 이 머지되기
  전 바이너리에서는 exit 2 다. `capabilities` 로 먼저 확인하는 것이 정석이다.
- `Plan.check()` 는 rhwp 의 `run --dry-run`(#3759)을 요구한다. 없으면 던진다 — 조용히
  실행으로 내려가지 않는다.
- `renderDiff` 는 단건(라운드트립·두 파일)만 감싼다. 폴더 일괄(`--batch`)은 NDJSON 이라
  반환 타입이 달라 이 함수에 넣지 않았다 — `runNdjson` 으로 직접 부른다.
- `exportTables` 에 `-o` 를 주면 stdout 이 사람용 문장으로 바뀌어 봉투 계약이 깨진다.
  바인딩이 옵션을 열지 않는 것은 회피일 뿐이고, 근본 수정은 본체 쪽 별도 이슈다.

## 라이선스

MIT — rhwp 본체와 동일합니다.
