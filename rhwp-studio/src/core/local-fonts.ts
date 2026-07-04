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
}

export type LocalFontDetectionSource = 'local-font-access' | 'font-presence-probe';

export interface LocalFontSnapshot {
  version: 1;
  detectedAt: string;
  families: string[];
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
  queryLocalFonts?: () => Promise<FontData[]>;
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

/** 캐시된 로컬 글꼴 snapshot (감지/저장소 로드 전 null) */
let cachedSnapshot: LocalFontSnapshot | null = null;
let storageLoaded = false;
let lastStorageError: string | null = null;

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

function normalizeSnapshot(value: unknown): LocalFontSnapshot | null {
  if (!value || typeof value !== 'object') return null;
  const data = value as Partial<LocalFontSnapshot>;
  if (data.version !== 1) return null;
  if (data.source !== 'local-font-access' && data.source !== 'font-presence-probe') return null;
  if (typeof data.detectedAt !== 'string' || !data.detectedAt) return null;
  const families = normalizeFamilies(data.families);
  return {
    version: 1,
    detectedAt: data.detectedAt,
    families,
    source: data.source,
    checkedFamilies: data.source === 'font-presence-probe'
      ? normalizeFamilies(data.checkedFamilies)
      : undefined,
  };
}

function makeSnapshot(
  families: string[],
  source: LocalFontDetectionSource,
  checkedFamilies?: readonly string[],
): LocalFontSnapshot {
  return {
    version: 1,
    detectedAt: new Date().toISOString(),
    families: normalizeFamilies(families),
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
  cachedSnapshot = snapshot;
  storageLoaded = true;
  return snapshot;
}

/** 저장된 로컬 글꼴 감지 결과와 런타임 캐시를 초기화한다. */
export async function clearStoredLocalFonts(): Promise<void> {
  cachedSnapshot = null;
  storageLoaded = true;
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
    const families = normalizeFamilies(fontDataList.map(fd => fd.family));
    snapshot = makeSnapshot(families, 'local-font-access');
  } else if (isFontPresenceProbeSupported() && options.candidateFamilies?.length) {
    const checkedFamilies = normalizeFamilies(options.candidateFamilies);
    const families = probeCandidateFamilies(checkedFamilies);
    snapshot = makeSnapshot(families, 'font-presence-probe', checkedFamilies);
  }

  if (!snapshot) return [];

  cachedSnapshot = snapshot;
  storageLoaded = true;
  await writeStoredSnapshot(snapshot);
  console.log(`[LocalFonts] ${snapshot.families.length}개 로컬 글꼴 감지됨 (${snapshot.source})`);
  return getLocalFonts({ includeRegistered: options.includeRegistered });
}

/** 캐시된 로컬 글꼴 목록을 동기적으로 반환 (감지 전이면 빈 배열) */
export function getLocalFonts(options: GetLocalFontsOptions = {}): string[] {
  const families = cachedSnapshot?.families ?? [];
  if (options.includeRegistered) return [...families];
  return families.filter(name => !REGISTERED_FONTS.has(name));
}

/** 캐시된 전체 로컬 글꼴 목록을 반환한다. */
export function getDetectedLocalFonts(): string[] {
  return getLocalFonts({ includeRegistered: true });
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
  cachedSnapshot = null;
  storageLoaded = false;
  lastStorageError = null;
}
