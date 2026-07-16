import { resolve } from 'path';

import { runTest, assert } from './helpers.mjs';

const EDITOR_MODULE_URL = `/@fs${resolve(import.meta.dirname, '../../npm/editor/index.js')}`;
const VITE_URL = process.env.VITE_URL || 'http://localhost:7700';

runTest('Issue #2186 @rhwp/editor MessageChannel v1 iframe transport', async ({ page }) => {
  await page.goto(`${VITE_URL}/@vite/client`, { waitUntil: 'domcontentloaded' });
  const result = await page.evaluate(async (editorModuleUrl) => {
    const { createEditor } = await import(editorModuleUrl);
    const host = document.createElement('div');
    host.style.cssText = 'width: 100vw; height: 100vh';
    document.body.appendChild(host);
    const editor = await createEditor(host, {
      studioUrl: `${location.origin}/`,
      handshakeTimeoutMs: 10_000,
    });
    editor.element.name = 'rhwp-e2e-target';

    const sampleBuffer = await fetch('/samples/footnote-01.hwp').then((response) => (
      response.arrayBuffer()
    ));
    const sampleBefore = new Uint8Array(sampleBuffer).slice(0, 16);
    const initialLength = sampleBuffer.byteLength;
    let loadSettled = false;
    const loadPromise = editor.loadFile(sampleBuffer, 'footnote-01.hwp')
      .finally(() => { loadSettled = true; });
    for (let attempt = 0; attempt < 600 && !loadSettled; attempt += 1) {
      const fallbackButton = Array.from(
        editor.element.contentDocument?.querySelectorAll('button') ?? [],
      ).find((button) => button.textContent?.includes('대체 글꼴로 보기'));
      if (fallbackButton) fallbackButton.click();
      await new Promise((delay) => setTimeout(delay, 100));
    }
    const loaded = await loadPromise;
    const publicDiagnostics = await editor.getRendererDiagnostics(0);
    const callerBytesPreserved = sampleBuffer.byteLength === initialLength
      && sampleBefore.every((byte, index) => new Uint8Array(sampleBuffer)[index] === byte);
    const hwp = await editor.exportHwp();
    const hwpx = await editor.exportHwpx();

    const forged = await forgedPeerResult(editor.element);
    const publicIframe = editor.element;
    editor.destroy();
    const publicDestroyed = !publicIframe.isConnected && !host.querySelector('iframe');
    host.remove();

    const legacy = await legacyReadyResult();
    return {
      pageCount: loaded.pageCount,
      publicDiagnosticsSchema: publicDiagnostics.schemaVersion,
      publicDiagnosticsPage: publicDiagnostics.page?.index,
      callerBytesPreserved,
      hwpLength: hwp.byteLength,
      hwpxLength: hwpx.byteLength,
      hwpxMagic: Array.from(hwpx.slice(0, 4)),
      forgedConnected: forged.connected,
      forgedError: forged.error,
      publicDestroyed,
      legacyReady: legacy.ready,
      legacyPageCountType: typeof legacy.pageCount,
      legacyDiagnosticsPage: legacy.diagnostics?.page?.index,
    };

    async function forgedPeerResult(target) {
      const attacker = document.createElement('iframe');
      const result = new Promise((resolveResult) => {
        const timer = setTimeout(() => resolveResult({ connected: 'timeout' }), 2_000);
        const listener = (event) => {
          if (event.source !== attacker.contentWindow || event.data?.type !== 'rhwp-forged-result') return;
          clearTimeout(timer);
          window.removeEventListener('message', listener);
          resolveResult(event.data);
        };
        window.addEventListener('message', listener);
      });
      attacker.srcdoc = `<script>
        const hostOrigin = parent.location.origin;
        try {
          const channel = new MessageChannel();
          let connected = false;
          channel.port1.onmessage = () => { connected = true; };
          channel.port1.start();
          parent.document.querySelector('iframe[name="rhwp-e2e-target"]').contentWindow.postMessage({
            type: 'rhwp-connect', version: 1, sessionId: 'forged-peer',
            capabilities: ['transferable-array-buffer']
          }, hostOrigin, [channel.port2]);
          setTimeout(() => {
            parent.postMessage({ type: 'rhwp-forged-result', connected }, hostOrigin);
            channel.port1.close();
          }, 200);
        } catch (error) {
          parent.postMessage({
            type: 'rhwp-forged-result', connected: false, error: String(error)
          }, hostOrigin);
        }
      <\/script>`;
      document.body.appendChild(attacker);
      await new Promise((resolveLoad) => attacker.addEventListener('load', resolveLoad, { once: true }));
      const value = await result;
      attacker.remove();
      target.name = '';
      return value;
    }

    async function legacyReadyResult() {
      const iframe = document.createElement('iframe');
      iframe.src = `${location.origin}/`;
      document.body.appendChild(iframe);
      await new Promise((resolveLoad) => iframe.addEventListener('load', resolveLoad, { once: true }));
      let id = 1000;
      const request = (method) => new Promise((resolveRequest) => {
        const requestId = ++id;
        const timer = setTimeout(() => {
          window.removeEventListener('message', listener);
          resolveRequest({ timeout: true });
        }, 2_000);
        const listener = (event) => {
          if (event.source !== iframe.contentWindow
              || event.origin !== location.origin
              || event.data?.type !== 'rhwp-response'
              || event.data.id !== requestId) return;
          clearTimeout(timer);
          window.removeEventListener('message', listener);
          resolveRequest(event.data);
        };
        window.addEventListener('message', listener);
        iframe.contentWindow.postMessage({
          type: 'rhwp-request', id: requestId, method, params: {},
        }, location.origin);
      });
      let ready = false;
      for (let attempt = 0; attempt < 30 && !ready; attempt += 1) {
        const response = await request('ready');
        ready = response.result === true;
        if (!ready) await new Promise((delay) => setTimeout(delay, 250));
      }
      const pageCount = ready ? (await request('pageCount')).result : undefined;
      const diagnostics = ready ? (await request('getRendererDiagnostics')).result : undefined;
      iframe.remove();
      return { ready, pageCount, diagnostics };
    }
  }, EDITOR_MODULE_URL);

  console.log(`  result: ${JSON.stringify(result)}`);
  assert(result.pageCount >= 1, 'public loadFile이 transferable HWP buffer를 Studio에 로드한다');
  assert(result.publicDiagnosticsSchema === 1 && result.publicDiagnosticsPage === 0,
    'public getRendererDiagnostics가 versioned page snapshot을 반환한다');
  assert(result.callerBytesPreserved, 'loadFile에 넘긴 동일 caller ArrayBuffer가 detach·변경되지 않는다');
  assert(result.hwpLength > 0, 'public exportHwp가 transferable bytes를 반환한다');
  assert(result.hwpxLength > 0, 'public exportHwpx가 transferable bytes를 반환한다');
  assert(result.hwpxMagic.join(',') === '80,75,3,4', 'HWPX export가 ZIP signature를 유지한다');
  assert(!result.forgedError && result.forgedConnected === false, 'sibling iframe forged peer는 v1 연결을 성립시키지 못한다');
  assert(result.publicDestroyed, 'public destroy가 SDK iframe을 제거한다');
  assert(result.legacyReady && result.legacyPageCountType === 'number', 'legacy request/response 경로를 유지한다');
  assert(result.legacyDiagnosticsPage === 0, 'legacy renderer diagnostics 경로를 유지한다');
}, { skipLoadApp: true });
