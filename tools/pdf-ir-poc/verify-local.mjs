import fs from "node:fs/promises";
import assert from "node:assert/strict";
import path from "node:path";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { gzipSync } from "node:zlib";
import { nestedPdf } from "./fixtures/nested-grid.mjs";

const root=path.dirname(new URL(import.meta.url).pathname);
const repo=path.resolve(process.argv[2] || "");
const bindgen=process.argv[3];
if (!process.argv[2] || !bindgen) throw new Error("usage: node verify-local.mjs RHWP_REPO WASM_BINDGEN_BINARY [OUTPUT]");
const output=path.resolve(process.argv[4] || path.join(repo,"output/poc/pdf-ir"));
const evidence=path.join(output,"verification");
await fs.mkdir(evidence,{recursive:true});
const commands=[];
function run(name,executable,args,env={}) {
  const started=performance.now();
  const result=spawnSync(executable,args,{cwd:root,env:{...process.env,...env},encoding:"utf8",timeout:600000,maxBuffer:16*1024*1024});
  const record={name,command:[executable,...args],exitCode:result.status,elapsedMs:performance.now()-started,
    stdout:result.stdout,stderr:result.stderr,error:result.error?String(result.error):undefined};
  commands.push(record);
  if (result.status!==0) throw new Error(`${name} failed: ${result.error || result.stderr}`);
  console.log(`${name}: PASS (${Math.round(record.elapsedMs)} ms)`);
}
try {
  const shared=["--release","--offline","--locked","--manifest-path",path.join(root,"Cargo.toml"),"--target-dir",path.join(repo,"target/pr-review")];
  run("fmt","cargo",["+1.93.1","fmt","--manifest-path",path.join(root,"Cargo.toml"),"--","--check"]);
  run("contracts","cargo",["+1.93.1","test",...shared,"--tests"],{RHWP_POC_REPO:repo});
  run("native-clippy","cargo",["+1.93.1","clippy",...shared,"--all-targets","--no-deps","--","-D","warnings"]);
  run("wasm-clippy","cargo",["+1.93.1","clippy",...shared,"--lib","--target","wasm32-unknown-unknown","--no-deps","--","-D","warnings"]);
  const inputs={text:"pdf/re-align-center-hancom-hwp-2020.pdf",table:"pdf/issue5875/nested_table_text_caption-2020.pdf"};
  run("upstream-detection-probe","cargo",["+1.93.1","run",...shared,"--example","table_probe","--",path.join(repo,inputs.table)]);
  for (const [name,input] of Object.entries(inputs)) {
    run(`${name}-native`,"cargo",["+1.93.1","run",...shared,"--",path.join(repo,input),path.join(output,name+"-page")]);
  }
  run("table-painted-native","cargo",["+1.93.1","run",...shared,"--",path.join(repo,inputs.table),path.join(output,"table-painted-page"),"--painted-rules"]);
  run("table-structured-native","cargo",["+1.93.1","run",...shared,"--",path.join(repo,inputs.table),path.join(output,"table-structured-page"),"--tables"]);
  await fs.writeFile(path.join(output,"nested-input.pdf"),await nestedPdf());
  for (const mode of ["painted","structured"]) {
    run(`nested-${mode}-native`,"cargo",["+1.93.1","run",...shared,"--",path.join(output,"nested-input.pdf"),path.join(output,`nested-${mode}-page`),mode==="painted"?"--painted-rules":"--tables"]);
  }
  await fs.mkdir(path.join(output,"source"),{recursive:true});
  run("split-real-nested-pages","pdfseparate",["-f","7","-l","8",path.join(repo,inputs.table),path.join(output,"source/page-%d.pdf")]);
  for (const page of [7,8]) run(`real${page}-structured-native`,"cargo",["+1.93.1","run",...shared,"--",path.join(output,`source/page-${page}.pdf`),path.join(output,`real${page}-structured-page`),"--tables"]);
  run("wasm-build","cargo",["+1.93.1","build",...shared,"--lib","--target","wasm32-unknown-unknown"]);
  run("wasm-bindgen",bindgen,[path.join(repo,"target/pr-review/wasm32-unknown-unknown/release/rhwp_pdf_oxide_poc.wasm"),"--target","web","--out-dir",path.join(output,"browser/pkg")]);
  run("browser",process.execPath,[path.join(root,"browser/verify.mjs"),repo,path.join(output,"browser/evidence")]);
  const sha=bytes=>createHash("sha256").update(bytes).digest("hex");
  const sourceSha=execFileSync("git",["rev-parse","HEAD"],{cwd:repo,encoding:"utf8"}).trim();
  const diagnostics=await Promise.all(["examples/host_probe.rs","examples/cell_flow_probe.rs"].map(async file=>({file,sha256:sha(await fs.readFile(path.join(root,file)))})));
  const comparisons=[];
  for (const [name,input] of Object.entries(inputs)) {
    run(`${name}-independent-text`,"pdftotext",["-f","1","-l","1","-layout","-enc","UTF-8",path.join(repo,input),"-"]);
    const compiled=JSON.parse(await fs.readFile(path.join(output,name+"-page/report.json"),"utf8"));
    const a=commands.at(-1).stdout.replace(/\s/g,"");
    const b=compiled.extraction.spans.map(s=>s.text).join("").replace(/\s/g,"");
    assert.equal([...a].sort().join(""),[...b].sort().join(""),"independent PDF text content differs");
    comparisons.push({name,independentExtractor:"Poppler pdftotext",sourceCharacters:[...a].length,
      recoveredCharacters:[...b].length,readingOrderEqual:a===b,
      characterMultisetEqual:[...a].sort().join("")===[...b].sort().join("")});
  }
  const wasm=await fs.readFile(path.join(output,"browser/pkg/rhwp_pdf_oxide_poc_bg.wasm"));
  await fs.writeFile(path.join(evidence,"manifest.json"),JSON.stringify({
    sourceSha,sourceTreeDirty:execFileSync("git",["status","--porcelain","--untracked-files=normal"],{cwd:repo,encoding:"utf8"}).trim().length>0,
    rustToolchain:"1.93.1",pdfOxide:"0.3.78",officeOxide:"0.1.9",diagnostics,
    inputHashes:await Promise.all(Object.entries(inputs).map(async([name,input])=>({name,input,sha256:sha(await fs.readFile(path.join(repo,input)))}))),
    wasmBytes:wasm.length,wasmGzipBytes:gzipSync(wasm).length,wasmSha256:sha(wasm),
    pocSources:await Promise.all(["Cargo.toml","Cargo.lock","src/lib.rs","src/lowering.rs","src/structured_lowering.rs","src/hwpx_compatibility.rs","src/main.rs","src/table_detection.rs","src/table_hierarchy.rs","src/table_topology.rs","src/recognition.rs","tests/nested_materialization.rs","tests/nested_tables.rs","tests/table_detection.rs","tests/contracts.rs","tests/structured.rs","fixtures/nested-grid.content","fixtures/nested-grid.mjs","browser/verify.mjs","browser/worker.js","browser/index.html","verify-local.mjs","verify-visual.mjs"].map(async file=>({file,sha256:sha(await fs.readFile(path.join(root,file)))}))),
    comparisons,hancomCompatibility:"unverified-awaiting-maintainer-retest",
    limits:"selected first pages only; optional inferred Table IR; new structured output requires Hancom verification"
  },null,2));
} finally {
  await fs.writeFile(path.join(evidence,"commands.json"),JSON.stringify(commands,null,2));
}
