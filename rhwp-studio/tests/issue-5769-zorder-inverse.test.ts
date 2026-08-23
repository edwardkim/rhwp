import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// [#5769 후속] z 순서 역연산화 소스 가드.
//
// z순서 변경(정렬 메뉴 4모드 + 개체 선택 시 자동 맨앞)은 스냅샷 대신 SetZOrderCommand
// (kind:'command')로 기록된다 — 되돌릴 것이 스칼라 1~2개인 조작이 문서 전체 클론을
// 스택에 얹지 않게 하는 #5769 다음 후보다. 행위 증명(저장 바이트 왕복 동일성)은 Rust
// 게이트 tests/cases/issue_5769_zorder_inverse_byte_identity.rs 가 담당하고, 여기선
// 배선을 핀한다 — 가드는 소스가 계약을 말하도록 유지한다(#2370 클러스터 관례).

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const read = (rel: string): string => readFileSync(join(rootDir, rel), 'utf8');

/** command.ts 에서 클래스 본문만 잘라낸다(다음 export class 직전까지). */
function classBody(cmdSrc: string, className: string): string {
  const start = cmdSrc.indexOf(`export class ${className}`);
  assert(start !== -1, `command.ts 에 ${className} 이 없다`);
  const rest = cmdSrc.slice(start);
  const next = rest.indexOf('\nexport class ', 1);
  return next === -1 ? rest : rest.slice(0, next);
}

test('insert.ts z순서 정렬 메뉴는 SetZOrderCommand 커맨드로 기록한다', () => {
  const s = read('src/command/commands/insert.ts');
  const funnelStart = s.indexOf('function changeZOrder');
  assert(funnelStart !== -1, 'changeZOrder 퍼널이 있어야 한다');
  const funnel = s.slice(funnelStart, s.indexOf('\n}', funnelStart));

  assert.match(funnel, /new SetZOrderCommand\(ref\.sec, ref\.ppi, ref\.ci, operation/,
    '퍼널은 속성쌍 커맨드를 생성해야 한다');
  assert.match(funnel, /kind: 'command'/, '커맨드 경로는 kind:command 다(snapshot 아님)');
  assert.doesNotMatch(funnel, /recordObjectMutation/, '퍼널은 스냅샷 헬퍼를 쓰지 않는다');
  assert.match(funnel, /getPositionOutsideSelectedPicture\(\)/,
    '[#3351] 캐럿은 개체 인접을 기록한다');

  // 4개 메뉴 호출부가 모두 퍼널로 수렴하는지 — 퍼널 앞에 직접 wasm 호출이 남으면 원장이 흔들린다.
  assert.doesNotMatch(
    s.slice(0, funnelStart),
    /changeShapeZOrder/,
    '메뉴 호출부에 직접 wasm.changeShapeZOrder 가 남아 있으면 안 된다',
  );
});

test('개체 선택 시 자동 맨앞(mouse)도 같은 커맨드 경로다', () => {
  const s = read('src/engine/input-handler-mouse.ts');
  assert.match(s, /new SetZOrderCommand\(picHit\.sec, picHit\.ppi, picHit\.ci, 'front'/,
    '선택 진입 front 도 SetZOrderCommand 로 기록해야 한다');
  assert.doesNotMatch(s, /operationType: 'changeZOrder'/,
    'changeZOrder 명의 snapshot 라우팅 잔류는 슬롯을 다시 먹는다');
});

test('SetZOrderCommand 배선 핀 — 캡처 선행, undo 는 old 대입 뒤 raw 복원', () => {
  const body = classBody(read('src/engine/command.ts'), 'SetZOrderCommand');

  // execute: 캡처가 상대 연산/절대 대입보다 먼저다(SetSectionPropsCommand 와 동일 생명주기).
  assert.match(body, /captureSectionRaw\(this\.sectionIdx\)/,
    'execute 는 변경 전 구역 raw 를 캡처해야 한다');
  assert.match(body, /captureSectionRaw[\s\S]{0,400}?changeShapeZOrder/,
    '최초 실행은 캡처 뒤 상대 연산을 실행한다');
  assert.match(body, /pairsJson\('after'\)/, 'redo 는 저장된 after 쌍으로 절대 대입한다');

  // undo: old 재적용(raw 재무효화) 뒤 passthrough 복원 — 순서가 바뀌면 수렴이 깨진다.
  assert.match(body, /pairsJson\('before'\)\)[\s\S]{0,120}?restoreSectionRaw/,
    'undo 는 old 대입 뒤 raw 를 복원해야 한다');

  // 무변경 경로: phantom 엔트리 방지 + 캡처 낭비 없음.
  assert.match(body, /noOp = true;[\s\S]{0,160}?discardSectionRaw/,
    '무변경이면 noOp 를 세우고 캡처를 버려야 한다');
  assert.match(body, /snapshotResourceCount\(\): number \{ return 0; \}/,
    '역연산 경로는 스냅샷 예산을 쓰지 않는다(#2328 수렴 계약)');
});

test('새 브리지 메서드와 레지스트리 분류가 짝을 이룬다', () => {
  const bridge = read('src/core/wasm-bridge.ts');
  assert.match(bridge, /applyShapeZOrderPairs\(sec: number, pairsJson: string\)/,
    'WasmBridge 에 절대 대입 메서드가 있어야 한다');

  const registry = read('src/core/mutation-method-registry.ts');
  assert.match(registry, /'applyShapeZOrderPairs'/,
    '문서 변형 메서드는 MUTATING_METHODS 에 분류돼야 한다(#2327)');
});
