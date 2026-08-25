/**
 * Vite dev server 를 띄운 뒤 그 위에서 인자로 받은 명령을 실행하는 공용 러너.
 *
 *   node e2e/run-with-vite.mjs -- <command...>
 *   node e2e/run-with-vite.mjs --npm <script> [<script args...>]
 *
 * 서버는 VITE_PORT(기본 7700)부터 비어 있는 포트를 택해 127.0.0.1 에 바인딩하고
 * readiness 를 기다린 뒤, VITE_URL 환경변수를 주입해 명령을 실행한다. 명령의
 * 종료 코드를 그대로 전파하며 성공·실패와 무관하게 서버를 끝낸다. 이미 같은
 * 포트로 떠 있는 dev server 가 있으면 다음 포트를 택하므로 로컬에서 겹치지 않는다.
 */

import {
  spawnNpm,
  spawnStudioCommand,
  startViteDevServer,
  waitForServer,
} from './vite-server.mjs';

const argv = process.argv.slice(2);
let mode = 'raw';
if (argv[0] === '--npm') {
  mode = 'npm';
  argv.shift();
} else if (argv[0] === '--') {
  argv.shift();
}
if (argv.length === 0) {
  console.error('usage: node e2e/run-with-vite.mjs [--npm <script> | -- <command...>]');
  process.exit(2);
}

let exitCode = 1;
const server = await startViteDevServer();
try {
  await waitForServer(server.url, server.child, server.logPath);
  const extraEnv = { VITE_URL: server.url };
  const child = mode === 'npm'
    ? spawnNpm(['run', ...argv], extraEnv)
    : spawnStudioCommand(argv[0], argv.slice(1), extraEnv);
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
