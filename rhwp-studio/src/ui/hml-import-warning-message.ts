import type { HmlOpenMetadata } from '../core/wasm-bridge.ts';

const MAX_WARNING_PATHS = 3;

export function buildHmlImportWarningMessage(metadata: HmlOpenMetadata): string {
  const version = metadata.hwpmlVersion ? ` ${metadata.hwpmlVersion}` : '';
  const savable = metadata.hmlSavable === true;
  const lines = [
    savable
      ? `HML${version} 문서를 열었습니다. HML로 의미를 보존해 저장할 수 있지만 원본 바이트와 동일하지는 않습니다.`
      : `HML${version} 문서를 열었습니다. 보존할 수 없는 요소가 있어 HML로는 저장할 수 없습니다. HWP 또는 HWPX로 저장하세요.`,
  ];
  if (metadata.warnings.length === 0) return lines.join('\n');

  lines.push(`지원하지 않거나 변환된 요소 ${metadata.warnings.length}건이 있습니다.`);
  for (const warning of metadata.warnings.slice(0, MAX_WARNING_PATHS)) {
    lines.push(`${warning.xmlPath}: ${warning.message}`);
  }
  if (metadata.warnings.length > MAX_WARNING_PATHS) {
    lines.push(`그 외 ${metadata.warnings.length - MAX_WARNING_PATHS}건`);
  }
  return lines.join('\n');
}
