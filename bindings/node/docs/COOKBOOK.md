# 요리책 — 실제로 하고 싶은 일들

각 레시피는 그대로 복사해 돌아간다. 설계 배경은
[`node_binding_guide.md`](../../../mydocs/manual/node_binding_guide.md).

예제는 전부 TypeScript(ESM)다. CJS 라면 `import` 를 `require` 로 바꾸면 그만이다.

## 목차

1. [서식 자동 채우기](#1-서식-자동-채우기)
2. [대량 메일머지](#2-대량-메일머지)
3. [RAG 색인 만들기](#3-rag-색인-만들기)
4. [표를 데이터셋으로](#4-표를-데이터셋으로)
5. [변환 품질 검증](#5-변환-품질-검증)
6. [눈검증 루프 닫기](#6-눈검증-루프-닫기)
7. [보호 문서 다루기](#7-보호-문서-다루기)
8. [아카이브 대장화](#8-아카이브-대장화)
9. [계획으로 안전하게 편집](#9-계획으로-안전하게-편집)
10. [문서 간 전사](#10-문서-간-전사)
11. [브라우저에서 쓰기](#11-브라우저에서-쓰기)
12. [Express 업로드 처리기](#12-express-업로드-처리기)

마지막에 [자주 하는 실수](#자주-하는-실수).

---

## 1. 서식 자동 채우기

**하고 싶은 것**: 정부 양식의 누름틀을 채우고, 제대로 채워졌는지 확인한다.

```ts
import { fields, fillFields } from '@rhwp/node';

interface FieldRecord { name: string }

export async function fillAndVerify(
  form: string,
  data: Record<string, string>,
  output: string,
): Promise<boolean> {
  // 먼저 어떤 칸이 있는지 본다 — 이름을 추측하지 않는다.
  const survey = await fields(form);
  const available = new Set((survey.get('fields') as FieldRecord[]).map((f) => f.name));
  const unknown = Object.keys(data).filter((k) => !available.has(k.split('[')[0]));
  if (unknown.length) {
    throw new Error(`없는 누름틀: ${unknown.join(', ')}\n있는 것: ${[...available].sort().join(', ')}`);
  }

  const result = await fillFields(form, data, { out: output, verify: true });

  const notFound = result.get('notFound') as string[];
  if (notFound.length) {
    console.warn(`채우지 못한 칸: ${notFound.join(', ')}`);
    return false;
  }

  // 판정은 예외가 아니라 값이다.
  const verify = result.verify;
  if (verify === null) throw new Error('verify 를 요청했는데 보고가 없다');
  if (!verify.identical) {
    console.warn(`저장본 검증 실패 (차이 ${verify.diffCount}건)`);
    return false;
  }

  console.log(`${result.get('filledCount')}칸 채움 → ${output}`);
  return true;
}
```

**동명 누름틀**이 여러 개면 `[순번]`(0 기준, `fields` 목록 순서)으로 지정한다.

```ts
await fillFields(form, { '성명[0]': '홍길동', '성명[1]': '김철수' }, { out });
```

순번 없이 주면 첫 번째만 채우고 봉투의 `ambiguous` 에 "몇 개 중 몇 개"를 보고한다.
그 필드를 읽지 않으면 조용히 한 칸만 채워진 문서가 나간다.

---

## 2. 대량 메일머지

**하고 싶은 것**: 서식 하나 + 데이터 N행 → 산출물 N개.

```ts
import { mkdir } from 'node:fs/promises';
import { join } from 'node:path';

import { fillFields } from '@rhwp/node';

export async function mailMerge(
  form: string,
  rows: Array<Record<string, string>>,
  outDir: string,
): Promise<string[]> {
  await mkdir(outDir, { recursive: true });
  const made: string[] = [];

  for (const [i, row] of rows.entries()) {
    const target = join(outDir, `${String(i + 1).padStart(4, '0')}_${row.성명 ?? 'noname'}.hwp`);
    const result = await fillFields(form, row, { out: target, verify: true });

    const verify = result.verify;
    if (verify && !verify.identical) {
      console.warn(`  ${target}: 검증 실패 (차이 ${verify.diffCount})`);
      continue;
    }
    made.push(target);
  }
  return made;
}
```

**직렬로 도는 것이 기본이다.** 호출 하나가 프로세스 하나이므로 `Promise.all` 로 수백 건을
동시에 던지면 프로세스 폭발이 난다. 병렬이 필요하면 동시성 상한을 직접 건다.

```ts
const LIMIT = 4;   // 코어 수 정도
for (let i = 0; i < rows.length; i += LIMIT) {
  await Promise.all(rows.slice(i, i + LIMIT).map(one));
}
```

한 건 실패로 전체를 멈추지 않는다 — 실패한 행만 보고하고 나머지를 계속한다.

---

## 3. RAG 색인 만들기

**하고 싶은 것**: 문서를 청크로 나누되 **주소를 잃지 않는다**. 인용할 때 "몇 쪽"을
답할 수 있어야 한다.

```ts
import { digest, search } from '@rhwp/node';

interface Chunk { text: string; source: string; page: number | null; heading: string | null }

export async function indexDocument(path: string): Promise<Chunk[]> {
  // 절 단위 청킹 — 주소가 보존된다.
  const env = await digest(path, { sections: true });
  const sections = env.get('sections') as Array<Record<string, unknown>>;

  return sections.map((s) => ({
    text: String(s.text ?? ''),
    source: path,
    page: (s.page as number | null) ?? null,
    heading: (s.heading as string | null) ?? null,
  }));
}
```

**인용 검증** — 답변이 실제로 그 쪽에 있는지 되짚는다.

```ts
export async function verifyCitation(path: string, quote: string): Promise<number[]> {
  const hits = await search(path, quote);
  const matches = hits.get('matches') as Array<{ page?: number }>;
  return matches.map((m) => m.page).filter((p): p is number => typeof p === 'number');
}
```

평문을 추출해 외부에서 검색하면 주소가 소멸한다 — `search` 는 조판 엔진을 거치므로
"몇 쪽"에 답할 수 있다. 결과의 `page` 는 **0 기준**이다.

---

## 4. 표를 데이터셋으로

**하고 싶은 것**: 보고서 표들을 하나의 행 배열로.

```ts
import { exportTables, setCell } from '@rhwp/node';

interface Cell { row: number; col: number; text: string }
interface Table { index: number; cells: Cell[] }

export async function tablesToRows(path: string): Promise<Array<Record<string, unknown>>> {
  const env = await exportTables(path);
  const rows: Array<Record<string, unknown>> = [];

  for (const table of env.get('tables') as Table[]) {
    // 병합된 셀은 좌상단(앵커) 좌표로만 나온다 — 덮인 좌표는 목록에 없다.
    const grid = new Map<string, string>();
    for (const cell of table.cells) grid.set(`${cell.row},${cell.col}`, cell.text);

    const maxRow = Math.max(-1, ...table.cells.map((c) => c.row));
    const maxCol = Math.max(-1, ...table.cells.map((c) => c.col));

    for (let r = 0; r <= maxRow; r += 1) {
      const row: Record<string, unknown> = { table: table.index, row: r };
      for (let c = 0; c <= maxCol; c += 1) row[`col${c}`] = grid.get(`${r},${c}`) ?? '';
      rows.push(row);
    }
  }
  return rows;
}
```

**셀에 값을 쓸 때**는 같은 좌표를 그대로 쓴다. 덮인 칸을 지목하면 앵커 좌표를 안내하며 실패한다.

```ts
await setCell(path, 1, 0, 2, '수정값', { out: '결과.hwpx' });
```

---

## 5. 변환 품질 검증

**하고 싶은 것**: HWP → HWPX 변환이 내용을 잃지 않았는지 확인한다.

```ts
import { exportHwpx, irDiff } from '@rhwp/node';

export async function convertSafely(source: string, target: string): Promise<boolean> {
  const result = await exportHwpx(source, { out: target, verify: true, verifyPages: true });

  const verify = result.verify;
  if (verify === null) throw new Error('verify 를 요청했는데 보고가 없다');
  if (verify.identical) return true;

  if (verify.reparseError) {
    // 저장본을 다시 읽지 못했다 — 판정 불가가 아니라 실패다.
    console.error(`재파싱 실패: ${verify.reparseError}`);
    return false;
  }

  console.warn(`IR 차이 ${verify.diffCount}건 — 무엇이 달라졌는지:`);
  const diff = await irDiff(source, target);
  for (const [category, items] of Object.entries(diff.get('categories') ?? {})) {
    console.warn(`  ${category}: ${Array.isArray(items) ? items.length : items}`);
  }
  return false;
}
```

`exportHwpx` 는 exit 3 으로 끝나도 **예외를 던지지 않는다**. 변환은 저장됐고, 실패한 것은
"내용이 같다"는 단언이다. 그 둘을 구분해야 재시도할지 사람을 부를지 정할 수 있다.

> **여기서 판정하는 것은 IR(내용)이지 기하(배치)가 아니다.** `verify.identical` 이
> 참이어도 렌더 결과가 밀렸을 수 있다. 기하는 `renderDiff` 가 본다 — 아래 참조.

### 기하까지 보기 — `renderDiff`

내용이 같아도 배치가 밀리면 사람에게는 다른 문서다. `renderDiff` 는 렌더 기하를 비교해
시각 회귀를 **판정**한다.

```ts
import { renderDiff } from '@rhwp/node';

export async function checkGeometry(source: string, target: string): Promise<boolean> {
  // 두 파일 직접 비교 — mode: 'pair'
  const geom = await renderDiff(source, target);

  if (!geom.get('regression')) return true;

  console.warn(`시각 회귀: ${geom.get('status')}`);
  console.warn(`  최대 변위 ${geom.get('maxDisp')}px (임계 ${geom.get('threshold')})`);
  console.warn(`  초과 쪽 ${geom.get('overPages')}개, 최악 ${geom.get('worstPage')}쪽`);
  if (geom.get('pageCountMismatch')) console.warn('  쪽 수 불일치 — 강신호');
  return false;
}
```

**변환 상대가 없어도 쓸 수 있다.** 두 번째 인자를 생략하면 원본을 한 번 저장했다 다시
읽는 자기 라운드트립(`mode: 'roundtrip'`)이 되어, **저장이 무엇을 잃는지**를 본다.

```ts
await renderDiff('보고서.hwp');                           // 기본 경유 hwpx
await renderDiff('보고서.hwp', undefined, { via: 'hwp' }); // HWP5 저장이 잃는 것
```

라운드트립인데 옵션을 주려면 `pathB` 자리에 `undefined` 를 넘긴다. 경유 포맷(`via`)은
두 파일 비교에서는 의미가 없어 **무시된다**(봉투의 `via` 가 `null`).

회귀가 난 쪽만 좁혀 보거나, 렌더 반올림 수준의 흔들림을 회귀로 세지 않으려면:

```ts
await renderDiff('보고서.hwp', undefined, { page: 3 });      // 그 쪽만 (0 기준)
await renderDiff('보고서.hwp', undefined, { maxDisp: 2.0 }); // 임계를 올린다
```

**회귀는 예외가 아니라 봉투의 `status`·`regression` 필드다.** `--json` 모드에서 회귀
검출은 exit 3(판정)이지 exit 1(고장)이 아니다. 도구는 정상 동작했고, 문서에 대한 단언이
실패한 것이다 — `verify.identical` 과 같은 규약이다.

> **CLI 를 직접 쓰던 스크립트를 옮길 때**: 사람용 출력 모드의 `render-diff` 는 회귀에
> **종전대로 exit 1** 을 낸다. 새 의미론(3)은 `--json` 소비자에게만 준다 — 이미 1 을
> 실패로 읽는 CI 스크립트를 깨지 않기 위해서다. 모드에 따라 코드가 다르다는 것만
> 기억하면 된다.

폴더를 통째로 도는 `--batch` 축은 이 함수가 감싸지 않는다. NDJSON 스트림이라 반환 타입이
다르기 때문이다(한 함수가 봉투와 배열을 다 돌려주면 호출자가 받은 값을 타입으로 알 수
없다). 필요하면 저수준으로 부른다.

```ts
import { runNdjson } from '@rhwp/node';

const rows = await runNdjson(['render-diff', '--batch', '샘플/', '-o', '결과/', '--json']);
for (const r of rows) if (r.regression) console.warn(r.sample, r.status);
```

---

## 6. 눈검증 루프 닫기

**하고 싶은 것**: 편집 후 **바뀐 쪽만** 그려서 확인한다. 전 쪽을 그리면 비용이 폭발한다.

```ts
import { fillFields, openDocument } from '@rhwp/node';

export async function editAndShow(
  source: string,
  target: string,
  data: Record<string, string>,
): Promise<string[]> {
  const result = await fillFields(source, data, { out: target, verify: true });

  const pages = result.changedPages;
  if (pages === null) {
    // 확정 불가 — 부분 목록보다 정직하다. 이럴 땐 전체를 보거나 포기한다.
    console.warn('바뀐 쪽을 확정할 수 없습니다 (전체 확인 필요)');
    return [];
  }

  const made: string[] = [];
  const doc = await openDocument(target);
  try {
    for (const page of pages) {
      const svg = `${target}.p${page}.svg`;
      await doc.renderPage(page, svg);
      made.push(svg);
    }
  } finally {
    await doc.close();
  }
  return made;
}
```

`null`(모름)과 `[]`(바뀐 쪽 없음)을 구분하는 것이 요점이다. 둘을 falsy 로 뭉뚱그리면
"확인할 게 없다"고 잘못 결론 낸다.

---

## 7. 보호 문서 다루기

```ts
import { openDocument } from '@rhwp/node';

const doc = await openDocument('보호문서.hwp', { password: process.env.DOC_PASSWORD });
try {
  const text = await doc.text({ page: 0 });
  console.log(String(text.get('pages')).slice(0, 200));
} finally {
  await doc.close();
}
```

암호가 틀리면 `RhwpRuntimeError` 다 — 인자 문제(`UsageError`)가 아니라 문서를 열 수 없는
것이므로 런타임 실패다.

**암호를 인자로 조립하지 않는다.** 서버는 응답·세션 상태에 보존하지 않고, 무상태 명령에서는
자식 stdin 으로만 전달한다. 그래서 `ps` 출력이나 셸 히스토리에 암호가 남지 않는다.
소스에 하드코딩하지 말고 환경변수·비밀 저장소에서 읽는다.

---

## 8. 아카이브 대장화

**하고 싶은 것**: 폴더의 문서 수백 개를 한 번에 조사한다.

```ts
import { readdir } from 'node:fs/promises';
import { extname, join } from 'node:path';

import { batch } from '@rhwp/node';

export async function catalog(folder: string) {
  const entries = await readdir(folder, { withFileTypes: true, recursive: true });
  const paths = entries
    .filter((e) => e.isFile() && ['.hwp', '.hwpx'].includes(extname(e.name).toLowerCase()))
    .map((e) => join(e.parentPath ?? folder, e.name))
    .sort();

  const records = await batch('info', paths);
  return {
    ok: records.filter((r) => !('error' in r)),
    failed: records.filter((r) => 'error' in r),
  };
}
```

**부분 실패도 실패지만 성공분은 남는다.** 실패 하나로 스트림을 통째로 버리면 수백 건의
성공까지 잃는다.

수천 건이면 배열로 모으지 말고 흘린다 — 메모리가 목록 길이에 비례해 늘지 않는다.

```ts
import { iterNdjson } from '@rhwp/node';

for await (const record of iterNdjson(['batch', 'info', '--json'], { stdin: paths.join('\n') })) {
  handle(record);
}
```

---

## 9. 계획으로 안전하게 편집

**하고 싶은 것**: 여러 편집 중 하나라도 불가능하면 **아무것도 하지 않는다**.

```ts
import { writeFile } from 'node:fs/promises';

import { Plan } from '@rhwp/node';

export async function submitForm(form: string, output: string, name: string, dept: string) {
  const plan = new Plan(form, output)
    .fillFields({ 성명: name, 부서: dept })
    .replaceText('2025년', '2026년')
    .setCheckbox(0)
    .requireAllFieldsFound()
    .verify();

  // 1) 검사 — 디스크를 건드리지 않는다.
  const preview = await plan.check();
  if (!preview.ok) {
    console.error('계획에 문제가 있습니다:');
    console.error(preview.describeViolations());
    return false;
  }
  for (const step of preview.preview) console.log('  예정:', step);

  // 2) 계획서를 남긴다 — 감사 추적·재현이 따라온다.
  await writeFile('제출계획.json', JSON.stringify(plan, null, 2), 'utf8');

  // 3) 실행 — 전 step 이 메모리에서 통과해야 저장한다.
  const journal = await plan.run();
  return journal.verify?.identical === true;
}
```

**위반은 예외가 아니라 결과다.** 계획을 고쳐 다시 검사하는 것이 정상 흐름이다.
`JSON.stringify(plan)` 이 그대로 계획서가 되는 것은 `toJSON()` 덕분이다.

### `check()` 가 던지는 경우 — 계획이 아니라 rhwp 문제다

`check()` 는 계획서를 보내기 전에 `capabilities` 로 rhwp 의 `run` 이 `--dry-run` 을
선언하는지 확인하고, 아니면 **실행하지 않고 예외를 던진다.**

```ts
try {
  preview = await plan.check();
} catch (e) {
  // "이 rhwp 는 계획 --dry-run 을 지원하지 않습니다 (#3759 이전 버전)."
  console.error('rhwp 를 갱신하세요. 검사 없이 run() 을 부를지는 사람이 정합니다.');
  return false;
}
```

`--dry-run` 을 모르는 rhwp 는 계획서의 `dryRun` 필드를 **무시하고 편집·저장한다**.
바인딩이 그대로 내려보내면 호출자는 "검사만 했다"고 믿는데 문서가 바뀐다 — 실패도
예외도 없이. 그래서 확실히 검사인 경우에만 진행한다. 이 예외는 계획 위반
(`preview.ok === false`)과 전혀 다른 사건이다.

---

## 10. 문서 간 전사

**하고 싶은 것**: A 문서의 값을 읽어 B 서식에 옮긴다.

```ts
import { fields, Plan } from '@rhwp/node';

interface FieldRecord { name: string; value?: string }

export async function transcribe(
  source: string,
  form: string,
  output: string,
  mapping: Record<string, string>,   // { 원본_누름틀: 대상_누름틀 }
): Promise<boolean> {
  const survey = await fields(source);
  const values = new Map(
    (survey.get('fields') as FieldRecord[]).map((f) => [f.name, f.value ?? '']),
  );

  const data: Record<string, string> = {};
  for (const [src, dst] of Object.entries(mapping)) {
    if (!values.has(src)) {
      console.error(`원본에 '${src}' 이 없습니다`);
      return false;
    }
    data[dst] = values.get(src)!;
  }

  const plan = new Plan(form, output).fillFields(data).requireAllFieldsFound().verify();
  if (!(await plan.check()).ok) return false;
  const journal = await plan.run();
  return journal.verify?.identical === true;
}
```

---

## 11. 브라우저에서 쓰기

**하고 싶은 것**: 같은 업무 로직을 서버와 브라우저 양쪽에서 돌린다.

서브프로세스는 브라우저에 없다. 그래서 브라우저 경로는 `@rhwp/editor` WASM 을 감싸되,
**같은 `RhwpClient` 인터페이스**를 구현한다.

```ts
// shared/report.ts — 환경을 모르는 업무 로직
import type { RhwpClient } from '@rhwp/node';

export async function summarize(client: RhwpClient, path: string) {
  const meta = await client.info(path);
  return { pages: meta.get('pageCount'), format: meta.get('format') };
}
```

```ts
// server.ts — 1층 함수가 그대로 인터페이스를 만족한다
import { info, exportText, search } from '@rhwp/node';

const nodeClient: RhwpClient = { info, exportText, search };
const result = await summarize(nodeClient, '보고서.hwp');
```

```ts
// browser.ts
import { createBrowserClient } from '@rhwp/node/browser';

const wasm = await loadRhwpWasm();          // @rhwp/editor 를 호출자가 로드한다
const client = createBrowserClient(wasm);
const result = await summarize(client, '/uploads/보고서.hwp');
```

**한계를 먼저 알고 쓰자.**

| 항목 | Node | 브라우저 |
|---|---|---|
| 실행 방식 | rhwp 서브프로세스 | WASM 인프로세스 |
| 파일 쓰기 명령(`-o`) | 가능 | 불가 — 결과를 값으로 받아 저장은 호출자 몫 |
| 세션(2층) | `mcp-serve` 핸들 | WASM 인스턴스가 곧 세션 |
| 번들 크기 | 무관 | WASM 만큼 늘어난다 |

WASM 모듈을 바인딩이 자동으로 끌어오지 않는 것은 의도다 — 번들 크기와 로딩 시점은 앱마다
다르고, 바인딩이 정하면 되돌릴 방법이 없다.

---

## 12. Express 업로드 처리기

**하고 싶은 것**: 업로드된 HWP 를 검사해 메타와 표를 돌려준다.

```ts
import express from 'express';
import { rm } from 'node:fs/promises';

import { openDocument, RhwpError, UsageError } from '@rhwp/node';

const app = express();

app.post('/inspect', async (req, res) => {
  const upload = req.file!.path;
  const doc = await openDocument(upload).catch((e: unknown) => {
    throw e;   // 아래 catch 로
  });
  try {
    const [meta, tables] = [await doc.info(), await doc.tables()];
    res.json({ pageCount: meta.get('pageCount'), tableCount: tables.get('tableCount') });
  } catch (e) {
    if (e instanceof UsageError) res.status(400).json({ error: e.message, hint: e.suggestion });
    else if (e instanceof RhwpError) res.status(422).json({ error: e.lastDiagnostic });
    else throw e;
  } finally {
    await doc.close();          // 세션을 남기면 파일이 잠긴다
    await rm(upload, { force: true });
  }
});
```

**요청마다 세션을 닫는 것이 핵심이다.** 자식 프로세스가 누적되면 서버는 조용히 느려지다가
파일 핸들이 고갈된다. 타임아웃도 반드시 건다(`timeoutMs`) — 신뢰할 수 없는 입력에
무제한 대기를 주면 그것이 곧 서비스 거부 경로다.

---

## 자주 하는 실수

### `verify` 를 요청하지 않고 통과로 읽기

```ts
const result = await fillFields(form, data, { out });   // verify 미요청
if (result.verify) { /* null → falsy → "실패"로 오독 */ }
```

`null` 은 "검증 안 함"이지 "검증 실패"가 아니다. 요청하지 않았으면 판정 자체가 없다.

### `changedPages` 의 `null` 과 `[]` 를 섞기

```ts
if (!result.changedPages) {
  console.log('바뀐 게 없네');    // 틀렸다 — null 은 "모른다"
}
```

```ts
if (result.changedPages === null) { /* 전체 확인 필요 */ }
else if (result.changedPages.length === 0) { /* 바뀐 쪽 없음 */ }
```

### `await` 를 빠뜨리기

```ts
const meta = info('보고서.hwp');   // Promise 다
meta.get('pageCount');             // 런타임 오류
```

1층 함수는 전부 비동기다. `strict` 타입 검사를 켜 두면 컴파일에서 잡힌다 — 켜자.

### 좌표를 추측하기

```ts
await setCell(path, 0, 0, 0, '값');   // 표 0 이 있다는 보장 없음
```

`exportTables` 로 실존 좌표를 먼저 확인한다. 병합으로 덮인 칸은 목록에 없다.

### 쪽 기준을 섞기

`extractPages` 의 `from`·`to` 만 **1 기준**이고 나머지 `page` 인자는 전부 **0 기준**이다.
그대로 옮겨 쓰면 오류 없이 한 쪽 밀린 문서가 나온다.

```ts
const hits = await search('보고서.hwp', '예산');
const page = hits.children('matches')[0]!.get('page') as number;   // 0 기준

await extractPages('보고서.hwp', page, page, { out: '발췌.hwp' });        // 한 쪽 밀린다
await extractPages('보고서.hwp', page + 1, page + 1, { out: '발췌.hwp' }); // 이것이 맞다
```

### `extractPages` 에 범위 문자열을 넘기기

```ts
await extractPages('보고서.hwp', '2-4', { out: '발췌.hwp' });   // 컴파일 오류
```

인자는 **두 개의 숫자**다. rhwp 의 `extract-pages` 가 `--from N --to M` 만 받기 때문이며,
`--pages` 는 `digest` 쪽 어휘라 섞으면 `알 수 없는 옵션` 으로 exit 2 다. 타입 검사를
켜 두면 실행 전에 잡힌다.

`out` 도 빠뜨리지 말 것 — 타입상 선택이지만 rhwp 는 출력 경로 없이는 exit 2 다
(원본을 덮어쓰지 않는 것이 이 명령의 안전 규약이다).

### `convert` 에 `out` 을 빠뜨리기

```ts
await convert('배포본.hwpx');                        // UsageError
await convert('배포본.hwpx', { out: '편집본.hwp' }); // 이것이 맞다
```

`exportHwpx` 는 생략하면 `<입력 stem>.hwpx` 로 가지만 `convert` 는 기본 경로가 없다.
바인딩이 프로세스를 띄우기 전에 던지는 이유는, 그래야 **무엇이 빠졌는지 이름으로** 알 수
있기 때문이다 — CLI 사용법 덤프는 "인자가 틀렸다"까지만 말한다.

두 명령 모두 산출 경로가 **`-o` 가 아니라 위치 인자**다(`-o` 를 주면 실물이 exit 2 다).
바인딩이 그 차이를 흡수하므로 코드에서는 `out` 하나로 통일돼 있지만, CLI 를 직접 부를
때는 다르다.

### `thumbnail` 에 `out` 과 `base64` 를 함께 주기

```ts
const t = await thumbnail('보고서.hwp', { out: '미리보기.png', base64: true });
// 파일이 생기지 않는다. t.get('output') 은 null 이다.
```

`base64`·`dataUri` 는 파일 출력을 **대체한다.** 파일과 문자열을 둘 다 원하면 파일로 뽑은
뒤 직접 읽는다. 두 플래그를 함께 켜는 것도 안 된다 — 나중 플래그가 이겨 `dataUri` 만
오므로, `base64` 를 읽으려던 코드가 `undefined` 를 만난다.

### `exportText` 결과를 파일로 받기를 기대하기

`exportText`·`exportStructure`·`exportTables` 에는 `out` 옵션이 **없다.** 앞의 둘은 `-o`
를 줘도 `--json` 모드에서 조용히 무시되고, `exportTables` 는 stdout 이 사람용 문장으로
바뀌어 봉투 계약이 깨진다. 파일이 필요하면 봉투 내용을 직접 쓴다.

```ts
import { writeFile } from 'node:fs/promises';
const env = await exportText('보고서.hwp');
await writeFile('본문.txt', (env.get('pages') as Array<{ text: string }>).map((p) => p.text).join('\n'), 'utf8');
```

### 세션을 닫지 않기

```ts
const doc = await openDocument('a.hwp');   // finally 없이
// ... 예외 발생 ...
// 서버가 남아 파일을 잡고 있다 → 다음 작업이 막힌다
```

`try/finally` 를 쓰거나, 런타임이 지원하면 `await using` 을 쓴다.

### 대량 처리를 `Promise.all` 로 던지기

```ts
await Promise.all(paths.map((p) => info(p)));   // 프로세스 수백 개
```

`batch` 를 쓰거나 동시성 상한을 건다. 프로세스 하나가 목록 전체를 처리하는 쪽이 항상 싸다.

### 오류를 문자열로 판별하기

```ts
if (String(e).includes('찾지 못')) { /* 메시지는 계약이 아니다 */ }
```

`instanceof UsageError` 처럼 클래스로 가른다. 메시지는 개선되면 바뀌지만 종료 코드 규약은 계약이다.
