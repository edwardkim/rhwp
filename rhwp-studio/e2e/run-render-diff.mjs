import {
  spawnNpm,
  startViteDevServer,
  waitForServer,
} from './vite-server.mjs';

async function runNpmScript(script, serverUrl) {
  const child = spawnNpm(['run', script], { VITE_URL: serverUrl });
  const exitCode = await new Promise((resolve, reject) => {
    child.once('error', reject);
    child.once('exit', (code, signal) => {
      if (signal) {
        reject(new Error(`${script} terminated by signal ${signal}`));
        return;
      }
      resolve(code ?? 1);
    });
  });
  if (exitCode !== 0) {
    throw new Error(`${script} failed with exit code ${exitCode}`);
  }
}

const server = await startViteDevServer();

try {
  await waitForServer(server.url, server.child, server.logPath);
  if (process.env.RHWP_RENDER_DIFF_SKIP_CANVAS !== '1') {
    await runNpmScript('e2e:render-diff', server.url);
  }
  if (process.env.RHWP_RENDER_DIFF_PDF === '1') {
    if (process.env.RHWP_RENDER_DIFF_DIRECT_PDF_GATE === '1') {
      await runNpmScript('e2e:pdf-render-diff', server.url);
    } else {
      try {
        await runNpmScript('e2e:pdf-render-diff', server.url);
      } catch (error) {
        console.error(`PDF render diff report failed without gating CI: ${error.message || error}`);
      }
    }
  }
} finally {
  await server.stop();
}
