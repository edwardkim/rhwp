/**
 * 확장 페이지에서 공유하는 viewer 설정을 읽는다.
 *
 * 일반 web/PWA 환경에서는 확장 storage API가 없으므로 기본값을 반환한다.
 */

export interface ExtensionViewerSettings {
  disableExternalWebFonts: boolean;
}

const DEFAULT_SETTINGS: ExtensionViewerSettings = {
  disableExternalWebFonts: false,
};

type StorageItems = Record<string, unknown>;

const DEFAULT_STORAGE_ITEMS: StorageItems = {
  disableExternalWebFonts: DEFAULT_SETTINGS.disableExternalWebFonts,
};

interface StorageAreaLike {
  get(
    keys?: string | string[] | StorageItems | null,
    callback?: (items: StorageItems) => void,
  ): Promise<StorageItems> | void;
}

interface BrowserLike {
  storage?: {
    sync?: StorageAreaLike;
    local?: StorageAreaLike;
  };
  runtime?: {
    lastError?: {
      message?: string;
    };
  };
}

function getChromeApi(): BrowserLike | null {
  return (globalThis as typeof globalThis & { chrome?: BrowserLike }).chrome ?? null;
}

function getBrowserApi(): BrowserLike | null {
  return (globalThis as typeof globalThis & { browser?: BrowserLike }).browser ?? null;
}

function chromeLastErrorMessage(): string | null {
  return getChromeApi()?.runtime?.lastError?.message ?? null;
}

function isThenable<T>(value: unknown): value is Promise<T> {
  return !!value && typeof (value as { then?: unknown }).then === 'function';
}

function chromeStorageGet(storage: StorageAreaLike, defaults: StorageItems): Promise<StorageItems> {
  return new Promise((resolve, reject) => {
    let settled = false;
    const settle = (fn: () => void) => {
      if (settled) return;
      settled = true;
      fn();
    };
    try {
      const result = storage.get(defaults, (items) => {
        const err = chromeLastErrorMessage();
        if (err) {
          settle(() => reject(new Error(err)));
        } else {
          settle(() => resolve(items ?? {}));
        }
      });
      if (isThenable<StorageItems>(result)) {
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

async function promiseStorageGet(storage: StorageAreaLike, defaults: StorageItems): Promise<StorageItems> {
  const result = storage.get(defaults);
  if (isThenable<StorageItems>(result)) {
    return result ?? {};
  }
  return {};
}

function normalizeSettings(value: StorageItems): ExtensionViewerSettings {
  return {
    disableExternalWebFonts: value.disableExternalWebFonts === true,
  };
}

async function readStorage(
  storage: StorageAreaLike | undefined,
  mode: 'chrome-callback' | 'promise',
): Promise<ExtensionViewerSettings | null> {
  if (!storage) return null;
  try {
    const items = mode === 'chrome-callback'
      ? await chromeStorageGet(storage, DEFAULT_STORAGE_ITEMS)
      : await promiseStorageGet(storage, DEFAULT_STORAGE_ITEMS);
    return normalizeSettings(items);
  } catch (error) {
    console.warn('[extension-settings] 확장 설정 로드 실패:', error);
    return null;
  }
}

/**
 * 확장 viewer 설정을 읽는다.
 *
 * Chrome/Firefox는 기존 options가 storage.sync를 사용하고, Safari는 storage.local을
 * 사용한다. sync를 먼저 확인하고 local을 fallback으로 둔다.
 */
export async function loadExtensionViewerSettings(): Promise<ExtensionViewerSettings> {
  const chromeStorage = getChromeApi()?.storage;
  const browserStorage = getBrowserApi()?.storage;

  return (
    await readStorage(chromeStorage?.sync, 'chrome-callback') ??
    await readStorage(browserStorage?.sync, 'promise') ??
    await readStorage(browserStorage?.local, 'promise') ??
    await readStorage(chromeStorage?.local, 'chrome-callback') ??
    DEFAULT_SETTINGS
  );
}
