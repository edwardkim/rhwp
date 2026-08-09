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
import { existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { beforeAll, describe, expect, it } from 'vitest';

const PKG_ROOT = path.join(path.dirname(fileURLToPath(import.meta.url)), '..');
const BIN = path.join(PKG_ROOT, 'bin', 'rhwp-mcp.cjs');
const DIST_ENTRY = path.join(PKG_ROOT, 'dist', 'index.cjs');

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
      // initialize 응답 한 줄이면 충분 — 서버를 종료시킨다.
      if (out.includes('"jsonrpc"')) child.kill();
    });
    child.stderr.on('data', (d) => (err += String(d)));
    child.on('exit', (code) => resolve({ code, out, err }));
    if (input !== undefined) child.stdin.write(input);
  });
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
});
