/**
 * 로컬 글꼴 감지 모듈
 *
 * Local Font Access API (queryLocalFonts) 를 사용하여 사용자 PC에 설치된
 * 글꼴 목록을 조회한다. 저장된 감지 결과는 재사용하되, 새 목록 조회는
 * 사용자 승인 흐름에서만 호출하도록 API를 분리한다.
 */
import { REGISTERED_FONTS } from './font-loader.ts';

/** queryLocalFonts 반환 타입 (DOM 표준 미포함) */
interface FontData {
  family: string;
  fullName: string;
  postscriptName: string;
  style: string;
  blob?: () => Promise<Blob>;
}

export type LocalFontDetectionSource = 'local-font-access' | 'font-presence-probe';

/**
 * 하나의 설치 글꼴 face를 나타내는 로컬 메타데이터다.
 *
 * `family`는 브라우저 CSS에서 사용할 canonical 이름이고, `aliases`에는 HWP가 저장한
 * 한글/영문 family, full name, PostScript name을 함께 둔다. 원본 SFNT 바이트는 저장하지 않는다.
 */
export interface LocalFontRecord {
  family: string;
  fullName: string;
  postscriptName: string;
  style: string;
  displayName: string;
  aliases: string[];
}

export interface LocalFontSnapshot {
  /** v1은 family 문자열만 저장한 이전 형식이며 로드 시 v2 레코드로 승격한다. */
  version: 1 | 2;
  detectedAt: string;
  families: string[];
  /** v2에서만 저장되는 설치 글꼴 face 메타데이터. */
  fontRecords?: LocalFontRecord[];
  source: LocalFontDetectionSource;
  /** font-presence-probe는 전체 목록이 아니라 문서 후보만 확인한다. */
  checkedFamilies?: string[];
}

export type LocalFontStorageKind =
  | 'chrome-storage-local'
  | 'browser-storage-local'
  | 'local-storage'
  | 'none';

export interface LocalFontState {
  supported: boolean;
  method: LocalFontDetectionSource | null;
  loaded: boolean;
  stored: boolean;
  source: LocalFontDetectionSource | null;
  complete: boolean;
  storage: LocalFontStorageKind;
  count: number;
  checkedFamilies: string[];
  detectedAt: string | null;
  lastError: string | null;
}

export interface DetectLocalFontsOptions {
  /** 저장/메모리 캐시가 있어도 Local Font Access API를 다시 호출한다. */
  force?: boolean;
  /** true면 REGISTERED_FONTS에 포함된 family도 반환한다. */
  includeRegistered?: boolean;
  /** Local Font Access API가 없는 브라우저에서 현재 문서 글꼴만 확인할 때 사용한다. */
  candidateFamilies?: readonly string[];
}

export interface GetLocalFontsOptions {
  /** true면 REGISTERED_FONTS에 포함된 family도 반환한다. */
  includeRegistered?: boolean;
}

type LocalFontGlobal = typeof globalThis & {
  queryLocalFonts?: (options?: { postscriptNames?: string[] }) => Promise<FontData[]>;
  document?: {
    createElement?: (tagName: string) => unknown;
  };
};

interface ChromeRuntimeLike {
  lastError?: { message?: string };
}

interface ChromeStorageAreaLike {
  get(
    keys: string | string[] | Record<string, unknown> | null,
    callback?: (items: Record<string, unknown>) => void,
  ): void | Promise<Record<string, unknown>>;
  set(items: Record<string, unknown>, callback?: () => void): void | Promise<void>;
  remove(keys: string | string[], callback?: () => void): void | Promise<void>;
}

interface ChromeLike {
  runtime?: ChromeRuntimeLike;
  storage?: {
    local?: ChromeStorageAreaLike;
  };
}

type BrowserLike = ChromeLike;

const STORAGE_KEY = 'rhwp-local-fonts';
const PROBE_FONT_SIZE = 72;
const PROBE_WIDTH_EPSILON = 0.1;
const PROBE_FALLBACKS = ['monospace', 'serif', 'sans-serif'];
const PROBE_TEXTS = [
  'mmmmmmmmmiiiiiiiiiWWW',
  '0123456789 ABCDEFG abcdefg',
  '가나다라마바사아자차카타파하',
  '한글과 English 12345',
];
const LOCAL_FONT_NAME_READ_CONCURRENCY = 4;
const HANGUL_RE = /[\u1100-\u11FF\u3130-\u318F\uAC00-\uD7A3]/;

/** 캐시된 로컬 글꼴 snapshot (감지/저장소 로드 전 null) */
let cachedSnapshot: LocalFontSnapshot | null = null;
let cachedFontRecords: LocalFontRecord[] = [];
let cachedFontLookup: LocalFontLookup = emptyLocalFontLookup();
let storageLoaded = false;
let lastStorageError: string | null = null;
/** 현재 페이지 수명 동안만 유지하는 CanvasKit용 SFNT 바이트 조회 캐시. */
const localFontBytesByPostscriptName = new Map<string, Promise<ArrayBuffer | null>>();

/** Local Font Access API 지원 여부 */
export function isLocalFontAccessSupported(): boolean {
  return typeof (globalThis as LocalFontGlobal).queryLocalFonts === 'function';
}

/** 문서 후보 글꼴 단위의 fallback probe 지원 여부 */
export function isFontPresenceProbeSupported(): boolean {
  try {
    const documentLike = (globalThis as LocalFontGlobal).document;
    const canvas = documentLike?.createElement?.('canvas') as {
      getContext?: (contextId: '2d') => unknown;
    } | null | undefined;
    const context = canvas?.getContext?.('2d') as { measureText?: unknown } | null | undefined;
    return typeof context?.measureText === 'function';
  } catch {
    return false;
  }
}

export function getLocalFontDetectionMethod(): LocalFontDetectionSource | null {
  if (isLocalFontAccessSupported()) return 'local-font-access';
  if (isFontPresenceProbeSupported()) return 'font-presence-probe';
  return null;
}

/** 로컬 글꼴 감지 지원 여부. Firefox에서는 문서 후보 글꼴 probe만 지원한다. */
export function isLocalFontSupported(): boolean {
  return getLocalFontDetectionMethod() !== null;
}

function normalizeFamilies(families: unknown): string[] {
  if (!Array.isArray(families)) return [];
  const set = new Set<string>();
  for (const family of families) {
    if (typeof family !== 'string') continue;
    const name = family.trim();
    if (name) set.add(name);
  }
  return Array.from(set).sort((a, b) => a.localeCompare(b, 'ko'));
}

function normalizeFontAlias(value: unknown): string {
  if (typeof value !== 'string') return '';
  return value
    .replace(/\u0000/g, '')
    .normalize('NFC')
    .replace(/\s+/g, ' ')
    .trim()
    .toLocaleLowerCase('en-US');
}

function normalizeFontNames(values: readonly unknown[]): string[] {
  const byAlias = new Map<string, string>();
  for (const value of values) {
    if (typeof value !== 'string') continue;
    const name = value.replace(/\u0000/g, '').normalize('NFC').replace(/\s+/g, ' ').trim();
    const alias = normalizeFontAlias(name);
    if (alias && !byAlias.has(alias)) byAlias.set(alias, name);
  }
  return Array.from(byAlias.values()).sort((a, b) => a.localeCompare(b, 'ko'));
}

interface SfntFontNames {
  families: string[];
  fullNames: string[];
  postscriptNames: string[];
  styles: string[];
}

interface LocalFontLookup {
  aliases: Map<string, LocalFontRecord[]>;
  postscriptNames: Map<string, LocalFontRecord[]>;
  fullNames: Map<string, LocalFontRecord[]>;
  familyStyles: Map<string, LocalFontRecord[]>;
  families: Map<string, LocalFontRecord[]>;
}

function emptyLocalFontLookup(): LocalFontLookup {
  return {
    aliases: new Map(),
    postscriptNames: new Map(),
    fullNames: new Map(),
    familyStyles: new Map(),
    families: new Map(),
  };
}

function addLocalFontLookupRecord(
  index: Map<string, LocalFontRecord[]>,
  name: string,
  record: LocalFontRecord,
  normalizedNameCache: Map<string, string>,
): void {
  let key = normalizedNameCache.get(name);
  if (key === undefined) {
    key = normalizeFontAlias(name);
    normalizedNameCache.set(name, key);
  }
  if (!key) return;
  const records = index.get(key);
  if (!records) {
    index.set(key, [record]);
  } else if (!records.includes(record)) {
    records.push(record);
  }
}

function buildLocalFontLookup(records: readonly LocalFontRecord[]): LocalFontLookup {
  const lookup = emptyLocalFontLookup();
  const normalizedNameCache = new Map<string, string>();
  for (const record of records) {
    for (const alias of record.aliases) {
      addLocalFontLookupRecord(lookup.aliases, alias, record, normalizedNameCache);
    }
    addLocalFontLookupRecord(lookup.postscriptNames, record.postscriptName, record, normalizedNameCache);
    addLocalFontLookupRecord(lookup.fullNames, record.fullName, record, normalizedNameCache);
    addLocalFontLookupRecord(lookup.familyStyles, `${record.family} ${record.style}`, record, normalizedNameCache);
    addLocalFontLookupRecord(lookup.families, record.family, record, normalizedNameCache);
  }
  return lookup;
}

function emptySfntFontNames(): SfntFontNames {
  return { families: [], fullNames: [], postscriptNames: [], styles: [] };
}

function byteRangeAvailable(view: DataView, offset: number, length: number): boolean {
  return Number.isSafeInteger(offset)
    && Number.isSafeInteger(length)
    && offset >= 0
    && length >= 0
    && offset <= view.byteLength
    && length <= view.byteLength - offset;
}

function sfntTag(view: DataView, offset: number): string {
  if (!byteRangeAvailable(view, offset, 4)) return '';
  return String.fromCharCode(
    view.getUint8(offset),
    view.getUint8(offset + 1),
    view.getUint8(offset + 2),
    view.getUint8(offset + 3),
  );
}

function decodeUtf16Be(bytes: Uint8Array): string {
  const codeUnits: number[] = [];
  for (let index = 0; index + 1 < bytes.length; index += 2) {
    codeUnits.push((bytes[index] << 8) | bytes[index + 1]);
  }
  let text = '';
  for (let index = 0; index < codeUnits.length; index += 4096) {
    text += String.fromCharCode(...codeUnits.slice(index, index + 4096));
  }
  return text;
}

function decodeSfntName(bytes: Uint8Array, platformId: number): string {
  if (platformId === 0 || platformId === 3) return decodeUtf16Be(bytes);
  let text = '';
  for (let index = 0; index < bytes.length; index += 4096) {
    text += String.fromCharCode(...bytes.slice(index, index + 4096));
  }
  return text;
}

/**
 * macOS legacy name record는 문자 인코딩을 platform ID만으로 확정할 수 없다.
 * 브라우저가 제공한 Unicode 이름은 유지하고, 이 record는 표시/별칭 후보에서 제외한다.
 */
function isUnicodeSfntPlatform(platformId: number): boolean {
  return platformId === 0 || platformId === 3;
}

function parseSfntNameTable(buffer: ArrayBuffer): SfntFontNames {
  const view = new DataView(buffer);
  if (!byteRangeAvailable(view, 0, 6)) return emptySfntFontNames();

  const count = view.getUint16(2, false);
  const stringOffset = view.getUint16(4, false);
  const recordsEnd = 6 + count * 12;
  if (!byteRangeAvailable(view, 6, count * 12) || stringOffset > view.byteLength || recordsEnd > view.byteLength) {
    return emptySfntFontNames();
  }

  const families: string[] = [];
  const fullNames: string[] = [];
  const postscriptNames: string[] = [];
  const styles: string[] = [];
  for (let index = 0; index < count; index += 1) {
    const recordOffset = 6 + index * 12;
    const platformId = view.getUint16(recordOffset, false);
    const nameId = view.getUint16(recordOffset + 6, false);
    const length = view.getUint16(recordOffset + 8, false);
    const relativeOffset = view.getUint16(recordOffset + 10, false);
    const valueOffset = stringOffset + relativeOffset;
    if (!byteRangeAvailable(view, valueOffset, length)) continue;
    if (!isUnicodeSfntPlatform(platformId)) continue;

    const name = decodeSfntName(
      new Uint8Array(buffer, valueOffset, length),
      platformId,
    ).replace(/\u0000/g, '').trim();
    if (!name) continue;
    if (nameId === 1 || nameId === 16) {
      families.push(name);
    } else if (nameId === 2 || nameId === 17) {
      styles.push(name);
    } else if (nameId === 4) {
      fullNames.push(name);
    } else if (nameId === 6) {
      postscriptNames.push(name);
    }
  }

  return {
    families: normalizeFontNames(families),
    fullNames: normalizeFontNames(fullNames),
    postscriptNames: normalizeFontNames(postscriptNames),
    styles: normalizeFontNames(styles),
  };
}

async function readSfntFontNames(fontData: FontData): Promise<SfntFontNames> {
  if (!fontData.blob) return emptySfntFontNames();
  try {
    const blob = await fontData.blob();
    if (blob.size < 12) return emptySfntFontNames();
    const header = new DataView(await blob.slice(0, 12).arrayBuffer());
    const tableCount = header.getUint16(4, false);
    const directoryLength = 12 + tableCount * 16;
    if (blob.size < directoryLength) return emptySfntFontNames();
    const directory = new DataView(await blob.slice(0, directoryLength).arrayBuffer());
    for (let index = 0; index < tableCount; index += 1) {
      const recordOffset = 12 + index * 16;
      if (sfntTag(directory, recordOffset) !== 'name') continue;
      const offset = directory.getUint32(recordOffset + 8, false);
      const length = directory.getUint32(recordOffset + 12, false);
      if (offset > blob.size || length > blob.size - offset) return emptySfntFontNames();
      return parseSfntNameTable(await blob.slice(offset, offset + length).arrayBuffer());
    }
  } catch {
    // 메타데이터 보강 실패는 감지 자체를 실패시키지 않고 API 기본 이름만 사용한다.
  }
  return emptySfntFontNames();
}

function preferredLocalFontDisplayName(fullNames: readonly string[], families: readonly string[]): string {
  return fullNames.find(name => HANGUL_RE.test(name))
    ?? families.find(name => HANGUL_RE.test(name))
    ?? fullNames[0]
    ?? families[0]
    ?? '';
}

/** UTF-8/UTF-16 이름을 legacy code page로 오독했을 때의 전형적인 깨짐을 제외한다. */
function isUsableFontDisplayName(value: string): boolean {
  if (!value.trim() || /[\u0000-\u001f\u007f-\u009f\ufffd]/.test(value)) return false;
  return !/[\u00ab\u00bb\u00c2\u00c3\u00d0\u00db]/.test(value);
}

function makeLocalFontRecord(fontData: Pick<FontData, 'family' | 'fullName' | 'postscriptName' | 'style'>, sfntNames: SfntFontNames = emptySfntFontNames()): LocalFontRecord | null {
  const families = normalizeFontNames([fontData.family, ...sfntNames.families]);
  const fullNames = normalizeFontNames([fontData.fullName, ...sfntNames.fullNames]);
  const postscriptNames = normalizeFontNames([fontData.postscriptName, ...sfntNames.postscriptNames]);
  const styles = normalizeFontNames([fontData.style, ...sfntNames.styles]);
  const family = normalizeFontNames([fontData.family])[0] ?? families[0] ?? fullNames[0] ?? postscriptNames[0];
  if (!family) return null;

  const aliases = normalizeFontNames([
    ...families,
    ...fullNames,
    ...postscriptNames,
    ...families.flatMap(familyName => styles.map(style => `${familyName} ${style}`)),
  ]);
  return {
    family,
    fullName: fullNames[0] ?? family,
    postscriptName: postscriptNames[0] ?? '',
    style: styles[0] ?? '',
    displayName: preferredLocalFontDisplayName(fullNames, families) || family,
    aliases,
  };
}

function normalizeLocalFontRecords(value: unknown): LocalFontRecord[] {
  if (!Array.isArray(value)) return [];
  const records = new Map<string, LocalFontRecord>();
  for (const candidate of value) {
    if (!candidate || typeof candidate !== 'object') continue;
    const data = candidate as Partial<LocalFontRecord>;
    const record = makeLocalFontRecord({
      family: typeof data.family === 'string' ? data.family : '',
      fullName: typeof data.fullName === 'string' ? data.fullName : '',
      postscriptName: typeof data.postscriptName === 'string' ? data.postscriptName : '',
      style: typeof data.style === 'string' ? data.style : '',
    }, {
      families: [],
      fullNames: [],
      postscriptNames: [],
      styles: [],
    });
    if (!record) continue;
    const aliases = normalizeFontNames([...record.aliases, ...(Array.isArray(data.aliases) ? data.aliases : [])]);
    const displayCandidates = [
      typeof data.displayName === 'string' ? data.displayName.trim() : '',
      typeof data.fullName === 'string' ? data.fullName.trim() : '',
      typeof data.family === 'string' ? data.family.trim() : '',
      ...aliases,
    ].filter(isUsableFontDisplayName);
    const displayName = displayCandidates.find(name => HANGUL_RE.test(name))
      ?? displayCandidates[0]
      ?? preferredLocalFontDisplayName(aliases.filter(name => HANGUL_RE.test(name)), [record.fullName, record.family])
      ?? record.family;
    const normalized = { ...record, displayName, aliases };
    const key = normalizeFontAlias(normalized.postscriptName || normalized.fullName || normalized.family);
    if (key && !records.has(key)) records.set(key, normalized);
  }
  return Array.from(records.values()).sort((a, b) => a.displayName.localeCompare(b.displayName, 'ko'));
}

function recordsFromFamilies(families: readonly string[]): LocalFontRecord[] {
  return normalizeLocalFontRecords(families.map(family => ({ family, fullName: family, postscriptName: '', style: '' })));
}

function snapshotRecords(snapshot: LocalFontSnapshot | null): LocalFontRecord[] {
  if (!snapshot) return [];
  return snapshot.fontRecords?.length ? snapshot.fontRecords : recordsFromFamilies(snapshot.families);
}

function cacheLocalFontSnapshot(snapshot: LocalFontSnapshot | null): void {
  cachedSnapshot = snapshot;
  cachedFontRecords = snapshotRecords(snapshot);
  cachedFontLookup = buildLocalFontLookup(cachedFontRecords);
}

async function collectLocalFontRecords(fontDataList: readonly FontData[]): Promise<LocalFontRecord[]> {
  const records: Array<LocalFontRecord | null> = new Array(fontDataList.length).fill(null);
  let nextIndex = 0;
  const worker = async (): Promise<void> => {
    while (nextIndex < fontDataList.length) {
      const index = nextIndex;
      nextIndex += 1;
      const fontData = fontDataList[index];
      records[index] = makeLocalFontRecord(fontData, await readSfntFontNames(fontData));
    }
  };
  await Promise.all(Array.from(
    { length: Math.min(LOCAL_FONT_NAME_READ_CONCURRENCY, fontDataList.length) },
    () => worker(),
  ));
  return normalizeLocalFontRecords(records.filter((record): record is LocalFontRecord => record !== null));
}

function normalizeSnapshot(value: unknown): LocalFontSnapshot | null {
  if (!value || typeof value !== 'object') return null;
  const data = value as Partial<LocalFontSnapshot>;
  if (data.source !== 'local-font-access' && data.source !== 'font-presence-probe') return null;
  if (typeof data.detectedAt !== 'string' || !data.detectedAt) return null;
  if (data.version !== 1 && data.version !== 2) return null;
  const records = data.version === 2
    ? normalizeLocalFontRecords(data.fontRecords)
    : [];
  return makeSnapshot(
    records.length > 0 ? records : recordsFromFamilies(normalizeFamilies(data.families)),
    data.source,
    data.checkedFamilies,
    data.detectedAt,
  );
}

function makeSnapshot(
  records: readonly LocalFontRecord[],
  source: LocalFontDetectionSource,
  checkedFamilies?: readonly string[],
  detectedAt = new Date().toISOString(),
): LocalFontSnapshot {
  const fontRecords = normalizeLocalFontRecords(records);
  return {
    version: 2,
    detectedAt,
    families: normalizeFamilies(fontRecords.map(record => record.family)),
    fontRecords,
    source,
    checkedFamilies: source === 'font-presence-probe'
      ? normalizeFamilies(checkedFamilies)
      : undefined,
  };
}

function cssQuoteFontFamily(name: string): string {
  return `"${name.replace(/\\/g, '\\\\').replace(/"/g, '\\"')}"`;
}

function createProbeContext(): CanvasRenderingContext2D | null {
  try {
    const documentLike = (globalThis as LocalFontGlobal).document;
    const canvas = documentLike?.createElement?.('canvas') as {
      getContext?: (contextId: '2d') => CanvasRenderingContext2D | null;
    } | null | undefined;
    return canvas?.getContext?.('2d') ?? null;
  } catch {
    return null;
  }
}

function measureWithFamily(
  context: Pick<CanvasRenderingContext2D, 'font' | 'measureText'>,
  family: string,
  text: string,
): number {
  context.font = `${PROBE_FONT_SIZE}px ${family}`;
  return context.measureText(text).width;
}

function isFamilyLikelyAvailable(
  context: Pick<CanvasRenderingContext2D, 'font' | 'measureText'>,
  family: string,
): boolean {
  const quoted = cssQuoteFontFamily(family);
  for (const fallback of PROBE_FALLBACKS) {
    for (const text of PROBE_TEXTS) {
      const baseWidth = measureWithFamily(context, fallback, text);
      const candidateWidth = measureWithFamily(context, `${quoted}, ${fallback}`, text);
      if (Math.abs(candidateWidth - baseWidth) > PROBE_WIDTH_EPSILON) {
        return true;
      }
    }
  }
  return false;
}

function probeCandidateFamilies(candidateFamilies: readonly string[]): string[] {
  const context = createProbeContext();
  if (!context) return [];
  return normalizeFamilies(candidateFamilies)
    .filter(family => !GENERIC_FONTS.has(family))
    .filter(family => !REGISTERED_FONTS.has(family))
    .filter(family => isFamilyLikelyAvailable(context, family));
}

const GENERIC_FONTS = new Set(['serif', 'sans-serif', 'monospace']);

function getChromeApi(): ChromeLike | null {
  return (globalThis as typeof globalThis & { chrome?: ChromeLike }).chrome ?? null;
}

function getChromeStorageLocal(): ChromeStorageAreaLike | null {
  return getChromeApi()?.storage?.local ?? null;
}

function getBrowserApi(): BrowserLike | null {
  return (globalThis as typeof globalThis & { browser?: BrowserLike }).browser ?? null;
}

function getBrowserStorageLocal(): ChromeStorageAreaLike | null {
  return getBrowserApi()?.storage?.local ?? null;
}

function getExtensionStorageLocal(): { kind: 'chrome-storage-local' | 'browser-storage-local'; storage: ChromeStorageAreaLike } | null {
  const chromeStorage = getChromeStorageLocal();
  if (chromeStorage) return { kind: 'chrome-storage-local', storage: chromeStorage };
  const browserStorage = getBrowserStorageLocal();
  if (browserStorage) return { kind: 'browser-storage-local', storage: browserStorage };
  return null;
}

function getStorageKind(): LocalFontStorageKind {
  const extensionStorage = getExtensionStorageLocal();
  if (extensionStorage) return extensionStorage.kind;
  try {
    if ((globalThis as typeof globalThis & { localStorage?: Storage }).localStorage) {
      return 'local-storage';
    }
  } catch {
    return 'none';
  }
  return 'none';
}

function chromeLastErrorMessage(): string | null {
  return getChromeApi()?.runtime?.lastError?.message ?? null;
}

function isThenable<T>(value: unknown): value is Promise<T> {
  return !!value && typeof (value as { then?: unknown }).then === 'function';
}

function chromeGet(storage: ChromeStorageAreaLike, key: string): Promise<Record<string, unknown>> {
  return new Promise((resolve, reject) => {
    let settled = false;
    const settle = (fn: () => void) => {
      if (settled) return;
      settled = true;
      fn();
    };
    try {
      const result = storage.get(key, (items) => {
        const err = chromeLastErrorMessage();
        if (err) {
          settle(() => reject(new Error(err)));
        } else {
          settle(() => resolve(items ?? {}));
        }
      });
      if (isThenable<Record<string, unknown>>(result)) {
        result.then(
          (items) => settle(() => resolve(items ?? {})),
          (error) => settle(() => reject(error)),
        );
      }
    } catch (error) {
      settle(() => reject(error));
    }
  });
}

function chromeSet(storage: ChromeStorageAreaLike, items: Record<string, unknown>): Promise<void> {
  return new Promise((resolve, reject) => {
    let settled = false;
    const settle = (fn: () => void) => {
      if (settled) return;
      settled = true;
      fn();
    };
    try {
      const result = storage.set(items, () => {
        const err = chromeLastErrorMessage();
        if (err) {
          settle(() => reject(new Error(err)));
        } else {
          settle(() => resolve());
        }
      });
      if (isThenable<void>(result)) {
        result.then(
          () => settle(() => resolve()),
          (error) => settle(() => reject(error)),
        );
      }
    } catch (error) {
      settle(() => reject(error));
    }
  });
}

function chromeRemove(storage: ChromeStorageAreaLike, key: string): Promise<void> {
  return new Promise((resolve, reject) => {
    let settled = false;
    const settle = (fn: () => void) => {
      if (settled) return;
      settled = true;
      fn();
    };
    try {
      const result = storage.remove(key, () => {
        const err = chromeLastErrorMessage();
        if (err) {
          settle(() => reject(new Error(err)));
        } else {
          settle(() => resolve());
        }
      });
      if (isThenable<void>(result)) {
        result.then(
          () => settle(() => resolve()),
          (error) => settle(() => reject(error)),
        );
      }
    } catch (error) {
      settle(() => reject(error));
    }
  });
}

function localStorageRef(): Storage | null {
  try {
    return (globalThis as typeof globalThis & { localStorage?: Storage }).localStorage ?? null;
  } catch {
    return null;
  }
}

async function readStoredSnapshot(): Promise<LocalFontSnapshot | null> {
  lastStorageError = null;
  const extensionStorage = getExtensionStorageLocal();
  if (extensionStorage) {
    try {
      const data = await chromeGet(extensionStorage.storage, STORAGE_KEY);
      return normalizeSnapshot(data[STORAGE_KEY]);
    } catch (error) {
      lastStorageError = error instanceof Error ? error.message : String(error);
      return null;
    }
  }

  const storage = localStorageRef();
  if (!storage) return null;
  try {
    const raw = storage.getItem(STORAGE_KEY);
    return raw ? normalizeSnapshot(JSON.parse(raw)) : null;
  } catch (error) {
    lastStorageError = error instanceof Error ? error.message : String(error);
    return null;
  }
}

async function writeStoredSnapshot(snapshot: LocalFontSnapshot): Promise<void> {
  lastStorageError = null;
  const extensionStorage = getExtensionStorageLocal();
  if (extensionStorage) {
    try {
      await chromeSet(extensionStorage.storage, { [STORAGE_KEY]: snapshot });
    } catch (error) {
      lastStorageError = error instanceof Error ? error.message : String(error);
    }
    return;
  }

  const storage = localStorageRef();
  if (!storage) return;
  try {
    storage.setItem(STORAGE_KEY, JSON.stringify(snapshot));
  } catch (error) {
    lastStorageError = error instanceof Error ? error.message : String(error);
  }
}

async function removeStoredSnapshot(): Promise<void> {
  lastStorageError = null;
  const extensionStorage = getExtensionStorageLocal();
  if (extensionStorage) {
    try {
      await chromeRemove(extensionStorage.storage, STORAGE_KEY);
    } catch (error) {
      lastStorageError = error instanceof Error ? error.message : String(error);
    }
    return;
  }

  const storage = localStorageRef();
  if (!storage) return;
  try {
    storage.removeItem(STORAGE_KEY);
  } catch (error) {
    lastStorageError = error instanceof Error ? error.message : String(error);
  }
}

/**
 * 저장된 로컬 글꼴 감지 결과를 로드한다.
 * 이 함수는 queryLocalFonts()를 호출하지 않는다.
 */
export async function loadStoredLocalFonts(): Promise<LocalFontSnapshot | null> {
  const snapshot = await readStoredSnapshot();
  cacheLocalFontSnapshot(snapshot);
  storageLoaded = true;
  return snapshot;
}

/** 저장된 로컬 글꼴 감지 결과와 런타임 캐시를 초기화한다. */
export async function clearStoredLocalFonts(): Promise<void> {
  cacheLocalFontSnapshot(null);
  storageLoaded = true;
  localFontBytesByPostscriptName.clear();
  await removeStoredSnapshot();
}

/**
 * 로컬 글꼴을 감지하여 family 목록을 반환한다.
 * - 중복 제거, 한국어 로케일 정렬
 * - 기본 반환값은 기존 UI 호환을 위해 REGISTERED_FONTS에 이미 등록된 글꼴을 제외
 * - includeRegistered=true면 문서 상태 분석용 전체 family 목록을 반환
 * - 캐시/저장소 결과가 있으면 권한 프롬프트 없이 즉시 반환
 */
export async function detectLocalFonts(options: DetectLocalFontsOptions = {}): Promise<string[]> {
  if (!options.force) {
    if (cachedSnapshot) return getLocalFonts({ includeRegistered: options.includeRegistered });
    if (!storageLoaded) {
      await loadStoredLocalFonts();
      if (cachedSnapshot) return getLocalFonts({ includeRegistered: options.includeRegistered });
    }
  }

  let snapshot: LocalFontSnapshot | null = null;
  if (isLocalFontAccessSupported()) {
    const queryLocalFonts = (globalThis as LocalFontGlobal).queryLocalFonts!;
    const fontDataList = await queryLocalFonts();
    snapshot = makeSnapshot(await collectLocalFontRecords(fontDataList), 'local-font-access');
  } else if (isFontPresenceProbeSupported() && options.candidateFamilies?.length) {
    const checkedFamilies = normalizeFamilies(options.candidateFamilies);
    const families = probeCandidateFamilies(checkedFamilies);
    snapshot = makeSnapshot(recordsFromFamilies(families), 'font-presence-probe', checkedFamilies);
  }

  if (!snapshot) return [];

  localFontBytesByPostscriptName.clear();
  cacheLocalFontSnapshot(snapshot);
  storageLoaded = true;
  await writeStoredSnapshot(snapshot);
  console.log(`[LocalFonts] ${cachedFontRecords.length}개 로컬 글꼴 감지됨 (${snapshot.source})`);
  return getLocalFonts({ includeRegistered: options.includeRegistered });
}

/** 캐시된 로컬 글꼴 face 레코드를 반환한다. */
export function getLocalFontRecords(options: GetLocalFontsOptions = {}): LocalFontRecord[] {
  if (options.includeRegistered) return [...cachedFontRecords];
  return cachedFontRecords.filter(record => !record.aliases.some(name => REGISTERED_FONTS.has(name)));
}

/** 캐시된 로컬 글꼴 목록을 동기적으로 반환 (감지 전이면 빈 배열) */
export function getLocalFonts(options: GetLocalFontsOptions = {}): string[] {
  return normalizeFamilies(getLocalFontRecords(options).map(record => record.displayName));
}

/** 캐시된 전체 로컬 글꼴 목록을 반환한다. */
export function getDetectedLocalFonts(): string[] {
  return cachedSnapshot ? [...cachedSnapshot.families] : [];
}

/** HWP/CSS 글꼴명에서 동일한 설치 글꼴 face를 찾는다. */
export function resolveLocalFont(fontName: string): LocalFontRecord | null {
  const target = normalizeFontAlias(fontName);
  if (!target) return null;
  const matches = cachedFontLookup.aliases.get(target) ?? [];
  if (matches.length === 0) return null;

  const uniqueMatch = (records: readonly LocalFontRecord[] | undefined): LocalFontRecord | null =>
    records?.length === 1 ? records[0] : null;
  return uniqueMatch(cachedFontLookup.postscriptNames.get(target))
    ?? uniqueMatch(cachedFontLookup.fullNames.get(target))
    ?? uniqueMatch(cachedFontLookup.familyStyles.get(target))
    // family만으로 여러 style face가 매칭되면 임의 face를 고르지 않는다.
    ?? uniqueMatch(cachedFontLookup.families.get(target))
    ?? (matches.length === 1 ? matches[0] : null);
}

/** CSS family와 달리 style별 native Typeface cache를 구분하는 안정 키다. */
export function localFontFaceKey(record: Pick<LocalFontRecord, 'family' | 'fullName' | 'postscriptName'>): string {
  return normalizeFontAlias(record.postscriptName || record.fullName || record.family);
}

function localFontRecordMatchesFontData(record: LocalFontRecord, fontData: FontData): boolean {
  const expectedPostscriptName = normalizeFontAlias(record.postscriptName);
  const actualPostscriptName = normalizeFontAlias(fontData.postscriptName);
  if (expectedPostscriptName && actualPostscriptName) {
    return expectedPostscriptName === actualPostscriptName;
  }
  const names = [fontData.family, fontData.fullName, fontData.postscriptName]
    .map(normalizeFontAlias)
    .filter(Boolean);
  const aliases = new Set(record.aliases.map(normalizeFontAlias));
  return names.some(name => aliases.has(name));
}

async function readLocalFontBytesBatch(records: readonly LocalFontRecord[]): Promise<Map<string, ArrayBuffer>> {
  const bytesByPostscriptName = new Map<string, ArrayBuffer>();
  if (cachedSnapshot?.source !== 'local-font-access') return bytesByPostscriptName;
  const queryLocalFonts = (globalThis as LocalFontGlobal).queryLocalFonts;
  const postscriptNames = normalizeFontNames(records.map(record => record.postscriptName));
  if (!queryLocalFonts || postscriptNames.length === 0) return bytesByPostscriptName;

  try {
    const candidates = await queryLocalFonts({ postscriptNames });
    await Promise.all(records.map(async (record) => {
      const fontData = candidates.find(candidate => localFontRecordMatchesFontData(record, candidate));
      if (!fontData?.blob) return;
      const bytes = await (await fontData.blob()).arrayBuffer();
      bytesByPostscriptName.set(normalizeFontAlias(record.postscriptName), bytes);
    }));
  } catch (error) {
    // 이미 승인된 글꼴을 다시 읽지 못해도 기본 Typeface fallback으로 계속 렌더한다.
    console.warn('[LocalFonts] CanvasKit용 SFNT 바이트 일괄 조회 실패:', error);
  }
  return bytesByPostscriptName;
}

/**
 * CanvasKit이 현재 문서의 local face를 등록할 때만 원본 SFNT 바이트를 일괄 조회한다.
 * 바이트는 localStorage에 저장하지 않고, 같은 PostScript face는 현재 페이지에서 한 번만 읽는다.
 */
export async function loadLocalFontBytesFor(fontNames: readonly string[]): Promise<Map<string, ArrayBuffer>> {
  const recordsByPostscriptName = new Map<string, LocalFontRecord>();
  for (const fontName of fontNames) {
    const record = resolveLocalFont(fontName);
    if (!record?.postscriptName) continue;
    recordsByPostscriptName.set(normalizeFontAlias(record.postscriptName), record);
  }

  const missing = Array.from(recordsByPostscriptName.entries())
    .filter(([postscriptName]) => !localFontBytesByPostscriptName.has(postscriptName));
  if (missing.length > 0) {
    const records = missing.map(([, record]) => record);
    const batch = readLocalFontBytesBatch(records);
    for (const [postscriptName] of missing) {
      localFontBytesByPostscriptName.set(
        postscriptName,
        batch.then(bytesByPostscriptName => bytesByPostscriptName.get(postscriptName) ?? null),
      );
    }
  }

  const result = new Map<string, ArrayBuffer>();
  for (const [postscriptName, record] of recordsByPostscriptName) {
    const bytes = await localFontBytesByPostscriptName.get(postscriptName);
    if (bytes) result.set(localFontFaceKey(record), bytes);
  }
  return result;
}

/** 단일 face 요청도 일괄 조회 cache를 경유하는 편의 함수다. */
export async function loadLocalFontBytes(fontName: string): Promise<ArrayBuffer | null> {
  const record = resolveLocalFont(fontName);
  if (!record) return null;
  return (await loadLocalFontBytesFor([fontName])).get(localFontFaceKey(record)) ?? null;
}

/** 현재 로컬 글꼴 감지/저장 상태를 반환한다. */
export function getLocalFontState(): LocalFontState {
  const method = getLocalFontDetectionMethod();
  const complete = cachedSnapshot?.source === 'local-font-access';
  const checkedFamilies = cachedSnapshot?.source === 'font-presence-probe'
    ? (cachedSnapshot.checkedFamilies ?? [])
    : (complete ? (cachedSnapshot?.families ?? []) : []);
  return {
    supported: method !== null,
    method,
    loaded: storageLoaded,
    stored: cachedSnapshot !== null,
    source: cachedSnapshot?.source ?? null,
    complete,
    storage: getStorageKind(),
    count: cachedSnapshot?.families.length ?? 0,
    checkedFamilies,
    detectedAt: cachedSnapshot?.detectedAt ?? null,
    lastError: lastStorageError,
  };
}

/** 테스트 전용: 모듈 내부 캐시를 초기화한다. */
export function resetLocalFontsForTests(): void {
  cacheLocalFontSnapshot(null);
  storageLoaded = false;
  lastStorageError = null;
  localFontBytesByPostscriptName.clear();
}
