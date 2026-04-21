import { spawn } from 'node:child_process';
import { setTimeout as delay } from 'node:timers/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const studioRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const npmCmd = process.platform === 'win32' ? 'npm.cmd' : 'npm';
const serverUrl = process.env.VITE_URL || 'http://127.0.0.1:7700';

function spawnCommand(args, extraEnv = {}) {
  return spawn(npmCmd, args, {
    cwd: studioRoot,
    stdio: 'inherit',
    env: {
      ...process.env,
      ...extraEnv,
    },
  });
}

function waitForExit(child, signal) {
  return new Promise((resolve) => {
    child.once('exit', () => resolve());
    child.kill(signal);
  });
}

async function stopServer(child) {
  if (child.exitCode !== null || child.signalCode) {
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

async function waitForServer(url, timeoutMs = 30000) {
  const deadline = Date.now() + timeoutMs;
  let lastError = null;

  while (Date.now() < deadline) {
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

  throw lastError ?? new Error(`timed out waiting for ${url}`);
}

async function runSuite() {
  const child = spawnCommand(['run', 'e2e:headless']);
  const exitCode = await new Promise((resolve, reject) => {
    child.once('error', reject);
    child.once('exit', (code, signal) => {
      if (signal) {
        reject(new Error(`e2e suite terminated by signal ${signal}`));
        return;
      }
      resolve(code ?? 1);
    });
  });
  if (exitCode !== 0) {
    throw new Error(`e2e suite failed with exit code ${exitCode}`);
  }
}

const devServer = spawnCommand(
  ['run', 'dev', '--', '--host', '0.0.0.0', '--port', '7700'],
  { BROWSER: 'none' },
);

try {
  await waitForServer(serverUrl);
  await runSuite();
} finally {
  await stopServer(devServer);
}
