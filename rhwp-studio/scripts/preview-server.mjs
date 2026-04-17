import http from 'node:http';
import { createReadStream } from 'node:fs';
import { access, readFile, stat, writeFile } from 'node:fs/promises';
import { extname, join, normalize, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const MIME_TYPES = new Map([
  ['.css', 'text/css; charset=utf-8'],
  ['.html', 'text/html; charset=utf-8'],
  ['.ico', 'image/x-icon'],
  ['.jpeg', 'image/jpeg'],
  ['.jpg', 'image/jpeg'],
  ['.js', 'text/javascript; charset=utf-8'],
  ['.json', 'application/json; charset=utf-8'],
  ['.png', 'image/png'],
  ['.svg', 'image/svg+xml'],
  ['.wasm', 'application/wasm'],
  ['.woff2', 'font/woff2'],
  ['.hwp', 'application/octet-stream'],
  ['.hwpx', 'application/octet-stream'],
]);

function parseCliOptions(argv) {
  const options = {
    host: '127.0.0.1',
    port: 7702,
  };

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === '--host' && argv[index + 1]) {
      options.host = argv[index + 1];
      index += 1;
      continue;
    }

    if (arg === '--port' && argv[index + 1]) {
      options.port = Number.parseInt(argv[index + 1], 10);
      index += 1;
    }
  }

  return options;
}

function getContentType(filePath) {
  return MIME_TYPES.get(extname(filePath).toLowerCase()) ?? 'application/octet-stream';
}

async function pathExists(filePath) {
  try {
    await access(filePath);
    return true;
  } catch {
    return false;
  }
}

async function readRequestBody(request) {
  const chunks = [];

  for await (const chunk of request) {
    chunks.push(Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk));
  }

  return Buffer.concat(chunks);
}

function getOpenedRoot(rootDir) {
  return resolve(join(rootDir, '__opened'));
}

function ensureSafeToken(token) {
  if (!/^[A-Za-z0-9_-]+$/.test(token)) {
    throw new Error(`Invalid token: ${token}`);
  }
}

async function readLaunchManifest(rootDir, token) {
  ensureSafeToken(token);

  const openedRoot = getOpenedRoot(rootDir);
  const manifestPath = resolve(join(openedRoot, `${token}.json`));

  if (!manifestPath.startsWith(openedRoot)) {
    throw new Error('Invalid manifest path');
  }

  const raw = await readFile(manifestPath, 'utf8');
  const manifest = JSON.parse(raw);
  const stagePath = resolve(join(openedRoot, manifest.stageFileName));

  if (!stagePath.startsWith(openedRoot)) {
    throw new Error('Invalid stage path');
  }

  return {
    originalFileName: manifest.originalFileName,
    originalFilePath: manifest.originalFilePath,
    stagePath,
  };
}

async function resolveRequestTarget(rootDir, pathname) {
  const decodedPath = decodeURIComponent(pathname || '/');
  const relativePath = decodedPath === '/' ? 'index.html' : normalize(decodedPath).replace(/^([/\\])+/, '');
  const rootPath = resolve(rootDir);
  const candidatePath = resolve(join(rootPath, relativePath));

  if (!candidatePath.startsWith(rootPath)) {
    return { kind: 'forbidden' };
  }

  if (await pathExists(candidatePath)) {
    const candidateStat = await stat(candidatePath);
    if (candidateStat.isFile()) {
      return { kind: 'file', filePath: candidatePath };
    }
  }

  if (extname(relativePath)) {
    return { kind: 'missing' };
  }

  return { kind: 'file', filePath: join(rootPath, 'index.html') };
}

export function createPreviewServer({ rootDir }) {
  return http.createServer(async (request, response) => {
    const requestUrl = new URL(request.url ?? '/', 'http://127.0.0.1');

    if (requestUrl.pathname === '/__rhwp_health') {
      response.writeHead(200, { 'content-type': 'application/json; charset=utf-8' });
      response.end(JSON.stringify({ ok: true, saveBridge: true }));
      return;
    }

    if (requestUrl.pathname.startsWith('/__rhwp_save/')) {
      if (request.method !== 'PUT') {
        response.writeHead(405, { 'content-type': 'application/json; charset=utf-8' });
        response.end(JSON.stringify({ ok: false, error: 'Method not allowed' }));
        return;
      }

      try {
        const token = requestUrl.pathname.split('/').pop() ?? '';
        const body = await readRequestBody(request);
        const manifest = await readLaunchManifest(rootDir, token);

        await writeFile(manifest.originalFilePath, body);
        await writeFile(manifest.stagePath, body);

        response.writeHead(200, { 'content-type': 'application/json; charset=utf-8' });
        response.end(JSON.stringify({
          ok: true,
          fileName: manifest.originalFileName,
        }));
      } catch (error) {
        response.writeHead(500, { 'content-type': 'application/json; charset=utf-8' });
        response.end(JSON.stringify({
          ok: false,
          error: error instanceof Error ? error.message : String(error),
        }));
      }
      return;
    }

    let target;
    try {
      target = await resolveRequestTarget(rootDir, requestUrl.pathname);
    } catch (error) {
      response.writeHead(400, { 'content-type': 'text/plain; charset=utf-8' });
      response.end(`Bad request: ${error instanceof Error ? error.message : String(error)}`);
      return;
    }

    if (target.kind === 'forbidden') {
      response.writeHead(403, { 'content-type': 'text/plain; charset=utf-8' });
      response.end('Forbidden');
      return;
    }

    if (target.kind === 'missing') {
      response.writeHead(404, { 'content-type': 'text/plain; charset=utf-8' });
      response.end('Not found');
      return;
    }

    response.writeHead(200, { 'content-type': getContentType(target.filePath) });
    createReadStream(target.filePath).pipe(response);
  });
}

const currentFilePath = fileURLToPath(import.meta.url);

if (process.argv[1] === currentFilePath) {
  const { host, port } = parseCliOptions(process.argv.slice(2));
  const rootDir = resolve(join(currentFilePath, '..', '..', 'dist'));
  const server = createPreviewServer({ rootDir });

  server.listen(port, host, () => {
    console.log(`rhwp preview server listening on http://${host}:${port}`);
  });
}

export { getContentType, parseCliOptions, resolveRequestTarget };
