// Chrome/Edge 확장 사용자 설정의 단일 저장소 adapter.
// storage.sync 를 key별 권위 저장소로 사용하고, 누락/일시 실패 시 storage.local snapshot 으로 복구한다.

export const SETTINGS_SCHEMA_VERSION = 1;
export const LOCAL_BACKUP_KEY = 'rhwpSettingsBackup';
export const SYNC_META_KEY = 'rhwpSettingsMeta';

export const SETTINGS_KEYS = Object.freeze([
  'autoOpen',
  'showBadges',
  'hoverPreview',
  'disableExternalWebFonts',
]);

export const DEFAULT_SETTINGS = Object.freeze({
  autoOpen: true,
  showBadges: true,
  hoverPreview: true,
  disableExternalWebFonts: false,
});

/**
 * sync 값을 우선하고, 유효한 boolean sync key 가 없을 때만 local snapshot 을 사용한다.
 * sync 를 읽을 수 없는 동시에 local snapshot 도 없으면 기본값으로 조용히 진행하지 않고 실패한다.
 */
export async function loadSettings(chromeApi = globalThis.chrome, options = {}) {
  const resolved = await resolveSettings(chromeApi, options);
  return resolved.settings;
}

/**
 * 탭 생성처럼 사용자가 원치 않을 수 있는 자동 동작은 sync 상태를 확인할 수 있을 때만 허용한다.
 * sync read 실패 또는 기존 설치 정황이 있는 partial sync에서 autoOpen을 확인할 수 없으면
 * fail-closed 처리한다. clean install이나 유효한 local snapshot은 기존 기본/복구 계약을 유지한다.
 */
export async function loadSettingsForAutomaticActions(chromeApi = globalThis.chrome, options = {}) {
  const resolved = await resolveSettings(chromeApi, options);
  if (!resolved.syncReadable) {
    return { ...resolved.settings, autoOpen: false };
  }
  if (resolved.syncAutoOpenKnown || resolved.localSnapshotKnown || !resolved.hasSyncEvidence) {
    return resolved.settings;
  }
  return { ...resolved.settings, autoOpen: false };
}

async function resolveSettings(chromeApi, options) {
  const now = options.now ?? Date.now;
  const syncArea = requireStorageArea(chromeApi, 'sync');
  const localArea = requireStorageArea(chromeApi, 'local');
  const [syncResult, localResult] = await Promise.all([
    // 기존 배포본의 legacy key도 설치 이력 신호이므로 전체 sync payload를 확인한다.
    readStorage(syncArea, null),
    readStorage(localArea, LOCAL_BACKUP_KEY),
  ]);

  const localSnapshot = localResult.ok
    ? normalizeSnapshot(localResult.items[LOCAL_BACKUP_KEY])
    : null;
  const syncAutoOpenKnown = syncResult.ok
    && typeof syncResult.items.autoOpen === 'boolean';
  const hasSyncEvidence = syncResult.ok
    && Object.keys(syncResult.items).length > 0;

  if (!syncResult.ok && !localSnapshot) {
    throw new Error('확장 설정을 불러오지 못했습니다.', { cause: syncResult.error ?? localResult.error });
  }

  const settings = {};
  for (const key of SETTINGS_KEYS) {
    if (syncResult.ok && typeof syncResult.items[key] === 'boolean') {
      settings[key] = syncResult.items[key];
    } else if (localSnapshot && typeof localSnapshot.settings[key] === 'boolean') {
      settings[key] = localSnapshot.settings[key];
    } else {
      settings[key] = DEFAULT_SETTINGS[key];
    }
  }

  // 기존 설치의 partial sync를 기본값으로 채운 결과는 last-known-good가 아니다.
  // 유효한 autoOpen 근거가 생길 때까지 local에 default true를 굳히지 않는다.
  const canCreateTrustworthySnapshot = localSnapshot
    || syncAutoOpenKnown
    || !hasSyncEvidence;
  if (localResult.ok
    && canCreateTrustworthySnapshot
    && !snapshotMatches(localSnapshot, settings)) {
    const syncMeta = syncResult.ok ? normalizeMeta(syncResult.items[SYNC_META_KEY]) : null;
    const snapshot = createSnapshot(settings, syncMeta?.updatedAt ?? now());
    try {
      await localArea.set({ [LOCAL_BACKUP_KEY]: snapshot });
    } catch (error) {
      console.warn('[rhwp-settings] local backup 갱신 실패:', error);
    }
  }

  return {
    settings,
    syncReadable: syncResult.ok,
    syncAutoOpenKnown,
    localSnapshotKnown: Boolean(localSnapshot),
    hasSyncEvidence,
  };
}

/**
 * 권위 저장소인 sync 기록에 실패하면 rejection 을 전달한다. local snapshot 은 복구용
 * best-effort 백업이므로 실패해도 이미 성공한 sync 저장을 실패로 되돌리지 않는다.
 */
export async function saveSettings(chromeApi = globalThis.chrome, candidate, options = {}) {
  const now = options.now ?? Date.now;
  const settings = normalizeCompleteSettings(candidate);
  const updatedAt = now();
  const localArea = requireStorageArea(chromeApi, 'local');
  const syncArea = requireStorageArea(chromeApi, 'sync');

  await syncArea.set({
    ...settings,
    [SYNC_META_KEY]: {
      schemaVersion: SETTINGS_SCHEMA_VERSION,
      updatedAt,
    },
  });
  try {
    await localArea.set({ [LOCAL_BACKUP_KEY]: createSnapshot(settings, updatedAt) });
  } catch (error) {
    console.warn('[rhwp-settings] local backup 저장 실패:', error);
  }

  return settings;
}

function requireStorageArea(chromeApi, name) {
  const area = chromeApi?.storage?.[name];
  if (!area || typeof area.get !== 'function' || typeof area.set !== 'function') {
    throw new Error(`chrome.storage.${name}을 사용할 수 없습니다.`);
  }
  return area;
}

async function readStorage(area, keys) {
  try {
    return { ok: true, items: (await area.get(keys)) ?? {}, error: null };
  } catch (error) {
    return { ok: false, items: {}, error };
  }
}

function normalizeCompleteSettings(candidate) {
  const settings = {};
  for (const key of SETTINGS_KEYS) {
    settings[key] = typeof candidate?.[key] === 'boolean'
      ? candidate[key]
      : DEFAULT_SETTINGS[key];
  }
  return settings;
}

function normalizeSnapshot(value) {
  if (!value || value.schemaVersion !== SETTINGS_SCHEMA_VERSION || !Number.isFinite(value.updatedAt)) {
    return null;
  }
  if (!value.settings || typeof value.settings !== 'object') return null;
  return {
    schemaVersion: SETTINGS_SCHEMA_VERSION,
    updatedAt: value.updatedAt,
    settings: normalizeCompleteSettings(value.settings),
  };
}

function normalizeMeta(value) {
  if (!value || value.schemaVersion !== SETTINGS_SCHEMA_VERSION || !Number.isFinite(value.updatedAt)) {
    return null;
  }
  return { schemaVersion: SETTINGS_SCHEMA_VERSION, updatedAt: value.updatedAt };
}

function createSnapshot(settings, updatedAt) {
  return {
    schemaVersion: SETTINGS_SCHEMA_VERSION,
    updatedAt,
    settings: { ...settings },
  };
}

function snapshotMatches(snapshot, settings) {
  if (!snapshot) return false;
  return SETTINGS_KEYS.every((key) => snapshot.settings[key] === settings[key]);
}
