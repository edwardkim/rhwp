export const EMBED_PROTOCOL_VERSION = 1 as const;
export const EMBED_CAPABILITIES = ['transferable-array-buffer', 'hml-export'] as const;

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
