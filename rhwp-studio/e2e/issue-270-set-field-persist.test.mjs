/**
 * E2E 테스트: 이슈 #270 — set_field 후 저장/재오픈 시 필드 값 유실 회귀
 *
 * 시나리오:
 *   1. samples/field-01.hwp 로드
 *   2. setFieldValueByName("회사명", "PERSIST_TEST")
 *   3. in-memory getFieldValueByName → "PERSIST_TEST" 확인
 *   4. exportHwp() → output/hwp/issue_270_persist_test.hwp 로 저장
 *   5. 같은 페이지에서 다시 loadDocument(저장된 bytes) → re-parse
 *   6. getFieldValueByName → "PERSIST_TEST" (회귀 게이트)
 *
 * 한컴 편집기 검증 (작업지시자 직접):
 *   - output/hwp/issue_270_persist_test.hwp 를 한컴 2010 / 2020 편집기로 열기
 *   - "회사명" 필드에 "PERSIST_TEST" 표시 확인
 *   - 파일 손상 메시지 없음 확인
 *
 * PR #446 옵션 C — 메인테이너 후속 정정 일환. 메모리 feedback_self_verification_not_hancom
 * 원칙 부합: rhwp 자기 라운드트립 (e2e) 통과 + 한컴 편집기 직접 검증으로 게이트 분리.
 */
import { runTest, loadHwpFile, screenshot, assert, setTestCase } from './helpers.mjs';
import fs from 'node:fs';
import path from 'node:path';

runTest('이슈 #270: set_field 후 저장/재오픈 시 필드 값 유실 회귀', async ({ page }) => {

  // ── TC-1: field-01.hwp 로드 ──────────────────────────────
  setTestCase('TC-1: field-01.hwp 로드');
  const { pageCount } = await loadHwpFile(page, 'field-01.hwp');
  assert(pageCount >= 1, `field-01.hwp 로드 성공 (${pageCount}페이지)`);
  await screenshot(page, 'issue270-01-loaded');

  // ── TC-2: 초기 "회사명" 필드 값 확인 (빈 문자열 또는 placeholder) ──
  // WASM API getFieldValueByName 은 JSON 문자열 반환:
  //   {"ok":true,"fieldId":<id>,"value":"<actual>"}
  setTestCase('TC-2: 초기 회사명 필드 값');
  const initialValue = await page.evaluate(() => {
    const w = window.__wasm;
    if (!w?.doc?.getFieldValueByName) return { error: 'API 없음' };
    try {
      const json = w.doc.getFieldValueByName('회사명');
      const parsed = JSON.parse(json);
      return { ok: parsed.ok, value: parsed.value };
    } catch (e) {
      return { error: e.message || String(e) };
    }
  });
  console.log('초기 회사명 값:', JSON.stringify(initialValue));
  assert(!initialValue.error && initialValue.ok, `초기 get_field 호출 성공`);
  assert(
    initialValue.value !== 'PERSIST_TEST',
    `초기 회사명은 PERSIST_TEST 가 아니어야 함 (실제: ${JSON.stringify(initialValue.value)})`
  );

  // ── TC-3: set_field 적용 ────────────────────────────────
  setTestCase('TC-3: set_field("회사명", "PERSIST_TEST")');
  const setResult = await page.evaluate(() => {
    const w = window.__wasm;
    if (!w?.doc?.setFieldValueByName) return { error: 'API 없음' };
    try {
      const json = w.doc.setFieldValueByName('회사명', 'PERSIST_TEST');
      const parsed = JSON.parse(json);
      return { ok: parsed.ok, oldValue: parsed.oldValue, newValue: parsed.newValue };
    } catch (e) {
      return { error: e.message || String(e) };
    }
  });
  console.log('set_field 결과:', JSON.stringify(setResult));
  assert(!setResult.error && setResult.ok, `set_field 호출 성공`);
  assert(setResult.newValue === 'PERSIST_TEST', `setField newValue = PERSIST_TEST`);

  // ── TC-4: in-memory 값 검증 ─────────────────────────────
  setTestCase('TC-4: in-memory 값 = PERSIST_TEST');
  const inMemoryValue = await page.evaluate(() => {
    const json = window.__wasm?.doc?.getFieldValueByName?.('회사명');
    if (!json) return null;
    return JSON.parse(json).value;
  });
  console.log('in-memory 값:', JSON.stringify(inMemoryValue));
  assert(
    inMemoryValue === 'PERSIST_TEST',
    `in-memory 값이 PERSIST_TEST 여야 함 (실제: ${JSON.stringify(inMemoryValue)})`
  );

  // ── TC-5: exportHwp() ─────────────────────────────────
  setTestCase('TC-5: exportHwp 저장');
  const exportedBase64 = await page.evaluate(() => {
    const w = window.__wasm;
    if (!w?.doc?.exportHwp) return { error: 'exportHwp API 없음' };
    try {
      const bytes = w.doc.exportHwp();
      // Uint8Array → base64 (브라우저)
      let binary = '';
      for (let i = 0; i < bytes.length; i++) {
        binary += String.fromCharCode(bytes[i]);
      }
      return { base64: btoa(binary), size: bytes.length };
    } catch (e) {
      return { error: e.message || String(e) };
    }
  });
  console.log('exportHwp 결과: size =', exportedBase64.size);
  assert(!exportedBase64.error, `exportHwp 호출 성공`);
  assert(exportedBase64.size > 0, `export 산출물 크기 > 0`);

  // ── TC-6: 파일 저장 (한컴 편집기 검증용) ──────────────
  setTestCase('TC-6: output/hwp/issue_270_persist_test.hwp 저장');
  const outputDir = path.resolve(process.cwd(), '..', 'output', 'hwp');
  fs.mkdirSync(outputDir, { recursive: true });
  const outputPath = path.join(outputDir, 'issue_270_persist_test.hwp');
  const buffer = Buffer.from(exportedBase64.base64, 'base64');
  fs.writeFileSync(outputPath, buffer);
  console.log(`출력 파일 저장: ${outputPath} (${buffer.length} bytes)`);
  assert(fs.existsSync(outputPath), `파일 저장 성공: ${outputPath}`);

  // ── TC-7: 같은 page 에서 재 loadDocument → re-parse 검증 ─────
  setTestCase('TC-7: 재 loadDocument → 회사명 = PERSIST_TEST (회귀 게이트)');
  const reopenResult = await page.evaluate((b64) => {
    const w = window.__wasm;
    try {
      // base64 → Uint8Array
      const binary = atob(b64);
      const bytes = new Uint8Array(binary.length);
      for (let i = 0; i < binary.length; i++) {
        bytes[i] = binary.charCodeAt(i);
      }
      const docInfo = w.loadDocument(bytes, 'issue_270_persist_test.hwp');
      if (!docInfo) return { error: 'loadDocument returned null' };
      const json = w.doc.getFieldValueByName('회사명');
      const parsed = JSON.parse(json);
      return { pageCount: docInfo.pageCount, value: parsed.value };
    } catch (e) {
      return { error: e.message || String(e) };
    }
  }, exportedBase64.base64);

  console.log('재오픈 결과:', JSON.stringify(reopenResult));
  assert(!reopenResult.error, `재 loadDocument 성공`);
  assert(
    reopenResult.value === 'PERSIST_TEST',
    `재오픈 후 회사명 값이 PERSIST_TEST 여야 함 — 이슈 #270 회귀 게이트 (실제: ${JSON.stringify(reopenResult.value)})`
  );

  await screenshot(page, 'issue270-02-after-reopen');

  console.log('');
  console.log('=== 자기 라운드트립 통과 ===');
  console.log('');
  console.log('=== 한컴 편집기 검증 (작업지시자 직접) ===');
  console.log(`  파일: ${outputPath}`);
  console.log('  1. 한컴 2010 / 2020 편집기로 열기');
  console.log('  2. "회사명" 필드 → "PERSIST_TEST" 표시 확인');
  console.log('  3. 파일 손상 메시지 없음 확인');
});
