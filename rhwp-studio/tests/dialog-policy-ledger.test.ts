/**
 * 다이얼로그 정책 원장 — "이 커맨드는 대화상자를 연다" 는 표기를 저작 시점에 강제한다.
 *
 * 외부 JS 자동화(`window.rhwpStudio.automation`, embed RPC)는 다이얼로그를 여는 커맨드를 그냥
 * 실행하면 **응답 없는 대기**가 된다. 사람이 눌러야 끝나는 것을 스크립트가 기다리기 때문이다.
 * 그래서 자동화는 `opensDialog` 를 보고 거절하거나 다른 경로를 안내한다.
 *
 * 표기를 사람 기억에 맡기면 새 다이얼로그 커맨드가 조용히 새어 나간다. 이 테스트가 그 자리를
 * 막는다 — 커맨드 정의 안에서 Dialog 를 쓰면서 표기가 없으면 실패한다.
 *
 * 계획: mydocs/plans/rhwp_studio_hwpctrl_plugin_impl.md §3.3
 */
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const commandsDir = join(rootDir, 'src/command/commands');

/** 커맨드 정의 하나 — 파일 안의 객체 리터럴 단위로 자른다. */
interface CommandBlock {
  file: string;
  id: string;
  body: string;
}

function commandBlocks(): CommandBlock[] {
  const blocks: CommandBlock[] = [];
  for (const name of readdirSync(commandsDir)) {
    if (!name.endsWith('.ts')) continue;
    const source = readFileSync(join(commandsDir, name), 'utf8');
    for (const part of source.split(/(?=\n {2}\{\n)/)) {
      const id = /\n?\s*id: '([^']+)',/.exec(part)?.[1];
      if (id) blocks.push({ file: name, id, body: part });
    }
  }
  return blocks;
}

/** 정의 본문이 다이얼로그를 여는가. `new *Dialog` 와 `Dialog.` 정적 호출을 본다. */
function opensDialog(body: string): boolean {
  return /\bnew [A-Za-z]*Dialog\b|\bDialog\.[a-zA-Z]/.test(body);
}

test('다이얼로그를 여는 커맨드는 opensDialog 를 표기한다', () => {
  const missing = commandBlocks()
    .filter((block) => opensDialog(block.body) && !/opensDialog:\s*true/.test(block.body))
    .map((block) => `${block.file}: ${block.id}`);

  assert.deepEqual(
    missing,
    [],
    '다이얼로그를 여는 커맨드에 `opensDialog: true` 가 없다. 자동화가 이 커맨드를 실행하면\n'
      + '사용자 조작을 기다리며 응답이 멈춘다. 정의에 표기를 추가하라:\n'
      + missing.map((entry) => `  - ${entry}`).join('\n'),
  );
});

test('opensDialog 표기는 실제 다이얼로그 사용과 짝이 맞는다', () => {
  // 반대 방향 — 표기해 놓고 다이얼로그를 안 여는 커맨드는 자동화를 괜히 막는다.
  const stale = commandBlocks()
    .filter((block) => /opensDialog:\s*true/.test(block.body) && !opensDialog(block.body))
    .map((block) => `${block.file}: ${block.id}`);

  assert.deepEqual(stale, [], `표기만 있고 다이얼로그를 열지 않는다: ${stale.join(', ')}`);
});

test('원장이 비어 있지 않다 — 파서가 깨지면 전수 검사가 조용히 통과한다', () => {
  const flagged = commandBlocks().filter((block) => /opensDialog:\s*true/.test(block.body));
  assert.ok(
    flagged.length >= 30,
    `표기된 커맨드가 ${flagged.length}개뿐이다 — 블록 분해가 깨졌을 수 있다`,
  );
});
