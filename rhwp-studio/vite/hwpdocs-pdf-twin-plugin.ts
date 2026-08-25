import { createHash, randomBytes, timingSafeEqual } from 'node:crypto';
import { spawn } from 'node:child_process';
import {
  createWriteStream,
  existsSync,
  mkdirSync,
  opendirSync,
  readdirSync,
  renameSync,
  rmSync,
  statSync,
  unlinkSync,
  type Stats,
} from 'node:fs';
import {
  mkdir as mkdirFile,
  open as openFile,
  opendir as openDirectory,
  rename as renameFile,
  stat as statFile,
  unlink as unlinkFile,
} from 'node:fs/promises';
import { homedir } from 'node:os';
import { basename, delimiter, dirname, extname, join, relative, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';
import type { IncomingMessage, ServerResponse } from 'node:http';
import { Transform, Writable } from 'node:stream';
import { pipeline } from 'node:stream/promises';
import type { Logger, Plugin } from 'vite';
import { isDocumentErrorLine } from '../src/dev/document-error-log.ts';
import {
  DOCUMENT_ERROR_CAPABILITY_HEADER,
  DOCUMENT_ERROR_LOG_PATH,
  PDF_PAGE_PATH_PREFIX,
  PDF_TWIN_LOOKUP_PATH,
  type PdfTwinLookupResponse,
} from '../src/dev/pdf-twin-contract.ts';
import {
  boundedPageRasterSize,
  PAGE_RASTER_BUDGET,
} from '../src/dev/page-raster-budget.ts';

const MAX_LOOKUP_BODY_BYTES = 1_024;
const MAX_DIFF_BODY_BYTES = 32_768;
const COMMAND_TIMEOUT_MS = 30_000;
const COMMAND_KILL_GRACE_MS = 1_000;
const MAX_PDF_PROCESSES = 2;
const MAX_QUEUED_PDF_PROCESSES = 8;
const MAX_INDEX_ENTRIES = 100_000;
const MAX_INDEX_DEPTH = 64;
const PDF_PAGE_RASTER_ALGORITHM = 'ghostscript-media-rgb-v5';
const DEFAULT_PDF_PAGE_RASTER_WIDTH = 2_048;
const PLUGIN_DIRECTORY = dirname(fileURLToPath(import.meta.url));
const TARGET_DIRECTORY = resolve(PLUGIN_DIRECTORY, '..', '..', 'target');
const PDF_CACHE_PARENT = join(TARGET_DIRECTORY, 'rhwp-pdf-reference-cache-v2');
interface PdfCacheProcessState {
  cacheOwnerId: string;
  closedCacheRoots: Set<string>;
  cacheRootsPendingCleanup: Set<string>;
  cacheServerCounts: Map<string, number>;
  pdfSnapshotsByToken: Map<string, string>;
  recentPdfTokens: Map<string, number>;
  pdfTokenLeases: Map<string, number>;
  pdfTokenEvictions: Set<string>;
  activeStagingArtifacts: Set<string>;
  pdfProcessQueue?: BoundedAsyncWorkQueue;
  sourceIoQueue?: BoundedAsyncWorkQueue;
  sourceIoInFlight: Map<string, Promise<unknown>>;
  rasterInFlight: Map<string, Promise<string>>;
}
const PDF_CACHE_PROCESS_STATE_KEY = Symbol.for('rhwp.pdf-reference-cache.process-state.v2');
const processRealm = globalThis as typeof globalThis
  & Record<symbol, PdfCacheProcessState | undefined>;
const pdfCacheProcessState = processRealm[PDF_CACHE_PROCESS_STATE_KEY] ?? {
  cacheOwnerId: randomBytes(12).toString('hex'),
  closedCacheRoots: new Set(),
  cacheRootsPendingCleanup: new Set(),
  cacheServerCounts: new Map(),
  pdfSnapshotsByToken: new Map(),
  recentPdfTokens: new Map(),
  pdfTokenLeases: new Map(),
  pdfTokenEvictions: new Set(),
  activeStagingArtifacts: new Set(),
  sourceIoInFlight: new Map(),
  rasterInFlight: new Map(),
};
pdfCacheProcessState.cacheOwnerId ??= randomBytes(12).toString('hex');
pdfCacheProcessState.closedCacheRoots ??= new Set();
pdfCacheProcessState.cacheRootsPendingCleanup ??= new Set();
pdfCacheProcessState.cacheServerCounts ??= new Map();
pdfCacheProcessState.sourceIoInFlight ??= new Map();
processRealm[PDF_CACHE_PROCESS_STATE_KEY] = pdfCacheProcessState;
const {
  pdfSnapshotsByToken,
  recentPdfTokens,
  pdfTokenLeases,
  pdfTokenEvictions,
  activeStagingArtifacts,
  sourceIoInFlight,
  rasterInFlight,
} = pdfCacheProcessState;
export function pdfCacheProcessRoot(
  targetDirectory: string,
  processId: number,
  ownerId: string,
): string {
  return join(targetDirectory, 'rhwp-pdf-reference-cache-v2', `${processId}-${ownerId}`);
}
const PDF_CACHE_PROCESS_ROOT = pdfCacheProcessRoot(
  TARGET_DIRECTORY,
  process.pid,
  pdfCacheProcessState.cacheOwnerId,
);
const PDF_PAGE_CACHE = join(PDF_CACHE_PROCESS_ROOT, 'pages');
const PDF_SOURCE_CACHE = join(PDF_CACHE_PROCESS_ROOT, 'source');
const CACHE_MAX_AGE_MS = 7 * 24 * 60 * 60 * 1_000;
const SOURCE_CACHE_MAX_ENTRIES = 16;
const RASTER_CACHE_MAX_ENTRIES = 1_024;
const CACHE_MAX_BYTES = 2 * 1024 * 1024 * 1024;
const CACHE_INSPECTION_MAX_ENTRIES = 20_000;
export const PDF_TWIN_SOURCE_BUDGET = {
  documentBytes: 256 * 1024 * 1024,
  pdfBytes: 512 * 1024 * 1024,
} as const;

export class SourceAdmissionError extends Error {
  readonly code = 'RHWP_PDF_SOURCE_ADMISSION';
}

export function isSourceAdmissionError(error: unknown): boolean {
  return typeof error === 'object'
    && error !== null
    && 'code' in error
    && error.code === 'RHWP_PDF_SOURCE_ADMISSION';
}
export class WorkQueueSaturatedError extends Error {
  readonly code = 'RHWP_PDF_HARNESS_BUSY';
}

export function isWorkQueueSaturatedError(error: unknown): boolean {
  return typeof error === 'object'
    && error !== null
    && 'code' in error
    && error.code === 'RHWP_PDF_HARNESS_BUSY';
}

function processIsAlive(processId: number): boolean {
  try {
    process.kill(processId, 0);
    return true;
  } catch (error) {
    return (error as NodeJS.ErrnoException).code === 'EPERM';
  }
}

export function reclaimDeadPdfCacheProcesses(
  parentDirectory: string,
  currentRoot: string,
  isAlive: (processId: number) => boolean = processIsAlive,
  maximum = 256,
  remove: (root: string) => void = root => rmSync(root, { recursive: true, force: true }),
  onError: (error: unknown) => void = reportCachePruneFailure,
): number {
  let directory: ReturnType<typeof opendirSync>;
  try {
    mkdirSync(parentDirectory, { recursive: true });
    directory = opendirSync(parentDirectory);
  } catch (error) {
    onError(error);
    return 0;
  }
  let inspected = 0;
  let removed = 0;
  try {
    for (let entry = directory.readSync(); entry && inspected < maximum; entry = directory.readSync()) {
      inspected += 1;
      if (!entry.isDirectory()) continue;
      const match = /^(\d+)-[a-f0-9]{24}$/.exec(entry.name);
      if (!match) continue;
      const root = join(parentDirectory, entry.name);
      if (root === currentRoot || isAlive(Number(match[1]))) continue;
      try {
        remove(root);
        removed += 1;
      } catch (error) {
        onError(error);
      }
    }
  } finally {
    try {
      directory.closeSync();
    } catch (error) {
      onError(error);
    }
  }
  return removed;
}

export function registerPdfCacheServerLifetime(
  server: { once(event: 'close', listener: () => void): unknown } | null,
  cacheRoot = PDF_CACHE_PROCESS_ROOT,
): void {
  if (!server) return;
  pdfCacheProcessState.closedCacheRoots.delete(cacheRoot);
  pdfCacheProcessState.cacheRootsPendingCleanup.delete(cacheRoot);
  pdfCacheProcessState.cacheServerCounts.set(
    cacheRoot,
    (pdfCacheProcessState.cacheServerCounts.get(cacheRoot) ?? 0) + 1,
  );
  let released = false;
  server.once('close', () => {
    if (released) return;
    released = true;
    const remaining = (pdfCacheProcessState.cacheServerCounts.get(cacheRoot) ?? 1) - 1;
    if (remaining > 0) {
      pdfCacheProcessState.cacheServerCounts.set(cacheRoot, remaining);
      return;
    }
    pdfCacheProcessState.cacheServerCounts.delete(cacheRoot);
    pdfCacheProcessState.closedCacheRoots.add(cacheRoot);
    pdfCacheProcessState.cacheRootsPendingCleanup.add(cacheRoot);
    releaseUnusedPdfCacheProcessRoots();
  });
}

function releaseUnusedPdfCacheProcessRoots(): void {
  if (
    pdfTokenLeases.size
    || activeStagingArtifacts.size
    || sourceIoInFlight.size
    || rasterInFlight.size
  ) return;
  removeUnownedPdfCacheRoots(
    pdfCacheProcessState.cacheRootsPendingCleanup,
    pdfCacheProcessState.cacheServerCounts,
  );
}

export function removeUnownedPdfCacheRoots(
  pending: Set<string>,
  serverCounts: ReadonlyMap<string, number>,
  remove: (root: string) => void = root => rmSync(root, { recursive: true, force: true }),
  onError: (error: unknown) => void = reportCachePruneFailure,
): number {
  let removed = 0;
  for (const root of pending) {
    if ((serverCounts.get(root) ?? 0) > 0) continue;
    try {
      remove(root);
      pending.delete(root);
      removed += 1;
    } catch (error) {
      try {
        onError(error);
      } catch {
        // Cleanup reporting must not replace the primary operation either.
      }
    }
  }
  return removed;
}

export interface ReferenceCacheEntry {
  id: string;
  owner: string;
  bytes: number;
  lastAccess: number;
}

export function selectReferenceCacheEvictions(
  entries: readonly ReferenceCacheEntry[],
  protectedOwners: ReadonlySet<string>,
  policy: { maxEntries: number; maxBytes: number; maxAgeMs: number; now: number },
): string[] {
  const oldest = [...entries].sort((left, right) => left.lastAccess - right.lastAccess);
  const evicted = new Set<string>();
  let retainedEntries = entries.length;
  let retainedBytes = entries.reduce((sum, entry) => sum + entry.bytes, 0);
  for (const entry of oldest) {
    if (
      protectedOwners.has(entry.owner)
      || (
        policy.now - entry.lastAccess <= policy.maxAgeMs
        && retainedEntries <= policy.maxEntries
        && retainedBytes <= policy.maxBytes
      )
    ) continue;
    evicted.add(entry.id);
    retainedEntries -= 1;
    retainedBytes -= entry.bytes;
  }
  return Array.from(evicted);
}

function isPdfTokenProtected(token: string): boolean {
  return recentPdfTokens.has(token) || isPdfTokenLeased(token);
}

export function withPdfTokenEviction(
  token: string,
  work: () => void,
): boolean {
  if (isPdfTokenProtected(token) || pdfTokenEvictions.has(token)) return false;
  pdfTokenEvictions.add(token);
  try {
    if (isPdfTokenProtected(token)) return false;
    work();
    return true;
  } finally {
    pdfTokenEvictions.delete(token);
  }
}

function setBoundedMap<K, V>(map: Map<K, V>, key: K, value: V, maximum: number): void {
  map.delete(key);
  map.set(key, value);
  while (map.size > maximum) map.delete(map.keys().next().value!);
}

function touchPdfToken(token: string): void {
  setBoundedMap(recentPdfTokens, token, Date.now(), 4);
}

function stagingOwner(path: string): string {
  return `staging:${path}`;
}

export async function withStagingArtifact<T>(
  path: string,
  work: () => Promise<T>,
): Promise<T> {
  activeStagingArtifacts.add(path);
  try {
    return await work();
  } finally {
    activeStagingArtifacts.delete(path);
    releaseUnusedPdfCacheProcessRoots();
  }
}

export function isPdfTokenLeased(token: string): boolean {
  return (pdfTokenLeases.get(token) ?? 0) > 0;
}

export async function withPdfTokenLease<T>(
  token: string,
  work: () => Promise<T>,
): Promise<T> {
  touchPdfToken(token);
  pdfTokenLeases.set(token, (pdfTokenLeases.get(token) ?? 0) + 1);
  try {
    return await work();
  } finally {
    const remaining = (pdfTokenLeases.get(token) ?? 1) - 1;
    if (remaining > 0) pdfTokenLeases.set(token, remaining);
    else pdfTokenLeases.delete(token);
    void pruneReferenceCaches();
    releaseUnusedPdfCacheProcessRoots();
  }
}

export function publishPdfSnapshotWithLease<T>(
  token: string,
  publish: () => Promise<T>,
): Promise<T> {
  return withPdfTokenLease(token, publish);
}

export function withPdfPageCountLease<T>(
  token: string,
  inspect: () => Promise<T>,
): Promise<T> {
  return withPdfTokenLease(token, inspect);
}

export class CoalescedAsyncMaintenance {
  private running: Promise<void> | null = null;
  private requested = false;
  private readonly work: () => Promise<void>;
  private readonly onError: (error: unknown) => void;

  constructor(
    work: () => Promise<void>,
    onError: (error: unknown) => void = () => {},
  ) {
    this.work = work;
    this.onError = onError;
  }

  request(): Promise<void> {
    this.requested = true;
    this.running ??= this.run();
    return this.running;
  }

  private async run(): Promise<void> {
    try {
      while (this.requested) {
        this.requested = false;
        try {
          await this.work();
        } catch (error) {
          try {
            this.onError(error);
          } catch {
            // Reporting is outside cache artifact ownership too.
          }
        }
      }
    } finally {
      this.running = null;
    }
  }
}

let cachePruneFailureReported = false;
function reportCachePruneFailure(error: unknown): void {
  if (cachePruneFailureReported) return;
  cachePruneFailureReported = true;
  const reason = (error instanceof Error ? error.message : String(error)).slice(0, 200);
  console.warn(`[hwpdocs-pdf] cache pruning failed: ${reason}`);
}

export interface PdfTwinLookupRequest {
  fileName: string;
  size: number;
  sha256: string;
}

export interface PdfTwinSession {
  rasterRevision: string;
  errorLogCapability: string;
}

interface DocumentCandidate {
  documentPath: string;
  pdfPath: string;
}

export interface HwpdocsPdfTwinIndex {
  root: string;
  byNameAndSize: Map<string, DocumentCandidate[]>;
}

function normalizedName(value: string): string {
  return value.normalize('NFC');
}

function candidateKey(fileName: string, size: number): string {
  return `${normalizedName(fileName)}\0${size}`;
}

function pdfToken(root: string, pdfPath: string, digest: string): string {
  return createHash('sha256')
    .update(resolve(root).normalize('NFC'))
    .update('\0')
    .update(relative(root, pdfPath).normalize('NFC'))
    .update(`\0${digest}`)
    .digest('base64url')
    .slice(0, 24);
}

export function pdfRasterToolchainRevision(input: {
  path: string;
  ghostscriptVersion: string;
  pdfinfoVersion: string;
}): string {
  const fingerprint = createHash('sha256')
    .update(PDF_PAGE_RASTER_ALGORITHM)
    .update('\0')
    .update(input.path)
    .update('\0')
    .update(input.ghostscriptVersion.slice(0, 4_096))
    .update('\0')
    .update(input.pdfinfoVersion.slice(0, 4_096))
    .digest('hex')
    .slice(0, 16);
  return `${PDF_PAGE_RASTER_ALGORITHM}-${fingerprint}`;
}

function admitSourceBytes(size: number, maximum: number, label: string): void {
  if (!Number.isSafeInteger(size) || size < 0 || size > maximum) {
    throw new SourceAdmissionError(`${label} exceeds the source byte budget`);
  }
}

export function sourceGenerationKey(
  filePath: string,
  stats: Pick<Stats, 'dev' | 'ino' | 'size' | 'mtimeMs' | 'ctimeMs'>,
): string {
  return [
    resolve(filePath),
    stats.dev,
    stats.ino,
    stats.size,
    stats.mtimeMs,
    stats.ctimeMs,
  ].join(':');
}

function sameSourceGeneration(left: Stats, right: Stats): boolean {
  return left.dev === right.dev
    && left.ino === right.ino
    && left.size === right.size
    && left.mtimeMs === right.mtimeMs
    && left.ctimeMs === right.ctimeMs;
}

export async function isSourceGenerationCurrent(
  filePath: string,
  expected: string,
): Promise<boolean> {
  try {
    return sourceGenerationKey(filePath, await statFile(filePath)) === expected;
  } catch {
    return false;
  }
}

export async function isSourcePairCurrent(pair: {
  documentPath: string;
  documentGeneration: string;
  pdfPath: string;
  pdfGeneration: string;
}): Promise<boolean> {
  return await isSourceGenerationCurrent(pair.documentPath, pair.documentGeneration)
    && await isSourceGenerationCurrent(pair.pdfPath, pair.pdfGeneration);
}

async function fileSha256(
  filePath: string,
  maximum: number,
  label: string,
): Promise<{ digest: string; generation: string }> {
  const file = await openFile(filePath, 'r');
  let admitted: Stats;
  try {
    admitted = await file.stat();
    admitSourceBytes(admitted.size, maximum, label);
  } catch (error) {
    await file.close();
    throw error;
  }
  return await runSourceIo(
    `digest:${sourceGenerationKey(filePath, admitted)}:${maximum}`,
    async () => {
    try {
      const hash = createHash('sha256');
      let streamed = 0;
      const sink = new Writable({
        write(chunk: Buffer, _encoding, done) {
          streamed += chunk.length;
          if (streamed > maximum) return done(new SourceAdmissionError(`${label} exceeds the source byte budget`));
          hash.update(chunk);
          done();
        },
      });
      const source = file.createReadStream({ autoClose: false });
      await pipeline(source, sink);
      if (!sameSourceGeneration(admitted, await file.stat())) {
        throw new SourceAdmissionError(`${label} changed while it was read`);
      }
      return {
        digest: hash.digest('hex'),
        generation: sourceGenerationKey(filePath, admitted),
      };
    } finally {
      await file.close();
    }
    },
    () => { void file.close().catch(() => {}); },
  );
}

async function materializePdfSnapshot(
  index: HwpdocsPdfTwinIndex,
  candidate: DocumentCandidate,
  expectedPdfGeneration: string,
): Promise<{ token: string; snapshotPath: string }> {
  const file = await openFile(candidate.pdfPath, 'r');
  let admitted: Stats;
  try {
    admitted = await file.stat();
    admitSourceBytes(admitted.size, PDF_TWIN_SOURCE_BUDGET.pdfBytes, 'PDF twin');
    if (sourceGenerationKey(candidate.pdfPath, admitted) !== expectedPdfGeneration) {
      throw new SourceAdmissionError('PDF twin changed after pair admission');
    }
  } catch (error) {
    await file.close();
    throw error;
  }
  const snapshot = await runSourceIo(
    `snapshot:${resolve(index.root)}:${sourceGenerationKey(candidate.pdfPath, admitted)}`,
    async () => {
      const stagingDirectory = join(PDF_SOURCE_CACHE, '.staging');
      await mkdirFile(stagingDirectory, { recursive: true });
      const temporaryPath = join(
        stagingDirectory,
        `${process.pid}-${Date.now()}-${Math.random().toString(36).slice(2)}.tmp`,
      );
      try {
        return await withStagingArtifact(temporaryPath, async () => {
          const hash = createHash('sha256');
          let streamed = 0;
          const digestingCopy = new Transform({
            transform(chunk: Buffer, _encoding, done) {
              streamed += chunk.length;
              if (streamed > PDF_TWIN_SOURCE_BUDGET.pdfBytes) {
                return done(new SourceAdmissionError('PDF twin exceeds the source byte budget'));
              }
              hash.update(chunk);
              done(null, chunk);
            },
          });
          const source = file.createReadStream({ autoClose: false });
          await pipeline(source, digestingCopy, createWriteStream(temporaryPath, { flags: 'wx' }));
          if (!sameSourceGeneration(admitted, await file.stat())) {
            throw new SourceAdmissionError('PDF twin changed while it was read');
          }
          const token = pdfToken(index.root, candidate.pdfPath, hash.digest('hex'));
          const snapshotDirectory = join(PDF_SOURCE_CACHE, token);
          const snapshotPath = join(snapshotDirectory, basename(candidate.pdfPath));
          await publishPdfSnapshotWithLease(token, async () => {
            await mkdirFile(snapshotDirectory, { recursive: true });
            try {
              await renameFile(temporaryPath, snapshotPath);
            } catch (error) {
              await unlinkFile(temporaryPath).catch(() => {});
              if (!existsSync(snapshotPath)) throw error;
            }
          });
          return { token, snapshotPath };
        });
      } catch (error) {
        await unlinkFile(temporaryPath).catch(() => {});
        throw error;
      } finally {
        await file.close();
      }
    },
    () => { void file.close().catch(() => {}); },
  );
  return snapshot;
}

export function resolvePdfSnapshot(token: string): string | null {
  if (!/^[A-Za-z0-9_-]{24}$/.test(token)) return null;
  const registered = pdfSnapshotsByToken.get(token);
  if (registered && existsSync(registered)) {
    touchPdfToken(token);
    return registered;
  }
  if (registered) pdfSnapshotsByToken.delete(token);
  const directory = join(PDF_SOURCE_CACHE, token);
  if (!existsSync(directory)) return null;
  const snapshots = readdirSync(directory)
    .filter(name => extname(name).toLowerCase() === '.pdf')
    .map(name => join(directory, name))
    .filter(path => {
      try {
        return statSync(path).isFile();
      } catch {
        return false;
      }
    });
  if (snapshots.length !== 1) return null;
  setBoundedMap(pdfSnapshotsByToken, token, snapshots[0], 64);
  touchPdfToken(token);
  return snapshots[0];
}

export async function inspectSourceCache(
  root = PDF_SOURCE_CACHE,
  maximumEntries = CACHE_INSPECTION_MAX_ENTRIES,
): Promise<{ entries: ReferenceCacheEntry[]; truncated: boolean }> {
  if (!existsSync(root)) return { entries: [], truncated: false };
  const entries: ReferenceCacheEntry[] = [];
  let inspectedNamespaces = 0;
  let inspectedStagingFiles = 0;
  let truncated = false;
  for await (const tokenEntry of await openDirectory(root)) {
    if (tokenEntry.isDirectory() && tokenEntry.name === '.staging') {
      const stagingDirectory = join(root, tokenEntry.name);
      for await (const fileEntry of await openDirectory(stagingDirectory)) {
        if (!fileEntry.isFile()) continue;
        if (inspectedStagingFiles >= maximumEntries) {
          truncated = true;
          break;
        }
        inspectedStagingFiles += 1;
        const path = join(stagingDirectory, fileEntry.name);
        try {
          const metadata = await statFile(path);
          entries.push({
            id: path,
            owner: stagingOwner(path),
            bytes: metadata.size,
            lastAccess: metadata.mtimeMs,
          });
        } catch {
          // Concurrent publication or cleanup already moved it.
        }
      }
      continue;
    }
    if (!tokenEntry.isDirectory() || !/^[A-Za-z0-9_-]{24}$/.test(tokenEntry.name)) continue;
    if (inspectedNamespaces >= maximumEntries) {
      truncated = true;
      break;
    }
    inspectedNamespaces += 1;
    const directory = join(root, tokenEntry.name);
    let bytes = 0;
    let lastAccess = 0;
    for await (const fileEntry of await openDirectory(directory)) {
      if (!fileEntry.isFile()) continue;
      try {
        const metadata = await statFile(join(directory, fileEntry.name));
        bytes += metadata.size;
        lastAccess = Math.max(lastAccess, metadata.mtimeMs);
      } catch {
        // Concurrent cache cleanup already removed it.
      }
    }
    entries.push({ id: directory, owner: tokenEntry.name, bytes, lastAccess });
  }
  return { entries, truncated };
}

export function evictSourceCacheEntries(
  entries: readonly ReferenceCacheEntry[],
  selected: readonly string[],
): number {
  const owners = new Map(entries.map(entry => [entry.id, entry.owner]));
  let removed = 0;
  for (const path of selected) {
    const owner = owners.get(path);
    if (!owner) continue;
    if (owner.startsWith('staging:')) {
      const stagingPath = owner.slice('staging:'.length);
      if (!activeStagingArtifacts.has(stagingPath) && existsSync(stagingPath)) {
        unlinkSync(stagingPath);
        removed += 1;
      }
      continue;
    }
    if (withPdfTokenEviction(owner, () => {
      rmSync(path, { recursive: true, force: true });
      pdfSnapshotsByToken.delete(owner);
    })) removed += 1;
  }
  return removed;
}

export async function inspectRasterCache(
  root = PDF_PAGE_CACHE,
  maximumEntries = CACHE_INSPECTION_MAX_ENTRIES,
): Promise<{ entries: ReferenceCacheEntry[]; truncated: boolean; removedDirectories: number }> {
  if (!existsSync(root)) return { entries: [], truncated: false, removedDirectories: 0 };
  const entries: ReferenceCacheEntry[] = [];
  let inspectedNamespaces = 0;
  let inspectedFiles = 0;
  let truncated = false;
  let removedDirectories = 0;
  const visit = async (directory: string, owner: string | null): Promise<boolean> => {
    let nonempty = false;
    for await (const entry of await openDirectory(directory)) {
      if (owner === null && inspectedNamespaces >= maximumEntries) {
        truncated = true;
        return true;
      }
      if (owner === null) inspectedNamespaces += 1;
      const path = join(directory, entry.name);
      const entryOwner = owner ?? entry.name;
      if (entry.isDirectory()) {
        if (await visit(path, entryOwner)) nonempty = true;
        else {
          const removed = withPdfTokenEviction(entryOwner, () => {
            rmSync(path, { recursive: true, force: true });
          });
          if (removed) removedDirectories += 1;
          else nonempty = true;
        }
      }
      else if (entry.isFile()) {
        if (inspectedFiles >= maximumEntries) {
          truncated = true;
          return true;
        }
        inspectedFiles += 1;
        nonempty = true;
        try {
          const metadata = await statFile(path);
          entries.push({
            id: path,
            owner: entryOwner,
            bytes: metadata.size,
            lastAccess: metadata.mtimeMs,
          });
        } catch {
          // Concurrent cache cleanup already removed it.
        }
      }
    }
    return nonempty;
  };
  await visit(root, null);
  return { entries, truncated, removedDirectories };
}

function removeEmptyRasterAncestors(path: string): void {
  let directory = dirname(path);
  while (directory !== PDF_PAGE_CACHE && directory.startsWith(`${PDF_PAGE_CACHE}${sep}`)) {
    try {
      if (readdirSync(directory).length > 0) return;
    } catch {
      return;
    }
    rmSync(directory, { recursive: true, force: true });
    directory = dirname(directory);
  }
}

const cachePruner = new CoalescedAsyncMaintenance(async () => {
    const protectedOwners = new Set([
      ...recentPdfTokens.keys(),
      ...pdfTokenLeases.keys(),
      ...Array.from(activeStagingArtifacts, stagingOwner),
    ]);
    const now = Date.now();
    const sourceInspection = await inspectSourceCache();
    const sourceEntries = sourceInspection.entries;
    const sourceEvictions = selectReferenceCacheEvictions(sourceEntries, protectedOwners, {
        maxEntries: SOURCE_CACHE_MAX_ENTRIES,
        maxBytes: CACHE_MAX_BYTES,
        maxAgeMs: CACHE_MAX_AGE_MS,
        now,
    });
    const removedSources = evictSourceCacheEntries(sourceEntries, sourceEvictions);
    const rasterInspection = await inspectRasterCache();
    const rasterEvictions = selectReferenceCacheEvictions(rasterInspection.entries, protectedOwners, {
        maxEntries: RASTER_CACHE_MAX_ENTRIES,
        maxBytes: CACHE_MAX_BYTES,
        maxAgeMs: CACHE_MAX_AGE_MS,
        now,
      });
    const rasterOwners = new Map(rasterInspection.entries.map(entry => [entry.id, entry.owner]));
    for (const path of rasterEvictions) {
      const token = rasterOwners.get(path);
      if (!token) continue;
      withPdfTokenEviction(token, () => {
        if (existsSync(path)) unlinkSync(path);
        removeEmptyRasterAncestors(path);
      });
    }
    if (
      rasterInspection.truncated
      && (rasterInspection.removedDirectories > 0 || rasterEvictions.length > 0)
    ) void cachePruner.request();
    if (sourceInspection.truncated && removedSources > 0) void cachePruner.request();
}, reportCachePruneFailure);

function pruneReferenceCaches(): Promise<void> {
  return cachePruner.request();
}

export async function buildHwpdocsPdfTwinIndex(
  root: string,
  limits: { maxEntries?: number; maxDepth?: number } = {},
): Promise<HwpdocsPdfTwinIndex> {
  const maxEntries = limits.maxEntries ?? MAX_INDEX_ENTRIES;
  const maxDepth = limits.maxDepth ?? MAX_INDEX_DEPTH;
  const byDirectoryStem = new Map<string, { documents: string[]; pdfs: string[] }>();
  let entryCount = 0;
  const visit = async (directory: string, depth: number): Promise<void> => {
    if (depth > maxDepth) throw new SourceAdmissionError('PDF twin index depth budget exceeded');
    for await (const entry of await openDirectory(directory)) {
      entryCount += 1;
      if (entryCount > maxEntries) {
        throw new SourceAdmissionError('PDF twin index entry budget exceeded');
      }
      const filePath = join(directory, entry.name);
      if (entry.isDirectory()) {
        await visit(filePath, depth + 1);
        continue;
      }
      if (!entry.isFile()) continue;
      const extension = extname(filePath).toLowerCase();
      if (extension !== '.hwp' && extension !== '.hwpx' && extension !== '.pdf') continue;
      const stem = basename(filePath, extname(filePath)).normalize('NFC');
      const key = `${dirname(filePath).normalize('NFC')}\0${stem}`;
      const pair = byDirectoryStem.get(key) ?? { documents: [], pdfs: [] };
      if (extension === '.pdf') pair.pdfs.push(filePath);
      else pair.documents.push(filePath);
      byDirectoryStem.set(key, pair);
    }
  };
  await visit(root, 0);

  const byNameAndSize = new Map<string, DocumentCandidate[]>();
  for (const pair of byDirectoryStem.values()) {
    if (pair.pdfs.length !== 1) continue;
    const pdfPath = pair.pdfs[0];
    for (const documentPath of pair.documents) {
      const size = (await statFile(documentPath)).size;
      const key = candidateKey(basename(documentPath), size);
      const candidates = byNameAndSize.get(key) ?? [];
      candidates.push({ documentPath, pdfPath });
      byNameAndSize.set(key, candidates);
    }
  }

  return { root, byNameAndSize };
}

export async function refreshPdfTwinIndexes(
  roots: readonly string[],
  previous: ReadonlyMap<string, HwpdocsPdfTwinIndex>,
  onRootError: (root: string, error: unknown) => void = () => {},
  limits: { maxEntries?: number; maxDepth?: number } = {},
): Promise<Map<string, HwpdocsPdfTwinIndex>> {
  const indexes = new Map(previous);
  for (const root of roots) {
    try {
      if (!existsSync(root)) continue;
      if (!(await statFile(root)).isDirectory()) throw new Error('not a directory');
      indexes.set(root, await buildHwpdocsPdfTwinIndex(root, limits));
    } catch (error) {
      onRootError(root, error);
    }
  }
  return indexes;
}

async function findPdfTwinCandidates(
  index: HwpdocsPdfTwinIndex,
  request: PdfTwinLookupRequest,
): Promise<Array<{
  candidate: DocumentCandidate;
  documentGeneration: string;
  pdfGeneration: string;
}>> {
  const candidates = index.byNameAndSize.get(candidateKey(request.fileName, request.size)) ?? [];
  const matching: Array<{
    candidate: DocumentCandidate;
    documentGeneration: string;
    pdfGeneration: string;
  }> = [];
  for (const candidate of candidates) {
    try {
      if (!existsSync(candidate.pdfPath)) continue;
      const pdfStats = statSync(candidate.pdfPath);
      admitSourceBytes(
        pdfStats.size,
        PDF_TWIN_SOURCE_BUDGET.pdfBytes,
        'PDF twin',
      );
      if (statSync(candidate.documentPath).size === request.size) {
        const document = await fileSha256(
          candidate.documentPath,
          PDF_TWIN_SOURCE_BUDGET.documentBytes,
          'HWP document',
        );
        if (document.digest === request.sha256) {
          matching.push({
            candidate,
            documentGeneration: document.generation,
            pdfGeneration: sourceGenerationKey(candidate.pdfPath, pdfStats),
          });
        }
      }
    } catch (error) {
      if (isWorkQueueSaturatedError(error)) throw error;
      // A missing, unreadable, or oversized indexed document is not a match.
    }
  }
  return matching;
}

async function publishPdfTwin(
  index: HwpdocsPdfTwinIndex,
  candidate: DocumentCandidate,
  documentGeneration: string,
  pdfGeneration: string,
  session: PdfTwinSession,
): Promise<PdfTwinLookupResponse> {
  let snapshot: { token: string; snapshotPath: string };
  try {
    snapshot = await materializePdfSnapshot(index, candidate, pdfGeneration);
  } catch (error) {
    if (isSourceAdmissionError(error)) return { status: 'none' };
    throw error;
  }
  if (!await isSourceGenerationCurrent(candidate.documentPath, documentGeneration)) {
    return { status: 'none' };
  }
  setBoundedMap(pdfSnapshotsByToken, snapshot.token, snapshot.snapshotPath, 64);
  touchPdfToken(snapshot.token);
  await pruneReferenceCaches();
  return {
    status: 'found',
    pdfName: basename(candidate.pdfPath),
    pdfPageUrl: `${PDF_PAGE_PATH_PREFIX}${snapshot.token}/${session.rasterRevision}`,
    pdfPageWidth: DEFAULT_PDF_PAGE_RASTER_WIDTH,
    pdfPageCount: null,
    relativeDirectory: relative(index.root, dirname(candidate.pdfPath)),
    errorLogCapability: session.errorLogCapability,
  };
}

export async function findPdfTwinAcrossIndexes(
  indexes: HwpdocsPdfTwinIndex[],
  request: PdfTwinLookupRequest,
  session: PdfTwinSession,
): Promise<PdfTwinLookupResponse> {
  const results: Array<{
    index: HwpdocsPdfTwinIndex;
    matches: Array<{
      candidate: DocumentCandidate;
      documentGeneration: string;
      pdfGeneration: string;
    }>;
  }> = [];
  for (const index of indexes) {
    results.push({ index, matches: await findPdfTwinCandidates(index, request) });
  }
  const found: Array<{
    index: HwpdocsPdfTwinIndex;
    match: {
      candidate: DocumentCandidate;
      documentGeneration: string;
      pdfGeneration: string;
    };
  }> = [];
  for (const result of results) {
    for (const match of result.matches) {
      if (await isSourcePairCurrent({
        documentPath: match.candidate.documentPath,
        documentGeneration: match.documentGeneration,
        pdfPath: match.candidate.pdfPath,
        pdfGeneration: match.pdfGeneration,
      })) found.push({ index: result.index, match });
    }
  }
  if (found.length > 1) return { status: 'ambiguous' };
  if (found.length === 1) {
    const result = await publishPdfTwin(
      found[0].index,
      found[0].match.candidate,
      found[0].match.documentGeneration,
      found[0].match.pdfGeneration,
      session,
    );
    return result;
  }
  return { status: 'none' };
}

export interface PdfPageRasterRequest {
  token: string;
  rasterRevision: string;
  pageIndex: number;
  pixelWidth: number;
}

export function parsePdfPageRasterRequest(url: URL): PdfPageRasterRequest | null {
  const match = new RegExp(
    `^/__rhwp_harness/pdf-page/([A-Za-z0-9_-]{24})/(${PDF_PAGE_RASTER_ALGORITHM}-[a-f0-9]{16})/(\\d+)\\.png$`,
  ).exec(url.pathname);
  if (!match) return null;
  const pageIndex = Number(match[3]);
  const pixelWidth = Number(url.searchParams.get('width'));
  if (
    !Number.isSafeInteger(pageIndex)
    || pageIndex < 0
    || !Number.isSafeInteger(pixelWidth)
    || pixelWidth < PAGE_RASTER_BUDGET.minWidth
    || pixelWidth > PAGE_RASTER_BUDGET.maxWidth
  ) return null;
  return { token: match[1], rasterRevision: match[2], pageIndex, pixelWidth };
}

export function pdfPageRasterKey(request: PdfPageRasterRequest): string {
  return `${request.rasterRevision}:${request.token}:${request.pageIndex}:${request.pixelWidth}`;
}

export function admitRasterRevision(requested: string, current: string): void {
  if (requested !== current) {
    throw new Error('PDF raster toolchain revision is not current');
  }
}

export function runCommand(
  command: string,
  args: string[],
  options: { timeoutMs?: number; killGraceMs?: number } = {},
): Promise<{ stdout: string; stderr: string }> {
  return new Promise((resolveCommand, rejectCommand) => {
    const child = spawn(command, args, { stdio: ['ignore', 'pipe', 'pipe'] });
    let stdout = '';
    let stderr = '';
    let timedOut = false;
    let completed = false;
    let killTimer: ReturnType<typeof setTimeout> | null = null;
    const timeoutMs = Math.max(1, options.timeoutMs ?? COMMAND_TIMEOUT_MS);
    const killGraceMs = Math.max(1, options.killGraceMs ?? COMMAND_KILL_GRACE_MS);
    const timeout = setTimeout(() => {
      timedOut = true;
      child.kill('SIGTERM');
      killTimer = setTimeout(() => child.kill('SIGKILL'), killGraceMs);
    }, timeoutMs);
    const finish = (error: Error | null, result?: { stdout: string; stderr: string }): void => {
      if (completed) return;
      completed = true;
      clearTimeout(timeout);
      if (killTimer !== null) clearTimeout(killTimer);
      if (error) rejectCommand(error);
      else resolveCommand(result!);
    };
    child.stdout.setEncoding('utf8');
    child.stdout.on('data', (chunk: string) => {
      stdout += chunk.slice(0, Math.max(0, 32_768 - stdout.length));
    });
    child.stderr.setEncoding('utf8');
    child.stderr.on('data', (chunk: string) => {
      stderr += chunk.slice(0, Math.max(0, 8_192 - stderr.length));
    });
    child.once('error', error => finish(error));
    child.once('close', (code) => {
      if (timedOut) finish(new Error(`${basename(command)} timed out after ${timeoutMs}ms`));
      else if (code === 0) finish(null, { stdout, stderr });
      else finish(new Error(`${basename(command)} exited ${code}: ${stderr.trim()}`));
    });
  });
}

export class BoundedAsyncWorkQueue {
  private active = 0;
  private readonly queued: Array<() => void> = [];
  private readonly maxConcurrent: number;
  private readonly maxQueued: number;

  constructor(
    maxConcurrent = MAX_PDF_PROCESSES,
    maxQueued = MAX_QUEUED_PDF_PROCESSES,
  ) {
    if (maxConcurrent < 1 || maxQueued < 0) throw new Error('invalid raster queue limits');
    this.maxConcurrent = maxConcurrent;
    this.maxQueued = maxQueued;
  }

  run<T>(work: () => Promise<T>): Promise<T> {
    return new Promise((resolveWork, rejectWork) => {
      const start = (): void => {
        this.active += 1;
        void Promise.resolve()
          .then(work)
          .then(resolveWork, rejectWork)
          .finally(() => {
            this.active -= 1;
            this.queued.shift()?.();
          });
      };
      if (this.active < this.maxConcurrent) start();
      else if (this.queued.length < this.maxQueued) this.queued.push(start);
      else rejectWork(new WorkQueueSaturatedError('PDF harness work queue is saturated'));
    });
  }
}

const pdfProcessQueue = pdfCacheProcessState.pdfProcessQueue ??= new BoundedAsyncWorkQueue();
const sourceIoQueue = pdfCacheProcessState.sourceIoQueue ??= new BoundedAsyncWorkQueue(2, 8);

export function runSourceIo<T>(
  key: string,
  work: () => Promise<T>,
  releaseUnused?: () => void,
  queue = sourceIoQueue,
  cacheRoot = PDF_CACHE_PROCESS_ROOT,
): Promise<T> {
  const existing = sourceIoInFlight.get(key) as Promise<T> | undefined;
  if (existing) {
    releaseUnused?.();
    return existing;
  }
  if (pdfCacheProcessState.closedCacheRoots.has(cacheRoot)) {
    releaseUnused?.();
    return Promise.reject(new SourceAdmissionError('PDF cache owner is closed'));
  }
  let started = false;
  const pending = queue.run(async () => {
    started = true;
    return await work();
  }).catch((error) => {
    if (!started) releaseUnused?.();
    throw error;
  }).finally(() => {
    if (sourceIoInFlight.get(key) === pending) sourceIoInFlight.delete(key);
    releaseUnusedPdfCacheProcessRoots();
  });
  sourceIoInFlight.set(key, pending);
  return pending;
}

function runPdfCommand(
  command: string,
  args: string[],
): Promise<{ stdout: string; stderr: string }> {
  return pdfProcessQueue.run(() => runCommand(command, args));
}

async function readPdfRasterToolchainRevision(): Promise<string> {
  const [ghostscript, pdfinfo] = await Promise.all([
    runPdfCommand('gs', ['--version']),
    runPdfCommand('pdfinfo', ['-v']),
  ]);
  return pdfRasterToolchainRevision({
    path: process.env.PATH ?? '',
    ghostscriptVersion: `${ghostscript.stdout}\n${ghostscript.stderr}`,
    pdfinfoVersion: `${pdfinfo.stdout}\n${pdfinfo.stderr}`,
  });
}

const pdfPageSizes = new Map<string, { width: number; height: number }>();
const pdfPageCounts = new Map<string, number>();
const pdfPageSizeInFlight = new Map<string, Promise<{ width: number; height: number }>>();
const pdfPageCountInFlight = new Map<string, Promise<number>>();

async function readPdfPageCount(
  pdfPath: string,
  token: string,
  rasterRevision: string,
): Promise<number> {
  const key = `${rasterRevision}:${token}`;
  const cached = pdfPageCounts.get(key);
  if (cached !== undefined) return cached;
  return await withPdfPageCountLease(token, async () => {
    const cachedInsideLease = pdfPageCounts.get(key);
    if (cachedInsideLease !== undefined) return cachedInsideLease;
    const existing = pdfPageCountInFlight.get(key);
    if (existing) return await existing;
    const pending = runPdfCommand('pdfinfo', [pdfPath]).then(({ stdout }) => {
      const match = /^Pages:\s+(\d+)$/m.exec(stdout);
      if (!match) throw new Error('pdfinfo did not report page count');
      const pageCount = Number(match[1]);
      if (!Number.isSafeInteger(pageCount) || pageCount <= 0) {
        throw new Error('invalid PDF page count');
      }
      setBoundedMap(pdfPageCounts, key, pageCount, 1_024);
      return pageCount;
    }).finally(() => {
      if (pdfPageCountInFlight.get(key) === pending) pdfPageCountInFlight.delete(key);
    });
    pdfPageCountInFlight.set(key, pending);
    return await pending;
  });
}

async function readPdfPageSize(
  pdfPath: string,
  token: string,
  rasterRevision: string,
  pageIndex: number,
): Promise<{ width: number; height: number }> {
  const key = `${rasterRevision}\0${token}\0${pageIndex}`;
  const cached = pdfPageSizes.get(key);
  if (cached) return cached;
  const existing = pdfPageSizeInFlight.get(key);
  if (existing) return existing;
  const pageNumber = pageIndex + 1;
  const pending = runPdfCommand('pdfinfo', [
    '-box',
    '-f', String(pageNumber),
    '-l', String(pageNumber),
    pdfPath,
  ]).then(({ stdout }) => {
    const size = parsePdfMediaBox(stdout, pageNumber);
    setBoundedMap(pdfPageSizes, key, size, 2_048);
    return size;
  }).finally(() => {
    if (pdfPageSizeInFlight.get(key) === pending) pdfPageSizeInFlight.delete(key);
  });
  pdfPageSizeInFlight.set(key, pending);
  return pending;
}

export function parsePdfMediaBox(
  pdfInfoOutput: string,
  pageNumber: number,
): { width: number; height: number } {
  const match = new RegExp(
    `Page\\s+${pageNumber}\\s+MediaBox:\\s+([\\d.-]+)\\s+([\\d.-]+)\\s+([\\d.-]+)\\s+([\\d.-]+)`,
  ).exec(pdfInfoOutput);
  if (!match) throw new Error(`pdfinfo did not report page ${pageNumber} MediaBox`);
  const coordinates = match.slice(1, 5).map(Number);
  const size = {
    width: coordinates[2] - coordinates[0],
    height: coordinates[3] - coordinates[1],
  };
  if (!coordinates.every(Number.isFinite) || !(size.width > 0) || !(size.height > 0)) {
    throw new Error(`invalid PDF page ${pageNumber} size`);
  }
  const rotationMatch = new RegExp(`Page\\s+${pageNumber}\\s+rot:\\s+(-?\\d+)`, 'i')
    .exec(pdfInfoOutput);
  const rotation = rotationMatch
    ? ((Number(rotationMatch[1]) % 360) + 360) % 360
    : 0;
  if (![0, 90, 180, 270].includes(rotation)) {
    throw new Error(`invalid PDF page ${pageNumber} rotation`);
  }
  if (rotation === 90 || rotation === 270) {
    return { width: size.height, height: size.width };
  }
  return size;
}

export function pdfRasterSize(
  pageSize: { width: number; height: number },
  pixelWidth: number,
): { width: number; height: number } {
  return boundedPageRasterSize(pageSize, pixelWidth, 'PDF page');
}

export function dedupeRasterRequest(
  key: string,
  render: () => Promise<string>,
): Promise<string> {
  const existing = rasterInFlight.get(key);
  if (existing) return existing;
  const pending = render().finally(() => {
    if (rasterInFlight.get(key) === pending) rasterInFlight.delete(key);
    releaseUnusedPdfCacheProcessRoots();
  });
  rasterInFlight.set(key, pending);
  return pending;
}

async function rasterizePdfPage(
  pdfPath: string,
  request: PdfPageRasterRequest,
): Promise<string> {
  const cacheDirectory = join(PDF_PAGE_CACHE, request.token, request.rasterRevision);
  const outputPath = join(cacheDirectory, `${request.pageIndex}-${request.pixelWidth}.png`);
  touchPdfToken(request.token);
  if (existsSync(outputPath)) return outputPath;

  return dedupeRasterRequest(pdfPageRasterKey(request), async () => {
    try {
      return await withPdfTokenLease(request.token, async () => {
        if (existsSync(outputPath)) return outputPath;
        const pageSize = await readPdfPageSize(
          pdfPath,
          request.token,
          request.rasterRevision,
          request.pageIndex,
        );
        const rasterSize = pdfRasterSize(pageSize, request.pixelWidth);
        mkdirSync(cacheDirectory, { recursive: true });
        const temporaryPath = `${outputPath}.${process.pid}.${Date.now()}.tmp`;
        const pageNumber = request.pageIndex + 1;
        try {
          await runPdfCommand('gs', ghostscriptRasterArgs(
            pdfPath,
            pageNumber,
            rasterSize.width,
            rasterSize.height,
            temporaryPath,
          ));
          renameSync(temporaryPath, outputPath);
          return outputPath;
        } catch (error) {
          if (existsSync(temporaryPath)) unlinkSync(temporaryPath);
          throw error;
        }
      });
    } finally {
      await pruneReferenceCaches();
    }
  });
}

export function ghostscriptRasterArgs(
  pdfPath: string,
  pageNumber: number,
  pixelWidth: number,
  pixelHeight: number,
  outputPath: string,
): string[] {
  return [
    '-q',
    '-dSAFER',
    '-dBATCH',
    '-dNOPAUSE',
    '-dPDFSTOPONERROR',
    '-dPDFFitPage',
    '-dAutoRotatePages=/PageByPage',
    '-dTextAlphaBits=4',
    '-dGraphicsAlphaBits=4',
    '-sDEVICE=png16m',
    `-dFirstPage=${pageNumber}`,
    `-dLastPage=${pageNumber}`,
    `-g${pixelWidth}x${pixelHeight}`,
    '-r72',
    `-sOutputFile=${outputPath}`,
    pdfPath,
  ];
}

async function openFileSource(
  filePath: string,
) {
  const file = await openFile(filePath, 'r');
  try {
    const size = (await file.stat()).size;
    return { size, source: file.createReadStream() };
  } catch (error) {
    await file.close();
    throw error;
  }
}

async function servePng(res: ServerResponse, pngPath: string): Promise<void> {
  const { size, source } = await openFileSource(pngPath);
  try {
    res.statusCode = 200;
    res.setHeader('Content-Type', 'image/png');
    res.setHeader('Content-Length', size);
    res.setHeader('Cache-Control', 'private, max-age=31536000, immutable');
    await pipeline(source, res);
  } catch (error) {
    source.destroy();
    throw error;
  }
}

function json(res: ServerResponse, statusCode: number, value: unknown): void {
  res.statusCode = statusCode;
  res.setHeader('Content-Type', 'application/json; charset=utf-8');
  res.setHeader('Cache-Control', 'no-store');
  res.end(JSON.stringify(value));
}

async function readJsonBody(req: IncomingMessage, limit = MAX_LOOKUP_BODY_BYTES): Promise<unknown> {
  const chunks: Buffer[] = [];
  let size = 0;
  for await (const chunk of req) {
    const buffer = Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk);
    size += buffer.length;
    if (size > limit) throw new Error('JSON body too large');
    chunks.push(buffer);
  }
  return JSON.parse(Buffer.concat(chunks).toString('utf8'));
}

async function readTextBody(req: IncomingMessage, limit: number): Promise<string> {
  const chunks: Buffer[] = [];
  let size = 0;
  for await (const chunk of req) {
    const buffer = Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk);
    size += buffer.length;
    if (size > limit) throw new Error('text body too large');
    chunks.push(buffer);
  }
  return Buffer.concat(chunks).toString('utf8');
}

function isLookupRequest(value: unknown): value is PdfTwinLookupRequest {
  if (!value || typeof value !== 'object') return false;
  const candidate = value as Partial<PdfTwinLookupRequest>;
  return typeof candidate.fileName === 'string'
    && candidate.fileName.length > 0
    && candidate.fileName.length <= 512
    && Number.isSafeInteger(candidate.size)
    && (candidate.size ?? -1) >= 0
    && typeof candidate.sha256 === 'string'
    && /^[a-f0-9]{64}$/.test(candidate.sha256);
}

export function hasDocumentErrorCapability(
  provided: string | string[] | undefined,
  expected: string,
): boolean {
  if (typeof provided !== 'string') return false;
  const actualBytes = Buffer.from(provided);
  const expectedBytes = Buffer.from(expected);
  return actualBytes.length === expectedBytes.length
    && timingSafeEqual(actualBytes, expectedBytes);
}

export async function serveDocumentErrorLog(
  req: IncomingMessage,
  res: ServerResponse,
  expectedCapability: string,
  logger: Pick<Logger, 'error'>,
): Promise<void> {
  if (req.method !== 'POST') return json(res, 405, { status: 'error' });
  const providedCapability = req.headers[DOCUMENT_ERROR_CAPABILITY_HEADER];
  if (!hasDocumentErrorCapability(providedCapability, expectedCapability)) {
    return json(res, 403, { status: 'error' });
  }
  try {
    const line = await readTextBody(req, MAX_DIFF_BODY_BYTES);
    if (!isDocumentErrorLine(line)) return json(res, 400, { status: 'error' });
    logger.error(line, { timestamp: true, error: null });
    return json(res, 202, { status: 'accepted' });
  } catch {
    return json(res, 400, { status: 'error' });
  }
}

export function failStreamResponse(res: ServerResponse, label: string, error: unknown): void {
  const reason = (error instanceof Error ? error.message : String(error)).slice(0, 200);
  console.warn(`[hwpdocs-pdf] ${label} failed: ${reason}`);
  if (!res.headersSent && !res.destroyed) json(res, 500, { status: 'error' });
  else if (!res.destroyed) res.destroy(error instanceof Error ? error : undefined);
}

export function hwpdocsPdfTwinPlugin(options: {
  root?: string;
  additionalRoots?: string[];
} = {}): Plugin {
  const primaryRoot = options.root
    ?? process.env.RHWP_HWP_DOCS_ROOT
    ?? join(homedir(), 'hwpdocs_10k');
  const environmentRoots = process.env.RHWP_PDF_TWIN_ROOTS
    ?.split(delimiter)
    .map(root => root.trim())
    .filter(Boolean);
  const additionalRoots = options.additionalRoots
    ?? (environmentRoots?.length ? environmentRoots : [join(homedir(), 'Downloads')]);
  const roots = Array.from(new Set([primaryRoot, ...additionalRoots].map(root => resolve(root))));
  const errorLogCapability = randomBytes(32).toString('base64url');
  reclaimDeadPdfCacheProcesses(PDF_CACHE_PARENT, PDF_CACHE_PROCESS_ROOT);
  void pruneReferenceCaches();
  let indexes: HwpdocsPdfTwinIndex[] | null = null;
  let indexesByRoot = new Map<string, HwpdocsPdfTwinIndex>();
  let indexBuild: Promise<HwpdocsPdfTwinIndex[]> | null = null;
  let toolchainRevision: Promise<string> | null = null;
  const warnedRootFailures = new Set<string>();
  const getIndexes = async (force = false): Promise<HwpdocsPdfTwinIndex[]> => {
    if (indexBuild) return indexBuild;
    if (indexes?.length && !force) return indexes;
    const pending = runSourceIo(
      `index:${roots.join('\0')}`,
      () => refreshPdfTwinIndexes(roots, indexesByRoot, (root, error) => {
        if (warnedRootFailures.has(root)) return;
        warnedRootFailures.add(root);
        const reason = (error instanceof Error ? error.message : String(error)).slice(0, 200);
        console.warn(`[hwpdocs-pdf] skipped twin root ${JSON.stringify(root)}: ${reason}`);
      }),
    ).then((built) => {
      indexesByRoot = built;
      indexes = Array.from(built.values());
      return indexes;
    }).finally(() => {
      if (indexBuild === pending) indexBuild = null;
    });
    indexBuild = pending;
    return pending;
  };
  const pdfPathForToken = (token: string): string | null => {
    return resolvePdfSnapshot(token);
  };
  const getToolchainRevision = (): Promise<string> => {
    toolchainRevision ??= readPdfRasterToolchainRevision().catch((error) => {
      toolchainRevision = null;
      throw error;
    });
    return toolchainRevision;
  };
  const lookup = async (
    request: PdfTwinLookupRequest,
    force = false,
  ): Promise<PdfTwinLookupResponse> => {
    return await findPdfTwinAcrossIndexes(
      await getIndexes(force),
      request,
      {
        rasterRevision: await getToolchainRevision(),
        errorLogCapability,
      },
    );
  };

  return {
    name: 'hwpdocs-pdf-twin-harness',
    apply: 'serve',
    configureServer(server) {
      registerPdfCacheServerLifetime(server.httpServer);
      server.middlewares.use(async (req, res, next) => {
        const url = new URL(req.url ?? '/', 'http://localhost');
        if (url.pathname === DOCUMENT_ERROR_LOG_PATH) {
          return serveDocumentErrorLog(req, res, errorLogCapability, server.config.logger);
        }
        if (url.pathname === PDF_TWIN_LOOKUP_PATH) {
          if (req.method !== 'POST') return json(res, 405, { status: 'error' });
          try {
            if ((await getIndexes()).length === 0) return json(res, 404, { status: 'none' });
            const body = await readJsonBody(req);
            if (!isLookupRequest(body)) return json(res, 400, { status: 'error' });
            let result = await lookup(body);
            if (result.status === 'none') result = await lookup(body, true);
            if (result.status === 'found') {
              const pageRequest = parsePdfPageRasterRequest(new URL(
                `${result.pdfPageUrl}/0.png?width=${result.pdfPageWidth}`,
                'http://localhost',
              ));
              const pdfPath = pageRequest ? resolvePdfSnapshot(pageRequest.token) : null;
              if (pageRequest && pdfPath) {
                result.pdfPageCount = await readPdfPageCount(
                  pdfPath,
                  pageRequest.token,
                  pageRequest.rasterRevision,
                );
                void rasterizePdfPage(pdfPath, pageRequest).catch((error) => {
                  console.warn('[hwpdocs-pdf] Ghostscript prewarm failed:', error);
                });
              }
            }
            return json(res, 200, result);
          } catch (error) {
            if (isWorkQueueSaturatedError(error)) {
              res.setHeader('Retry-After', '1');
              return json(res, 503, { status: 'busy', retryAfterMs: 1_000 });
            }
            return json(res, 400, { status: 'error' });
          }
        }

        if (url.pathname.startsWith(PDF_PAGE_PATH_PREFIX)) {
          if (req.method !== 'GET') return json(res, 405, { status: 'error' });
          const request = parsePdfPageRasterRequest(url);
          if (!request) return json(res, 400, { status: 'error' });
          try {
            const currentRevision = await getToolchainRevision();
            try {
              admitRasterRevision(request.rasterRevision, currentRevision);
            } catch {
              return json(res, 409, { status: 'stale-toolchain' });
            }
            const pdfPath = pdfPathForToken(request.token);
            if (!pdfPath) return json(res, 404, { status: 'none' });
            await withPdfTokenLease(request.token, async () => {
              const pngPath = await rasterizePdfPage(pdfPath, request);
              await servePng(res, pngPath);
            });
            return;
          } catch (error) {
            if (isWorkQueueSaturatedError(error) && !res.headersSent) {
              res.setHeader('Retry-After', '1');
              return json(res, 503, { status: 'busy', retryAfterMs: 1_000 });
            }
            failStreamResponse(res, 'PNG raster/stream', error);
            return;
          }
        }

        next();
      });
    },
  };
}
