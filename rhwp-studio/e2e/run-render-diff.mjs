import { spawn } from 'node:child_process';
import fs from 'node:fs';
import net from 'node:net';
import path from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const studioRoot = path.resolve(__dirname, '..');
const repoRoot = path.resolve(studioRoot, '..');
const npmCmd = process.platform === 'win32' ? 'npm.cmd' : 'npm';
const preferredPort = Number(process.env.VITE_PORT || '7700');

function spawnCommand(args, extraEnv = {}, stdio = 'inherit') {
  return spawn(npmCmd, args, {
    cwd: studioRoot,
    stdio,
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

async function waitForServer(url, child, logPath, timeoutMs = 30000) {
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

async function findAvailablePort(startPort, attempts = 20) {
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

async function runRenderDiff(serverUrl) {
  const child = spawnCommand(['run', 'e2e:render-diff'], { VITE_URL: serverUrl });
  const exitCode = await new Promise((resolve, reject) => {
    child.once('error', reject);
    child.once('exit', (code, signal) => {
      if (signal) {
        reject(new Error(`render diff terminated by signal ${signal}`));
        return;
      }
      resolve(code ?? 1);
    });
  });
  if (exitCode !== 0) {
    throw new Error(`render diff failed with exit code ${exitCode}`);
  }
}

const serverPort = await findAvailablePort(preferredPort);
const serverUrl = `http://127.0.0.1:${serverPort}`;
const logPath = path.join(repoRoot, 'target', 'rhwp-studio-vite.log');
fs.mkdirSync(path.dirname(logPath), { recursive: true });
const logFile = fs.openSync(logPath, 'w');
const devServer = spawnCommand(
  ['run', 'dev', '--', '--host', '127.0.0.1', '--port', String(serverPort), '--strictPort'],
  { BROWSER: 'none' },
  ['ignore', logFile, logFile],
);

try {
  await waitForServer(serverUrl, devServer, logPath);
  await runRenderDiff(serverUrl);
} finally {
  await stopServer(devServer);
  fs.closeSync(logFile);
}
