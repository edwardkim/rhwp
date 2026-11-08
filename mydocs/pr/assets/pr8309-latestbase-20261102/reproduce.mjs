// Reproduce the public synthetic storage actions with a selected real WASM package.
// Usage: node reproduce.mjs <absolute-pkg-dir> <new-output-dir>
import {mkdirSync, readFileSync, writeFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';

const [packageArg, outputArg] = process.argv.slice(2);
if (!packageArg || !outputArg) throw new Error('Pass a WASM package and a new output directory');
const pkg = resolve(packageArg);
const output = resolve(outputArg);
mkdirSync(output, {recursive: true});
const {default: init, HwpDocument} = await import(pathToFileURL(resolve(pkg, 'rhwp.js')).href);
const sha256 = (bytes) => createHash('sha256').update(bytes).digest('hex');
const wasm = readFileSync(resolve(pkg, 'rhwp_bg.wasm'));
await init({module_or_path: wasm});
const document = HwpDocument.createEmpty();
document.createBlankDocument();
const location = JSON.parse(document.createShapeControl(JSON.stringify({
  sectionIdx: 0, paraIdx: 0, charOffset: 0, width: 18000, height: 12000,
  treatAsChar: true, shapeType: 'textbox',
})));
document.insertTextInCell(0, location.paraIdx, location.controlIdx, 0, 0, 0,
  'Text box alignment ABC 123');
const cases = [];
for (const action of ['default', 'Center', 'Bottom', 'Top']) {
  if (action !== 'default') {
    document.setShapeProperties(0, location.paraIdx, location.controlIdx,
      JSON.stringify({tbVerticalAlign: action}));
  }
  const expected = action === 'default' ? 'Top' : action;
  for (const format of ['hwp', 'hwpx']) {
    const bytes = format === 'hwp' ? document.exportHwp() : document.exportHwpx();
    const filename = `textbox-saved-${action.toLowerCase()}.${format}`;
    writeFileSync(resolve(output, filename), bytes, {flag: 'wx'});
    const reopened = new HwpDocument(bytes);
    const actual = JSON.parse(reopened.getShapeProperties(0, location.paraIdx, location.controlIdx));
    cases.push({action, format, expected, actual: actual.tbVerticalAlign,
      preserved: actual.tbVerticalAlign === expected, pages: reopened.pageCount(),
      textPresent: reopened.getPageRenderTree(0).includes('Text box alignment ABC 123'),
      input: filename, sha256: sha256(bytes)});
    reopened.free();
  }
}
document.free();
const report = {wasm_sha256: sha256(wasm), cases};
writeFileSync(resolve(output, 'roundtrip.json'), `${JSON.stringify(report, null, 2)}\n`, {flag: 'wx'});
console.log(JSON.stringify(report));
if (!cases.every((item) => item.preserved && item.textPresent)) process.exitCode = 1;
