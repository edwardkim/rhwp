import { readFile, writeFile } from 'node:fs/promises';
import { resolve, join } from 'node:path';
import { pathToFileURL } from 'node:url';

const [pkg, file, generate] = process.argv.slice(2);
if (!pkg || !file || (generate && generate !== '--make')) {
  throw new Error('Usage: node equation-probe.mjs <wasm-pack web pkg> <sample.hwpx> [--make]');
}
const { default: initialize, HwpDocument } = await import(pathToFileURL(resolve(pkg, 'rhwp.js')).href);
await initialize({ module_or_path: await readFile(join(pkg, 'rhwp_bg.wasm')) });
globalThis.measureTextWidth = (font, text) => {
  const pixels = Number(/(\d+(?:\.\d+)?)px/.exec(font)?.[1] ?? 13);
  return Array.from(text).reduce((width, ch) => width + pixels * (ch.charCodeAt(0) > 0x2e80 ? 1 : 0.5), 0);
};
if (generate) {
  const author = HwpDocument.createEmpty();
  try {
    author.createBlankDocument();
    author.insertText(0, 0, 0, 'next');
    author.insertEquation(0, 0, 0, 'x+1', 1200, 0);
    await writeFile(file, author.exportHwpx());
  } finally {
    author.free();
  }
}
const doc = new HwpDocument(await readFile(file));
try {
  const tree = JSON.parse(doc.getPageRenderTree(0));
  const findEquation = node => node.type === 'Equation' ? node.bbox : node.children?.map(findEquation).find(Boolean);
  const equation = findEquation(tree);
  if (!equation) throw new Error('The input should contain an inline equation in its first character slot.');
  const caret = [0, 1, 2].map(offset => JSON.parse(doc.getCursorRect(0, 0, offset)));
  const selected = [[0, 1], [0, 2], [1, 2]].map(([start, end]) => ({
    range: [start, end],
    rectangles: JSON.parse(doc.getSelectionRects(0, 0, start, 0, end)),
  }));
  console.log(JSON.stringify({equation, caret, selected}, null, 2));
  if (!selected[0].rectangles.some(rect => rect.x <= equation.x + 1 && rect.x + rect.width >= equation.x + equation.w - 1)) {
    process.exitCode = 1;
  }
} finally {
  doc.free();
}
