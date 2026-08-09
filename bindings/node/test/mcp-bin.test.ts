/**
 * [#4349] npx 원라인 MCP bin 계약.
 *
 * ① 실 바이너리로 initialize JSON-RPC 왕복이 되고(스크립트가 곧 stdio MCP 서버),
 * ② 바이너리 미발견이면 설치 안내와 exit 2 — 조용한 실패 금지.
 */
import { spawn } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { describe, expect, it } from 'vitest';

const BIN = path.join(path.dirname(fileURLToPath(import.meta.url)), '..', 'bin', 'rhwp-mcp.cjs');

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
