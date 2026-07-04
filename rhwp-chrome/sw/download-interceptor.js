// 다운로드 관찰자 (Chrome)
// - .hwp/.hwpx 다운로드 감지 → 뷰어로 열기
// - 사용자 설정(autoOpen)에 따라 동작
//
// #198 (chrome-fd-001): HWP 가 아닌 일반 파일 다운로드에는 suggest() 를 호출하지 않아
//                       Chrome 의 마지막 저장 위치 기억 동작을 보존한다.
// #207: 판정 로직은 rhwp-shared/sw/download-interceptor-common.js 와 공유.
// #1131: 로컬 file:// HWP 는 이미 디스크에 있으므로 자체 뷰어로 열 때 중복 다운로드를
//        억제한다 (cancel + erase, best-effort). 원격(http) 파일은 기존 동작 유지.
// #1471: onDeterminingFilename 리스너 등록 자체가 다른 확장의 download({filename})
//        경로 결정을 무효화하므로 filename 결정 단계에서 완전히 빠진다.

import { openViewer } from './viewer-launcher.js';
import { shouldInterceptDownload } from './download-interceptor-common.js';
import {
  DEFAULT_STATE_TTL_MS,
  evaluateDownloadChanged,
  evaluateDownloadCreated,
  isDownloadStateExpired,
  isTerminalDelta,
  markDownloadHandled,
  markDownloadTerminal,
  shouldRecheckDownload,
} from './download-observer-state.js';

const STORAGE_PREFIX = 'rhwpDownloadState:';
const TERMINAL_CLEANUP_MS = 30_000;
const memoryStateFallback = new Map();

/** 다운로드 항목이 로컬 file:// 인지 판별. */
function isLocalFileDownload(item) {
  return typeof item?.url === 'string' && item.url.startsWith('file:');
}

/**
 * 다운로드 관찰자를 설정한다.
 *
 * - 로컬 file:// HWP: 뷰어를 열고 다운로드는 cancel + erase 로 억제 (#1131)
 * - 원격 HWP/HWPX 다운로드: 뷰어 트리거
 * - 일반 파일: filename 결정 단계에 참여하지 않음 (#1471)
 */
export function setupDownloadInterceptor() {
  chrome.downloads.onCreated.addListener((item) => {
    void handleCreated(item);
  });

  chrome.downloads.onChanged.addListener(async (delta) => {
    await handleChanged(delta);
  });
}

async function handleCreated(item) {
  const now = Date.now();
  const previousState = await getDownloadState(item?.id, now);
  const decision = evaluateDownloadCreated(item, previousState, now);

  if (decision.action !== 'track') return;
  await setDownloadState(decision.state);
  await processDownloadCandidate(item, decision.state);
}

async function handleChanged(delta) {
  const now = Date.now();
  const state = await getDownloadState(delta?.id, now);

  if (state && !state.handledAt && shouldRecheckDownload(delta)) {
    try {
      const [item] = await chrome.downloads.search({ id: delta.id });
      const decision = evaluateDownloadChanged(delta, item, state, now);
      if (decision.action === 'candidate') {
        await setDownloadState(decision.state);
        await processDownloadCandidate(item, decision.state);
      }
    } catch (err) {
      console.error('[rhwp] 다운로드 항목 재조회 오류:', err);
    }
  }

  if (isTerminalDelta(delta)) {
    const terminalState = markDownloadTerminal(state, now);
    if (terminalState) {
      await setDownloadState(terminalState);
    }
    scheduleRemoveDownloadState(delta.id);
  }
}

async function processDownloadCandidate(item, state) {
  if (!item || state?.handledAt) return;
  if (!shouldInterceptDownload(item)) return;

  try {
    const settings = await chrome.storage.sync.get({ autoOpen: true });
    const reason = settings.autoOpen ? 'opened' : 'auto-open-disabled';
    await setDownloadState(markDownloadHandled(state, Date.now(), reason));
    if (!settings.autoOpen) return;

    handleHwpDownload(item);

    if (isLocalFileDownload(item)) {
      void suppressLocalDownload(item);
    }
  } catch (err) {
    console.error('[rhwp] 다운로드 인터셉터 오류:', err);
  }
}

/**
 * 로컬 file:// 다운로드를 취소하고 다운로드 목록에서 제거한다 (#1131, best-effort).
 *
 * 로컬 복사는 거의 즉시 완료되어 cancel 이 늦을 수 있으나, erase 로 항목을 정리한다.
 * 실패해도 뷰어 동작에는 영향이 없으므로 예외는 무시한다.
 */
async function suppressLocalDownload(item) {
  try {
    await chrome.downloads.cancel(item.id);
  } catch {
    // 이미 완료/취소됨 — 무시하고 erase 로 진행
  }
  try {
    await chrome.downloads.erase({ id: item.id });
  } catch {
    // 항목 제거 실패는 치명적이지 않음 — 무시
  }
}

function handleHwpDownload(item) {
  // 대용량 파일 경고 (50MB 초과)
  if (item.fileSize > 50 * 1024 * 1024) {
    console.warn(`[rhwp] 대용량 파일: ${item.filename} (${(item.fileSize / 1024 / 1024).toFixed(1)}MB)`);
  }

  openViewer({
    url: item.url,
    filename: item.filename,
  });
}

function stateKey(id) {
  return `${STORAGE_PREFIX}${id}`;
}

function getSessionStorage() {
  return chrome.storage?.session || null;
}

async function getDownloadState(id, now = Date.now()) {
  if (typeof id !== 'number') return null;

  const key = stateKey(id);
  const session = getSessionStorage();
  let state = null;

  if (session) {
    const result = await session.get(key);
    state = result?.[key] || null;
  } else {
    state = memoryStateFallback.get(id) || null;
  }

  if (state && isDownloadStateExpired(state, now, DEFAULT_STATE_TTL_MS)) {
    await removeDownloadState(id);
    return null;
  }

  return state;
}

async function setDownloadState(state) {
  if (!state || typeof state.id !== 'number') return;

  const session = getSessionStorage();
  if (session) {
    await session.set({ [stateKey(state.id)]: state });
    return;
  }

  memoryStateFallback.set(state.id, state);
}

async function removeDownloadState(id) {
  if (typeof id !== 'number') return;

  const session = getSessionStorage();
  if (session) {
    await session.remove(stateKey(id));
    return;
  }

  memoryStateFallback.delete(id);
}

function scheduleRemoveDownloadState(id) {
  if (typeof id !== 'number') return;

  const session = getSessionStorage();
  const key = stateKey(id);
  const timer = setTimeout(() => {
    if (session) {
      void session.remove(key);
      return;
    }
    memoryStateFallback.delete(id);
  }, TERMINAL_CLEANUP_MS);
  if (typeof timer?.unref === 'function') timer.unref();
}
