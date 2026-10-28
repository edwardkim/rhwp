import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { codeOnly } from './support/source-guard.ts';

// [#5769 후속] z 순서 역연산화 소스 가드 — 클래스 라이프사이클 전용.
//
// 퍼널(insert.ts)과 개체 선택 자동 맨앞(mouse)의 커맨드 경유 여부는 이미
// undo-menu-object-ops 와 undo-drag-click-routing 이 담당한다 — 같은 배선을
// 여기서 다시 pin 하면 배선 변경 시 세 곳을 고쳐야 한다. 이 파일은 다른 어디에도
// 없는 것만 본다: **SetZOrderCommand 본체의 저널 생명주기 순서**(probe → 캡처 →
// 적용, undo 는 old 대입 뒤 raw 복원, 무변경 캡처 폐기). 행위 증명(바이트 왕복)
// 은 Rust 게이트 tests/cases/issue_5769_zorder_inverse_byte_identity.rs 다.

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const cmdSrc = codeOnly(readFileSync(join(rootDir, 'src/engine/command.ts'), 'utf8'));

test('SetZOrderCommand 배선 핀 — probe 선차단, 캡처 선행, undo 는 old 대입 뒤 raw 복원', () => {
  const start = cmdSrc.indexOf('export class SetZOrderCommand');
  assert(start !== -1, 'command.ts 에 SetZOrderCommand 이 없다');
  const rest = cmdSrc.slice(start);
  const body = rest.slice(0, rest.indexOf('\nexport class ', 1));

  // 스큐 선제 차단(gpt 3차 리뷰) — probe 가 캡처·뮤테이션보다 먼저여야 한다.
  // 정규식 거리 창은 분기 추가에 깨지므로 인덱스 순서로 핀한다.
  const probeIdx = body.indexOf('hasShapeZOrderInverse()');
  const capIdx = body.indexOf('captureSectionRaw(this.sectionIdx)');
  const redoIdx = body.indexOf("pairsJson('after')");
  const firstRunIdx = body.indexOf('changeShapeZOrder');
  assert.notEqual(probeIdx, -1, '구버전 wasm 판별 probe 가 있어야 한다');
  assert.notEqual(capIdx, -1, 'execute 는 변경 전 구역 raw 를 캡처해야 한다');
  assert.ok(
    probeIdx < capIdx && capIdx < redoIdx && capIdx < firstRunIdx,
    'probe → 캡처 → redo 절대 대입/최초 상대 연산 순서여야 한다',
  );
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
