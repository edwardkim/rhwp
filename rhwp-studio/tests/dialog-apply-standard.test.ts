import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { applyThroughRouter } from '../src/ui/dialog-apply.ts';

// [Task #2370 클러스터 C] 다이얼로그 [확인] 실패 처리 표준화 가드.
//
// Track 1(#2362·#2364·#2368)에서 여섯 다이얼로그를 라우터로 이관할 때 실패 처리가
// 제각각 남았다 — 다단·새 번호는 try/catch 로 삼키고 닫혔고, 편집 용지·구역·쪽 테두리·
// 미주 모양은 throw 를 전파했다(처리되지 않은 예외 + 우연히 열린 채). 같은 실패에
// 사용자가 보는 것이 달랐다. 공용 헬퍼 하나로 모으고, 다시 갈라지지 않게 핀한다.

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const src = (rel: string) => readFileSync(join(rootDir, rel), 'utf8');

const ROUTED_DIALOGS = [
  'src/ui/page-setup-dialog.ts',
  'src/ui/section-settings-dialog.ts',
  'src/ui/column-settings-dialog.ts',
  'src/ui/page-border-dialog.ts',
  'src/ui/new-number-dialog.ts',
  'src/ui/endnote-shape-dialog.ts',
];

// 라우팅되지만 이 표준화에서 **의도적으로 제외**한 다이얼로그. 목록이 조용히 굳는 것을
// 막으려고 여기 적고, 아래 테스트가 "제외 사유가 유효한지"를 함께 확인한다.
const ROUTED_BUT_EXCLUDED = [
  {
    file: 'src/ui/formula-dialog.ts',
    // 계산식은 입력 검증이 본질이라 실패를 **다이얼로그 안에** 보여 준다
    // (`showError(...)` + `return false`). 콘솔 경고만 남기는 공용 헬퍼보다 강한
    // 처리이므로 헬퍼로 끌어내리지 않는다 — 오히려 이쪽이 참고할 선례다.
    reason: 'showError',
  },
];

test('공용 헬퍼는 실패를 삼키지 않고 false 로 알려 다이얼로그를 열어 둔다', () => {
  const helperSrc = src('src/ui/dialog-apply.ts');
  const helper = helperSrc.slice(helperSrc.indexOf('export function applyThroughRouter'));
  assert.match(helper, /return true;/, '성공 시 true');
  assert.match(helper, /catch \(err\) \{[\s\S]*console\.warn[\s\S]*return false;/, '실패 시 경고 + false');
  assert.match(helper, /kind: 'snapshot'/, 'snapshot 으로 라우팅');
  assert.match(helper, /fallback\(\)/, 'services 미주입 fallback 유지');
  // ModalDialog 계약: onConfirm 이 false 를 반환하면 닫지 않는다. 이 전제가 깨지면
  // 실패해도 다이얼로그가 닫혀 사용자가 입력을 잃는다.
  assert.match(
    src('src/ui/dialog.ts'),
    /const shouldClose = this\.onConfirm\(\);\s*\n\s*if \(shouldClose !== false\) this\.hide\(\);/,
    'ModalDialog 는 onConfirm() === false 일 때 닫지 않아야 함(헬퍼의 전제)',
  );
});

test('라우터와 fallback 예외를 실제로 false로 바꾼다', () => {
  const originalWarn = console.warn;
  const warnings: unknown[][] = [];
  console.warn = (...args: unknown[]) => { warnings.push(args); };
  try {
    const routed = applyThroughRouter({
      services: {
        getInputHandler: () => ({
          executeOperation: () => { throw new Error('router failure'); },
        }),
      } as never,
      label: 'router-test',
      operationType: 'test',
      operation: () => null,
      fallback: () => { throw new Error('fallback must not run'); },
    });
    const fallback = applyThroughRouter({
      services: undefined,
      label: 'fallback-test',
      operationType: 'test',
      operation: () => null,
      fallback: () => { throw new Error('fallback failure'); },
    });
    assert.equal(routed, false);
    assert.equal(fallback, false);
    assert.equal(warnings.length, 2);
  } finally {
    console.warn = originalWarn;
  }
});

test('라우팅된 다이얼로그 여섯은 모두 공용 헬퍼를 통과한다', () => {
  for (const rel of ROUTED_DIALOGS) {
    const s = src(rel);
    // [#5769 Stage 4→후속2] 헬퍼 계열 확장 수용 — snapshot(applyThroughRouter)과
    // command(applyCommandThroughRouter) 어느 쪽이든 공용 헬퍼 경유를 핀한다.
    assert.match(s, /import \{[^}]*apply(Command)?ThroughRouter[^}]*\} from '\.\/dialog-apply'/, `${rel}: 헬퍼 import`);
    assert.match(s, /return apply(Command)?ThroughRouter\(\{/, `${rel}: onConfirm 이 헬퍼 결과를 반환`);
    // 헬퍼 밖에서 직접 라우터를 부르면 표준화가 무너진다.
    assert.doesNotMatch(
      s,
      /ih\.executeOperation\(\{/,
      `${rel}: executeOperation 직접 호출 금지 — 공용 헬퍼 경유`,
    );
    // onConfirm 은 헬퍼의 성공 여부를 그대로 돌려줘야 한다(실패 시 다이얼로그 유지).
    assert.match(s, /protected onConfirm\(\): boolean \{/, `${rel}: onConfirm 반환형 boolean`);
  }
});

test('제외한 라우팅 다이얼로그는 더 강한 실패 처리를 갖고 있어서 제외된 것이다', () => {
  // 이 표준화의 대상은 "실패를 콘솔에만 남기던" 여섯이다. 그보다 나은 처리를 이미
  // 가진 다이얼로그를 헬퍼로 끌어내리면 UX 가 후퇴한다 — 제외 사유가 사라지면
  // (showError 를 잃으면) 이 테스트가 실패해 재검토를 요구한다.
  for (const { file, reason } of ROUTED_BUT_EXCLUDED) {
    const s = src(file);
    assert.match(s, /operationType:\s*'\w+'/, `${file}: 라우팅된 다이얼로그여야 함(제외 목록의 전제)`);
    assert.match(
      s,
      new RegExp(`this\\.${reason}\\(`),
      `${file}: 제외 사유(${reason})가 사라졌다면 공용 헬퍼로 통일할지 재검토할 것`,
    );
    assert.match(s, /return false; \/\/ 실패 → 대화상자 유지/, `${file}: 실패 시 다이얼로그 유지`);
  }
});
