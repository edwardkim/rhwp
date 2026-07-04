// 다운로드 관찰자 상태 머신 (Chrome / Firefox 공용)
//
// 브라우저 API 호출 없이 DownloadItem/DownloadDelta와 저장 상태만으로
// 새 다운로드 추적, 과거 항목 무시, 중복 처리 방지를 판정한다.

export const DEFAULT_FRESH_GRACE_MS = 5_000;
export const DEFAULT_STATE_TTL_MS = 10 * 60 * 1000;

export function parseDownloadTime(value) {
  if (typeof value !== 'string' || value.length === 0) return null;
  const ms = Date.parse(value);
  return Number.isFinite(ms) ? ms : null;
}

export function isTerminalDelta(delta) {
  return delta?.state?.current === 'complete' || delta?.state?.current === 'interrupted' || Boolean(delta?.error);
}

export function shouldRecheckDownload(delta) {
  return Boolean(delta?.filename?.current || delta?.finalUrl?.current || delta?.state?.current === 'complete');
}

export function isDownloadStateExpired(state, now = Date.now(), ttlMs = DEFAULT_STATE_TTL_MS) {
  if (!state) return false;
  const latest = Math.max(
    state.firstSeenAt || 0,
    state.handledAt || 0,
    state.terminalAt || 0,
  );
  return latest > 0 && latest < now - ttlMs;
}

function downloadIdOf(itemOrDelta) {
  return typeof itemOrDelta?.id === 'number' ? itemOrDelta.id : null;
}

function isPastDownloadItem(item, referenceTime, options = {}) {
  if (!item) return true;

  const graceMs = options.freshGraceMs ?? DEFAULT_FRESH_GRACE_MS;
  const boundary = referenceTime - graceMs;
  const startMs = parseDownloadTime(item.startTime);
  const endMs = parseDownloadTime(item.endTime);

  if (startMs !== null && startMs < boundary) return true;
  if (endMs !== null && endMs < boundary) return true;

  // 완료 상태인데 시간 정보가 전혀 없으면 새 다운로드로 증명할 수 없다.
  if (item.state === 'complete' && startMs === null && endMs === null) return true;

  return false;
}

function buildState(item, now, previous = null, reason = 'track') {
  return {
    ...(previous || {}),
    id: downloadIdOf(item),
    firstSeenAt: previous?.firstSeenAt || now,
    itemStartTime: parseDownloadTime(item?.startTime),
    itemEndTime: parseDownloadTime(item?.endTime),
    handledAt: previous?.handledAt || null,
    terminalAt: previous?.terminalAt || null,
    lastReason: reason,
  };
}

export function evaluateDownloadCreated(item, previousState = null, now = Date.now(), options = {}) {
  const id = downloadIdOf(item);
  if (id === null) return { action: 'ignore', reason: 'missing-id', state: previousState };

  if (isDownloadStateExpired(previousState, now, options.stateTtlMs)) {
    previousState = null;
  }

  if (previousState?.handledAt) {
    return { action: 'ignore', reason: 'already-handled', state: previousState };
  }

  if (isPastDownloadItem(item, now, options)) {
    return { action: 'ignore', reason: 'past-created', state: previousState };
  }

  return {
    action: 'track',
    reason: 'fresh-created',
    item,
    state: buildState(item, now, previousState, 'fresh-created'),
  };
}

export function evaluateDownloadChanged(delta, item, previousState = null, now = Date.now(), options = {}) {
  const id = downloadIdOf(delta);
  if (id === null) return { action: 'ignore', reason: 'missing-id', state: previousState };

  if (!previousState || isDownloadStateExpired(previousState, now, options.stateTtlMs)) {
    return { action: 'ignore', reason: 'untracked-changed', state: previousState };
  }

  if (previousState.handledAt) {
    return { action: 'ignore', reason: 'already-handled', state: previousState };
  }

  if (!item) {
    return { action: 'ignore', reason: 'missing-item', state: previousState };
  }

  if (isPastDownloadItem(item, previousState.firstSeenAt || now, options)) {
    return { action: 'ignore', reason: 'past-changed', state: previousState };
  }

  return {
    action: 'candidate',
    reason: 'tracked-changed',
    item,
    state: buildState(item, now, previousState, 'tracked-changed'),
  };
}

export function markDownloadHandled(state, now = Date.now(), reason = 'handled') {
  if (!state) return state;
  return {
    ...state,
    handledAt: state.handledAt || now,
    lastReason: reason,
  };
}

export function markDownloadTerminal(state, now = Date.now(), reason = 'terminal') {
  if (!state) return state;
  return {
    ...state,
    terminalAt: state.terminalAt || now,
    lastReason: reason,
  };
}
