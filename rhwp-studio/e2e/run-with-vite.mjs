/**
 * Vite dev server 를 띄운 뒤 그 위에서 인자로 받은 명령을 실행하는 공용 러너.
 *
 *   node e2e/run-with-vite.mjs -- <command...>
 *
 * 서버는 VITE_PORT(기본 7700)부터 비어 있는 포트를 vite 가 직접 골라 127.0.0.1 에
 * 바인딩한다. listen 이 끝나면 이미 받을 준비가 된 상태라 별도 readiness 폴링이 없다.
 * VITE_URL 환경변수를 주입해 명령을 실행하고, 명령의 종료 코드를 그대로 전파하며
 * 성공·실패와 무관하게 서버를 끝낸다.
 */

import { spawnIn, startViteDevServer } from './vite-server.mjs';

const argv = process.argv.slice(2);
if (argv[0] === '--') {
  argv.shift();
}
if (argv.length === 0) {
  console.error('usage: node e2e/run-with-vite.mjs -- <command...>');
  process.exit(2);
}

let exitCode = 1;
const server = await startViteDevServer();
try {
  const child = spawnIn(argv[0], argv.slice(1), { VITE_URL: server.url });
  exitCode = await new Promise((resolve, reject) => {
    child.once('error', reject);
    child.once('exit', (code, signal) => {
      if (signal) {
        reject(new Error(`command terminated by signal ${signal}`));
        return;
      }
      resolve(code ?? 1);
    });
  });
} catch (error) {
  console.error(error?.message || error);
  exitCode = 1;
} finally {
  await server.stop();
}

process.exit(exitCode);
