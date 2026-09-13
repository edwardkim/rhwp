// D2 transport contract, not a Hancom visual oracle or a Gym dependency.
// Run from repository root after the standard Docker WASM build.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {pathToFileURL} from 'node:url';

const root = process.cwd();
const output = path.resolve(process.argv[2] ?? 'output/3587/d2-wasm');
const wasm = fs.readFileSync(path.join(root, 'pkg/rhwp_bg.wasm'));
const {default: init, HwpDocument} = await import(pathToFileURL(path.join(root, 'pkg/rhwp.js')).href);
await init({module_or_path: wasm});
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const input = fs.readFileSync('samples/table-in-tbox.hwp');
const base = new HwpDocument(input);
const derived = base.exportHwpx();
base.free();
const results = [];
for (const [format, bytes] of [['hwp', input], ['derived-hwpx', derived]]) {
  const source = new HwpDocument(bytes);
  const target = HwpDocument.createEmpty();
  try {
    target.createBlankDocument();
    target.insertText(0, 0, 0, 'BEFORE');
    target.splitParagraph(0, 0, 6);
    target.insertText(0, 1, 0, 'AFTER');
    const state = d => [sha(d.exportHwp()), sha(d.exportHwpx()), d.getDocumentInfo()];
    const sourceBefore = state(source), targetBefore = state(target);
    const unchanged = () => {
      assert.deepEqual(state(source), sourceBefore);
      assert.deepEqual(state(target), targetBefore);
    };
    const request = {sourceSection:0, sourceStart:4, sourceEnd:5,
      targetSection:0, insertBefore:1, count:2};
    // Distinct handles are an ABI precondition. Reject aliasing in JS before
    // wasm-bindgen acquires a mutable/shared borrow of the same Rust value.
    const call = (destination, origin, options) => {
      if (destination === origin) throw new TypeError('import requires distinct document handles');
      return destination.importParagraphBlock(origin, JSON.stringify(options));
    };
    const invoke = options => JSON.parse(call(target, source, options));
    const preview = invoke({request, dryRun:true});
    unchanged();
    assert.equal(preview.dryRun, true);
    assert.equal(preview.schemaVersion, '1.0');
    assert.equal(preview.changedPages, null);
    assert.equal(preview.operationResult.action, 'import_paragraph_block');
    assert.deepEqual(preview.operationResult.result.inserted, {start:1, end:3});
    assert.equal(preview.operationResult.result.copies.length, 2);
    const errors = [];
    for (const [label, json] of [
      ['json', '{'], ['oversize', ' '.repeat(8 * 1024 * 1024 + 1)],
      ['boolean', JSON.stringify({request, dryRun:'false'})],
      ['unknown', JSON.stringify({request, sourceBytes:[]})],
      ...[true, false].map(dryRun => ['boundary-' + dryRun,
        JSON.stringify({request:{...request, insertBefore:999}, dryRun})]),
    ]) {
      assert.throws(() => target.importParagraphBlock(source, json), error => {
        errors.push({label, message:String(error)});
        return true;
      });
      unchanged();
    }
    assert.equal(errors.at(-1).message, errors.at(-2).message);
    // This tests caller-side preflight, NOT native recovery from ABI misuse.
    assert.throws(() => call(target, target, {request}), /distinct document handles/);
    unchanged();
    const noop = invoke({request:{...request, count:0}});
    assert.equal(noop.operationResult.result.copies.length, 0);
    unchanged();
    const applied = invoke({request});
    assert.equal(applied.dryRun, false);
    assert.deepEqual(applied.operationResult, preview.operationResult);
    assert.equal(target.getParagraphCount(0), 4);
    assert.deepEqual(state(source), sourceBefore);
    assert(target.getTextFileText().includes('BEFORE'));
    assert(target.getTextFileText().includes('AFTER'));
    const reopenedResults = [];
    for (const [savedFormat, saved] of [['hwp', target.exportHwp()], ['hwpx', target.exportHwpx()]]) {
      const reopened = new HwpDocument(saved);
      try {
        assert.equal(reopened.getParagraphCount(0), 4);
        assert(reopened.getTextFileText().includes('BEFORE'));
        assert(reopened.getTextFileText().includes('AFTER'));
        assert(reopened.getTextFileText().includes('수질검사'));
        reopenedResults.push({format:savedFormat, sha256:sha(saved)});
      } finally { reopened.free(); }
    }
    results.push({format, inputSha256:sha(bytes), preview, applied, errors, reopenedResults});
  } finally { source.free(); target.free(); }
}
fs.mkdirSync(output, {recursive:true});
fs.writeFileSync(path.join(output, 'result.json'), JSON.stringify({
  wasmSha256:sha(wasm), sourceSha256:sha(input),
  scope:'real WASM transport; derived HWPX is not an independent Hancom oracle', results,
}, null, 2) + '\n');
console.log(`PASS: ${results.length} input formats, dry-run/apply, errors, caller alias preflight, zero count and both save formats`);
