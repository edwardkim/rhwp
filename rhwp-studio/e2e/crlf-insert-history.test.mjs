/** #8001: 실제 fresh WASM과 Studio 명령의 CRLF 커서·Undo/Redo·저장 계약. */
import { runTest, createNewDocument, assert } from './helpers.mjs';

runTest('CRLF 삽입·되돌리기 보존', async ({ page }) => {
  await createNewDocument(page);
  const result = await page.evaluate(async () => {
    const { WasmBridge } = await import('/src/core/wasm-bridge.ts');
    const { InsertTextCommand } = await import('/src/engine/command.ts');
    const { CommandHistory } = await import('/src/engine/history.ts');
    const bridge = new WasmBridge();
    await bridge.initialize();
    const cases = [];
    const searches = [];
    try {
      for (const kind of ['body', 'cell', 'nested']) {
        for (const input of ['가\r\n😀\r\n끝', '가\r😀\r끝', '가\n😀\n끝']) {
          const position = { sectionIndex: 0, paragraphIndex: 0, charOffset: 1 };
          let path;
          if (kind === 'body') {
            bridge.createNewDocument();
            bridge.insertText(0, 0, 0, '앞뒤😀');
          } else {
            const cell = content => `<TABLE RowCount="1" ColCount="1"><SHAPEOBJECT><SIZE Width="5000" Height="2000"/></SHAPEOBJECT><ROW><CELL ColAddr="0" RowAddr="0" Width="5000" Height="2000"><PARALIST><P><TEXT>${content}</TEXT></P></PARALIST></CELL></ROW></TABLE>`;
            const content = '<CHAR>앞뒤😀</CHAR>';
            const hml = `<HWPML Version="2.91"><HEAD/><BODY><SECTION><P><TEXT>${cell(kind === 'nested' ? cell(content) : content)}</TEXT></P></SECTION></BODY><TAIL/></HWPML>`;
            bridge.loadDocument(new TextEncoder().encode(hml), 'contract.hml');
            path = Array.from({ length: kind === 'nested' ? 2 : 1 }, () => ({
              controlIndex: 0, cellIndex: 0, cellParaIndex: 0,
            }));
            Object.assign(position, { parentParaIndex: 0, controlIndex: 0,
              cellIndex: 0, cellParaIndex: 0, cellPath: path });
          }
          // HWP 재열기 때 secd가 추가될 수 있어 컨트롤 번호를 원본과 같다고 가정하지 않는다.
          const locate = doc => {
            const [hit] = JSON.parse(doc.searchAllText('앞', true, true));
            if (!hit) throw new Error('문단의 앞 표지가 누락됐습니다');
            const cell = hit.cellContext;
            return { para: cell?.parentPara ?? hit.para, path: hit.cellPath ?? [{
              controlIndex: cell.ctrlIdx, cellIndex: cell.cellIdx, cellParaIndex: cell.cellPara,
            }] };
          };
          const read = doc => {
            if (kind === 'body') return doc.getTextRange(0, 0, 0, 100);
            const at = locate(doc);
            return doc.getTextInCellByPath(0, at.para, JSON.stringify(at.path), 0, 100);
          };
          // 명령의 정규화가 오래된 WASM을 가리지 않도록 raw 공개 API도 실행한다.
          const Document = bridge.borrowDocumentHandle().constructor;
          const raw = new Document(bridge.exportHwp());
          let direct;
          try {
            const at = kind === 'body' ? null : locate(raw);
            const response = JSON.parse(at
              ? raw.insertTextInCellByPath(0, at.para, JSON.stringify(at.path), 1, input)
              : raw.insertText(0, 0, 1, input));
            const savedRaw = new Document(raw.exportHwp());
            try { direct = { caret: response.charOffset, reopened: read(savedRaw) }; }
            finally { savedRaw.free(); }
          } finally { raw.free(); }
          const history = new CommandHistory();
          const command = new InsertTextCommand(position, input);
          const after = history.execute(command, bridge);
          const inserted = read(bridge.borrowDocumentHandle());
          const saved = new Document(bridge.exportHwp());
          let reopened;
          try { reopened = read(saved); } finally { saved.free(); }
          history.undo(bridge);
          const undone = read(bridge.borrowDocumentHandle());
          const redo = history.redo(bridge);
          const redone = read(bridge.borrowDocumentHandle());
          cases.push({ kind, input, inserted, reopened, undone, redone, direct,
            caret: after.charOffset, redoCaret: redo.charOffset,
            invalidation: command.getPageLocalTextEditOptions().insertedText });
        }
      }
      // #7999도 같은 fresh package의 공개 검색·치환과 저장 경계에서 확인한다.
      const Document = bridge.borrowDocumentHandle().constructor;
      for (const [before, query, replacement, sensitive, expected, count] of [
        ['검토......완료', '...', '…', true, '검토……완료', 2],
        ['AAAA끝', 'AA', 'Z', true, 'ZZ끝', 2],
        ['AaAa끝', 'aa', 'Z', false, 'ZZ끝', 2],
        ['İİİ끝', 'i\u0307i\u0307', 'Z', false, 'Zİ끝', 1],
      ]) {
        const doc = Document.createEmpty();
        try {
          doc.createBlankDocument();
          doc.insertText(0, 0, 0, before);
          const hits = JSON.parse(doc.searchAllText(query, sensitive, true));
          const replaced = JSON.parse(doc.replaceAll(query, replacement, sensitive));
          const saved = new Document(doc.exportHwp());
          try { searches.push({ before, expected, count, hits: hits.length,
            replaced: replaced.count, reopened: saved.getTextRange(0, 0, 0, 100) }); }
          finally { saved.free(); }
        } finally { doc.free(); }
      }
    } finally { bridge.releaseDocument(); }

    // 실제 InputHandler의 operation·history·dirty 경로도 대조한다.
    const wasm = window.__wasm;
    const handler = window.__inputHandler;
    wasm.insertText(0, 0, 0, '앞뒤😀');
    handler.cursor.moveTo({ sectionIndex: 0, paragraphIndex: 0, charOffset: 1 });
    handler.executeOperation({ kind: 'command', command: new InsertTextCommand(
      handler.cursor.getPosition(), '가\r\n😀\r\n끝',
    ) });
    const ui = { inserted: wasm.getTextRange(0, 0, 0, 100),
      caret: handler.cursor.getPosition().charOffset,
      canUndo: window.rhwpStudio.automation.getContext().canUndo };
    handler.performUndo();
    ui.undone = wasm.getTextRange(0, 0, 0, 100);
    handler.performRedo();
    ui.redone = wasm.getTextRange(0, 0, 0, 100);
    return { cases, searches, ui };
  });
  console.log(JSON.stringify(result));
  for (const row of result.cases) {
    const label = `${row.kind} ${JSON.stringify(row.input)}`;
    assert(row.direct.caret === 6 && row.direct.reopened === '앞가\n😀\n끝뒤😀',
      `${label}: raw WASM API의 정규화·저장 보존`);
    assert(row.inserted === '앞가\n😀\n끝뒤😀' && row.reopened === row.inserted,
      `${label}: 정규화·HWP 저장 후 suffix 보존`);
    assert(row.caret === 6 && row.redoCaret === 6, `${label}: 정규화된 scalar 커서`);
    assert(row.undone === '앞뒤😀' && row.redone === row.inserted, `${label}: Undo/Redo 보존`);
    assert(row.invalidation === '가\n😀\n끝', `${label}: repaint도 같은 입력 소비`);
  }
  for (const row of result.searches) {
    assert(row.hits === row.count && row.replaced === row.count && row.reopened === row.expected,
      `${JSON.stringify(row.before)}: 겹치지 않는 검색·치환·저장 보존`);
  }
  assert(result.ui.inserted === '앞가\n😀\n끝뒤😀' && result.ui.caret === 6 && result.ui.canUndo,
    '실제 InputHandler 삽입·캐럿·history 기록');
  assert(result.ui.undone === '앞뒤😀' && result.ui.redone === result.ui.inserted,
    '실제 InputHandler Undo/Redo suffix 보존');
});
