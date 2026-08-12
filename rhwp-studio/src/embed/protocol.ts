export const EMBED_PROTOCOL_VERSION = 1 as const;
export const EMBED_CAPABILITIES = [
  'transferable-array-buffer',
  'hml-export',
  'renderer-diagnostics-v1',
  'notify-saved-v1',
  // 아래 넷은 브리지 확장(P4). 프로토콜 세대는 1 을 유지하고 capability 로만 넓힌다 —
  // 구버전 studio 에 붙은 신버전 SDK 는 기능만 비활성되고 기존 임베드는 그대로 돈다.
  'automation-v1',
  'plugin-loader-v1',
  'hwpctrl-v1',
  'chrome-v1',
] as const;

export interface EmbedConnectAttempt {
  type: 'rhwp-connect';
  version: number;
  sessionId: string;
  capabilities?: unknown;
}

export interface EmbedConnectMessage {
  type: 'rhwp-connect';
  version: typeof EMBED_PROTOCOL_VERSION;
  sessionId: string;
  capabilities: readonly string[];
}

export interface EmbedRequestEnvelope {
  type: 'rhwp-request';
  version: typeof EMBED_PROTOCOL_VERSION;
  sessionId: string;
  id: number;
  method: string;
  params?: unknown;
}

export interface EmbedRequestAttempt {
  type: 'rhwp-request';
  version?: unknown;
  sessionId: string;
  id: number;
  method?: unknown;
  params?: unknown;
}

export interface EmbedResponseEnvelope {
  type: 'rhwp-response';
  version: typeof EMBED_PROTOCOL_VERSION;
  sessionId: string;
  id: number;
  result?: unknown;
  error?: EmbedProtocolError;
}

export interface EmbedProtocolError {
  code: string;
  message: string;
  supportedVersions?: number[];
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null;
}

export function isConnectMessage(value: unknown): value is EmbedConnectMessage {
  return isConnectAttempt(value)
    && value.version === EMBED_PROTOCOL_VERSION
    && Array.isArray(value.capabilities)
    && value.capabilities.includes('transferable-array-buffer');
}

export function isConnectAttempt(value: unknown): value is EmbedConnectAttempt {
  return isRecord(value)
    && value.type === 'rhwp-connect'
    && Number.isSafeInteger(value.version)
    && typeof value.sessionId === 'string'
    && value.sessionId.length > 0;
}

export function isRequestEnvelope(
  value: unknown,
  sessionId: string,
): value is EmbedRequestEnvelope {
  return isRequestAttempt(value, sessionId)
    && value.version === EMBED_PROTOCOL_VERSION
    && typeof value.method === 'string'
    && value.method.length > 0;
}

export function isRequestAttempt(
  value: unknown,
  sessionId: string,
): value is EmbedRequestAttempt {
  return isRecord(value)
    && value.type === 'rhwp-request'
    && value.sessionId === sessionId
    && Number.isSafeInteger(value.id);
}

export function isUsableParentOrigin(origin: string): boolean {
  if (!origin || origin === 'null') return false;
  try {
    const protocol = new URL(origin).protocol;
    return protocol === 'http:' || protocol === 'https:';
  } catch {
    return false;
  }
}
