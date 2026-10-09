import { createHash, randomUUID } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';

// Observe the bytes received during app initialization, not a second diagnostic fetch.
export async function observeWasmArtifact(page, manifestPath = process.env.RHWP_WASM_BUILD_MANIFEST) {
  if (!manifestPath) return { finish: async () => {}, stop: () => {} };
  const manifest = JSON.parse(readFileSync(manifestPath, 'utf8'));
  const expected = manifest.artifacts?.['rhwp_bg.wasm']?.sha256;
  if (manifest.success !== true || !/^[a-f0-9]{64}$/.test(expected || '')) {
    throw new Error('WASM build manifest is incomplete or unsuccessful');
  }
  const session = await page.createCDPSession();
  // Dev WASM can exceed Chrome's default inspector body buffer (~41 MB here).
  const bufferBytes = Math.max(64 * 1024 * 1024,
    Number(manifest.artifacts['rhwp_bg.wasm'].bytes || 0) * 2);
  await session.send('Network.enable', {
    maxResourceBufferSize: bufferBytes, maxTotalBufferSize: bufferBytes * 2,
  });
  const responses = [];
  session.on('Network.responseReceived', ({ requestId, response }) => {
    if (new URL(response.url).pathname.endsWith('/rhwp_bg.wasm')) {
      responses.push({ requestId, url: response.url, status: response.status });
    }
  });
  let detached = false;
  const stop = async () => {
    if (!detached) {
      detached = true;
      await session.detach();
    }
  };
  return {
    stop,
    async finish() {
      const received = await Promise.all(responses.map(async ({ requestId, ...response }) => {
        try {
          const { body, base64Encoded } = await session.send('Network.getResponseBody', { requestId });
          const bytes = Buffer.from(body, base64Encoded ? 'base64' : 'utf8');
          return { ...response, sha256: createHash('sha256').update(bytes).digest('hex') };
        } catch (error) {
          return { ...response, error: String(error) };
        }
      }));
      await stop();
      const matched = received.length > 0 && received.every(
        (item) => item.status === 200 && item.sha256 === expected,
      );
      const directory = path.join(path.dirname(manifestPath), 'consumption');
      mkdirSync(directory, { recursive: true });
      writeFileSync(path.join(directory, `${process.pid}-${randomUUID()}.json`), JSON.stringify({
        source_sha: manifest.source_sha, profile: manifest.profile, expected_sha256: expected,
        page: page.url(), matched, received,
      }, null, 2) + '\n');
      if (!matched) throw new Error(`WASM artifact mismatch or missing response: expected ${expected}`);
      console.log(`  [wasm-artifact] matched ${expected}`);
    },
  };
}
