import {
  normalizeHmlSaveState,
  resolveHmlSaveCapability,
  type HmlSaveBlocker,
} from '../core/hml-save-capability.ts';

const MAX_BLOCKER_PATHS = 3;

interface HmlSaveMessageMetadata {
  hmlSavable?: unknown;
  saveBlockers: HmlSaveBlocker[];
}

export function buildHmlSaveFormatMessage(
  metadata: HmlSaveMessageMetadata | null,
  exporterAvailable: boolean,
): string {
  const capability = resolveHmlSaveCapability(metadata, exporterAvailable);
  if (capability.hmlEnabled) {
    return 'HML로 의미를 보존해 저장할 수 있지만 원본 바이트와 동일하지는 않습니다.\n저장할 형식을 선택하세요.';
  }

  const lines = [
    `${capability.diagnostic ?? 'HML 저장을 사용할 수 없습니다.'}\nHWP 또는 HWPX로 저장할 수 있습니다.`,
  ];
  const blockers = normalizeHmlSaveState(metadata)?.saveBlockers ?? [];
  for (const blocker of blockers.slice(0, MAX_BLOCKER_PATHS)) {
    lines.push(`${blocker.xmlPath}: ${blocker.message}`);
  }
  if (blockers.length > MAX_BLOCKER_PATHS) {
    lines.push(`그 외 ${blockers.length - MAX_BLOCKER_PATHS}건`);
  }
  return lines.join('\n');
}
