/**
 * E2E 테스트 — 큰 문서 로딩 동안 대기 커서 (#5740)
 *
 * 검증 항목:
 * 1. 로딩 시작 전에는 대기 커서가 아니다
 * 2. 파싱·쪽 계산이 도는 동안 루트에 rhwp-busy 가 걸리고 커서가 wait 로 계산된다
 * 3. 로딩이 끝나면 되돌아간다
 */

import { runTest, assert } from './helpers.mjs';

process.env.VITE_URL = process.env.VITE_URL || 'http://localhost:7700';

// 10MB 급 실제 샘플 — 빈 화면이 오래 보이는 상황을 그대로 재현한다.
const SAMPLE = '2025 행정업무운영 편람(최종).hwp';

runTest('큰 문서 로딩 중 대기 커서', async ({ page }) => {
  // ── TC1: 로딩 전 ───────────────────────────────────────────
  const before = await page.evaluate(() => ({
    busy: document.documentElement.classList.contains('rhwp-busy'),
    cursor: getComputedStyle(document.body).cursor,
  }));
  assert(before.busy === false, `TC1: 로딩 전에는 대기 커서가 아님 (busy=${before.busy})`);

  // ── TC2/TC3: 실제 열기 경로(loadFile → loadBytes)를 태우며 샘플링 ──
  const result = await page.evaluate(async ({ sample }) => {
    const samples = [];
    let running = true;
    const tick = () => {
      if (!running) return;
      samples.push({
        busy: document.documentElement.classList.contains('rhwp-busy'),
        cursor: getComputedStyle(document.body).cursor,
      });
      requestAnimationFrame(tick);
    };
    requestAnimationFrame(tick);

    const resp = await fetch(`/samples/${encodeURIComponent(sample)}`);
    if (!resp.ok) {
      running = false;
      return { error: `HTTP ${resp.status}` };
    }
    const bytes = new Uint8Array(await resp.arrayBuffer());
    const file = new File([bytes], sample, { type: 'application/octet-stream' });
    const input = document.getElementById('file-input');
    const dt = new DataTransfer();
    dt.items.add(file);
    input.files = dt.files;
    input.dispatchEvent(new Event('change', { bubbles: true }));

    // 로딩이 끝날 때까지 대기 — rhwp-busy 가 걸렸다가 풀리는 것을 관찰한다.
    await new Promise((resolve) => {
      const started = performance.now();
      const poll = () => {
        const busy = document.documentElement.classList.contains('rhwp-busy');
        if (busy) window.__sawBusy = true;
        if (window.__sawBusy && !busy) return resolve();
        if (performance.now() - started > 120000) return resolve();
        setTimeout(poll, 16);
      };
      poll();
    });
    running = false;

    return {
      samples: samples.length,
      busyFrames: samples.filter(s => s.busy).length,
      waitCursorFrames: samples.filter(s => s.cursor === 'wait').length,
      after: {
        busy: document.documentElement.classList.contains('rhwp-busy'),
        cursor: getComputedStyle(document.body).cursor,
      },
    };
  }, { sample: SAMPLE });

  assert(!result.error, `TC2: 샘플 로드 (${result.error ?? 'ok'})`);
  assert(result.busyFrames > 0,
    `TC2: 로딩 동안 rhwp-busy 가 걸린 프레임 있음 (${result.busyFrames}/${result.samples})`);
  assert(result.waitCursorFrames > 0,
    `TC2: 로딩 동안 커서가 wait 로 계산된 프레임 있음 (${result.waitCursorFrames}/${result.samples})`);
  assert(result.after.busy === false && result.after.cursor !== 'wait',
    `TC3: 로딩이 끝나면 되돌아감 (busy=${result.after.busy}, cursor=${result.after.cursor})`);
});
