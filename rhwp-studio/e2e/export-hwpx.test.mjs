/**
 * Issue #557 — npm/editor RPC + Wrapper 에 exportHwpx / exportHwpVerify 노출
 *
 * 본 파일은 fail-first 로 작성되었다 (Stage 0).
 * - 현재 RPC switch 에 'exportHwpx' / 'exportHwpVerify' case 가 없으므로
 *   default 분기에 떨어져 'Unknown method: ...' 응답이 와야 한다.
 * - Stage 1 에서 RPC case 추가 → 본 단언이 깨짐 (의도된 부분 실패).
 * - Stage 4 에서 통과 단언 + 라운드트립 + verify 객체 정합성 단언으로 교체.
 *
 * 실행:
 *   node e2e/export-hwpx.test.mjs --mode=headless
 */
import { runTest, assert } from './helpers.mjs';

runTest('issue-557 fail-first: exportHwpx / exportHwpVerify RPC 노출 갭', async ({ page }) => {
  // RPC 호출 헬퍼를 페이지 컨텍스트에 주입
  await page.evaluate(() => {
    window.__callRpc = (method, params = {}) => new Promise((resolve) => {
      const id = Math.floor(Math.random() * 1e9);
      const handler = (e) => {
        if (e.data?.type === 'rhwp-response' && e.data.id === id) {
          window.removeEventListener('message', handler);
          resolve(e.data);
        }
      };
      window.addEventListener('message', handler);
      window.postMessage({ type: 'rhwp-request', id, method, params }, '*');
      setTimeout(() => resolve({ timeout: true, method }), 5000);
    });
  });

  // 기준선: exportHwp 는 method 인식되어야 한다 (문서 미로드 시 다른 에러)
  console.log('\n[1] exportHwp (기준선) — method 인식 여부');
  const baseline = await page.evaluate(() => window.__callRpc('exportHwp'));
  console.log(`    응답: ${JSON.stringify(baseline).slice(0, 200)}`);
  assert(!/Unknown method/.test(baseline.error || ''),
    `exportHwp 는 RPC 가 인식해야 함 (current error: ${baseline.error})`);

  // exportHwpx — 현재 미구현 → Unknown method 단언 (red)
  console.log('\n[2] exportHwpx — RPC default 단언 (현재 미구현 기대)');
  const r1 = await page.evaluate(() => window.__callRpc('exportHwpx'));
  console.log(`    응답: ${JSON.stringify(r1)}`);
  assert(/Unknown method: exportHwpx/.test(r1.error || ''),
    `exportHwpx 가 'Unknown method' 응답이어야 함 (current: ${r1.error || '(no error)'})`);

  // exportHwpVerify — 동일
  console.log('\n[3] exportHwpVerify — RPC default 단언 (현재 미구현 기대)');
  const r2 = await page.evaluate(() => window.__callRpc('exportHwpVerify'));
  console.log(`    응답: ${JSON.stringify(r2)}`);
  assert(/Unknown method: exportHwpVerify/.test(r2.error || ''),
    `exportHwpVerify 가 'Unknown method' 응답이어야 함 (current: ${r2.error || '(no error)'})`);

  console.log('\nSTAGE 0 RED — RPC 노출 갭 자동화 캡처 완료');
});
