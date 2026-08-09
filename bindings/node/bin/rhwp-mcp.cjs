#!/usr/bin/env node
/**
 * [#4349] npx 원라인 MCP — `npx -p @rhwp/node rhwp-mcp` 로 rhwp MCP 서버를 연다.
 *
 * 바이너리 탐색은 패키지의 `findBinary()`(RHWP_BIN → 패키지 동봉 → PATH, D-14
 * 계약)를 **그대로 재사용**한다 — 여기서 탐색을 다시 쓰면 두 규칙이 갈라진다.
 * 찾으면 `rhwp mcp-serve` 를 stdio 상속으로 실행해 이 프로세스가 곧 MCP
 * 서버(stdio JSON-RPC)가 된다. 못 찾으면 설치 경로를 안내하고 exit 2(사용법
 * 오류 계열 — #2707 사전과 정합).
 */
'use strict';

const { spawn } = require('node:child_process');

const SHUTDOWN_GRACE_MS = 5_000;

function fail(message) {
  process.stderr.write(message + '\n');
  process.exit(2);
}

function main() {
  let findBinary;
  try {
    ({ findBinary } = require('../dist/index.cjs'));
  } catch (err) {
    fail(
      '@rhwp/node 빌드 산출물(dist)이 없습니다. 저장소 클론이라면 먼저: cd bindings/node && npm ci && npm run build\n' +
        `원인: ${err && err.message}`
    );
  }

  let binary;
  try {
    binary = findBinary();
  } catch (err) {
    fail(
      (err && err.message ? err.message + '\n' : '') +
        'rhwp 실행 파일이 필요합니다 — 설치 1줄:\n' +
        '  · Releases 바이너리: https://github.com/edwardkim/rhwp/releases/latest\n' +
        '  · Windows(scoop): scoop install https://raw.githubusercontent.com/edwardkim/rhwp/devel/contrib/packaging/scoop/rhwp.json\n' +
        '  · 또는 RHWP_BIN=/path/to/rhwp 지정'
    );
  }

  const child = spawn(binary, ['mcp-serve', ...process.argv.slice(2)], {
    stdio: 'inherit',
  });

  let shutdownSignal;
  let forceKillTimer;
  const signalHandlers = new Map();

  const childIsRunning = () => child.exitCode === null && child.signalCode === null;
  const killChild = (signal) => {
    if (!childIsRunning()) return false;
    try {
      return child.kill(signal);
    } catch {
      return false;
    }
  };

  // MCP hosts usually stop stdio servers by closing stdin. If they signal the
  // wrapper PID instead, the real server must receive the same signal rather
  // than surviving as an orphan. A second signal (or the grace timeout) is an
  // explicit request to stop waiting for graceful shutdown.
  const forwardSignal = (signal) => {
    if (shutdownSignal !== undefined) {
      killChild('SIGKILL');
      return;
    }
    shutdownSignal = signal;
    if (!killChild(signal)) return;
    forceKillTimer = setTimeout(() => killChild('SIGKILL'), SHUTDOWN_GRACE_MS);
  };

  for (const signal of ['SIGINT', 'SIGTERM']) {
    const handler = () => forwardSignal(signal);
    signalHandlers.set(signal, handler);
    process.on(signal, handler);
  }

  // Covers an explicit process.exit() path. Signal handlers above keep the
  // wrapper alive until the child is reaped; this hook is only a last-resort,
  // synchronous termination request.
  const stopChildOnWrapperExit = () => {
    killChild('SIGTERM');
  };
  process.once('exit', stopChildOnWrapperExit);

  const cleanup = () => {
    if (forceKillTimer !== undefined) clearTimeout(forceKillTimer);
    process.removeListener('exit', stopChildOnWrapperExit);
    for (const [signal, handler] of signalHandlers) {
      process.removeListener(signal, handler);
    }
  };

  child.once('error', (err) => {
    cleanup();
    fail(`rhwp 실행 실패: ${err.message}`);
  });
  child.once('exit', (code, signal) => {
    cleanup();
    process.exitCode = signal ? 1 : code == null ? 1 : code;
  });
}

main();
