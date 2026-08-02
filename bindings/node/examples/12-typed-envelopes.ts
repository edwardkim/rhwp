/*
 * 생성 타입 — TypeScript 에서만 얻는 이득.
 *
 * 파이썬판은 동적 매핑으로 충분했다. TypeScript 는 다르다 — 타입이 계약을
 * **강제**해야 값어치가 있다. 그래서 이 바인딩은 봉투 타입(`src/envelopes.ts`)과
 * IR 타입(`src/ir.ts`)을 도구의 자기서술에서 **생성**하고, `Envelope<T>` 의 타입
 * 파라미터로 끼운다. 손으로 쓴 목록은 반드시 뒤처지고, 뒤처졌다는 사실조차
 * 드러나지 않는다.
 *
 * 이 예제가 보이는 것은 두 가지다.
 *   1. `Envelope<InfoEnvelope>` 로 `.raw.pageCount` 가 **정적으로** 좁혀진다.
 *   2. 그 정적 타입이 런타임 스키마와 어긋나면 **그 사실이 드러난다**.
 *
 *   npx tsx examples/12-typed-envelopes.ts 문서.hwp
 *
 * 주의: 4)번 스키마 확인은 `export-ir-schema` 를 쓴다. 이 명령은 아직 모든 rhwp 에
 * 있지 않다(M18 / #3762) — 없는 바이너리는 사용법 오류(exit 2)로 답한다.
 */

import process from 'node:process';

import * as rhwp from '../src/index.js';

/**
 * 1층 봉투를 생성 타입으로 **다시 본다**.
 *
 * `info()` 는 `Envelope<RawEnvelope>` 를 돌려준다 — 1층 함수는 명령별 봉투 타입을
 * 모른다. 명령 이름과 봉투 타입의 짝은 생성물(`EnvelopeByCommand`)이 알고 있으므로,
 * 그 짝을 아는 호출부에서 이름을 붙여 준다. 캐스팅이 한 곳에 모여 있고 이름이
 * 붙어 있는 편이, 사용처마다 `as` 가 흩어지는 것보다 낫다.
 *
 * 캐스팅이 싫다면 저수준 `runJson<T>` 이 타입 파라미터를 직접 받는다(1층 내부가
 * 쓰는 것도 그것이다). 대신 인자 조립을 손으로 하게 되므로, 여기서는 1층을 쓴다.
 */
function typed<T extends rhwp.RawEnvelope>(envelope: rhwp.Envelope): rhwp.Envelope<T> {
  return new rhwp.Envelope(envelope.raw as T);
}

/**
 * `src/ir.ts` 의 `Paragraph` 가 요구하는 필수 필드 — 컴파일러가 강제한다.
 *
 * rhwp 가 IR 에 필수 필드를 더하고 타입을 다시 생성하면 **이 리터럴이 컴파일되지
 * 않는다**. 즉 스키마 변화가 문서가 아니라 빌드에서 드러난다.
 */
const PARAGRAPH_SHAPE: rhwp.Paragraph = {
  charCount: 0,
  controls: [],
  paraShapeId: 0,
  text: '',
};

/** `export-ir-schema` 가 없는 rhwp 를 만났을 때의 안내. */
function reportMissingSchemaCommand(error: rhwp.UsageError): number {
  console.error('\n4) IR 스키마: 이 rhwp 는 export-ir-schema 를 모릅니다 (M18 / #3762 미머지).');
  console.error(`   ${error.message.split('\n')[0] ?? error.message}`);
  console.error('   위 1~3 번(생성 봉투 타입)은 이 명령과 무관하게 동작합니다.');
  console.error('   명령이 있는지 먼저 보려면: (await rhwp.capabilities()).raw.commands');
  // 명령이 없는 것은 **사용법 오류**다. 문서 판정 실패(3)도 도구 고장(1)도 아니므로
  // 도구가 준 코드를 그대로 전달한다.
  return 2;
}

/** 생성 IR 타입이 런타임 스키마와 어긋났는지 본다. 어긋나면 재생성이 필요하다는 신호다. */
function checkParagraphDrift(schema: rhwp.IrSchema): number {
  if (!schema.has('Paragraph')) {
    // 정의 이름이 사라졌다는 것은 드리프트를 넘어 스키마가 다른 물건이 됐다는 뜻이다.
    // 없는 정의를 `get()` 하면 예외가 나므로, 여기서 사실만 보고하고 끊는다.
    console.log(`   스키마에 Paragraph 정의가 없습니다. 있는 정의: ${schema.names().join(', ')}`);
    return 1;
  }

  const required = schema
    .get('Paragraph')
    .fields.filter((f) => f.required)
    .map((f) => f.name);
  const declared = new Set(Object.keys(PARAGRAPH_SHAPE));
  const missing = required.filter((name) => !declared.has(name));

  console.log(`   Paragraph 필수 필드(런타임): ${[...required].sort().join(', ')}`);
  if (missing.length === 0) {
    console.log('   생성 타입과 일치합니다.');
    return 0;
  }

  console.log(`   생성 타입에 없는 필수 필드: ${missing.join(', ')}`);
  console.log('   → `npm run gen:types` 로 다시 생성하세요 (스키마가 단일 출처입니다).');
  // 도구도 문서도 정상이다. 틀린 것은 "생성 타입이 최신"이라는 단언이므로 3.
  return 3;
}

async function main(path: string): Promise<number> {
  // 1) 봉투에 이름 붙이기.
  const meta = typed<rhwp.InfoEnvelope>(await rhwp.info(path));

  // `.raw` 가 정적으로 좁혀진다 — 타입을 명시해 두면 컴파일러가 대조한다.
  const pageCount: number | undefined = meta.raw.pageCount;
  const fonts: readonly string[] | undefined = meta.raw.fonts;
  console.log(`1) 정적 타입: pageCount=${pageCount}, 글꼴 ${fonts?.length ?? 0}종`);

  // 오타를 어디까지 잡아 주는지 정확히 말해 두자.
  //
  //   봉투는 **추가-전용** 계약이라 생성 인터페이스에 인덱스 시그니처가 있다.
  //   그래서 `meta.raw.pageCont` 는 "없는 속성"이 아니라 `unknown` 이다. 그 자체로는
  //   오류가 아니지만, 위처럼 **타입을 명시한 자리에 담는 순간** 오류가 된다:
  //
  //     const n: number | undefined = meta.raw.pageCont;
  //     //    'unknown' 형식은 'number | undefined' 형식에 할당할 수 없습니다.
  //
  //   그러니 `.raw` 는 받는 자리의 타입을 적어서 쓴다. `const n = meta.raw.pageCont`
  //   처럼 받으면 `unknown` 을 그대로 안고 가게 되고, 오타는 나중에 드러난다.

  // 2) 정적 타입과 런타임 조회는 **서로 다른 것**을 보증한다. 둘 다 필요하다.
  //
  //    `.raw.pageCount`      — 타입은 보증하지만 **존재는 보증하지 않는다**(`| undefined`).
  //    `.get<number>('...')`  — 존재는 보증하지만(없으면 예외) **타입은 주장일 뿐**이다.
  //
  //    그래서 "반드시 있어야 하는 값"은 `.get()` 으로 읽고, "있으면 쓰는 값"은
  //    `.raw`(또는 `.getOr`)로 읽는다.
  console.log(`2) 런타임 보증: ${meta.get<number>('pageCount')}쪽, 포맷 ${meta.get<string>('format')}`);

  // 3) 명령 → 봉투 타입 표. 어떤 명령이 어떤 봉투를 내는지가 생성물에 적혀 있으므로,
  //    소비자가 그 짝을 손으로 외울 필요가 없다.
  type TablesRaw = rhwp.EnvelopeByCommand['export-tables'];
  const tables = typed<TablesRaw>(await rhwp.exportTables(path));
  const tableCount: number | undefined = tables.raw.tableCount;
  console.log(`3) export-tables 봉투: 표 ${tableCount ?? 0}개`);

  // 4) 생성 타입의 원천을 런타임에 확인한다.
  let schema: rhwp.IrSchema;
  try {
    schema = await rhwp.irSchema();
  } catch (error) {
    if (!(error instanceof rhwp.UsageError)) throw error;
    return reportMissingSchemaCommand(error);
  }

  console.log(`\n4) IR 스키마 v${schema.version} — 정의 ${schema.size}개, 루트 ${schema.root.name}`);
  return checkParagraphDrift(schema);
}

const argv = process.argv.slice(2);
const [docPath] = argv;
if (argv.length !== 1 || docPath === undefined) {
  console.error('사용법: npx tsx examples/12-typed-envelopes.ts 문서.hwp');
  process.exit(2);
}

process.exit(await main(docPath));
