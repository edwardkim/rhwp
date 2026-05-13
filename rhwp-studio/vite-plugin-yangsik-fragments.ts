/**
 * vite plugin — `/api/personal-templates/yangsik-fragments/*` endpoint
 *
 * dev server 와 preview server 양쪽에 동일한 미들웨어를 등록해 양식 부품 데이터를 정적 제공한다.
 * 데이터 디렉터리는 `YANGSIK_FRAGMENTS_DIR` 환경변수로 명시한 경우에만 사용한다.
 */

import { existsSync, readFileSync, statSync } from 'fs';
import { resolve } from 'path';
import type { Plugin, Connect } from 'vite';

const FRAGMENTS_DIR = process.env.YANGSIK_FRAGMENTS_DIR
  ? resolve(process.env.YANGSIK_FRAGMENTS_DIR)
  : '';

const ROUTE_TAIL = '/api/personal-templates/yangsik-fragments';
const MANIFEST_TAIL = '/api/personal-templates/yangsik-fragments/manifest';

function sendJson(res: any, status: number, payload: unknown): void {
  const body = JSON.stringify(payload);
  res.statusCode = status;
  res.setHeader('Content-Type', 'application/json; charset=utf-8');
  res.setHeader('Content-Length', Buffer.byteLength(body));
  res.end(body);
}

function sendStatus(res: any, status: number, message: string): void {
  res.statusCode = status;
  res.setHeader('Content-Type', 'text/plain; charset=utf-8');
  res.end(message);
}

function safeFragmentPath(fragmentFile: string): string | null {
  if (!FRAGMENTS_DIR) {
    return null;
  }
  // `..`, 절대 경로, slash 차단
  if (!fragmentFile || fragmentFile.includes('/') || fragmentFile.includes('..')) {
    return null;
  }
  if (!fragmentFile.endsWith('.xml')) {
    return null;
  }
  const candidate = resolve(FRAGMENTS_DIR, fragmentFile);
  // resolve 결과가 FRAGMENTS_DIR 밖으로 나가면 거부
  const root = resolve(FRAGMENTS_DIR) + '/';
  if (!candidate.startsWith(root)) {
    return null;
  }
  try {
    if (!statSync(candidate).isFile()) return null;
  } catch {
    return null;
  }
  return candidate;
}

/**
 * vite base 가 `/rhwp/` 등으로 설정되면 들어오는 path 도 prefix 를 가진다.
 * ROUTE_TAIL 이 워낙 unique 하므로 첫 출현 위치부터 잘라 매칭한다.
 */
function stripBase(path: string): string {
  const idx = path.indexOf(ROUTE_TAIL);
  if (idx >= 0) {
    return path.slice(idx);
  }
  return path;
}

const handler: Connect.NextHandleFunction = (req, res, next) => {
  const rawUrl = req.url ?? '';
  const url = new URL(rawUrl, 'http://localhost');
  const path = stripBase(url.pathname);

  if (path === MANIFEST_TAIL) {
    if (!FRAGMENTS_DIR) {
      return sendJson(res, 200, { fragments: [] });
    }
    const manifestPath = resolve(FRAGMENTS_DIR, 'manifest.json');
    if (!existsSync(manifestPath)) {
      return sendJson(res, 200, { fragments: [] });
    }
    try {
      const raw = readFileSync(manifestPath, 'utf-8');
      const data = JSON.parse(raw);
      return sendJson(res, 200, data);
    } catch {
      return sendJson(res, 200, { fragments: [] });
    }
  }

  if (path.startsWith(`${ROUTE_TAIL}/`)) {
    const tail = decodeURIComponent(path.slice(`${ROUTE_TAIL}/`.length));
    const resolved = safeFragmentPath(tail);
    if (!resolved) {
      return sendStatus(res, 404, 'fragment not found');
    }
    try {
      const data = readFileSync(resolved);
      res.statusCode = 200;
      res.setHeader('Content-Type', 'application/xml; charset=utf-8');
      res.setHeader('Content-Length', String(data.byteLength));
      return res.end(data);
    } catch {
      return sendStatus(res, 500, 'fragment read failed');
    }
  }

  next();
};

export function yangsikFragmentsPlugin(): Plugin {
  return {
    name: 'yangsik-fragments',
    configureServer(server) {
      server.middlewares.use(handler);
    },
    configurePreviewServer(server) {
      server.middlewares.use(handler);
    },
  };
}
