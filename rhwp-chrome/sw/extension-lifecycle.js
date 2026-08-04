import { loadSettings } from './settings-store.js';

export const LIFECYCLE_KEY = 'rhwpSettingsLifecycle';

/**
 * update/chrome_update 직전의 유효한 설정은 local snapshot으로 선보존하고,
 * 사용자 설정과 분리된 최소 수명주기 진단을 local storage에 기록한다.
 * install/update/chrome_update 어느 경로에서도 sync 사용자 설정은 수정하지 않는다.
 */
export async function handleExtensionInstalled(chromeApi = globalThis.chrome, details = {}, options = {}) {
  const now = options.now ?? Date.now;
  if (details.reason === 'update' || details.reason === 'chrome_update') {
    try {
      await loadSettings(chromeApi, { now });
    } catch (error) {
      console.warn('[rhwp-settings] 업데이트 전 설정 snapshot 보존 실패:', error);
    }
  }
  const record = {
    reason: details.reason ?? 'unknown',
    previousVersion: details.previousVersion ?? null,
    currentVersion: chromeApi.runtime.getManifest().version,
    recordedAt: now(),
  };
  await chromeApi.storage.local.set({ [LIFECYCLE_KEY]: record });
  return record;
}
