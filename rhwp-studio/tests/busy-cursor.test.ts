import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

import {
  BUSY_CLASS,
  busyDepth,
  withBusyCursor,
  type BusyRoot,
} from '../src/view/busy-cursor.ts';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

function fakeRoot() {
  const tokens = new Set<string>();
  const log: string[] = [];
  const root: BusyRoot = {
    classList: {
      add(token: string) {
        tokens.add(token);
        log.push(`+${token}`);
      },
      remove(token: string) {
        tokens.delete(token);
        log.push(`-${token}`);
      },
    },
  };
  return { root, tokens, log };
}

test('처리 동안 대기 커서를 걸고 끝나면 되돌린다', async () => {
  const { root, tokens, log } = fakeRoot();

  const seen: boolean[] = [];
  await withBusyCursor(root, async () => {
    seen.push(tokens.has(BUSY_CLASS));
  });

  assert.deepEqual(seen, [true], '처리 중에는 대기 커서가 걸려 있어야 한다');
  assert.equal(tokens.has(BUSY_CLASS), false, '끝나면 되돌려야 한다');
  assert.deepEqual(log, [`+${BUSY_CLASS}`, `-${BUSY_CLASS}`]);
  assert.equal(busyDepth(), 0);
});

test('겹쳐 불러도 가장 바깥 처리가 끝날 때 한 번만 되돌린다', async () => {
  const { root, tokens, log } = fakeRoot();

  await withBusyCursor(root, async () => {
    // loadFile → loadBytes 처럼 안쪽에서 다시 감싸는 경우
    await withBusyCursor(root, async () => {
      assert.equal(busyDepth(), 2);
    });
    assert.equal(tokens.has(BUSY_CLASS), true, '안쪽이 끝나도 유지되어야 한다');
  });

  assert.equal(tokens.has(BUSY_CLASS), false);
  assert.deepEqual(log, [`+${BUSY_CLASS}`, `-${BUSY_CLASS}`], '클래스 조작은 1회씩');
  assert.equal(busyDepth(), 0);
});

test('처리가 실패해도 대기 커서를 반드시 되돌린다', async () => {
  const { root, tokens } = fakeRoot();

  await assert.rejects(
    withBusyCursor(root, async () => {
      throw new Error('파싱 실패');
    }),
    /파싱 실패/,
  );

  assert.equal(tokens.has(BUSY_CLASS), false);
  assert.equal(busyDepth(), 0);
});

test('대기 커서 CSS 는 편집 영역의 인라인 커서까지 덮는다', () => {
  const css = readFileSync(join(rootDir, 'src/style.css'), 'utf8');
  // 편집 영역이 style.cursor 를 직접 넣으므로 !important 가 없으면 지지 않는다.
  assert.match(css, new RegExp(`:root\\.${BUSY_CLASS}[^{]*\\{[^}]*cursor:\\s*wait\\s*!important`));
  assert.match(css, new RegExp(`:root\\.${BUSY_CLASS}\\s*\\*`), '자식 요소까지 덮어야 한다');
});
