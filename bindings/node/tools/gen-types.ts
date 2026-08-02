/**
 * 두 출처 → TypeScript 타입 생성기.
 *
 * `bindings_foundation.md` §3 이 못박은 규약을 코드로 강제한다:
 * **필드명은 봉투 키를 기계 변환한다 — 수기 개명 금지.**
 *
 * 파이썬판(M18, `bindings/python/tools/gen_models.py`)은 IR 하나만 생성했다.
 * 동적 언어에서는 `Envelope` 하나가 봉투에 있는 것을 전부 노출하므로 구조적으로
 * 뒤처질 수 없었기 때문이다. TypeScript 는 다르다 — 사용자가 이 언어를 고른 이유의
 * 상당 부분이 "필드 이름을 컴파일러가 확인해 준다"인데, 봉투를 `Record<string,
 * unknown>` 으로만 주면 바인딩이 그 값어치를 통째로 버린다. 그래서 여기서는 **두
 * 출처**에서 생성한다:
 *
 * | 출처 | 서술하는 것 | 산출 |
 * |---|---|---|
 * | `rhwp export-ir-schema` | 문서 모델(IR) | `src/ir.ts` |
 * | `rhwp capabilities` | 명령별 봉투 | `src/envelopes.ts` |
 *
 * 어느 한쪽만으로는 봉투 필드에 정적 타입을 붙일 수 없다. IR 로 봉투를 흉내 내는
 * 순간 수기 매핑이 부활한다.
 *
 * 사용법:
 *
 * ```bash
 * npm run gen:types                       # src/ir.ts · src/envelopes.ts 재생성
 * npm run gen:check                       # 디스크와 다르면 exit 1 (CI 용)
 * tsx tools/gen-types.ts --help
 * ```
 *
 * `--check` 는 생성 결과가 디스크와 다르면 **exit 1** 이다. 스키마가 바뀌었는데
 * 타입을 다시 만들지 않은 PR 을 CI 가 잡는다. 낡은 생성 타입은 "컴파일은 되는데
 * 런타임에 필드가 없는" 가장 나쁜 형태로 드러나기 때문이다.
 *
 * @packageDocumentation
 */

import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, relative, resolve } from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

import { ENV_VAR } from '../src/binary.js';
import { BinaryNotFoundError, UsageError } from '../src/errors.js';
import { propertyKey, toCamel } from '../src/naming.js';
import { runJson } from '../src/process.js';

// ── 출력 규약 ─────────────────────────────────────────────────────────────

/** 들여쓰기 한 단계. */
const INDENT = '  ';

/** 줄 바꿈 기준 폭. 넘으면 JSDoc·유니온을 접는다. */
const MAX_WIDTH = 96;

/**
 * 인터페이스·필드는 **이름순**으로 낸다.
 *
 * 파이썬판은 dataclass 정의 순서 때문에 위상 정렬이 필요했지만, TypeScript 의
 * `interface` 는 호이스팅되므로 순서가 의미를 갖지 않는다. 그렇다면 남는 기준은
 * "diff 가 안정적인가" 하나뿐이고, 이름순이 그것을 만족한다 — 스키마에 필드가
 * 하나 끼어들어도 그 줄만 바뀐다.
 */
function sortedKeys(record: Readonly<Record<string, unknown>>): string[] {
  return Object.keys(record).sort();
}

// ── 스키마 노드 ───────────────────────────────────────────────────────────

/**
 * JSON Schema 노드 — `export-ir-schema` 가 실제로 내는 형태만 좁혀 둔다.
 *
 * 범용 JSON Schema 파서를 만들지 않는 이유: 이 스키마는 `src/ir_schema.rs` 가
 * **손으로** 쓴 것이라 문법 표면이 좁고 고정돼 있다. 넓게 받으면 정작 새 문법이
 * 들어왔을 때 `unknown` 으로 조용히 뭉개진다. 모르는 모양은 드러나야 한다.
 */
interface SchemaNode {
  readonly $ref?: string;
  readonly type?: string;
  readonly enum?: readonly string[];
  readonly const?: string;
  readonly oneOf?: readonly SchemaNode[];
  readonly items?: SchemaNode;
  readonly properties?: Readonly<Record<string, SchemaNode>>;
  readonly required?: readonly string[];
  readonly description?: string;
  readonly discriminator?: { readonly propertyName?: string };
}

/** `#/$defs/Paragraph` → `Paragraph`. */
function refName(ref: string): string {
  const cut = ref.lastIndexOf('/');
  return cut === -1 ? ref : ref.slice(cut + 1);
}

/** 문자열 리터럴 타입 표기. 작은따옴표가 이 저장소의 관례다. */
function literal(value: string): string {
  return `'${value.replace(/\\/g, '\\\\').replace(/'/g, "\\'")}'`;
}

/** 유니온이면 배열 원소로 쓸 때 괄호가 필요하다. */
function needsParens(type: string): boolean {
  return type.includes(' | ');
}

/**
 * 한글·한자는 두 칸을 차지한다 — 글자 수가 아니라 **칸 수**로 접는다.
 *
 * 이 저장소의 주석은 한국어라 글자 수로 접으면 실제 폭이 두 배가 된다. 생성물만
 * 유독 화면 밖으로 흘러나가면 사람이 읽지 않게 되고, 읽지 않는 문서는 없는 것과
 * 같다. 폭이 애매한 문자(`—`·`·`·`…`)는 좁게 센다 — 대부분의 편집기가 그렇게 그린다.
 */
function displayWidth(text: string): number {
  let width = 0;
  for (const char of text) {
    const code = char.codePointAt(0) ?? 0;
    width += isWide(code) ? 2 : 1;
  }
  return width;
}

/** East Asian Wide/Fullwidth 범위. */
function isWide(code: number): boolean {
  return (
    (code >= 0x1100 && code <= 0x115f) ||
    (code >= 0x2e80 && code <= 0x303e) ||
    (code >= 0x3041 && code <= 0x33ff) ||
    (code >= 0x3400 && code <= 0x4dbf) ||
    (code >= 0x4e00 && code <= 0x9fff) ||
    (code >= 0xa000 && code <= 0xa4cf) ||
    (code >= 0xac00 && code <= 0xd7a3) ||
    (code >= 0xf900 && code <= 0xfaff) ||
    (code >= 0xfe30 && code <= 0xfe4f) ||
    (code >= 0xff00 && code <= 0xff60) ||
    (code >= 0xffe0 && code <= 0xffe6)
  );
}

// ── JSDoc ────────────────────────────────────────────────────────────────

/**
 * 설명을 한 줄로 눌러 JSDoc 에 넣을 수 있게 만든다.
 *
 * `*&#47;` 를 escape 하는 이유: 스키마 설명은 사람이 쓰는 자유 문자열이라 언젠가
 * 주석 종료 문자열이 섞인다. 그때 생성물이 **문법 오류로** 깨지면 원인을 찾기가
 * 몹시 어렵다.
 */
function normalizeText(text: string): string {
  return text.replace(/\s+/g, ' ').trim().replace(/\*\//g, '*\\/');
}

/** 폭에 맞춰 단어 단위로 접는다. */
function wrapText(text: string, width: number): string[] {
  const words = text.split(' ').filter((word) => word.length > 0);
  const lines: string[] = [];
  let current = '';
  for (const word of words) {
    if (current.length === 0) {
      current = word;
    } else if (displayWidth(current) + 1 + displayWidth(word) > width) {
      lines.push(current);
      current = word;
    } else {
      current = `${current} ${word}`;
    }
  }
  if (current.length > 0) lines.push(current);
  return lines;
}

/**
 * JSDoc 블록. 짧은 한 문단이면 한 줄로, 아니면 접어서.
 *
 * 스키마 `description` 을 여기로 옮기는 것이 이 생성기의 절반이다 — 생성 바인딩의
 * IDE 힌트는 이 주석 말고는 원천이 없다. 설명을 버리면 사용자는 필드 이름만 보고
 * 뜻을 짐작해야 한다.
 */
function jsDoc(indent: string, paragraphs: readonly (string | undefined)[]): string[] {
  const cleaned: string[] = [];
  for (const paragraph of paragraphs) {
    const text = normalizeText(paragraph ?? '');
    if (text.length > 0) cleaned.push(text);
  }
  if (cleaned.length === 0) return [];

  const only = cleaned[0];
  if (cleaned.length === 1 && only !== undefined) {
    const oneLine = `${indent}/** ${only} */`;
    if (displayWidth(oneLine) <= MAX_WIDTH) return [oneLine];
  }

  const lines: string[] = [`${indent}/**`];
  cleaned.forEach((paragraph, index) => {
    if (index > 0) lines.push(`${indent} *`);
    for (const line of wrapText(paragraph, MAX_WIDTH - indent.length - 3)) {
      lines.push(`${indent} * ${line}`);
    }
  });
  lines.push(`${indent} */`);
  return lines;
}

// ── IR 스키마 → 타입 표기 ─────────────────────────────────────────────────

/**
 * 스키마 노드 하나를 TypeScript 타입 표기로.
 *
 * `indent` 는 인라인 객체 리터럴을 낼 때만 쓴다(중첩 들여쓰기를 맞추려고).
 */
function tsType(node: SchemaNode, indent: string): string {
  if (node.$ref !== undefined) return refName(node.$ref);
  if (node.const !== undefined) return literal(node.const);

  if (node.oneOf !== undefined) {
    const parts: string[] = [];
    for (const child of node.oneOf) {
      const part = tsType(child, indent);
      if (!parts.includes(part)) parts.push(part);
    }
    return parts.length > 0 ? parts.join(' | ') : 'unknown';
  }

  if (node.enum !== undefined) {
    return node.enum.map(literal).join(' | ');
  }

  switch (node.type) {
    case 'string':
      return 'string';
    case 'integer':
    case 'number':
      return 'number';
    case 'boolean':
      return 'boolean';
    case 'null':
      return 'null';
    case 'array': {
      const items = node.items;
      if (items === undefined) return 'readonly unknown[]';
      const element = tsType(items, indent);
      return `readonly ${needsParens(element) ? `(${element})` : element}[]`;
    }
    case 'object':
      return node.properties === undefined
        ? 'Record<string, unknown>'
        : inlineObject(node, indent);
    default:
      // 모르는 모양은 `unknown` 이 정직하다 — 짐작한 타입은 컴파일러의 보증으로 둔갑한다.
      return 'unknown';
  }
}

/**
 * 이름 없는 중첩 객체를 인라인 타입 리터럴로.
 *
 * 이름을 지어 최상위 인터페이스로 끌어올리지 않는 이유: 그 이름은 **스키마에 없는
 * 것**이고, 생성기가 지어낸 이름은 스키마가 바뀔 때 근거 없이 흔들린다. 인라인은
 * 조금 장황하지만 출처가 분명하다.
 */
function inlineObject(node: SchemaNode, indent: string): string {
  const inner = indent + INDENT;
  const properties = node.properties ?? {};
  const required = new Set(node.required ?? []);
  const blocks: MemberBlock[] = [];
  for (const key of sortedKeys(properties)) {
    const child = properties[key];
    if (child === undefined) continue;
    blocks.push(renderField(key, child, required.has(key), inner));
  }
  // 인라인 객체는 이미 중첩돼 있어 빈 줄까지 넣으면 부모 필드가 화면을 덮는다.
  const lines: string[] = ['{'];
  for (const block of blocks) lines.push(...block.lines);
  lines.push(`${inner}readonly [key: string]: unknown;`);
  lines.push(`${indent}}`);
  return lines.join('\n');
}

/** 유니온 멤버를 한 줄에 하나씩. 마지막에만 세미콜론. */
function foldUnion(members: readonly string[], indent: string): string[] {
  return members.map(
    (member, index) => `${indent}| ${member}${index === members.length - 1 ? ';' : ''}`,
  );
}

/** 멤버 하나 — 주석이 붙었는지까지 알아야 빈 줄을 어디에 넣을지 정할 수 있다. */
interface MemberBlock {
  readonly lines: readonly string[];
  readonly documented: boolean;
}

/**
 * 멤버 블록을 이어 붙인다. **주석이 붙은 블록의 앞뒤에만** 빈 줄을 넣는다.
 *
 * 무조건 빈 줄을 넣으면 주석 하나 없는 봉투 인터페이스가 두 배로 늘어나 한눈에
 * 안 들어오고, 아예 안 넣으면 IR 처럼 필드마다 설명이 붙은 곳에서 주석과 선언이
 * 서로 달라붙는다. 기준을 "주석이 있는가" 하나로 두면 둘 다 읽을 만해진다.
 */
function joinMembers(blocks: readonly MemberBlock[]): string[] {
  const lines: string[] = [];
  blocks.forEach((block, index) => {
    const previous = index > 0 ? blocks[index - 1] : undefined;
    if (previous !== undefined && (previous.documented || block.documented)) lines.push('');
    lines.push(...block.lines);
  });
  return lines;
}

/** 필드 하나 — JSDoc + 선언. 긴 유니온은 접는다. */
function renderField(
  name: string,
  node: SchemaNode,
  required: boolean,
  indent: string,
): MemberBlock {
  const lines = jsDoc(indent, [node.description]);
  const documented = lines.length > 0;
  const type = tsType(node, indent);
  const head = `${indent}readonly ${propertyKey(name)}${required ? '' : '?'}:`;
  const single = `${head} ${type};`;

  // 인라인 객체(여러 줄)나 배열은 접지 않는다 — 접으면 오히려 읽기 어려워진다.
  const foldable =
    displayWidth(single) > MAX_WIDTH &&
    !type.includes('\n') &&
    type.includes(' | ') &&
    !type.startsWith('readonly ');
  if (foldable) {
    lines.push(head);
    lines.push(...foldUnion(type.split(' | '), indent + INDENT));
  } else {
    lines.push(single);
  }
  return { lines, documented };
}

/** `export type X = A | B;` — 길면 접는다. */
function renderAlias(
  name: string,
  doc: readonly (string | undefined)[],
  parts: readonly string[],
): string[] {
  const lines = jsDoc('', doc);
  const members = parts.length > 0 ? parts : ['unknown'];
  const single = `export type ${name} = ${members.join(' | ')};`;
  if (displayWidth(single) <= MAX_WIDTH) {
    lines.push(single);
    return lines;
  }
  lines.push(`export type ${name} =`);
  lines.push(...foldUnion(members, INDENT));
  return lines;
}

/** 정의 하나를 인터페이스 또는 타입 별칭으로. */
function renderDefinition(name: string, node: SchemaNode): string[] {
  if (node.oneOf !== undefined) {
    const discriminator = node.discriminator?.propertyName;
    return renderAlias(
      name,
      [node.description, discriminator === undefined ? undefined : `판별자: \`${discriminator}\``],
      node.oneOf.map((variant) => tsType(variant, '')),
    );
  }

  if (node.enum !== undefined) {
    return renderAlias(name, [node.description], node.enum.map(literal));
  }

  if (node.type !== 'object' || node.properties === undefined) {
    return renderAlias(name, [node.description], [tsType(node, '')]);
  }

  const lines = jsDoc('', [node.description]);
  lines.push(`export interface ${name} {`);
  const properties = node.properties;
  const required = new Set(node.required ?? []);
  const blocks: MemberBlock[] = [];
  for (const key of sortedKeys(properties)) {
    const child = properties[key];
    if (child === undefined) continue;
    blocks.push(renderField(key, child, required.has(key), INDENT));
  }
  lines.push(...joinMembers(blocks));
  if (blocks.length > 0) lines.push('');
  lines.push(`${INDENT}readonly [key: string]: unknown;`);
  lines.push('}');
  return lines;
}

// ── IR 스키마 적재·검증 ───────────────────────────────────────────────────

/** 생성에 필요한 만큼만 추린 IR 스키마. */
interface IrSource {
  readonly version: string;
  readonly defs: Readonly<Record<string, SchemaNode>>;
}

/** 노드 하나가 가리키는 `$ref` 를 전부 모은다 (중첩 포함). */
function collectRefs(node: SchemaNode, out: string[]): void {
  if (node.$ref !== undefined) out.push(refName(node.$ref));
  if (node.items !== undefined) collectRefs(node.items, out);
  for (const child of node.oneOf ?? []) collectRefs(child, out);
  for (const key of sortedKeys(node.properties ?? {})) {
    const child = (node.properties ?? {})[key];
    if (child !== undefined) collectRefs(child, out);
  }
}

/**
 * 끊어진 참조 — 어떤 필드가 존재하지 않는 타입을 가리킨다.
 *
 * 이건 문서의 문제가 아니라 **스키마 자체가 깨진 것**이므로 생성을 중단한다.
 * 절반쯤 만들다 죽은 파일은 컴파일은 되면서 뜻은 틀린, 최악의 산출물이 된다.
 */
function danglingReferences(defs: Readonly<Record<string, SchemaNode>>): string[] {
  const problems: string[] = [];
  for (const name of sortedKeys(defs)) {
    const node = defs[name];
    if (node === undefined) continue;
    const refs: string[] = [];
    collectRefs(node, refs);
    for (const target of refs) {
      if (!(target in defs)) problems.push(`${name} → ${target}`);
    }
  }
  return problems;
}

/** `export-ir-schema` 봉투에서 스키마 본문과 버전을 꺼낸다. */
function parseIrSchema(envelope: Record<string, unknown>): IrSource {
  const schema = envelope.schema ?? envelope;
  if (schema === null || typeof schema !== 'object' || Array.isArray(schema)) {
    throw new Error('IR 스키마 봉투에 `schema` 객체가 없습니다.');
  }
  const body = schema as Record<string, unknown>;
  const defs = body.$defs;
  if (defs === null || defs === undefined || typeof defs !== 'object' || Array.isArray(defs)) {
    throw new Error('IR 스키마에 `$defs` 가 없습니다 — 봉투 모양이 바뀌었는지 확인하세요.');
  }
  const version =
    typeof envelope.irSchemaVersion === 'string'
      ? envelope.irSchemaVersion
      : typeof body.irSchemaVersion === 'string'
        ? body.irSchemaVersion
        : '(알 수 없음)';
  return { version, defs: defs as Readonly<Record<string, SchemaNode>> };
}

// ── IR 산출물 ────────────────────────────────────────────────────────────

/** 자동 생성 표시. 사람이 이 파일을 고치지 않게 하는 것이 첫 줄의 임무다. */
function irHeader(source: IrSource, count: number): string[] {
  return [
    '/**',
    ' * IR 타입 — **자동 생성 파일. 손으로 고치지 마세요.**',
    ' *',
    ' * 재생성: `npm run gen:types` (tools/gen-types.ts)',
    ` * 출처:   \`rhwp export-ir-schema\` — irSchemaVersion ${source.version}, 정의 ${count}개`,
    ' *',
    ' * 이 파일을 직접 고치면 다음 생성에서 사라집니다. 모양을 바꾸려면 rhwp 본체의',
    ' * `src/ir_schema.rs` 를 고치세요 — 스키마가 단일 출처이고 이 파일은 그 그림자입니다.',
    ' *',
    ' * 모든 인터페이스에 두 규약이 적용됩니다:',
    ' *',
    ' * - **전 필드 `readonly`** — 봉투는 도구가 준 관찰값이지 편집 대상이 아닙니다.',
    ' *   여기서 값을 고쳐도 문서는 바뀌지 않으므로, 고칠 수 있게 두면 그 오해가 조용히',
    ' *   자란다.',
    ' * - **인덱스 시그니처(`readonly [key: string]: unknown`)** — IR 은 추가-전용으로',
    ' *   진화합니다(`additionalProperties: true`). rhwp 가 필드를 하나 더할 때마다 모든',
    ' *   소비자가 타입 오류로 깨지면 계약이 아니라 족쇄가 됩니다.',
    ' *',
    ' * 정의와 필드는 이름순입니다 — `interface` 는 호이스팅되므로 순서에 의미가 없고,',
    ' * 이름순이어야 스키마가 조금 바뀔 때 diff 도 조금만 바뀝니다.',
    ' *',
    ' * @packageDocumentation',
    ' */',
    '',
    '/** 이 파일을 만들어 낸 IR 스키마 버전. 봉투 `schemaVersion`(명령별)과는 별개입니다. */',
    `export const IR_SCHEMA_VERSION = ${literal(source.version)};`,
  ];
}

/** IR 스키마 전체를 `src/ir.ts` 소스로. */
function generateIr(source: IrSource): string {
  const dangling = danglingReferences(source.defs);
  if (dangling.length > 0) {
    throw new Error(
      `끊어진 참조가 있어 생성을 중단합니다: ${dangling.join(', ')}\n` +
        '  (스키마가 존재하지 않는 정의를 가리킵니다 — src/ir_schema.rs 를 확인하세요)',
    );
  }

  const names = sortedKeys(source.defs);
  const lines = irHeader(source, names.length);
  for (const name of names) {
    const node = source.defs[name];
    if (node === undefined) continue;
    lines.push('');
    lines.push(...renderDefinition(name, node));
  }
  return `${lines.join('\n')}\n`;
}

// ── capabilities → 봉투 타입 ─────────────────────────────────────────────

/**
 * 이름만으로 타입이 확실한 봉투 필드 — **여러 명령에 같은 뜻으로 나오는 것만.**
 *
 * `capabilities` 는 명령마다 *어떤 필드가 있는지*(`recordFields`)만 선언하고 타입은
 * 말하지 않는다. 그래서 여기 없는 필드는 전부 `unknown` 이다. 짐작한 타입을 적는
 * 순간 그 짐작이 컴파일러의 보증으로 둔갑하고, 사용자는 검사받았다고 믿은 채 틀린
 * 코드를 쓴다. `unknown` 은 한 번 좁히도록 강제하고, 그 지점이 곧 "여기서 계약을
 * 가정했다"는 표시가 된다.
 *
 * 이 표의 항목은 전부 실제 봉투에서 값을 확인한 것이다.
 */
const GLOBAL_FIELD_TYPES: Readonly<Record<string, string>> = {
  bytes: 'number',
  /** 편집이 바꾼 쪽 목록. `null`(확정 불가)과 `[]`(없음)은 다른 결론이다. */
  changedPages: 'readonly number[] | null',
  format: 'string',
  output: 'string',
  outputDir: 'string',
  outputFormat: 'string',
  pageCount: 'number',
  paraCount: 'number',
  renderedCount: 'number',
  schemaVersion: 'string',
  source: 'string',
  truncated: 'boolean',
  /** `--verify` 미요청이면 `null` — "검증 안 함"은 "검증 실패"가 아니다. */
  verify: 'RawVerifyReport | null',
};

/**
 * 한 명령에만 나오거나, 명령마다 뜻이 다른 필드.
 *
 * 전역 표에 올리면 거짓말이 되는 실례가 있다: `sections` 는 `info` 에서 구역
 * **개수**(`number`)지만 `digest --sections` 에서는 절 **목록**(배열)이다. 이름이
 * 같다고 타입이 같지 않다 — 그래서 이 표는 명령별이다.
 */
const COMMAND_FIELD_TYPES: Readonly<Record<string, Readonly<Record<string, string>>>> = {
  batch: { error: 'string' },
  'build-from-ingest': { paragraphCount: 'number', questionCount: 'number' },
  capabilities: { tool: 'string', version: 'string' },
  convert: { wasDistribution: 'boolean' },
  digest: { excerpt: 'string', nextStep: 'string' },
  edit: { dryRun: 'boolean', filledCount: 'number', replacedCount: 'number' },
  'export-capabilities-schema': {
    capabilitiesSchemaVersion: 'string',
    definitionCount: 'number',
    dialect: 'string',
  },
  'export-doclang': {
    assetCount: 'number',
    assetsDir: 'string',
    doclangVersion: 'string',
    lossCount: 'number',
  },
  'export-markdown': { imageCount: 'number' },
  'export-pdf': { backend: 'string' },
  'export-structure': { mode: 'string', nodeCount: 'number' },
  'export-tables': { tableCount: 'number' },
  'extract-pages': {
    from: 'number',
    pagesAfter: 'number',
    pagesBefore: 'number',
    paragraphsKept: 'number',
    paragraphsRemoved: 'number',
    to: 'number',
  },
  fields: { fieldCount: 'number' },
  info: {
    fonts: 'readonly string[]',
    sections: 'number',
    sizeBytes: 'number',
    title: 'string',
    version: 'string',
  },
  'ir-diff': { a: 'string', b: 'string', diffCount: 'number', identical: 'boolean' },
  run: { planVersion: 'string' },
  search: {
    caseSensitive: 'boolean',
    matchCount: 'number',
    query: 'string',
    totalMatchCount: 'number',
  },
  thumbnail: { height: 'number', mime: 'string', width: 'number' },
};

/** `recordFields` 를 선언한 명령 하나. */
interface CommandSpec {
  readonly name: string;
  readonly summary: string | undefined;
  readonly fields: readonly string[];
  /** `--json` 을 실제로 받는지. `capabilities` 는 봉투를 내면서도 이 플래그를 거부한다. */
  readonly takesJsonFlag: boolean;
}

/** `export-tables` → `ExportTablesEnvelope`. 개명이 아니라 기계 변환이다. */
function envelopeTypeName(command: string): string {
  // `toCamel` 은 `_` 만 가른다. 명령 이름의 구분자는 `-` 이므로 규칙을 빌려 쓰기 위해
  // 먼저 치환한다 — 여기서 별도 규칙을 만들면 naming.ts 와 갈라진다.
  const camel = toCamel(command.replace(/-/g, '_'));
  return `${camel.charAt(0).toUpperCase()}${camel.slice(1)}Envelope`;
}

/**
 * `steps[].confusable` 같은 중첩 표기에서 최상위 필드 이름만 떼어 낸다.
 *
 * 중첩을 최상위 필드로 환산하지 않는 이유: `capabilities` 는 중첩의 *모양*을
 * 서술하지 않는다. 환산하면 없는 필드를 있다고 선언하게 된다.
 */
function fieldHead(field: string): string {
  const cut = field.search(/[\[.]/);
  return cut === -1 ? field : field.slice(0, cut);
}

/** 명령 하나의 봉투 인터페이스. */
function renderEnvelope(spec: CommandSpec): string[] {
  const overrides = COMMAND_FIELD_TYPES[spec.name] ?? {};

  // 중첩 표기는 최상위 이름으로 합치되, 원문 표기는 주석으로 남긴다.
  const heads: string[] = [];
  const notations = new Map<string, string[]>();
  for (const field of spec.fields) {
    const head = fieldHead(field);
    if (!heads.includes(head)) {
      heads.push(head);
      notations.set(head, []);
    }
    if (field !== head) notations.get(head)?.push(field);
  }

  const invocation = `\`rhwp ${spec.name}${spec.takesJsonFlag ? ' --json' : ''}\` 봉투.`;
  const lines = jsDoc('', [invocation, spec.summary]);
  lines.push(`export interface ${envelopeTypeName(spec.name)} {`);

  const blocks: MemberBlock[] = [];
  for (const head of heads.slice().sort()) {
    const nested = notations.get(head) ?? [];
    const note =
      `중첩 표기: ${nested.join(', ')} — 중첩의 모양은 capabilities 가 서술하지 않으므로` +
      ' 최상위 필드만 선언한다.';
    const doc = nested.length === 0 ? [] : jsDoc(INDENT, [note]);
    const type = overrides[head] ?? GLOBAL_FIELD_TYPES[head] ?? 'unknown';
    blocks.push({
      lines: [...doc, `${INDENT}readonly ${propertyKey(head)}?: ${type};`],
      documented: doc.length > 0,
    });
  }
  lines.push(...joinMembers(blocks));
  if (blocks.length > 0) lines.push('');
  lines.push(`${INDENT}readonly [key: string]: unknown;`);
  lines.push('}');
  return lines;
}

/** 자동 생성 표시 + 규약 설명. */
function envelopesHeader(version: string, count: number, needsVerify: boolean): string[] {
  const lines = [
    '/**',
    ' * 명령별 봉투 타입 — **자동 생성 파일. 손으로 고치지 마세요.**',
    ' *',
    ' * 재생성: `npm run gen:types` (tools/gen-types.ts)',
    ` * 출처:   \`rhwp capabilities\` — version ${version}, \`--json\` 봉투 ${count}개`,
    ' *',
    ' * `capabilities` 는 명령마다 **어떤 필드가 있는지**(`recordFields`)만 선언하고 타입은',
    ' * 말하지 않습니다. 그래서 대부분의 필드가 `unknown` 입니다 — 짐작한 타입을 적으면 그',
    ' * 짐작이 컴파일러의 보증으로 둔갑하고, 사용자는 검사받았다고 믿은 채 틀린 코드를 씁니다.',
    ' * 이름만으로 확실한 소수(`schemaVersion`·`pageCount`·`verify` …)에만 타입을 줍니다.',
    ' *',
    ' * 모든 필드가 선택(`?`)인 이유: 옵션에 따라 나오지 않는 필드가 있는데 `capabilities`',
    ' * 는 그 조건을 서술하지 않습니다. 없을 수 있다는 사실을 타입에 남깁니다.',
    ' *',
    ' * 인덱스 시그니처는 봉투의 **추가-전용** 계약이자, 각 인터페이스가',
    ' * `Envelope<T extends RawEnvelope>` 의 제약을 만족하게 하는 장치입니다.',
    ' *',
    ' * @packageDocumentation',
    ' */',
  ];
  if (needsVerify) {
    lines.push('');
    lines.push("import type { RawVerifyReport } from './envelope.js';");
  }
  lines.push('');
  lines.push('/** 이 파일을 만들어 낸 capabilities 스냅샷 버전(= rhwp 버전). */');
  lines.push(`export const CAPABILITIES_SNAPSHOT_VERSION = ${literal(version)};`);
  return lines;
}

/** capabilities 봉투에서 `--json` 명령 목록을 추린다. */
function parseCommands(envelope: Record<string, unknown>): CommandSpec[] {
  const raw = envelope.commands;
  if (!Array.isArray(raw)) {
    throw new Error('capabilities 봉투에 `commands` 배열이 없습니다.');
  }
  const specs: CommandSpec[] = [];
  for (const item of raw) {
    if (item === null || typeof item !== 'object') continue;
    const record = item as Record<string, unknown>;
    const name = record.name;
    const fields = record.recordFields;
    // `recordFields` 가 없는 명령은 `--json` 봉투를 내지 않는다 (dump·bench 등).
    if (typeof name !== 'string' || !Array.isArray(fields)) continue;
    const flags = Array.isArray(record.flags) ? record.flags : [];
    specs.push({
      name,
      summary: typeof record.summary === 'string' ? record.summary : undefined,
      fields: fields.filter((field): field is string => typeof field === 'string'),
      takesJsonFlag: flags.includes('--json'),
    });
  }
  if (specs.length === 0) {
    throw new Error('capabilities 에 `recordFields` 를 선언한 명령이 하나도 없습니다.');
  }
  return specs.sort((a, b) => (a.name < b.name ? -1 : a.name > b.name ? 1 : 0));
}

/** capabilities 전체를 `src/envelopes.ts` 소스로. */
function generateEnvelopes(version: string, specs: readonly CommandSpec[]): string {
  const needsVerify = specs.some((spec) => {
    const overrides = COMMAND_FIELD_TYPES[spec.name] ?? {};
    return spec.fields.some((field) => {
      const head = fieldHead(field);
      const type = overrides[head] ?? GLOBAL_FIELD_TYPES[head];
      return type !== undefined && type.includes('RawVerifyReport');
    });
  });

  const lines = envelopesHeader(version, specs.length, needsVerify);
  for (const spec of specs) {
    lines.push('');
    lines.push(...renderEnvelope(spec));
  }

  lines.push('');
  lines.push(
    ...jsDoc('', [
      '명령 이름 → 봉투 타입.',
      '`recordFields` 를 선언한 명령만 들어 있습니다 — 나머지는 `--json` 봉투를 내지 않습니다.',
    ]),
  );
  lines.push('export interface EnvelopeByCommand {');
  for (const spec of specs) {
    lines.push(`${INDENT}${propertyKey(spec.name)}: ${envelopeTypeName(spec.name)};`);
  }
  lines.push('}');

  lines.push('');
  lines.push('/** `--json` 봉투를 내는 명령 이름. */');
  lines.push('export type EnvelopeCommand = keyof EnvelopeByCommand;');
  return `${lines.join('\n')}\n`;
}

// ── 출처 적재 ────────────────────────────────────────────────────────────

/** rhwp 를 못 찾았거나 명령을 모를 때, 무엇을 하면 되는지까지 적는다. */
function explainSourceFailure(command: string, cause: unknown): string {
  if (cause instanceof BinaryNotFoundError) {
    return (
      `rhwp 실행 파일을 찾지 못했습니다.\n  ${cause.message.split('\n').join('\n  ')}\n` +
      `  예) ${ENV_VAR}=../../target/release/rhwp npm run gen:types`
    );
  }
  if (cause instanceof UsageError) {
    return (
      `rhwp 가 \`${command}\` 를 모릅니다 (해당 명령이 없는 옛 빌드).\n` +
      `  최신 rhwp 를 빌드해 ${ENV_VAR} 로 가리키거나, 스키마 JSON 을 파일로 넘기세요\n` +
      '  (--ir-schema <파일> / --capabilities <파일>).'
    );
  }
  return `\`rhwp ${command}\` 실행에 실패했습니다 — ${String(cause)}`;
}

/** 파일에서 JSON 봉투를 읽는다 (재현·이행기 전용 경로). */
function readJsonFile(path: string): Record<string, unknown> {
  const text = readFileSync(path, 'utf8');
  const parsed: unknown = JSON.parse(text);
  if (parsed === null || typeof parsed !== 'object' || Array.isArray(parsed)) {
    throw new Error(`${path} 의 최상위가 JSON 객체가 아닙니다.`);
  }
  return parsed as Record<string, unknown>;
}

/**
 * 출처 하나를 적재한다.
 *
 * 파일 경로를 주면 rhwp 를 부르지 않는다. **재현·이행기 전용**이다 — 명령이 아직
 * 머지되지 않은 브랜치에서 생성해야 할 때가 있고, 무엇으로 생성했는지 손에 쥐고
 * 확인할 수 있어야 한다. 평상시 경로는 언제나 rhwp 자신이다.
 */
async function loadSource(
  args: readonly string[],
  file: string | undefined,
): Promise<Record<string, unknown>> {
  if (file !== undefined) return readJsonFile(file);
  const command = args[0] ?? '';
  try {
    return await runJson(args);
  } catch (cause) {
    throw new Error(explainSourceFailure(command, cause));
  }
}

// ── 쓰기·검사 ────────────────────────────────────────────────────────────

/**
 * 디스크와 비교한다. CRLF 는 정규화한다.
 *
 * 윈도우 체크아웃(`core.autocrlf=true`)은 LF 산출물을 CRLF 로 펼쳐 놓는다. 그건
 * 생성물이 낡았다는 뜻이 아니라 git 의 작업 트리 규칙이다. 정규화하지 않으면 CI 는
 * 통과하는데 로컬은 항상 실패하는, 아무도 믿지 않는 게이트가 된다.
 */
function readNormalized(path: string): string {
  return readFileSync(path, 'utf8').replace(/\r\n/g, '\n');
}

/** 첫 차이 지점을 사람이 읽을 수 있게. */
function describeDifference(disk: string, generated: string): string[] {
  const diskLines = disk.split('\n');
  const genLines = generated.split('\n');
  const limit = Math.max(diskLines.length, genLines.length);
  for (let i = 0; i < limit; i += 1) {
    const left = diskLines[i];
    const right = genLines[i];
    if (left === right) continue;
    return [
      `  첫 차이: ${i + 1}번째 줄 (디스크 ${diskLines.length}줄 · 생성 ${genLines.length}줄)`,
      `    디스크: ${left === undefined ? '(줄 없음)' : left}`,
      `    생성:   ${right === undefined ? '(줄 없음)' : right}`,
    ];
  }
  return ['  줄 단위 차이는 없고 내용 길이만 다릅니다 (끝 개행을 확인하세요).'];
}

/** 생성 결과 하나를 디스크에 반영하거나 비교한다. */
function emit(
  label: string,
  target: string,
  content: string,
  check: boolean,
  summary: string,
): boolean {
  // 패키지 밖을 가리키면 `..\..\..` 가 절대 경로보다 읽기 어렵다.
  const short = relative(process.cwd(), target);
  const shown = short && !short.startsWith('..') ? short : target;
  if (!check) {
    mkdirSync(dirname(target), { recursive: true });
    writeFileSync(target, content, 'utf8');
    console.log(`생성: ${shown} (${summary})`);
    return true;
  }
  if (!existsSync(target)) {
    console.error(`오류: ${shown} 이 없습니다 — 먼저 \`npm run gen:types\` 를 돌리세요.`);
    return false;
  }
  const disk = readNormalized(target);
  if (disk === content) {
    console.log(`${shown} 최신 (${summary})`);
    return true;
  }
  console.error(`오류: ${shown} 이 최신이 아닙니다 — 출처(${label})가 바뀌었습니다.`);
  for (const line of describeDifference(disk, content)) console.error(line);
  console.error('  고치는 법: `npm run gen:types` 를 돌리고 결과를 커밋하세요.');
  return false;
}

// ── CLI ──────────────────────────────────────────────────────────────────

/** 명령줄 옵션. */
interface Options {
  readonly outIr: string;
  readonly outEnvelopes: string;
  readonly irSchemaFile: string | undefined;
  readonly capabilitiesFile: string | undefined;
  readonly check: boolean;
}

const USAGE = `사용법: tsx tools/gen-types.ts [옵션]

  rhwp 자신이 보고하는 두 스키마에서 TypeScript 타입을 생성합니다.
  산출물은 손으로 고치지 않습니다 — 고치면 다음 생성에서 사라집니다.

옵션:
  --out-ir <경로>          IR 타입 출력           (기본: src/ir.ts)
  --out-envelopes <경로>   봉투 타입 출력         (기본: src/envelopes.ts)
  --ir-schema <파일>       rhwp 대신 파일에서 IR 스키마 JSON 을 읽는다
  --capabilities <파일>    rhwp 대신 파일에서 capabilities JSON 을 읽는다
  --check                  디스크와 다르면 exit 1. 파일을 쓰지 않는다 (CI 용)
  -h, --help               이 도움말

환경변수:
  ${ENV_VAR}   rhwp 실행 파일 경로 (탐색 1순위)`;

/** 값이 필요한 옵션에서 값을 꺼낸다. 빠졌으면 조용히 넘기지 않는다. */
function takeValue(argv: readonly string[], index: number, flag: string): string {
  const value = argv[index];
  if (value === undefined || value.startsWith('--')) {
    throw new Error(`${flag} 에 값이 필요합니다.`);
  }
  return value;
}

function parseArgs(argv: readonly string[], root: string): Options | 'help' {
  let outIr = join(root, 'src', 'ir.ts');
  let outEnvelopes = join(root, 'src', 'envelopes.ts');
  let irSchemaFile: string | undefined;
  let capabilitiesFile: string | undefined;
  let check = false;

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    switch (arg) {
      case '-h':
      case '--help':
        return 'help';
      case '--check':
        check = true;
        break;
      case '--out-ir':
        i += 1;
        outIr = resolve(takeValue(argv, i, '--out-ir'));
        break;
      case '--out-envelopes':
        i += 1;
        outEnvelopes = resolve(takeValue(argv, i, '--out-envelopes'));
        break;
      case '--ir-schema':
        i += 1;
        irSchemaFile = resolve(takeValue(argv, i, '--ir-schema'));
        break;
      case '--capabilities':
        i += 1;
        capabilitiesFile = resolve(takeValue(argv, i, '--capabilities'));
        break;
      default:
        throw new Error(`알 수 없는 옵션입니다: ${arg ?? ''}\n\n${USAGE}`);
    }
  }
  return { outIr, outEnvelopes, irSchemaFile, capabilitiesFile, check };
}

async function main(argv: readonly string[]): Promise<number> {
  const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');

  let options: Options | 'help';
  try {
    options = parseArgs(argv, root);
  } catch (cause) {
    console.error(`오류: ${cause instanceof Error ? cause.message : String(cause)}`);
    return 2;
  }
  if (options === 'help') {
    console.log(USAGE);
    return 0;
  }

  let irSource: IrSource;
  let capabilitiesVersion: string;
  let specs: CommandSpec[];
  try {
    const irEnvelope = await loadSource(['export-ir-schema', '--json'], options.irSchemaFile);
    irSource = parseIrSchema(irEnvelope);

    // `capabilities` 는 `--json` 을 받지 않는다 (기본 출력이 이미 봉투다).
    const capabilities = await loadSource(['capabilities'], options.capabilitiesFile);
    capabilitiesVersion = typeof capabilities.version === 'string' ? capabilities.version : '0.0.0';
    specs = parseCommands(capabilities);
  } catch (cause) {
    console.error(`오류: ${cause instanceof Error ? cause.message : String(cause)}`);
    return 1;
  }

  let irText: string;
  let envelopeText: string;
  try {
    irText = generateIr(irSource);
    envelopeText = generateEnvelopes(capabilitiesVersion, specs);
  } catch (cause) {
    console.error(`오류: ${cause instanceof Error ? cause.message : String(cause)}`);
    return 1;
  }

  const definitionCount = Object.keys(irSource.defs).length;
  const okIr = emit(
    'IR 스키마',
    options.outIr,
    irText,
    options.check,
    `IR v${irSource.version}, 정의 ${definitionCount}개`,
  );
  const okEnvelopes = emit(
    'capabilities',
    options.outEnvelopes,
    envelopeText,
    options.check,
    `capabilities v${capabilitiesVersion}, 봉투 ${specs.length}개`,
  );
  return okIr && okEnvelopes ? 0 : 1;
}

process.exitCode = await main(process.argv.slice(2));
