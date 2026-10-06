const fs = require('node:fs');
const vm = require('node:vm');
const assert = require('node:assert/strict');
const ts = require(require('node:path').resolve('rhwp-vscode/node_modules/typescript'));
const source = fs.readFileSync('rhwp-vscode/src/webview/viewer.ts', 'utf8');
function extract(name) {
  const start = source.indexOf(`function ${name}(`);
  const open = source.indexOf('{', start);
  let depth = 1, end = open + 1;
  for (; depth; end++) { if (source[end] === '{') depth++; if (source[end] === '}') depth--; }
  return source.slice(start, end);
}
const calls = [];
const context = {
  hwpDoc: {
    getCursorRect(...args) { calls.push(['body', ...args]); return JSON.stringify({pageIndex: 1, y: 25, height: 10}); },
    getCursorRectByPath(...args) { calls.push(['cell', ...args]); return JSON.stringify({pageIndex: 2, y: 80, height: 15}); },
  },
  scrollToDocumentPosition(...args) { calls.push(['scroll', ...args]); },
  scrollToPage(...args) { calls.push(['fallback', ...args]); },
};
vm.createContext(context);
vm.runInContext(ts.transpileModule(extract('outlineKey') + '\n' + extract('navigateToOutline'), {compilerOptions:{target:ts.ScriptTarget.ES2022}}).outputText, context);
const body = {section:0, paragraph:4, page:2};
const cell = {...body, cellPath:[{controlIndex:0,cellIndex:0,cellParaIndex:1},{controlIndex:0,cellIndex:0,cellParaIndex:0}]};
assert.notEqual(context.outlineKey(body), context.outlineKey(cell));
assert.notEqual(context.outlineKey(cell), context.outlineKey({...cell, cellPath:[{controlIndex:0,cellIndex:0,cellParaIndex:2},cell.cellPath[1]]}));
context.navigateToOutline(body);
context.navigateToOutline(cell);
assert.deepEqual(calls, [['body',0,4,0],['scroll',1,25,10],['cell',0,4,JSON.stringify(cell.cellPath),0],['scroll',2,80,15]]);
context.hwpDoc.getCursorRectByPath = () => {throw new Error('no rect');};
context.navigateToOutline(cell);
assert.deepEqual(calls.at(-1), ['fallback',1]);
console.log('PASS: body/cell routes, nested paragraph identity, exact coordinates, fallback');
