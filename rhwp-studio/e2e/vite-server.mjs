/**
 * Vite dev server 기동·종료의 공용 헬퍼.
 *
 * 서버는 **이 프로세스 안에서** 돈다 — vite 의 Node API(`createServer`)가 포트 선택·
 * readiness·종료를 모두 책임진다. 종전에는 `node vite.js` 를 자식으로 띄우고 포트를
 * 직접 스캔한 뒤 HTTP 로 readiness 를 폴링하고 SIGTERM/SIGKILL·taskkill 로 트리를
 * 정리했는데, 그 전부가 라이브러리가 이미 하는 일이었다. 자식 프로세스 설계의 근거였던
 * win32 `.cmd` EINVAL 도 spawn 을 안 하면 성립하지 않는다.
 *
 * 로그는 `target/rhwp-studio-vite*.log` 에 그대로 남긴다 — CI 가 아티팩트로 올린다
 * (`.github/workflows/render-diff.yml`). in-process 라 stdout 으로도 보이지만, 기존
 * 아티팩트 계약을 깨지 않도록 `customLogger` 로 파일에도 쓴다.
 *
 * 테스트 명령 자체는 여전히 자식 프로세스다 — `spawnIn` 이 그 몫이다.
 */

import { spawn } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createLogger, createServer } from 'vite';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
export const studioRoot = path.resolve(__dirname, '..');
const repoRoot = path.resolve(studioRoot, '..');
const npmCmd = process.platform === 'win32' ? 'npm.cmd' : 'npm';

/**
 * studio 루트에서 명령을 자식으로 띄운다.
 *
 * win32 의 npm 은 `npm.cmd` 이고 Node 20.12+ 는 `.cmd` 직접 spawn 을 EINVAL 로 거절하므로
 * 그때만 shell 을 경유한다(인자는 공백·메타문자 없는 고정값뿐이다).
 */
export function spawnIn(command, args, extraEnv = {}) {
  return spawn(command, args, {
    cwd: studioRoot,
    stdio: 'inherit',
    shell: process.platform === 'win32' && command.endsWith('.cmd'),
    env: { ...process.env, ...extraEnv },
  });
}

export function spawnNpm(args, extraEnv = {}) {
  return spawnIn(npmCmd, ['run', ...args], extraEnv);
}

export function viteLogPath(suffix = '') {
  return path.join(repoRoot, 'target', `rhwp-studio-vite${suffix}.log`);
}

/**
 * vite dev server 를 이 프로세스에서 띄운다.
 *
 * 반환 `url` 은 **origin 만** 담는다 — `resolvedUrls.local[0]` 은 base 가 붙어 끝에
 * 슬래시가 있고, 호출부가 `${url}/path` 로 이어 붙이면 `//path` 가 되어 dev server 의
 * SPA 폴백이 index.html 을 돌려준다(조용히 엉뚱한 응답).
 */
export async function startViteDevServer({
  preferredPort = Number(process.env.VITE_PORT || '7700'),
  logPath = viteLogPath(),
} = {}) {
  fs.mkdirSync(path.dirname(logPath), { recursive: true });
  const logFile = fs.openSync(logPath, 'w');
  const write = (msg) => fs.writeSync(logFile, `${msg}
`);
  const base = createLogger('info', { allowClearScreen: false });
  const logger = {
    ...base,
    info: (msg, opts) => { write(msg); base.info(msg, opts); },
    warn: (msg, opts) => { write(msg); base.warn(msg, opts); },
    error: (msg, opts) => { write(msg); base.error(msg, opts); },
  };

  const server = await createServer({
    root: studioRoot,
    configFile: path.join(studioRoot, 'vite.config.ts'),
    customLogger: logger,
    server: { host: '127.0.0.1', port: preferredPort },
  });
  await server.listen();
  server.printUrls();

  const resolved = server.resolvedUrls?.local?.[0];
  if (!resolved) {
    await server.close();
    fs.closeSync(logFile);
    throw new Error(`vite dev server 가 주소를 내놓지 않았다 (로그: ${logPath})`);
  }
  const url = new URL(resolved).origin;

  return {
    url,
    port: server.config.server.port,
    logPath,
    async stop() {
      await server.close();
      fs.closeSync(logFile);
    },
  };
}
