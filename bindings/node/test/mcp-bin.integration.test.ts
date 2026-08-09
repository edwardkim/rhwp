/**
 * [#4349] npx 원라인 MCP bin 계약.
 *
 * ① 실 바이너리로 initialize JSON-RPC 왕복이 되고(스크립트가 곧 stdio MCP 서버),
 * ② 바이너리 미발견이면 설치 안내와 exit 2 — 조용한 실패 금지.
 *
 * **통합 파일인 이유** (vitest.config.ts 의 파일 이름 규칙): ① 은 실 rhwp 바이너리가
 * 필요하고, bin 스크립트 자체가 소스가 아니라 배포 산출물(`dist/index.cjs`)의
 * `findBinary` 를 재사용한다 — "바이너리 없이 도는" 단위 프로젝트에서는 이 계약을
 * 밟을 수 없다. dist 는 npm 배포물에는 항상 있지만 클론·CI 에는 없으므로,
 * 여기서 한 번 빌드해 실제 배포물과 같은 경로를 검증한다.
 */
import { execSync, spawn } from 'node:child_process';
import type { ChildProcess } from 'node:child_process';
import { existsSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { beforeAll, describe, expect, it } from 'vitest';

const PKG_ROOT = path.join(path.dirname(fileURLToPath(import.meta.url)), '..');
const BIN = path.join(PKG_ROOT, 'bin', 'rhwp-mcp.cjs');
const DIST_ENTRY = path.join(PKG_ROOT, 'dist', 'index.cjs');
const require = createRequire(import.meta.url);
const { forceKillChildTree } = require(BIN) as {
  forceKillChildTree: (child: ChildProcess) => boolean;
};

beforeAll(() => {
  // bin 이 require 하는 배포 산출물이 없으면 만들어 둔다 — 통합 프로젝트의
  // hookTimeout(120s) 안에서 tsup+tsc 가 끝난다.
  if (!existsSync(DIST_ENTRY)) {
    execSync('npm run build', { cwd: PKG_ROOT, stdio: 'inherit' });
  }
});

function runOnce(env: NodeJS.ProcessEnv, input?: string): Promise<{ code: number | null; out: string; err: string }> {
  return new Promise((resolve) => {
    const child = spawn(process.execPath, [BIN], { env, stdio: ['pipe', 'pipe', 'pipe'] });
    let out = '';
    let err = '';
    child.stdout.on('data', (d) => {
      out += String(d);
      // initialize 응답 한 줄이면 충분 — stdin EOF 로 서버를 정상 종료시킨다.
      if (out.includes('"jsonrpc"') && !child.stdin.destroyed) child.stdin.end();
    });
    child.stderr.on('data', (d) => (err += String(d)));
    child.on('exit', (code) => resolve({ code, out, err }));
    if (input !== undefined) child.stdin.write(input);
  });
}

async function expectSignalForwarded(signal: 'SIGINT' | 'SIGTERM'): Promise<void> {
  const dir = mkdtempSync(path.join(tmpdir(), 'rhwp-mcp-signal-'));
  const fakeServer = path.join(dir, 'mcp-serve');
  writeFileSync(
    fakeServer,
    [
      "'use strict';",
      'process.stdout.write(`READY ${process.pid}\\n`);',
      'const stop = (signal) => {',
      '  process.stdout.write(`FORWARDED ${signal}\\n`, () => process.exit(0));',
      '};',
      "process.on('SIGINT', () => stop('SIGINT'));",
      "process.on('SIGTERM', () => stop('SIGTERM'));",
      'setInterval(() => {}, 1_000);',
    ].join('\n'),
  );

  const wrapper = spawn(process.execPath, [BIN], {
    cwd: dir,
    env: { ...process.env, RHWP_BIN: process.execPath },
    stdio: ['ignore', 'pipe', 'pipe'],
  });
  let out = '';
  let err = '';
  let serverPid: number | undefined;

  try {
    const ready = new Promise<void>((resolve, reject) => {
      const timer = setTimeout(() => reject(new Error(`fake server 준비 시간 초과: ${out}\n${err}`)), 10_000);
      wrapper.stdout.on('data', (chunk) => {
        out += String(chunk);
        const match = out.match(/READY (\d+)/);
        if (match?.[1] !== undefined) {
          serverPid = Number(match[1]);
          clearTimeout(timer);
          resolve();
        }
      });
      wrapper.stderr.on('data', (chunk) => (err += String(chunk)));
      wrapper.once('error', reject);
    });
    const exited = new Promise<{ code: number | null; signal: NodeJS.Signals | null }>((resolve) => {
      wrapper.once('exit', (code, exitSignal) => resolve({ code, signal: exitSignal }));
    });

    await ready;
    expect(wrapper.kill(signal)).toBe(true);
    const status = await exited;

    expect(status).toEqual({ code: 0, signal: null });
    expect(out).toContain(`FORWARDED ${signal}`);
    expect(serverPid).toBeDefined();
    expect(() => process.kill(serverPid!, 0)).toThrow();
  } finally {
    if (wrapper.exitCode === null && wrapper.signalCode === null) wrapper.kill('SIGKILL');
    if (serverPid !== undefined) {
      try {
        process.kill(serverPid, 'SIGKILL');
      } catch {
        // 이미 wrapper가 회수했다.
      }
    }
    rmSync(dir, { recursive: true, force: true });
  }
}

async function waitForProcessExit(pid: number, timeoutMs: number): Promise<void> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    try {
      process.kill(pid, 0);
    } catch {
      return;
    }
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
  throw new Error(`PID ${pid}가 ${timeoutMs}ms 뒤에도 실행 중입니다`);
}

async function expectWindowsProcessTreeKilled(): Promise<void> {
  const dir = mkdtempSync(path.join(tmpdir(), 'rhwp-mcp-tree-'));
  const parentScript = path.join(dir, 'parent.cjs');
  writeFileSync(
    parentScript,
    [
      "'use strict';",
      "const { spawn } = require('node:child_process');",
      "const grandchild = spawn(process.execPath, ['-e', 'setInterval(() => {}, 1000)'], { stdio: 'ignore' });",
      'process.stdout.write(`TREE ${grandchild.pid}\\n`);',
      'setInterval(() => {}, 1_000);',
    ].join('\n'),
  );

  const parent = spawn(process.execPath, [parentScript], {
    stdio: ['ignore', 'pipe', 'pipe'],
    windowsHide: true,
  });
  let out = '';
  let err = '';
  let grandchildPid: number | undefined;

  try {
    await new Promise<void>((resolve, reject) => {
      const timer = setTimeout(() => reject(new Error(`process tree 준비 시간 초과: ${out}\n${err}`)), 10_000);
      parent.stdout.on('data', (chunk) => {
        out += String(chunk);
        const match = out.match(/TREE (\d+)/);
        if (match?.[1] !== undefined) {
          grandchildPid = Number(match[1]);
          clearTimeout(timer);
          resolve();
        }
      });
      parent.stderr.on('data', (chunk) => (err += String(chunk)));
      parent.once('error', reject);
    });
    const exited = new Promise<void>((resolve) => parent.once('exit', () => resolve()));

    expect(forceKillChildTree(parent)).toBe(true);
    await Promise.race([
      exited,
      new Promise<never>((_, reject) =>
        setTimeout(() => reject(new Error(`parent 종료 시간 초과: ${out}\n${err}`)), 10_000),
      ),
    ]);
    expect(grandchildPid).toBeDefined();
    await waitForProcessExit(grandchildPid!, 10_000);
  } finally {
    if (parent.exitCode === null && parent.signalCode === null) forceKillChildTree(parent);
    if (grandchildPid !== undefined) {
      try {
        process.kill(grandchildPid, 'SIGKILL');
      } catch {
        // tree kill로 이미 종료했다.
      }
    }
    rmSync(dir, { recursive: true, force: true });
  }
}

describe('rhwp-mcp bin', () => {
  it('실 바이너리로 initialize 왕복이 된다', async () => {
    const request =
      JSON.stringify({ jsonrpc: '2.0', id: 1, method: 'initialize', params: { protocolVersion: '2025-06-18', capabilities: {}, clientInfo: { name: 'bin-test', version: '0' } } }) + '\n';
    const { out } = await runOnce(process.env, request);
    expect(out).toContain('"jsonrpc"');
    expect(out).toContain('"result"');
  }, 30_000);

  it('바이너리 미발견이면 설치 안내와 exit 2', async () => {
    const env = { ...process.env, PATH: '', Path: '', RHWP_BIN: '' };
    const { code, err } = await runOnce(env);
    expect(code).toBe(2);
    expect(err).toContain('releases');
  }, 30_000);

  it.skipIf(process.platform === 'win32')(
    'SIGINT와 SIGTERM을 실제 서버에 전달하고 자식을 회수한다',
    async () => {
      await expectSignalForwarded('SIGINT');
      await expectSignalForwarded('SIGTERM');
    },
    30_000,
  );

  it.skipIf(process.platform !== 'win32')(
    'wrapper가 통제하는 강제 종료는 Windows child tree 전체를 닫는다',
    expectWindowsProcessTreeKilled,
    30_000,
  );
});
