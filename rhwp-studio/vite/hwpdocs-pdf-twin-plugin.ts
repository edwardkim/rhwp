import { spawn } from 'node:child_process';
import { createHash, randomBytes, timingSafeEqual } from 'node:crypto';
import {
  appendFileSync, closeSync, createReadStream, createWriteStream, existsSync, mkdirSync,
  openSync, readFileSync, readSync, renameSync, rmSync, statSync, truncateSync, unlinkSync,
  writeFileSync,
} from 'node:fs';
import { opendir } from 'node:fs/promises';
import type { IncomingMessage, ServerResponse } from 'node:http';
import { homedir } from 'node:os';
import { basename, delimiter, dirname, extname, join, relative, resolve } from 'node:path';
import { Transform } from 'node:stream';
import { pipeline } from 'node:stream/promises';
import { styleText } from 'node:util';
import { fileURLToPath } from 'node:url';
import type { Logger, Plugin } from 'vite';
import { formatDocumentErrorForTerminal } from '../src/dev/document-error-log.ts';
import { boundedPageRasterSize } from '../src/dev/page-raster-budget.ts';
import {
  DOCUMENT_ERROR_LOG_PATH,
  PDF_PAGE_PATH_PREFIX,
  PDF_TWIN_LOOKUP_PATH,
  type PdfTwinFound,
  type PdfTwinLookupResponse,
} from '../src/dev/pdf-twin-contract.ts';

const MAX_BODY_BYTES = 32_768;
const DEFAULT_RASTER_WIDTH = 2_048;
const COMMAND_TIMEOUT_MS = 30_000;
const CACHE_PARENT = resolve(dirname(fileURLToPath(import.meta.url)), '../../target/rhwp-pdf-reference-cache');
const CACHE_OWNER_RECORD_BYTES = 24;
const CACHE_LOCK_WAIT = new Int32Array(new SharedArrayBuffer(4));
export const cacheOwnerName = (pid: number, nonce: string): string =>
  `${String(pid).padStart(10, '0')}-${nonce}`;
const CACHE_OWNER = cacheOwnerName(process.pid, randomBytes(6).toString('hex'));
const CACHE = join(CACHE_PARENT, CACHE_OWNER);
const CACHE_OWNER_LOG = '.owners';
const CACHE_RECLAIM_CURSOR = '.reclaim-cursor';
export const MAX_SOURCE_BYTES = 512 * 1024 * 1024;

function processIsAlive(pid: number): boolean {
  try { process.kill(pid, 0); return true; }
  catch (error) { return (error as NodeJS.ErrnoException).code !== 'ESRCH'; }
}

export function reclaimDeadCacheRoots(
  parent: string,
  isAlive: (pid: number) => boolean = processIsAlive,
): void {
  const log = join(parent, CACHE_OWNER_LOG);
  if (!existsSync(log)) return;
  const size = statSync(log).size;
  const cursorPath = join(parent, CACHE_RECLAIM_CURSOR);
  let cursor = Number(existsSync(cursorPath) ? readFileSync(cursorPath, 'utf8') : 0);
  if (!Number.isSafeInteger(cursor) || cursor < 0 || cursor >= size) cursor = 0;
  cursor -= cursor % CACHE_OWNER_RECORD_BYTES;
  const file = openSync(log, 'r');
  const buffer = Buffer.alloc(CACHE_OWNER_RECORD_BYTES * 128);
  let bytes = 0;
  try { bytes = readSync(file, buffer, 0, buffer.length, cursor); }
  finally { closeSync(file); }
  const consumed = bytes - (bytes % CACHE_OWNER_RECORD_BYTES);
  for (let offset = 0; offset < consumed; offset += CACHE_OWNER_RECORD_BYTES) {
    const owner = buffer.subarray(offset, offset + CACHE_OWNER_RECORD_BYTES - 1).toString('utf8');
    const match = /^(\d{10})-[a-f0-9]{12}$/.exec(owner);
    if (!match || isAlive(Number(match[1]))) continue;
    try { rmSync(join(parent, owner), { recursive: true, force: true }); } catch {}
  }
  writeFileSync(cursorPath, String(cursor + consumed >= size ? 0 : cursor + consumed));
}

export function registerCacheRoot(parent: string, owner: string): void {
  if (!/^\d{10}-[a-f0-9]{12}$/.test(owner)) throw new Error('invalid cache owner');
  mkdirSync(parent, { recursive: true });
  const lock = join(parent, '.owners.lock');
  const claim = `${lock}.${process.pid}-${randomBytes(6).toString('hex')}`;
  const deadline = Date.now() + 5_000;
  while (true) {
    mkdirSync(claim);
    writeFileSync(join(claim, 'pid'), String(process.pid));
    try { renameSync(claim, lock); break; }
    catch (error) {
      rmSync(claim, { recursive: true, force: true });
      if ((error as NodeJS.ErrnoException).code !== 'EEXIST'
        && (error as NodeJS.ErrnoException).code !== 'ENOTEMPTY') throw error;
      let holder: number;
      try { holder = Number(readFileSync(join(lock, 'pid'), 'utf8')); }
      catch (readError) {
        if ((readError as NodeJS.ErrnoException).code === 'ENOENT') continue;
        throw readError;
      }
      if (Number.isSafeInteger(holder) && !processIsAlive(holder)) {
        rmSync(lock, { recursive: true, force: true });
        continue;
      }
      if (Date.now() >= deadline) throw new Error('cache owner log lock timed out');
      Atomics.wait(CACHE_LOCK_WAIT, 0, 0, 10);
    }
  }
  try {
    const log = join(parent, CACHE_OWNER_LOG);
    if (existsSync(log)) {
      const size = statSync(log).size;
      if (size % CACHE_OWNER_RECORD_BYTES) {
        truncateSync(log, size - (size % CACHE_OWNER_RECORD_BYTES));
      }
    }
    appendFileSync(log, `${owner}\n`);
  } finally {
    rmSync(lock, { recursive: true, force: true });
  }
}

export interface PdfTwinLookupRequest {
  fileName: string;
  size: number;
  sha256: string;
}

type Candidate = { documentPath: string; pdfPath: string };
export type HwpdocsPdfTwinIndex = { root: string; candidates: Map<string, Candidate[]> };
type Match = { index: HwpdocsPdfTwinIndex; pdfPath: string };

const key = (name: string, size: number): string => `${name.normalize('NFC')}\0${size}`;

function admittedStat(path: string) {
  const stat = statSync(path);
  if (!stat.isFile() || stat.size > MAX_SOURCE_BYTES) {
    throw new SourceAdmissionError(`source exceeds ${MAX_SOURCE_BYTES} bytes: ${path}`);
  }
  return stat;
}

const sameFileGeneration = (left: ReturnType<typeof statSync>, right: ReturnType<typeof statSync>) =>
  left.dev === right.dev && left.ino === right.ino && left.size === right.size
  && left.mtimeMs === right.mtimeMs && left.ctimeMs === right.ctimeMs;

async function sha256(path: string): Promise<string> {
  admittedStat(path);
  return sourceQueue.run(async () => {
    const before = admittedStat(path);
    const hash = createHash('sha256');
    let length = 0;
    for await (const chunk of createReadStream(path)) {
      const bytes = Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk);
      if ((length += bytes.length) > MAX_SOURCE_BYTES) throw new SourceAdmissionError('source grew while hashing');
      hash.update(bytes);
    }
    if (length !== before.size || !sameFileGeneration(before, admittedStat(path))) {
      throw new SourceAdmissionError('source changed while hashing');
    }
    return hash.digest('hex');
  });
}

async function filesUnder(root: string): Promise<string[]> {
  const files: string[] = [];
  let entries = 0;
  const visit = async (directory: string, depth: number): Promise<void> => {
    if (depth > 64) throw new Error(`PDF twin root exceeds 64 levels: ${root}`);
    for await (const entry of await opendir(directory)) {
      if (++entries > 100_000) throw new Error(`PDF twin root exceeds 100000 entries: ${root}`);
      const path = join(directory, entry.name);
      if (entry.isDirectory() && !entry.name.startsWith('.')
        && !['node_modules', 'target', 'dist', 'build'].includes(entry.name)) {
        await visit(path, depth + 1);
      }
      else if (entry.isFile() && /\.(hwp|hwpx|pdf)$/i.test(entry.name)) files.push(path);
    }
  };
  await visit(root, 0);
  return files;
}

export async function buildHwpdocsPdfTwinIndex(root: string): Promise<HwpdocsPdfTwinIndex> {
  const pairs = new Map<string, { documents: string[]; pdfs: string[] }>();
  for (const path of await filesUnder(root)) {
    const extension = extname(path).toLowerCase();
    const pairKey = `${dirname(path).normalize('NFC')}\0${basename(path, extension).normalize('NFC')}`;
    const pair = pairs.get(pairKey) ?? { documents: [], pdfs: [] };
    (extension === '.pdf' ? pair.pdfs : pair.documents).push(path);
    pairs.set(pairKey, pair);
  }

  const candidates = new Map<string, Candidate[]>();
  for (const { documents, pdfs } of pairs.values()) {
    if (pdfs.length !== 1) continue;
    for (const documentPath of documents) {
      const candidateKey = key(basename(documentPath), statSync(documentPath).size);
      const values = candidates.get(candidateKey) ?? [];
      values.push({ documentPath, pdfPath: pdfs[0] });
      candidates.set(candidateKey, values);
    }
  }
  return { root, candidates };
}

export async function findPdfTwin(
  indexes: readonly HwpdocsPdfTwinIndex[],
  request: PdfTwinLookupRequest,
): Promise<Match | Exclude<PdfTwinLookupResponse, PdfTwinFound>> {
  const matches: Match[] = [];
  for (const index of indexes) {
    for (const candidate of index.candidates.get(key(request.fileName, request.size)) ?? []) {
      try {
        if (statSync(candidate.documentPath).size === request.size
          && await sha256(candidate.documentPath) === request.sha256) {
          matches.push({ index, pdfPath: candidate.pdfPath });
        }
      } catch (error) {
        if (error instanceof SourceAdmissionError || error instanceof WorkQueueBusyError) throw error;
        // A corpus file changed after indexing; the forced rebuild on `none` retries it.
      }
    }
  }
  if (matches.length === 0) return { status: 'none' };
  return matches.length === 1 ? matches[0] : { status: 'ambiguous' };
}

export async function snapshotPdf(
  pdfPath: string,
  cache = CACHE,
): Promise<{ token: string; path: string; release: () => void }> {
  admittedStat(pdfPath);
  const owned = resolve(cache) === CACHE;
  const work = () => sourceQueue.run(async () => {
    const before = admittedStat(pdfPath);
    const directory = join(cache, 'source');
    const temporary = join(directory, `.${process.pid}-${Date.now()}-${randomBytes(6).toString('hex')}`);
    const hash = createHash('sha256');
    let length = 0;
    mkdirSync(directory, { recursive: true });
    try {
      const tap = new Transform({
        transform(chunk: Buffer, _encoding, callback) {
          if ((length += chunk.length) > MAX_SOURCE_BYTES) {
            return callback(new SourceAdmissionError('PDF grew while snapshotting'));
          }
          hash.update(chunk);
          callback(null, chunk);
        },
      });
      await pipeline(createReadStream(pdfPath), tap, createWriteStream(temporary, { flags: 'wx' }));
      if (length !== before.size || !sameFileGeneration(before, admittedStat(pdfPath))) {
        throw new SourceAdmissionError('PDF changed while snapshotting');
      }
      const token = hash.digest('base64url').slice(0, 24);
      const path = join(directory, `${token}.pdf`);
      if (existsSync(path)) unlinkSync(temporary);
      else renameSync(temporary, path);
      if (owned) referenceCache.track(path, token, true);
      return { token, path, release: owned ? referenceCache.acquire(token, path) : () => {} };
    } finally {
      if (existsSync(temporary)) unlinkSync(temporary);
    }
  });
  return owned ? referenceCache.withOperation(work) : work();
}

export class WorkQueueBusyError extends Error {}

export class SourceAdmissionError extends Error {}

export class BoundedWorkQueue {
  private running = 0;
  private readonly waiting: Array<() => void> = [];
  private readonly maxRunning: number;
  private readonly maxWaiting: number;

  constructor(maxRunning: number, maxWaiting: number) {
    this.maxRunning = maxRunning;
    this.maxWaiting = maxWaiting;
  }

  run<T>(work: () => Promise<T>): Promise<T> {
    if (this.running < this.maxRunning) return this.start(work);
    if (this.waiting.length >= this.maxWaiting) return Promise.reject(new WorkQueueBusyError());
    return new Promise<T>((resolve, reject) => {
      this.waiting.push(() => { void this.start(work).then(resolve, reject); });
    });
  }

  private async start<T>(work: () => Promise<T>): Promise<T> {
    this.running += 1;
    try { return await work(); }
    finally {
      this.running -= 1;
      this.waiting.shift()?.();
    }
  }
}

const sourceQueue = new BoundedWorkQueue(2, 8);

type CacheEntry = { token: string; source: boolean; size: number; touchedAt: number };

export class ReferenceCacheOwner {
  private readonly entries = new Map<string, CacheEntry>();
  private readonly recentTokens: string[] = [];
  private readonly tokenLeases = new Map<string, number>();
  private readonly pathLeases = new Map<string, number>();
  private operations = 0;
  private servers = 0;
  private resetPending = false;
  readonly root: string;
  private readonly maxEntries: number;
  private readonly maxBytes: number;
  private readonly maxAgeMs: number;
  private readonly recentLimit: number;

  constructor(
    root: string,
    maxEntries = 256,
    maxBytes = 2 * 1024 * 1024 * 1024,
    maxAgeMs = 24 * 60 * 60 * 1_000,
    recentLimit = 4,
  ) {
    this.root = root;
    this.maxEntries = maxEntries;
    this.maxBytes = maxBytes;
    this.maxAgeMs = maxAgeMs;
    this.recentLimit = recentLimit;
  }

  reset(): void {
    this.resetPending = true;
    this.maybeReset();
  }

  attachServer(): () => void {
    if (this.servers === 0 && this.operations === 0 && this.tokenLeases.size === 0) {
      this.resetNow();
    }
    this.resetPending = false;
    this.servers += 1;
    let attached = true;
    return () => {
      if (!attached) return;
      attached = false;
      this.servers -= 1;
      if (this.servers === 0) this.reset();
    };
  }

  async withOperation<T>(work: () => Promise<T>): Promise<T> {
    this.operations += 1;
    try { return await work(); }
    finally {
      this.operations -= 1;
      this.maybeReset();
    }
  }

  private resetNow(): void {
    rmSync(this.root, { recursive: true, force: true });
    this.entries.clear();
    this.recentTokens.length = 0;
    this.resetPending = false;
  }

  private maybeReset(): void {
    if (this.resetPending && this.servers === 0 && this.operations === 0
      && this.tokenLeases.size === 0 && this.pathLeases.size === 0) this.resetNow();
  }

  touchToken(token: string): void {
    const existing = this.recentTokens.indexOf(token);
    if (existing >= 0) this.recentTokens.splice(existing, 1);
    this.recentTokens.push(token);
    if (this.recentTokens.length > this.recentLimit) this.recentTokens.shift();
  }

  track(path: string, token: string, source: boolean): void {
    if (!existsSync(path)) return;
    if (source) this.touchToken(token);
    this.entries.delete(path);
    this.entries.set(path, { token, source, size: statSync(path).size, touchedAt: Date.now() });
    this.prune();
  }

  acquire(token: string, path: string): () => void {
    this.tokenLeases.set(token, (this.tokenLeases.get(token) ?? 0) + 1);
    this.pathLeases.set(path, (this.pathLeases.get(path) ?? 0) + 1);
    let active = true;
    return () => {
      if (!active) return;
      active = false;
      const remaining = (this.tokenLeases.get(token) ?? 1) - 1;
      if (remaining > 0) this.tokenLeases.set(token, remaining);
      else this.tokenLeases.delete(token);
      const pathRemaining = (this.pathLeases.get(path) ?? 1) - 1;
      if (pathRemaining > 0) this.pathLeases.set(path, pathRemaining);
      else this.pathLeases.delete(path);
      this.prune();
      this.maybeReset();
    };
  }

  async withLease<T>(token: string, path: string, work: () => Promise<T>): Promise<T> {
    const release = this.acquire(token, path);
    try { return await work(); }
    finally { release(); }
  }

  private prune(now = Date.now()): void {
    let bytes = Array.from(this.entries.values()).reduce((sum, entry) => sum + entry.size, 0);
    for (const [path, entry] of this.entries) {
      const over = this.entries.size > this.maxEntries || bytes > this.maxBytes;
      if (!over && now - entry.touchedAt <= this.maxAgeMs) continue;
      if ((entry.source && (this.recentTokens.includes(entry.token)
        || this.tokenLeases.has(entry.token))) || this.pathLeases.has(path)) continue;
      try { unlinkSync(path); } catch { continue; }
      this.entries.delete(path);
      bytes -= entry.size;
    }
  }
}

const referenceCache = new ReferenceCacheOwner(CACHE);
let cacheRootRegistered = false;

function runCommand(command: string, args: string[]): Promise<string> {
  return new Promise((resolveCommand, rejectCommand) => {
    const child = spawn(command, args, { stdio: ['ignore', 'pipe', 'pipe'] });
    let stdout = '';
    let stderr = '';
    const timer = setTimeout(() => child.kill('SIGKILL'), COMMAND_TIMEOUT_MS);
    child.stdout.setEncoding('utf8');
    child.stderr.setEncoding('utf8');
    child.stdout.on('data', chunk => { if (stdout.length < 32_768) stdout += chunk; });
    child.stderr.on('data', chunk => { if (stderr.length < 8_192) stderr += chunk; });
    child.once('error', (error) => {
      clearTimeout(timer);
      rejectCommand(error);
    });
    child.once('close', (code) => {
      clearTimeout(timer);
      if (code === 0) resolveCommand(stdout);
      else rejectCommand(new Error(`${command} exited ${code}: ${stderr.trim()}`));
    });
  });
}

const commandQueue = new BoundedWorkQueue(2, 8);
const run = (command: string, args: string[]): Promise<string> =>
  commandQueue.run(() => runCommand(command, args));

const pageCounts = new Map<string, number>();
const pageSizes = new Map<string, { width: number; height: number }>();

function remember<K, V>(map: Map<K, V>, key: K, value: V, limit: number): void {
  map.delete(key);
  map.set(key, value);
  while (map.size > limit) map.delete(map.keys().next().value!);
}

async function pdfPageCount(path: string, token: string): Promise<number> {
  const cached = pageCounts.get(token);
  if (cached !== undefined) return cached;
  const match = /^Pages:\s+(\d+)$/m.exec(await run('pdfinfo', [path]));
  const count = Number(match?.[1]);
  if (!Number.isSafeInteger(count) || count <= 0) throw new Error('pdfinfo did not report page count');
  remember(pageCounts, token, count, 64);
  return count;
}

export function parsePdfPageSize(output: string, page: number): { width: number; height: number } {
  const number = '[-+]?(?:\\d+(?:\\.\\d*)?|\\.\\d+)';
  const media = new RegExp(
    `Page\\s+${page}\\s+MediaBox:\\s+(${number})\\s+(${number})\\s+(${number})\\s+(${number})`,
  ).exec(output);
  const size = new RegExp(`Page\\s+${page}\\s+size:\\s+(${number})\\s+x\\s+(${number})`).exec(output);
  let width = media ? Number(media[3]) - Number(media[1]) : Number(size?.[1]);
  let height = media ? Number(media[4]) - Number(media[2]) : Number(size?.[2]);
  const rawRotation = Number(new RegExp(`Page\\s+${page}\\s+rot:\\s+(-?\\d+)`).exec(output)?.[1] ?? 0);
  const rotation = ((rawRotation % 360) + 360) % 360;
  if (![0, 90, 180, 270].includes(rotation)) throw new Error(`invalid page ${page} rotation`);
  if (rotation === 90 || rotation === 270) [width, height] = [height, width];
  if (!Number.isFinite(width) || !Number.isFinite(height) || !(width > 0) || !(height > 0)) {
    throw new Error(`pdfinfo did not report page ${page} size`);
  }
  return { width, height };
}

async function pdfPageSize(path: string, token: string, pageIndex: number): Promise<{ width: number; height: number }> {
  const cacheKey = `${token}:${pageIndex}`;
  const cached = pageSizes.get(cacheKey);
  if (cached) return cached;
  const page = pageIndex + 1;
  const size = parsePdfPageSize(await run('pdfinfo', ['-box', '-f', String(page), '-l', String(page), path]), page);
  remember(pageSizes, cacheKey, size, 512);
  return size;
}

export function ghostscriptRasterArgs(
  path: string,
  page: number,
  size: { width: number; height: number },
  output: string,
): string[] {
  return [
    '-q', '-dSAFER', '-dBATCH', '-dNOPAUSE', '-dPDFSTOPONERROR', '-dPDFFitPage',
    '-dTextAlphaBits=4', '-dGraphicsAlphaBits=4', '-sDEVICE=png16m',
    `-dFirstPage=${page}`, `-dLastPage=${page}`, `-g${size.width}x${size.height}`,
    '-r72', `-sOutputFile=${output}`, path,
  ];
}

const rasterInFlight = new Map<string, Promise<string>>();

async function rasterize(token: string, pageIndex: number, width: number): Promise<string> {
  const pdfPath = join(CACHE, 'source', `${token}.pdf`);
  if (!/^[A-Za-z0-9_-]{24}$/.test(token) || !existsSync(pdfPath)) throw new Error('unknown PDF token');
  const directory = join(CACHE, 'pages', token);
  const output = join(directory, `${pageIndex}-${width}.png`);
  referenceCache.touchToken(token);
  if (existsSync(output)) {
    referenceCache.track(output, token, false);
    return output;
  }
  const requestKey = `${token}:${pageIndex}:${width}`;
  const existing = rasterInFlight.get(requestKey);
  if (existing) return existing;
  const pending = (async () => {
    const size = boundedPageRasterSize(await pdfPageSize(pdfPath, token, pageIndex), width, 'PDF page');
    mkdirSync(directory, { recursive: true });
    const temporary = `${output}.${process.pid}.${Date.now()}.tmp`;
    try {
      await run('gs', ghostscriptRasterArgs(pdfPath, pageIndex + 1, size, temporary));
      renameSync(temporary, output);
      referenceCache.track(output, token, false);
      return output;
    } finally {
      if (existsSync(temporary)) unlinkSync(temporary);
    }
  })().finally(() => rasterInFlight.delete(requestKey));
  rasterInFlight.set(requestKey, pending);
  return pending;
}

async function withRaster<T>(
  token: string,
  pageIndex: number,
  width: number,
  consume: (path: string) => T | Promise<T>,
): Promise<T> {
  const output = join(CACHE, 'pages', token, `${pageIndex}-${width}.png`);
  return referenceCache.withLease(token, output, async () => consume(
    await rasterize(token, pageIndex, width),
  ));
}

function json(res: ServerResponse, status: number, value: unknown): void {
  res.statusCode = status;
  res.setHeader('Content-Type', 'application/json; charset=utf-8');
  res.setHeader('Cache-Control', 'no-store');
  res.end(JSON.stringify(value));
}

async function body(req: IncomingMessage): Promise<string> {
  const chunks: Buffer[] = [];
  let length = 0;
  for await (const chunk of req) {
    const bytes = Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk);
    if ((length += bytes.length) > MAX_BODY_BYTES) throw new Error('request body too large');
    chunks.push(bytes);
  }
  return Buffer.concat(chunks).toString('utf8');
}

function lookupRequest(value: unknown): value is PdfTwinLookupRequest {
  const request = value as Partial<PdfTwinLookupRequest> | null;
  return !!request
    && typeof request.fileName === 'string' && request.fileName.length <= 512
    && Number.isSafeInteger(request.size) && (request.size ?? -1) >= 0
    && typeof request.sha256 === 'string' && /^[a-f0-9]{64}$/.test(request.sha256);
}

function hasCapability(req: IncomingMessage, expected: string): boolean {
  const provided = req.headers['x-rhwp-harness-capability'];
  if (typeof provided !== 'string') return false;
  const left = Buffer.from(expected);
  const right = Buffer.from(provided);
  return left.length === right.length && timingSafeEqual(left, right);
}

export async function serveDocumentErrorLog(
  req: IncomingMessage,
  res: ServerResponse,
  logger: Pick<Logger, 'error'>,
  capability: string,
): Promise<void> {
  if (req.method !== 'POST') return json(res, 405, { status: 'error' });
  if (!hasCapability(req, capability)) return json(res, 403, { status: 'error' });
  try {
    const line = await body(req);
    const display = formatDocumentErrorForTerminal(line);
    if (!display) return json(res, 400, { status: 'error' });
    logger.error(styleText('red', display, { stream: process.stderr }), {
      timestamp: true,
      error: null,
    });
    json(res, 202, { status: 'accepted' });
  } catch {
    json(res, 400, { status: 'error' });
  }
}

function logFailure(logger: Logger, label: string, error: unknown): void {
  logger.error(`[pdf-reference] ${label}: ${error instanceof Error ? error.message : String(error)}`, {
    timestamp: true,
    error: error instanceof Error ? error : null,
  });
}

export function rejectOperationalFailure(res: ServerResponse, error: unknown): boolean {
  if (error instanceof WorkQueueBusyError) {
    res.setHeader('Retry-After', '1');
    json(res, 503, { status: 'busy' });
    return true;
  }
  if (error instanceof SourceAdmissionError) {
    json(res, 413, { status: 'error' });
    return true;
  }
  return false;
}

export function hwpdocsPdfTwinPlugin(options: { root?: string; additionalRoots?: string[] } = {}): Plugin {
  const errorLogCapability = randomBytes(32).toString('base64url');
  const configured = process.env.RHWP_PDF_TWIN_ROOTS?.split(delimiter).filter(Boolean);
  const roots = Array.from(new Set([
    options.root ?? process.env.RHWP_HWP_DOCS_ROOT ?? join(homedir(), 'hwpdocs_10k'),
    ...(options.additionalRoots ?? configured ?? [join(homedir(), 'Downloads')]),
  ].map(root => resolve(root))));
  let indexes: Promise<HwpdocsPdfTwinIndex[]> | null = null;
  const getIndexes = (refresh = false): Promise<HwpdocsPdfTwinIndex[]> => {
    if (indexes && !refresh) return indexes;
    indexes = Promise.all(roots.map(async (root) => {
      try { return existsSync(root) && statSync(root).isDirectory() ? await buildHwpdocsPdfTwinIndex(root) : null; }
      catch { return null; }
    })).then(values => values.filter((value): value is HwpdocsPdfTwinIndex => value !== null));
    return indexes;
  };

  return {
    name: 'hwpdocs-pdf-twin-harness',
    apply: 'serve',
    configureServer(server) {
      if (!cacheRootRegistered) {
        registerCacheRoot(CACHE_PARENT, CACHE_OWNER);
        cacheRootRegistered = true;
      }
      reclaimDeadCacheRoots(CACHE_PARENT);
      const detachCache = referenceCache.attachServer();
      const closeCache = () => {
        detachCache();
        pageCounts.clear();
        pageSizes.clear();
      };
      pageCounts.clear();
      pageSizes.clear();
      server.httpServer?.once('close', closeCache);
      server.middlewares.use(async (req, res, next) => {
        const url = new URL(req.url ?? '/', 'http://localhost');
        if (url.pathname === DOCUMENT_ERROR_LOG_PATH) {
          return serveDocumentErrorLog(req, res, server.config.logger, errorLogCapability);
        }
        if (url.pathname === PDF_TWIN_LOOKUP_PATH) {
          if (req.method !== 'POST') return json(res, 405, { status: 'error' });
          try {
            const request = JSON.parse(await body(req));
            if (!lookupRequest(request)) return json(res, 400, { status: 'error' });
            let match = await findPdfTwin(await getIndexes(), request);
            if ('status' in match && match.status === 'none') {
              match = await findPdfTwin(await getIndexes(true), request);
            }
            if ('status' in match) return json(res, 200, match);
            const snapshot = await snapshotPdf(match.pdfPath);
            try {
              const pageCount = await pdfPageCount(snapshot.path, snapshot.token);
              await withRaster(snapshot.token, 0, DEFAULT_RASTER_WIDTH, () => undefined);
              const result: PdfTwinFound = {
                status: 'found',
                pdfName: basename(match.pdfPath),
                pdfPageUrl: `${PDF_PAGE_PATH_PREFIX}${snapshot.token}`,
                pdfPageWidth: DEFAULT_RASTER_WIDTH,
                pdfPageCount: pageCount,
                relativeDirectory: relative(match.index.root, dirname(match.pdfPath)),
                errorLogCapability,
              };
              return json(res, 200, result);
            } finally {
              snapshot.release();
            }
          } catch (error) {
            if (rejectOperationalFailure(res, error)) return;
            logFailure(server.config.logger, 'PDF twin lookup failed', error);
            return json(res, 500, { status: 'error' });
          }
        }
        if (url.pathname.startsWith(PDF_PAGE_PATH_PREFIX)) {
          if (req.method !== 'GET') return json(res, 405, { status: 'error' });
          const match = /^\/__rhwp_harness\/pdf-page\/([A-Za-z0-9_-]{24})\/(\d+)\.png$/.exec(url.pathname);
          const pageIndex = Number(match?.[2]);
          const width = Number(url.searchParams.get('width'));
          if (!match || !Number.isSafeInteger(pageIndex) || pageIndex < 0) return json(res, 400, { status: 'error' });
          try {
            return await withRaster(match[1], pageIndex, width, (path) => {
              const png = readFileSync(path);
              res.statusCode = 200;
              res.setHeader('Content-Type', 'image/png');
              res.setHeader('Cache-Control', 'private, max-age=31536000, immutable');
              res.end(png);
            });
          } catch (error) {
            if (rejectOperationalFailure(res, error)) return;
            logFailure(server.config.logger, 'Ghostscript raster failed', error);
            return json(res, 500, { status: 'error' });
          }
        }
        next();
      });
    },
  };
}
