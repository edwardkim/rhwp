import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// [Task #2327] 계급 1(기록 옵트인) 회귀를 저작 시점에 차단하는 소스 가드.
//
// 두 정적 검사:
//  (1) 드리프트(양방향) — mutation-method-registry.ts 의 MUTATING_METHODS /
//      EXCLUDED_NON_DOCUMENT 가 브리지의 문서-변경형 공개 메서드를 전수 분류하고
//      (신규 뮤테이터 누락 차단), 역으로 목록 전 항목이 실제 브리지 메서드인지도
//      강제한다(rename/제거 무통보 차단).
//  (2) 원장 트립와이어 — ui/ + command/ + engine/input-handler* 에서 뮤테이터를
//      직접 호출하는 표면(파일별 호출 수)을 동결한다. 신규 파일·증가는 실패 →
//      의식적 baseline 갱신 + 리뷰 강제. (텍스트 카운트는 executeOperation 콜백
//      내부 호출도 세므로 "라우팅 여부"가 아니라 "뮤테이션 표면 증가"의
//      트립와이어다.)

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

function source(rel: string): string {
  return readFileSync(join(rootDir, rel), 'utf8');
}

/** mutation-method-registry.ts 의 배열 상수를 단일 권위원으로 파싱한다. */
function parseStringArray(guardSrc: string, name: string): string[] {
  // `export const NAME ...= [ ... ];` 선언에 앵커(상단 주석의 이름 언급 회피).
  const m = guardSrc.match(new RegExp(`export const ${name}\\b[^=]*=\\s*\\[([\\s\\S]*?)\\];`));
  assert.ok(m, `${name} 상수를 파싱하지 못함`);
  return [...m![1].matchAll(/'([A-Za-z0-9_]+)'/g)].map((x) => x[1]);
}

const guardSrc = source('src/core/mutation-method-registry.ts');
const MUTATING = parseStringArray(guardSrc, 'MUTATING_METHODS');
const EXCLUDED = parseStringArray(guardSrc, 'EXCLUDED_NON_DOCUMENT');

// ── (1) 드리프트: 브리지 공개 메서드 분류 강제 ──────────────────────────────

/** WasmBridge 클래스 본문의 공개 메서드 이름을 추출한다(2칸 들여쓰기 최상위). */
function bridgePublicMethods(): string[] {
  const src = source('src/core/wasm-bridge.ts');
  const names = new Set<string>();
  const priv = new Set<string>();
  const re = /^ {2}(public |private |protected )?(async )?([a-zA-Z_]\w*)\s*\(/gm;
  for (const m of src.matchAll(re)) {
    const name = m[3];
    if (['constructor', 'if', 'for', 'while', 'switch', 'catch', 'return'].includes(name)) continue;
    if (m[1] === 'private ' || m[1] === 'protected ') priv.add(name);
    else names.add(name);
  }
  return [...names].filter((n) => !priv.has(n));
}

// 문서 변경을 시사하는 동사 접두어. MUTATING_METHODS 의 모든 이름을 커버해야
// 하며(아래 자기정합 단언이 강제), 그래야 새 브리지 뮤테이터가 drift 에 걸린다.
// find* 는 쿼리(findNextEditableControl 등)가 많아 findOrCreate 로 좁힌다.
const MUTATING_VERB = /^(insert|delete|create|apply|add|remove|move|resize|merge|split|update|toggle|replace|paste|assign|group|ungroup|change|clear|evaluate|transpose|ensure|findOrCreate|reflow|setPage|setSection|setColumn|setCell|setTable|setPicture|setShape|setEquation|setNote|setChar|setPara|setField|setForm|setNumbering|setHeaderFooter|setActiveField|renameBookmark)/;

test('MUTATING_VERB 는 MUTATING_METHODS 전 항목을 커버한다(drift 사각 방지)', () => {
  // 목록에 있으나 동사 패턴에 안 걸리는 이름이 있으면, 그 계열의 신규 브리지
  // 뮤테이터가 drift 에서 누락된다(ensure*/find* 계열 사각이 실제였음).
  const uncovered = MUTATING.filter((m) => !MUTATING_VERB.test(m));
  assert.deepEqual(uncovered, [],
    `MUTATING_METHODS 에 MUTATING_VERB 가 못 잡는 이름: ${uncovered.join(', ')} → 동사 추가 필요`);
});

test('드리프트: 문서-변경형 브리지 공개 메서드는 모두 분류돼야 한다', () => {
  const classified = new Set([...MUTATING, ...EXCLUDED]);
  const unclassified = bridgePublicMethods()
    .filter((n) => MUTATING_VERB.test(n))
    .filter((n) => !classified.has(n));
  assert.deepEqual(
    unclassified,
    [],
    `WasmBridge 신규(?) 뮤테이터가 분류되지 않음: ${unclassified.join(', ')}\n` +
      `→ mutation-method-registry.ts 의 MUTATING_METHODS(기록 대상) 또는 ` +
      `EXCLUDED_NON_DOCUMENT(문서 비변경 사유)에 추가하라.`,
  );
});

test('MUTATING_METHODS / EXCLUDED_NON_DOCUMENT 는 서로 겹치지 않는다', () => {
  const dup = MUTATING.filter((m) => EXCLUDED.includes(m));
  assert.deepEqual(dup, [], `양쪽에 중복 분류됨: ${dup.join(', ')}`);
});

test('MUTATING_METHODS 는 모두 실제 브리지 공개 메서드여야 한다(rename skip 방지)', () => {
  // 브리지에서 메서드가 rename/제거되면 목록의 옛 이름이 드리프트·원장 검사에서
  // 무의미해지므로(권위 목록 사각), 목록 항목이 전부 실재하는지 역방향으로 강제한다.
  const bridge = new Set(bridgePublicMethods());
  const missing = MUTATING.filter((m) => !bridge.has(m));
  assert.deepEqual(
    missing,
    [],
    `MUTATING_METHODS 에 브리지에 없는 이름: ${missing.join(', ')}\n` +
      `→ 브리지에서 rename/제거된 메서드. 목록을 갱신하라(방치 시 가드 무통보 비활성).`,
  );
});

// ── (2) 원장 트립와이어: 뮤테이션 표면 동결 ─────────────────────────────────

/**
 * 뮤테이션 표면 스캔 대상: ui/ + command/ + engine/input-handler*.ts + hwpctl/.
 * input-handler* 는 드래그/키보드 nudge 등 최고밀도 직접-뮤테이션 영역이므로
 * 반드시 포함한다(누락 시 엔진 층의 신규 미라우팅이 원장에 안 걸림).
 *
 * [#3648] `src/hwpctl` 을 추가했다. 자동화 표면은 정책상 히스토리를 경유하지 않지만
 * (아래 원장 주석), 스캔에서 빼 두면 **그 정책이 지켜지는지도 볼 수 없다** — 미라우팅이
 * 늘어나는 것을 아무도 모른다. 정책은 "라우팅하지 않는다"이고 원장의 일은 "현상 동결"이다.
 */
function scanFiles(): string[] {
  const out: string[] = [];
  const walk = (rel: string) => {
    for (const ent of readdirSync(join(rootDir, rel), { withFileTypes: true })) {
      const child = `${rel}/${ent.name}`;
      if (ent.isDirectory()) walk(child);
      else if (ent.name.endsWith('.ts') && !ent.name.endsWith('.test.ts')) out.push(child);
    }
  };
  walk('src/ui');
  walk('src/command');
  walk('src/hwpctl');
  for (const ent of readdirSync(join(rootDir, 'src/engine'), { withFileTypes: true })) {
    if (ent.isFile() && ent.name.startsWith('input-handler') && ent.name.endsWith('.ts')
      && !ent.name.endsWith('.test.ts')) {
      out.push(`src/engine/${ent.name}`);
    }
  }
  return out.sort();
}

/**
 * 문서를 직접 변형하는 수신자 이름 (#3648).
 *
 * 종전에는 `wasm` 하나에 앵커돼 있어 hwpctl 의 23곳이 통째로 빠졌다 — hwpctl 은
 * `ctrl.getWasmDoc()` 이 준 raw doc 을 `doc`·`this.wasmDoc` 으로 받아 쓴다.
 *
 * 타입이 아니라 이름으로 판정하는 한 이 목록은 불완전할 수밖에 없다. 그래서 목록을 없애는
 * 대신 **빠뜨렸을 때 조용히 넘어가지 않게** 했다 — 아래
 * `모든 뮤테이터 호출의 수신자는 분류돼 있어야 한다` 가 미등재 수신자를 실패로 만든다.
 * #3648 을 만든 실패 양상(새 수신자가 원장에서 사라짐)이 다시 나면 그 테스트가 잡는다.
 */
const MUTATION_RECEIVERS = ['wasm', 'wasmDoc', 'doc'] as const;

/**
 * 뮤테이터와 이름이 같지만 문서를 변형하지 않는 호출의 수신자.
 *
 * `this`/`ih` 는 `InputHandler` 자기 위임이다 — 실제 뮤테이션은 그 메서드 안의 `wasm.` 호출
 * 지점에서 이미 세므로, 위임까지 세면 한 변경을 두 번 센다.
 */
const NON_MUTATION_RECEIVERS: Readonly<Record<string, string>> = {
  this: 'InputHandler 자기 위임 (applyCharFormat/applyParaFormat/applyStyle) — 내부 wasm. 호출을 이미 센다',
  ih: 'InputHandler 위임 — 위와 같다',
};

/**
 * 표준 JS 메서드와 이름이 겹치는 뮤테이터.
 *
 * `replaceAll` 은 브리지의 찾아 바꾸기이면서 `String.prototype.replaceAll` 이기도 하다.
 * 다만 이름만으로 모든 비브리지 수신자를 건너뛰면 새 raw-document alias까지 조용히 빠진다.
 * 현재 문자열 수신자만 명시적으로 비뮤테이션으로 분류하고, 모르는 수신자는 원장의
 * `모든 뮤테이터 호출` 검사에서 실패시킨다.
 */
const BUILTIN_NAME_COLLISIONS = new Set(['replaceAll']);
const BUILTIN_NON_MUTATION_RECEIVERS: Readonly<Record<string, string>> = {
  text: 'String.prototype.replaceAll을 쓰는 HTML escape 문자열',
  value: 'String.prototype.replaceAll을 쓰는 비교 문자열',
};

function isKnownBuiltinNonMutationCall(receiver: string, method: string): boolean {
  return BUILTIN_NAME_COLLISIONS.has(method) && receiver in BUILTIN_NON_MUTATION_RECEIVERS;
}

/** `수신자.메서드(` 를 훑는다. 수신자는 점 앞의 마지막 식별자(`this.wasmDoc` → `wasmDoc`). */
function mutatorCalls(src: string): { receiver: string; method: string }[] {
  const re = new RegExp(`([A-Za-z0-9_$]+)\\s*\\.\\s*(${MUTATING.join('|')})\\s*\\(`, 'g');
  return [...src.matchAll(re)].map((m) => ({ receiver: m[1], method: m[2] }));
}

function mutatorCallCount(src: string): number {
  return mutatorCalls(src).filter(
    ({ receiver }) => (MUTATION_RECEIVERS as readonly string[]).includes(receiver),
  ).length;
}

// 뮤테이션 표면 원장 (2026-07-17 동결). 이관/추가 시 이 표를 의식적으로 갱신한다.
// 값을 낮추는 방향(이관)만 무해하며, 높이거나 신규 키 추가는 리뷰 대상이다.
const BASELINE: Readonly<Record<string, number>> = {
  'src/command/commands/edit.ts': 1,
  'src/command/commands/format.ts': 1,
  'src/command/commands/insert.ts': 16, // -3: z순서 4 호출부를 changeZOrder 헬퍼 1곳으로 합침(#2370 A)
  'src/command/commands/page.ts': 13,
  'src/command/commands/table.ts': 36, // +1: 블록계산 이관 시 evaluateTableFormula dry-run(write=false, 검증) 추가 / +2: 표 나누기·붙이기(splitTable·mergeTableWithNext) — 둘 다 executeOperation snapshot 으로 라우팅 (undo 기록됨)
  'src/ui/bookmark-dialog.ts': 3,
  'src/ui/cell-border-bg-dialog.ts': 5,
  'src/ui/column-settings-dialog.ts': 1,
  'src/ui/endnote-shape-dialog.ts': 1,
  'src/ui/equation-editor-dialog.ts': 2,
  'src/ui/equation-props-dialog.ts': 2,
  'src/ui/find-dialog.ts': 4,
  'src/ui/formula-dialog.ts': 4, // +1: [#2367] 쉼표 포맷이 원시 결과 위에 겹치지 않도록 deleteTextInCell 추가(commit() snapshot 내부 — 라우팅 유지)
  'src/ui/new-number-dialog.ts': 1,
  'src/ui/numbering-dialog.ts': 1,
  'src/ui/page-border-dialog.ts': 1,
  'src/ui/page-setup-dialog.ts': 1,
  'src/ui/picture-props-dialog.ts': 5,
  'src/ui/section-settings-dialog.ts': 2,
  'src/ui/style-dialog.ts': 2, // +1: [#3387] 삭제를 snapshot 으로 라우팅하며 services 미주입 fallback 분기 추가(원장은 표면 수만 세므로 라우팅해도 줄지 않는다)
  'src/ui/style-edit-dialog.ts': 6,
  'src/ui/table-cell-props-dialog.ts': 2,
  'src/ui/toolbar.ts': 4,
  // engine/input-handler* — 드래그/nudge 등 직접-뮤테이션 최고밀도 영역.
  'src/engine/input-handler.ts': 30, // +1: 누름틀 제거 이관 시 removeFieldAt 이 양식모드(직접 유지)/일반모드(snapshot) 두 분기로 분리 / +1: Edit 오버레이 커밋이 셀 내부는 setFormValueInCell 로 분기(CheckBox 와 동일 조건 — 기존 flat 호출은 표 컨트롤 슬롯을 가리켜 실패) / +1: 셀 블록 글자 서식이 applyCharFormatInCell 을 executeOperation snapshot 안에서 호출(여러 셀에 걸친 글자 서식 커맨드 부재) / +2: 머리말·꼬리말(applyParaFormatInHf)과 각주(applyParaFormatInFootnote) 문단 서식 배선 — 둘 다 snapshot 라우팅
  'src/engine/input-handler-connector.ts': 1,
  'src/engine/input-handler-keyboard.ts': 21,
  'src/engine/input-handler-mouse.ts': 3,
  'src/engine/input-handler-picture.ts': 11,
  'src/engine/input-handler-table.ts': 9, // +2: 한컴 3모드 셀 크기 조절 분화 — Alt(resizeCellLocalByKeyboard)·Shift(resizeCellBoundaryByKeyboard) 각각 executeOperation snapshot 으로 라우팅 (undo 기록됨)
  'src/engine/input-handler-text.ts': 11, // #2424: raw IME delete를 command 공통 typed helper로 이관
  // ── hwpctl — 의도적 미라우팅 (#3648 정책 판정, 2026-07-31) ──
  //
  // hwpctl 은 자동화·임베드 표면이고 소비자는 스크립트다. 스크립트는 자기 트랜잭션 경계를
  // 알기에 "N 개 작업 후 되돌리기"는 호출자가 문서 사본으로 관리할 문제로 판정됐다. 히스토리
  // 경유(①안)는 자동화 레이어를 앱 레이어에 종속시켜 의존 방향을 뒤집으므로 채택하지 않았다.
  //
  // 따라서 아래 23곳은 **줄여야 할 부채가 아니라 동결된 현상**이다. 다른 키처럼 "이관하면
  // 값을 낮춘다"가 아니라, 값이 **늘어나는 것만** 리뷰 대상이다 — 자동화 표면에 문서 변경이
  // 새로 붙었다는 뜻이므로 정책 재확인이 필요하다.
  'src/hwpctl/actions/clipboard.ts': 2,
  'src/hwpctl/actions/format.ts': 3,
  'src/hwpctl/actions/page.ts': 2,
  'src/hwpctl/actions/table-edit.ts': 6,
  'src/hwpctl/actions/table.ts': 1,
  'src/hwpctl/actions/text.ts': 4,
  'src/hwpctl/index.ts': 5,
};

/** 원장에서 "동결된 현상"으로 다루는 접두어 — 이관 권고(↓) 대상에서 뺀다. */
const FROZEN_BY_POLICY = ['src/hwpctl/'];

test('뮤테이션 표면 원장: 신규·증가 사이트는 baseline 갱신을 강제한다', () => {
  const current: Record<string, number> = {};
  for (const rel of scanFiles()) {
    const n = mutatorCallCount(source(rel));
    if (n > 0) current[rel] = n;
  }

  const violations: string[] = [];
  for (const [rel, n] of Object.entries(current)) {
    const base = BASELINE[rel];
    if (base === undefined) {
      violations.push(`  + ${rel}: ${n} (신규 뮤테이션 파일 — executeOperation 라우팅 또는 baseline 등재 필요)`);
    } else if (n > base) {
      violations.push(`  ↑ ${rel}: ${base} → ${n} (뮤테이션 사이트 증가 — 라우팅 확인 또는 baseline 갱신)`);
    }
  }
  // 이관으로 값이 줄면 baseline 을 낮추라고 안내(실패는 아님 — 진행을 막지 않음).
  const stale: string[] = [];
  for (const [rel, base] of Object.entries(BASELINE)) {
    // 정책상 동결된 표면은 이관 대상이 아니므로 "이관 권고"를 내지 않는다 (#3648).
    if (FROZEN_BY_POLICY.some((prefix) => rel.startsWith(prefix))) continue;
    const n = current[rel] ?? 0;
    if (n < base) stale.push(`  ↓ ${rel}: ${base} → ${n} (이관 완료 — baseline 을 ${n}${n === 0 ? ' 로 낮추거나 키 제거' : ''} 로 갱신 권장)`);
  }

  assert.deepEqual(
    violations,
    [],
    `뮤테이션 표면이 baseline 을 초과했습니다:\n${violations.join('\n')}\n\n` +
      `새 문서 변경은 InputHandler.executeOperation 라우팅으로 undo 에 기록돼야 합니다(#2327).` +
      (stale.length ? `\n\n참고(이관 진행 — 실패 아님):\n${stale.join('\n')}` : ''),
  );

  if (stale.length) {
    // 진행 상황 가시화 — 실패시키지 않고 로그만.
    console.log(`[mutation-routing] 이관 진행:\n${stale.join('\n')}`);
  }
});

// [#3648] 이름으로 수신자를 판정하는 한 `MUTATION_RECEIVERS` 는 불완전할 수밖에 없다.
// #3648 은 그 불완전함이 **조용히** 넘어갔기 때문에 생겼다 — hwpctl 이 `doc`·`this.wasmDoc`
// 을 쓰는데 원장은 `wasm` 만 봐서, 23곳이 없는 것처럼 보였고 가드는 5/5 통과했다.
//
// 그래서 목록을 없애는 대신 **빠뜨리면 실패하게** 만든다. 새 수신자 이름이 등장하면 세는
// 쪽(`MUTATION_RECEIVERS`)이든 세지 않는 쪽(`NON_MUTATION_RECEIVERS`)이든 **의식적으로**
// 분류해야 통과한다.
test('모든 뮤테이터 호출의 수신자는 분류돼 있어야 한다(미등재 수신자 침묵 차단)', () => {
  const unclassified = new Map<string, { files: Set<string>; methods: Set<string> }>();

  for (const rel of scanFiles()) {
    for (const { receiver, method } of mutatorCalls(source(rel))) {
      // 동명이인이라도 미리 분류한 문자열 수신자만 건너뛴다. 미지 수신자는 raw-document
      // alias일 수 있으므로 아래 분류 실패로 드러나야 한다.
      if (isKnownBuiltinNonMutationCall(receiver, method)) continue;
      if ((MUTATION_RECEIVERS as readonly string[]).includes(receiver)) continue;
      if (receiver in NON_MUTATION_RECEIVERS) continue;

      if (!unclassified.has(receiver)) {
        unclassified.set(receiver, { files: new Set(), methods: new Set() });
      }
      const entry = unclassified.get(receiver)!;
      entry.files.add(rel);
      entry.methods.add(method);
    }
  }

  const violations = [...unclassified.entries()].map(([receiver, e]) =>
    `  ${receiver}.{${[...e.methods].sort().join(',')}} — ${[...e.files].sort().join(', ')}`,
  );

  assert.deepEqual(
    violations,
    [],
    '분류되지 않은 뮤테이터 수신자가 있습니다:\n' + violations.join('\n') + '\n\n'
      + '문서를 직접 변형하면 MUTATION_RECEIVERS 에 추가하고 BASELINE 을 갱신하세요.\n'
      + '변형하지 않으면(위임·동명이인) NON_MUTATION_RECEIVERS 에 사유와 함께 등재하세요.\n'
      + '이 검사가 없으면 새 수신자의 뮤테이션이 원장에서 조용히 빠집니다(#3648).',
  );
});

test('replaceAll 동명이인은 알려진 문자열 수신자만 비뮤테이션으로 분류한다', () => {
  assert.equal(isKnownBuiltinNonMutationCall('text', 'replaceAll'), true);
  assert.equal(isKnownBuiltinNonMutationCall('value', 'replaceAll'), true);
  assert.equal(isKnownBuiltinNonMutationCall('wasm', 'replaceAll'), false);
  assert.equal(isKnownBuiltinNonMutationCall('rawDoc', 'replaceAll'), false,
    '새 raw-document alias는 수신자 분류 실패로 리뷰에 드러나야 한다');
});

// hwpctl 이 스캔 대상에 실제로 들어왔는지 — #3648 의 "이중 미스" 두 겹을 각각 못박는다.
// 한 겹만 고치면 원장에 올라오지 않으므로, 둘을 따로 확인해야 회귀를 잡는다.
test('[#3648] hwpctl 은 스캔 대상이고 raw doc 수신자도 계상된다', () => {
  const scanned = scanFiles();
  assert.ok(
    scanned.some((rel) => rel.startsWith('src/hwpctl/')),
    '스캔 디렉터리에 src/hwpctl 이 없으면 정책 준수 여부를 볼 수 없다',
  );

  // 겹 ②: 수신자 제약. `doc`·`wasmDoc` 을 세지 않으면 스캔해도 0 으로 보인다.
  for (const receiver of ['doc', 'wasmDoc']) {
    assert.ok(
      (MUTATION_RECEIVERS as readonly string[]).includes(receiver),
      `${receiver} 수신자를 세지 않으면 hwpctl 의 raw doc 호출이 원장에서 빠진다`,
    );
  }

  const total = scanned
    .filter((rel) => rel.startsWith('src/hwpctl/'))
    .reduce((sum, rel) => sum + mutatorCallCount(source(rel)), 0);
  assert.equal(
    total,
    23,
    'hwpctl 직접 뮤테이션은 #3648 정책으로 23곳에 동결됐다 — 늘었다면 자동화 표면에 '
      + '문서 변경이 새로 붙었다는 뜻이므로 정책을 재확인해야 한다',
  );
});
