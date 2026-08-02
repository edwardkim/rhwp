/*
 * IR 스키마 탐색 — 바인딩이 IR 모양을 하드코딩하지 않는 이유.
 *
 * `src/ir.ts` 의 타입은 손으로 쓴 것이 아니라 도구가 스스로 보고한 스키마에서
 * 생성된다. 그래서 IR 이 바뀌면 바인딩이 조용히 틀리는 게 아니라 생성 결과가
 * 달라진다. 이 예제는 그 원천을 그대로 들여다본다.
 *
 *   npx tsx examples/06-ir-schema.ts [타입이름]
 *
 * 주의: `export-ir-schema` 는 아직 모든 rhwp 에 있지 않다(M18 / #3762). 명령을
 * 모르는 바이너리는 **사용법 오류(exit 2)** 로 답한다 — 이 예제는 그 답을 그대로
 * 전달한다. 자세한 사정은 examples/README.md 를 보라.
 */

import process from 'node:process';

import * as rhwp from '../src/index.js';

/** `export-ir-schema` 가 없는 rhwp 를 만났을 때의 안내. */
function reportMissingCommand(error: rhwp.UsageError): void {
  console.error('이 rhwp 는 export-ir-schema 를 모릅니다 (M18 / #3762 미머지).');
  console.error(`  ${error.message}`);
  console.error('  명령 표면은 `rhwp capabilities` 가 단일 출처다 — 있는지 먼저 확인할 수 있다:');
  console.error("    (await rhwp.capabilities()).raw.commands");
}

async function main(target: string | undefined): Promise<number> {
  let schema: rhwp.IrSchema;
  try {
    schema = await rhwp.irSchema();
  } catch (error) {
    // 명령이 없는 것은 **사용법 오류**다(호출이 틀렸다). 문서 판정 실패도, 도구
    // 고장도 아니므로 종료 코드를 바꿔 포장하지 않고 그대로 2 로 전달한다.
    if (!(error instanceof rhwp.UsageError)) throw error;
    reportMissingCommand(error);
    return 2;
  }

  const names = schema.names();
  console.log(
    `IR 스키마 v${schema.version} (${schema.dialect}) — 정의 ${names.length}개, 루트 ${schema.root.name}`,
  );

  // 끊어진 참조 = 어떤 필드가 존재하지 않는 타입을 가리킨다.
  // 이건 문서의 문제가 아니라 스키마 자체가 깨진 것이므로 런타임 실패로 다룬다.
  const dangling = schema.danglingReferences();
  if (dangling.length > 0) {
    console.log('끊어진 참조:');
    for (const [where, missing] of dangling) {
      console.log(`  ${where} → ${missing} (정의 없음)`);
    }
    return 1;
  }

  if (target === undefined) {
    console.log('\n정의 목록:');
    // `for (const def of schema)` 로도 같은 순회가 된다. 여기서는 이름 순서를
    // 그대로 보여주려고 names() 를 쓴다.
    for (const name of names) {
      const def = schema.get(name);
      const kind = def.isUnion ? 'union' : 'object';
      console.log(`  ${name.padEnd(24)} ${kind.padEnd(7)} ${def.description.slice(0, 48)}`);
    }
    console.log('\n특정 타입을 보려면: npx tsx examples/06-ir-schema.ts Paragraph');
    return 0;
  }

  // 없는 이름을 물으면 "있는 것"을 같이 알려준다 — 오타를 사람이 스스로 고칠 수 있게.
  // `schema.get()` 도 같은 목록을 담아 예외를 던지지만, 여기서는 예외 대신 종료
  // 코드로 답하는 편이 CLI 처럼 쓰기 좋다.
  if (!schema.has(target)) {
    console.log(`'${target}' 정의가 없습니다. 있는 것: ${names.join(', ')}`);
    return 2;
  }

  const def = schema.get(target);
  console.log(`\n${def.name} — ${def.description}`);

  if (def.isUnion) {
    // 유니온은 필드가 아니라 변형을 갖는다. 필드만 보면 아무것도 안 보인다.
    console.log(`  유니온 변형: ${def.variants.join(', ')}`);
  }

  for (const f of def.fields) {
    const optional = f.required ? '' : ' (선택)';
    // tsType 은 생성기가 쓰는 것과 같은 표기다. 여기 보이는 것이 곧 `src/ir.ts` 에 나온다.
    console.log(`      ${f.name.padEnd(20)} ${f.tsType}${optional}`);
    // enumValues 만은 정말로 없을 수 있다 — 열거형이 아닌 필드가 대부분이다.
    const enums = f.enumValues ?? [];
    if (enums.length > 0) {
      console.log(`      ${' '.repeat(20)}   값: ${enums.join(' | ')}`);
    }
  }

  return 0;
}

const argv = process.argv.slice(2);
if (argv.length > 1) {
  console.error('사용법: npx tsx examples/06-ir-schema.ts [타입이름]');
  process.exit(2);
}

process.exit(await main(argv[0]));
