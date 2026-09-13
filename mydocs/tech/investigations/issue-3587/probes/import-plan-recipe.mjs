// Run from the repository root. Use a NEW output directory; never overwrite evidence.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {isDeepStrictEqual} from 'node:util';

const binary = path.resolve(process.argv[2] ?? 'target/pr-review/release-test/rhwp');
const output = path.resolve(process.argv[3] ?? 'output/3587/d2-cli');
fs.mkdirSync(path.dirname(output), {recursive:true});
fs.mkdirSync(output); // EEXIST intentionally stops a rerun from replacing accepted files.
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const sample = fs.readFileSync('samples/rnote/labnote-001.hwp');
const sourceHwp = path.join(output, 'source.hwp');
const sourceHwpx = path.join(output, 'source-derived.hwpx');
fs.writeFileSync(sourceHwp, sample, {flag:'wx'});
execFileSync(binary, ['export-hwpx', sourceHwp, sourceHwpx]);
const run = plan => JSON.parse(execFileSync(binary, ['run', '--plan-json', JSON.stringify(plan), '--json'], {encoding:'utf8'}));
const writeJson = (name, data) => fs.writeFileSync(path.join(output, name), JSON.stringify(data,null,2)+'\n', {flag:'wx'});
const results = [];
for (const [ext, source] of [['hwp',sourceHwp], ['hwpx',sourceHwpx]]) {
  const bytes = fs.readFileSync(source);
  const input = path.join(output, `target.${ext}`);
  const imported = path.join(output, `imported.${ext}`);
  const filled = path.join(output, `filled.${ext}`);
  fs.writeFileSync(input, bytes, {flag:'wx'});
  const request = {sourceSection:0,sourceStart:12,sourceEnd:13,targetSection:0,insertBefore:13,count:1};
  const plan = {planVersion:'1.0',input,output:imported,preconditions:{inputSha256:sha(bytes)},
    steps:[{action:'import_paragraph_block',source:{path:source,sha256:sha(bytes)},request}]};
  writeJson(`import-${ext}.plan.json`, plan);
  const preview = run({...plan,dryRun:true});
  assert(!fs.existsSync(imported));
  const applied = run(plan);
  assert.deepEqual(preview.preview[0].operationResult, applied.steps[0].operationResult);
  const result = applied.steps[0].operationResult.result;
  const relativeSource = [{kind:'paragraph',index:0},{kind:'control',index:1},{kind:'cell',index:5},{kind:'paragraph',index:0}];
  const mapping = result.copies[0].mappings.find(m => isDeepStrictEqual(m.source, relativeSource));
  assert(mapping, 'cell mapping required; never guess an output cell index');
  const relative = structuredClone(mapping.destination);
  relative[0].index -= result.inserted.start;
  const fillPlan = {planVersion:'1.0',input:imported,output:filled,
    preconditions:{inputSha256:applied.outputSha256},steps:[{action:'fill_template',request:{
      scope:{sectionIndex:result.targetSection,start:result.inserted.start,end:result.inserted.end},
      bindings:[{key:'body',target:{kind:'textRange',path:relative,start:0,end:0}}],
      record:{body:'가져온 연구노트'}}}]};
  writeJson(`fill-${ext}.plan.json`, fillPlan);
  const fill = run(fillPlan);
  const verified = JSON.parse(execFileSync(binary, ['verify',filled,'--expect-contains','가져온 연구노트','--json'], {encoding:'utf8'}));
  assert.equal(sha(fs.readFileSync(source)), sha(bytes));
  assert.equal(sha(fs.readFileSync(input)), sha(bytes));
  results.push({format:ext,preview,applied,fill,verified,filled});
}
writeJson('result.json', {binarySha256:sha(fs.readFileSync(binary)),sourceSha256:sha(sample),
  scope:'CLI import and returned-path fill; derived HWPX is not an independent Hancom oracle',results});
console.log(`PASS: HWP/HWPX import -> mapped fill -> save/reopen: ${output}`);
