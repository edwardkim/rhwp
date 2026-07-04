// 다운로드 관찰자 (Firefox)
// - .hwp/.hwpx 다운로드 감지 → 뷰어로 열기
// - 사용자 설정(autoOpen)에 따라 동작
// - browser.downloads.search 로 최신 DownloadItem 재조회
//
// #207: 판정 로직은 rhwp-shared/sw/download-interceptor-common.js 와 공유.
// #1516: 공통 다운로드 관찰자 상태 머신으로 과거 항목/중복 이벤트를 판정한다.

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

export function setupDownloadInterceptor() {
  browser.downloads.onCreated.addListener((item) => {
    void handleCreated(item);
  });

  browser.downloads.onChanged.addListener((delta) => {
    void handleChanged(delta);
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
  let state = await getDownloadState(delta?.id, now);

  if (state && !state.handledAt && shouldRecheckDownload(delta)) {
    try {
      const [item] = await browser.downloads.search({ id: delta.id });
      const decision = evaluateDownloadChanged(delta, item, state, now);
      if (decision.action === 'candidate') {
        state = decision.state;
        await setDownloadState(state);
        state = await processDownloadCandidate(item, state);
      }
    } catch (err) {
      console.error('[rhwp] 다운로드 항목 재조회 오류:', err);
    }
  }

  if (isTerminalDelta(delta)) {
    const terminalState = markDownloadTerminal(state, Date.now());
    if (terminalState) {
      await setDownloadState(terminalState);
    }
    scheduleRemoveDownloadState(delta.id);
  }
}

async function processDownloadCandidate(item, state) {
  if (!item || state?.handledAt) return state;
  if (!shouldInterceptDownload(item)) return state;

  try {
    const settings = await browser.storage.sync.get({ autoOpen: true });
    const reason = settings.autoOpen ? 'opened' : 'auto-open-disabled';
    const handledState = markDownloadHandled(state, Date.now(), reason);
    await setDownloadState(handledState);
    if (!settings.autoOpen) return handledState;

    handleHwpDownload(item);
    return handledState;
  } catch (err) {
    console.error('[rhwp] 다운로드 인터셉터 오류:', err);
    return state;
  }
}

function handleHwpDownload(item) {
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
  return browser.storage?.session || null;
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
  if (typeof timer?.unref === 'function') {
    timer.unref();
  }
}
