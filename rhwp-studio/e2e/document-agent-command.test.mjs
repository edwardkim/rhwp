import { resolve } from 'node:path';

import { runTest, assert } from './helpers.mjs';

const EDITOR_MODULE_PATH = resolve(import.meta.dirname, '../../npm/editor/index.js').replace(/\\/g, '/');
const EDITOR_MODULE_URL = EDITOR_MODULE_PATH.startsWith('/')
  ? `/@fs${EDITOR_MODULE_PATH}`
  : `/@fs/${EDITOR_MODULE_PATH}`;
const VITE_URL = process.env.VITE_URL || 'http://localhost:7700';
const FIXTURES = [
  { file: 'para-001.hwp', format: 'hwp' },
  { file: 'hwpx/para-001.hwpx', format: 'hwpx' },
];

runTest('document-agent exact command HWP/HWPX browser gate', async ({ page }) => {
  await page.goto(`${VITE_URL}/e2e/embed-harness.html`, { waitUntil: 'domcontentloaded' });

  for (const fixture of FIXTURES) {
    const setup = await page.evaluate(async ({ editorModuleUrl, sampleFile, expectedFormat }) => {
      const { createEditor } = await import(editorModuleUrl);
      const host = document.createElement('div');
      host.style.cssText = 'width: 100vw; height: 100vh';
      document.body.replaceChildren(host);
      const editor = await createEditor(host, {
        studioUrl: `${location.origin}/`,
        renderer: 'canvas2d',
        handshakeTimeoutMs: 10_000,
      });
      const sampleUrl = `/samples/${sampleFile.split('/').map(encodeURIComponent).join('/')}`;
      const bytes = await fetch(sampleUrl).then(response => response.arrayBuffer());
      await editor.loadFile(bytes, sampleFile, { suppressDialogs: true });
      const studioWindow = editor.element.contentWindow;
      const wasm = studioWindow.__wasm;
      if (!wasm) throw new Error('Studio WasmBridge is unavailable');

      const sha = async (value) => {
        const data = typeof value === 'string' ? new TextEncoder().encode(value) : value;
        const digest = await crypto.subtle.digest('SHA-256', data);
        return [...new Uint8Array(digest)].map(byte => byte.toString(16).padStart(2, '0')).join('');
      };
      const safeId = (value, label) => {
        if (!Number.isSafeInteger(value) || value < 0) throw new Error(`${label} unavailable`);
        return value;
      };
      const charShapeIds = (section, paragraph, length) => Array.from(
        { length: Math.max(length, 1) },
        (_, offset) => safeId(
          wasm.getCharPropertiesAt(section, paragraph, offset).charShapeId,
          'charShapeId',
        ),
      );
      const semantic = async (section, paragraph) => {
        const length = wasm.getParagraphLength(section, paragraph);
        const text = length > 0 ? wasm.getTextRange(section, paragraph, 0, length) : '';
        return {
          section,
          paragraph,
          length,
          textSha256: await sha(text),
          paraShapeId: safeId(
            wasm.getParaPropertiesAt(section, paragraph).paraShapeId,
            'paraShapeId',
          ),
          styleId: safeId(wasm.getStyleAt(section, paragraph).id, 'styleId'),
          charShapeIds: charShapeIds(section, paragraph, length),
          controls: wasm.getControlTextPositions(section, paragraph),
        };
      };
      const evidence = async (target) => {
        const text = target.length > 0
          ? wasm.getTextRange(target.section, target.paragraph, 0, target.length)
          : '';
        const shapes = charShapeIds(target.section, target.paragraph, target.length);
        if (!shapes.every(id => id === shapes[0])) throw new Error('mixed target format');
        const charShapeId = shapes[0];
        const paraShapeId = safeId(
          wasm.getParaPropertiesAt(target.section, target.paragraph).paraShapeId,
          'paraShapeId',
        );
        const styleId = safeId(wasm.getStyleAt(target.section, target.paragraph).id, 'styleId');
        const previous = target.paragraph > 0
          ? await semantic(target.section, target.paragraph - 1)
          : null;
        const next = target.paragraph + 1 < wasm.getParagraphCount(target.section)
          ? await semantic(target.section, target.paragraph + 1)
          : null;
        return {
          text,
          textSha256: await sha(text),
          formatSha256: await sha(JSON.stringify({
            schemaVersion: 1, charShapeId, paraShapeId, styleId,
          })),
          adjacentContextSha256: await sha(JSON.stringify({
            schemaVersion: 1, previous, next,
          })),
        };
      };
      const manifest = async (target) => {
        const sectionCount = wasm.getSectionCount();
        const paragraphCounts = Array.from(
          { length: sectionCount },
          (_, section) => wasm.getParagraphCount(section),
        );
        const paragraphs = [];
        for (let section = 0; section < sectionCount; section += 1) {
          for (let paragraph = 0; paragraph < paragraphCounts[section]; paragraph += 1) {
            if (section === target.section && paragraph === target.paragraph) continue;
            paragraphs.push(await semantic(section, paragraph));
          }
        }
        return sha(JSON.stringify({ schemaVersion: 1, sectionCount, paragraphCounts, paragraphs }));
      };
      const targetHasField = (target) => {
        for (let offset = 0; offset <= target.length; offset += 1) {
          if (wasm.getFieldInfoAt({
            sectionIndex: target.section,
            paragraphIndex: target.paragraph,
            charOffset: offset,
          }).inField) return true;
        }
        return false;
      };
      const findCandidate = async () => {
        for (let section = 0; section < wasm.getSectionCount(); section += 1) {
          for (let paragraph = 0; paragraph < wasm.getParagraphCount(section); paragraph += 1) {
            const length = wasm.getParagraphLength(section, paragraph);
            if (length < 2 || length > 160) continue;
            const target = { kind: 'body_paragraph', section, paragraph, charOffset: 0, length };
            const text = wasm.getTextRange(section, paragraph, 0, length);
            if (!text.trim() || wasm.getControlTextPositions(section, paragraph).length > 0
                || targetHasField(target)) continue;
            try {
              return { target, evidence: await evidence(target) };
            } catch {
              // 다음 plain body paragraph를 찾는다.
            }
          }
        }
        throw new Error(`safe body paragraph candidate not found: ${sampleFile}`);
      };

      const state = await editor.getDocumentState();
      if (state.format !== expectedFormat) {
        throw new Error(`unexpected source format: ${state.format}`);
      }
      const candidate = await findCandidate();
      const beforeManifest = await manifest(candidate.target);
      const chars = Array.from(candidate.evidence.text);
      const replacement = `${chars[0] === '가' ? '나' : '가'}${chars.slice(1).join('')}`;
      const events = [];
      const off = editor.onDocumentChanged(event => events.push(event));
      const apply = async (commandId, currentState, currentEvidence) => {
        const startedAt = performance.now();
        const receipt = await editor.applyTextCommand({
          schemaVersion: 1,
          commandId,
          expectedDocumentEpoch: currentState.documentEpoch,
          expectedChangeSeq: currentState.changeSeq,
          expectedDocumentSha256: currentState.documentSha256,
          target: candidate.target,
          expectedBeforeSha256: currentEvidence.textSha256,
          expectedFormatSha256: currentEvidence.formatSha256,
          expectedAdjacentContextSha256: currentEvidence.adjacentContextSha256,
          replacement,
        });
        return { receipt, elapsedMs: performance.now() - startedAt };
      };

      const first = await apply(crypto.randomUUID(), state, candidate.evidence);
      const afterTarget = { ...candidate.target, length: Array.from(replacement).length };
      const afterEvidence = await evidence(afterTarget);
      const manifestAfterApply = await manifest(afterTarget);
      const afterApplyState = await editor.getDocumentState();
      const focus = await editor.focusTarget(afterTarget);
      const selection = await editor.getSelectionContext();
      await editor.revertTextCommand({
        schemaVersion: 1,
        commandId: first.receipt.commandId,
        expectedDocumentEpoch: first.receipt.documentEpoch,
        expectedChangeSeq: first.receipt.afterChangeSeq,
        expectedAfterDocumentSha256: first.receipt.afterDocumentSha256,
        expectedAfterSha256: first.receipt.afterTextSha256,
      });
      const restoredEvidence = await evidence(candidate.target);
      const stateAfterRevert = await editor.getDocumentState();
      const second = await apply(crypto.randomUUID(), stateAfterRevert, restoredEvidence);
      await editor.focusTarget(afterTarget);

      window.__agentCommandE2E = {
        editor,
        off,
        wasm,
        target: candidate.target,
        afterTarget,
        original: candidate.evidence.text,
        replacement,
        beforeManifest,
        manifest,
        evidence,
        receipt: second.receipt,
        events,
        expectedFormat,
      };
      return {
        pageCountBefore: state.pageCount,
        pageCountAfter: afterApplyState.pageCount,
        applyElapsedMs: first.elapsedMs,
        appliedText: afterEvidence.text,
        formatStable: afterEvidence.formatSha256 === candidate.evidence.formatSha256,
        contextStable:
          afterEvidence.adjacentContextSha256 === candidate.evidence.adjacentContextSha256,
        nonTargetStable: manifestAfterApply === beforeManifest,
        revertedText: restoredEvidence.text,
        focus,
        selection,
        modalCount: studioWindow.document.querySelectorAll('.modal-overlay').length,
      };
    }, {
      editorModuleUrl: EDITOR_MODULE_URL,
      sampleFile: fixture.file,
      expectedFormat: fixture.format,
    });

    assert(setup.applyElapsedMs <= 3000, `${fixture.file}: apply+strict render 3초 이내`);
    assert(setup.pageCountBefore === setup.pageCountAfter, `${fixture.file}: page count 보존`);
    assert(setup.formatStable && setup.contextStable, `${fixture.file}: target format/context 보존`);
    assert(setup.nonTargetStable, `${fixture.file}: target 밖 semantic manifest 보존`);
    assert(setup.appliedText !== setup.revertedText, `${fixture.file}: apply와 exact revert 동작`);
    assert(setup.focus.focused && setup.focus.page === setup.selection.page,
      `${fixture.file}: exact target focus와 selection context 동기화`);
    assert(setup.modalCount === 0, `${fixture.file}: apply/revert 추가 modal 0회`);

    // focusTarget은 exact target 전체를 선택하고 focus는 문단 끝에 둔다. 일반 입력의
    // undo를 단일 history entry로 검증하기 위해 End로 같은 위치에서 선택만 접는다.
    await page.keyboard.press('End');
    await page.waitForFunction(async () => {
      const e2e = window.__agentCommandE2E;
      return e2e && (await e2e.editor.getSelectionContext()).collapsed;
    }, { timeout: 5000 });
    await page.keyboard.type('Z');
    await page.waitForFunction(() => {
      const e2e = window.__agentCommandE2E;
      const text = e2e?.wasm.getTextRange(
        e2e.afterTarget.section,
        e2e.afterTarget.paragraph,
        0,
        e2e.wasm.getParagraphLength(e2e.afterTarget.section, e2e.afterTarget.paragraph),
      );
      return typeof text === 'string' && text.endsWith('Z');
    }, { timeout: 5000 });

    const nativeText = await page.evaluate(() => {
      const e2e = window.__agentCommandE2E;
      return {
        actual: e2e.wasm.getTextRange(
          e2e.afterTarget.section,
          e2e.afterTarget.paragraph,
          0,
          e2e.wasm.getParagraphLength(e2e.afterTarget.section, e2e.afterTarget.paragraph),
        ),
        expected: `${e2e.replacement}Z`,
      };
    });
    assert(nativeText.actual === nativeText.expected,
      `${fixture.file}: collapsed target 끝 native typing`);

    const typed = await page.evaluate(async () => {
      const e2e = window.__agentCommandE2E;
      const state = await e2e.editor.getDocumentState();
      let revertError = null;
      try {
        await e2e.editor.revertTextCommand({
          schemaVersion: 1,
          commandId: e2e.receipt.commandId,
          expectedDocumentEpoch: e2e.receipt.documentEpoch,
          expectedChangeSeq: state.changeSeq,
          expectedAfterDocumentSha256: state.documentSha256,
          expectedAfterSha256: e2e.receipt.afterTextSha256,
        });
      } catch (error) {
        revertError = error?.code ?? String(error);
      }
      return { changeSeq: state.changeSeq, revertError };
    });
    assert(typed.revertError === 'COMMAND_NOT_LATEST',
      `${fixture.file}: native typing 뒤 agent revert fail-closed`);

    await page.keyboard.down('Control');
    await page.keyboard.press('KeyZ');
    await page.keyboard.up('Control');
    await page.waitForFunction(() => {
      const e2e = window.__agentCommandE2E;
      return e2e?.wasm.getTextRange(
        e2e.afterTarget.section,
        e2e.afterTarget.paragraph,
        0,
        e2e.wasm.getParagraphLength(e2e.afterTarget.section, e2e.afterTarget.paragraph),
      ) === e2e.replacement;
    }, { timeout: 5000 });
    await page.keyboard.down('Control');
    await page.keyboard.press('KeyZ');
    await page.keyboard.up('Control');
    await page.waitForFunction(() => {
      const e2e = window.__agentCommandE2E;
      return e2e?.wasm.getTextRange(
        e2e.target.section,
        e2e.target.paragraph,
        0,
        e2e.wasm.getParagraphLength(e2e.target.section, e2e.target.paragraph),
      ) === e2e.original;
    }, { timeout: 5000 });

    const final = await page.evaluate(async () => {
      const e2e = window.__agentCommandE2E;
      const finalManifest = await e2e.manifest(e2e.target);
      const state = await e2e.editor.getDocumentState();
      const bytes = e2e.expectedFormat === 'hwpx'
        ? await e2e.editor.exportHwpx()
        : await e2e.editor.exportHwp();
      const result = {
        finalText: e2e.wasm.getTextRange(
          e2e.target.section,
          e2e.target.paragraph,
          0,
          e2e.wasm.getParagraphLength(e2e.target.section, e2e.target.paragraph),
        ),
        original: e2e.original,
        manifestStable: finalManifest === e2e.beforeManifest,
        pageCount: state.pageCount,
        exportLength: bytes.byteLength,
        eventReasons: e2e.events.map(event => event.reason),
        modalCount: e2e.editor.element.contentDocument.querySelectorAll('.modal-overlay').length,
      };
      e2e.off();
      e2e.editor.destroy();
      delete window.__agentCommandE2E;
      return result;
    });
    assert(final.finalText === final.original, `${fixture.file}: native Ctrl+Z 2회로 원문 복원`);
    assert(final.manifestStable, `${fixture.file}: 일반 undo 뒤 target 밖 manifest 보존`);
    assert(final.pageCount === setup.pageCountBefore, `${fixture.file}: 일반 undo 뒤 page count 보존`);
    assert(final.exportLength > 0, `${fixture.file}: 현재 source format 다운로드 bytes 생성`);
    assert(final.modalCount === 0, `${fixture.file}: native typing/undo 중 modal 0회`);
    assert(JSON.stringify(final.eventReasons) === JSON.stringify([
      'agent_apply', 'agent_revert', 'agent_apply',
    ]), `${fixture.file}: public agent event exact 3회`);
  }
}, { skipLoadApp: true });
