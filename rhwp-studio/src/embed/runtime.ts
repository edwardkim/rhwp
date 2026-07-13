import {
  EMBED_PROTOCOL_VERSION,
  EMBED_CAPABILITIES,
  isConnectAttempt,
  isConnectMessage,
  isRequestAttempt,
  isRequestEnvelope,
  isUsableParentOrigin,
  type EmbedResponseEnvelope,
} from './protocol.ts';
import { routeEmbedRequest, type EmbedRpcHandlers } from './rpc-router.ts';

interface EmbedRuntimeOptions {
  hostWindow: Window;
  parentWindow: Window;
  handlers: EmbedRpcHandlers;
}

function errorText(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function postPortResponse(port: MessagePort, response: EmbedResponseEnvelope): void {
  if (!(response.result instanceof Uint8Array)) {
    port.postMessage(response);
    return;
  }
  const result = response.result.slice();
  port.postMessage({ ...response, result }, [result.buffer]);
}

function releasePort(port: MessagePort | undefined): void {
  if (!port) return;
  port.onmessage = null;
  port.close();
}

function releasePorts(ports: readonly MessagePort[]): void {
  for (const port of ports) releasePort(port);
}

function bindPort(port: MessagePort, sessionId: string, handlers: EmbedRpcHandlers): void {
  port.onmessage = async ({ data }) => {
    if (!isRequestAttempt(data, sessionId)) return;
    const response: EmbedResponseEnvelope = {
      type: 'rhwp-response', version: EMBED_PROTOCOL_VERSION, sessionId, id: data.id,
    };
    if (!isRequestEnvelope(data, sessionId)) {
      response.error = typeof data.version === 'number'
        && Number.isSafeInteger(data.version)
        && data.version !== EMBED_PROTOCOL_VERSION
        ? {
            code: 'UNSUPPORTED_VERSION',
            message: `Unsupported embed protocol version: ${data.version}`,
            supportedVersions: [EMBED_PROTOCOL_VERSION],
          }
        : { code: 'INVALID_REQUEST', message: 'Invalid embed request.' };
      postPortResponse(port, response);
      return;
    }
    try {
      response.result = await routeEmbedRequest(data.method, data.params, handlers);
    } catch (error) {
      response.error = { code: 'RPC_ERROR', message: errorText(error) };
    }
    postPortResponse(port, response);
  };
  port.start();
  port.postMessage({
    type: 'rhwp-connected', version: EMBED_PROTOCOL_VERSION, sessionId,
    capabilities: EMBED_CAPABILITIES,
  });
}

function rejectConnect(port: MessagePort, attempt: { version: number; sessionId: string }): void {
  port.start();
  port.postMessage({
    type: 'rhwp-connect-error',
    version: EMBED_PROTOCOL_VERSION,
    sessionId: attempt.sessionId,
    error: {
      code: attempt.version === EMBED_PROTOCOL_VERSION
        ? 'UNSUPPORTED_CAPABILITY'
        : 'UNSUPPORTED_VERSION',
      message: attempt.version === EMBED_PROTOCOL_VERSION
        ? '필수 embed capability를 지원하지 않습니다.'
        : `지원하지 않는 embed protocol version: ${attempt.version}`,
      supportedVersions: [EMBED_PROTOCOL_VERSION],
    },
  });
  releasePort(port);
}

async function handleLegacy(
  event: MessageEvent,
  handlers: EmbedRpcHandlers,
): Promise<void> {
  const message = event.data;
  const isHwpctl = message?.type === 'hwpctl-load' && message.data;
  if (!isHwpctl && (message?.type !== 'rhwp-request' || !message.method)) return;
  const method = isHwpctl ? 'loadFile' : message.method;
  const params = isHwpctl ? message : message.params;
  const response: Record<string, unknown> = { type: 'rhwp-response', id: message.id };
  try {
    const result = await routeEmbedRequest(method, params, handlers, true);
    response.result = result instanceof Uint8Array ? Array.from(result) : result;
  } catch (error) {
    response.error = errorText(error);
  }
  (event.source as WindowProxy | null)?.postMessage(response, { targetOrigin: event.origin });
}

export function installEmbedRuntime(options: EmbedRuntimeOptions): () => void {
  const ports = new Set<MessagePort>();
  let binding: { origin: string; sessionId: string; port: MessagePort } | null = null;
  const onMessage = (event: MessageEvent) => {
    const transferredPorts = Array.from(event.ports);
    if (event.source !== options.parentWindow || !isUsableParentOrigin(event.origin)) {
      releasePorts(transferredPorts);
      return;
    }
    const port = event.data?.type === 'rhwp-connect' ? transferredPorts.shift() : undefined;
    releasePorts(transferredPorts);
    if (binding && event.origin !== binding.origin) {
      releasePort(port);
      return;
    }
    if (port) {
      if (!isConnectAttempt(event.data)) {
        releasePort(port);
        return;
      }
      if (!isConnectMessage(event.data)) {
        rejectConnect(port, event.data);
        return;
      }
      if (binding) {
        releasePort(port);
        return;
      }
      binding = { origin: event.origin, sessionId: event.data.sessionId, port };
      ports.add(port);
      bindPort(port, event.data.sessionId, options.handlers);
      return;
    }
    if (binding) return;
    void handleLegacy(event, options.handlers);
  };
  options.hostWindow.addEventListener('message', onMessage);
  return () => {
    options.hostWindow.removeEventListener('message', onMessage);
    for (const port of ports) releasePort(port);
    ports.clear();
    binding = null;
  };
}
