/**
 * Vite dev server 기동·대기·종료의 공용 헬퍼.
 *
 * run-render-diff.mjs 와 run-with-vite.mjs 가 같은 기동 계약(포트 탐색 →
 * 127.0.0.1 바인딩 → readiness 폴링 → SIGTERM 뒤 5초 안에 SIGKILL)을 쓰도록
 * run-render-diff.mjs 에서 추출했다. 로그는 target/rhwp-studio-vite*.log 에
 * 남기고, 서버가 readiness 전에 죽으면 로그 내용을 함께 던진다.
 */

import { spawn } from 'node:child_process';
import fs from 'node:fs';
import net from 'node:net';
import path from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
export const studioRoot = path.resolve(__dirname, '..');
const repoRoot = path.resolve(studioRoot, '..');
const npmCmd = process.platform === 'win32' ? 'npm.cmd' : 'npm';

export function spawnNpm(args, extraEnv = {}, stdio = 'inherit') {
  return spawn(npmCmd, args, {
    cwd: studioRoot,
    stdio,
    // win32 의 npm 은 npm.cmd 다 — Node 20+ 부터 .cmd 직접 spawn 이 EINVAL 로
    // 거절되므로 shell 경유로 띄운다(인자는 공백·메타문자 없는 고정값뿐이다).
    shell: process.platform === 'win32',
    env: {
      ...process.env,
      ...extraEnv,
    },
  });
}

export function spawnStudioCommand(command, args, extraEnv = {}, stdio = 'inherit') {
  return spawn(command, args, {
    cwd: studioRoot,
    stdio,
    env: {
      ...process.env,
      ...extraEnv,
    },
  });
}

export function viteLogPath(suffix = '') {
  return path.join(repoRoot, 'target', `rhwp-studio-vite${suffix}.log`);
}

function waitForExit(child, signal) {
  return new Promise((resolve) => {
    child.once('exit', () => resolve());
    child.kill(signal);
  });
}

export async function stopServer(child) {
  if (child.exitCode !== null || child.signalCode) {
    return;
  }
  if (process.platform === 'win32' && child.pid) {
    // shell 경유로 띄운 자식(cmd.exe 래퍼)은 SIGTERM 으로 트리가 정리되지 않는다.
    await new Promise((resolve) => {
      spawn('taskkill', ['/pid', String(child.pid), '/T', '/F'], { stdio: 'ignore' })
        .once('exit', resolve);
    });
    return;
  }
  await Promise.race([
    waitForExit(child, 'SIGTERM'),
    delay(5000).then(async () => {
      if (child.exitCode === null && !child.signalCode) {
        await waitForExit(child, 'SIGKILL');
      }
    }),
  ]);
}

export async function waitForServer(url, child, logPath, timeoutMs = 30000) {
  const deadline = Date.now() + timeoutMs;
  let lastError = null;

  while (Date.now() < deadline) {
    if (child.exitCode !== null || child.signalCode) {
      const log = fs.existsSync(logPath) ? fs.readFileSync(logPath, 'utf8') : '';
      throw new Error(`Vite dev server exited before ${url} became ready.\n${log}`);
    }
    try {
      const response = await fetch(url);
      if (response.ok) {
        return;
      }
      lastError = new Error(`server responded with status ${response.status}`);
    } catch (error) {
      lastError = error;
    }
    await delay(500);
  }

  const log = fs.existsSync(logPath) ? fs.readFileSync(logPath, 'utf8') : '';
  throw new Error(`${lastError?.message || `timed out waiting for ${url}`}\n${log}`);
}

export async function findAvailablePort(startPort, attempts = 20) {
  for (let port = startPort; port < startPort + attempts; port += 1) {
    const available = await new Promise((resolve) => {
      const server = net.createServer();
      server.once('error', () => resolve(false));
      server.listen(port, '127.0.0.1', () => {
        server.close(() => resolve(true));
      });
    });
    if (available) {
      return port;
    }
  }
  throw new Error(`failed to find an available port starting at ${startPort}`);
}

/**
 * vite dev server 를 기동한다. npm.cmd 경유가 아니라 node 로 vite.js 를 직접
 * 띄운다 — win32 에서 .cmd spawn 은 EINVAL 로 거절되고 shell 우회는 트리
 * 종료(SIGTERM)를 망가뜨리기 때문. readiness 대기와 로그 핸들 닫기는 호출자의
 * 몫으로 남긴다(waitForServer · 반환 객체의 stop()).
 */
export async function startViteDevServer({
  preferredPort = Number(process.env.VITE_PORT || '7700'),
  logPath = viteLogPath(),
} = {}) {
  const port = await findAvailablePort(preferredPort);
  const url = `http://127.0.0.1:${port}`;
  fs.mkdirSync(path.dirname(logPath), { recursive: true });
  const logFile = fs.openSync(logPath, 'w');
  const viteJs = path.join(studioRoot, 'node_modules', 'vite', 'bin', 'vite.js');
  const child = spawn(
    process.execPath,
    [viteJs, '--host', '127.0.0.1', '--port', String(port), '--strictPort'],
    {
      cwd: studioRoot,
      stdio: ['ignore', logFile, logFile],
      env: {
        ...process.env,
        BROWSER: 'none',
      },
    },
  );
  return {
    url,
    port,
    child,
    logPath,
    async stop() {
      await stopServer(child);
      fs.closeSync(logFile);
    },
  };
}
