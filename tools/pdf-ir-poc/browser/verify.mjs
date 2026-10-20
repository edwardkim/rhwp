import assert from "node:assert/strict";
import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { createRequire } from "node:module";
import { nestedPdf } from "../fixtures/nested-grid.mjs";

const root = path.dirname(new URL(import.meta.url).pathname);
const repo = process.argv[2];
const out = process.argv[3];
const artifacts=path.resolve(out || ".","../..");
if (!repo || !out) throw new Error("usage: node verify.mjs REPO OUTPUT");
const require = createRequire(path.join(repo, "rhwp-studio/package.json"));
const puppeteer = require("puppeteer-core");
const inputs = {
  text: path.join(repo,"pdf/re-align-center-hancom-hwp-2020.pdf"),
  table: path.join(repo,"pdf/issue5875/nested_table_text_caption-2020.pdf"),
  nested: path.join(artifacts,"nested-input.pdf"),
  real7: path.join(artifacts,"source/page-7.pdf"),
  real8: path.join(artifacts,"source/page-8.pdf")
};
const mime = {".js":"text/javascript", ".html":"text/html", ".wasm":"application/wasm", ".pdf":"application/pdf"};
const server = http.createServer(async (request,response) => {
  try {
    const url = new URL(request.url, "http://localhost");
    if (url.pathname==="/favicon.ico") {response.writeHead(204);response.end();return;}
    const name = url.pathname.match(/^\/pdf\/(text|table|nested|real7|real8)\.pdf$/)?.[1];
    const fileRoot=url.pathname.startsWith("/pkg/")?path.join(artifacts,"browser"):root;
    const file = name ? inputs[name] : path.resolve(fileRoot, "." + (url.pathname === "/" ? "/index.html" : url.pathname));
    if (!name && !file.startsWith(fileRoot + path.sep)) throw new Error("invalid path");
    const bytes = await fs.readFile(file);
    response.writeHead(200, {"Content-Type":mime[path.extname(file)] || "application/octet-stream"});
    response.end(bytes);
  } catch { response.writeHead(404); response.end(); }
});
await new Promise(resolve=>server.listen(0,"127.0.0.1",resolve));
const base = `http://127.0.0.1:${server.address().port}`;
let browser;
const result = { passed:false, requests:[], consoleErrors:[], consoleMessages:[], cases:[],stage:"launch" };
try {
  await fs.mkdir(out,{recursive:true});
  browser = await puppeteer.launch({ executablePath:process.env.CHROME_PATH || "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome", headless:true,protocolTimeout:30000 });
  const page = await browser.newPage();
  await page.setViewport({width:1000,height:1200,deviceScaleFactor:1});
  page.on("request",request=>result.requests.push(request.url()));
  page.on("pageerror",error=>result.consoleErrors.push(String(error)));
  page.on("console",message=>result.consoleMessages.push({type:message.type(),text:message.text()}));
  await page.setRequestInterception(true);
  page.on("request",request=>{
    if (request.url().startsWith(base) || request.url().startsWith("blob:") || request.url().startsWith("data:")) request.continue();
    else request.abort();
  });
  result.stage="navigation";
  await page.goto(base,{waitUntil:"networkidle0"});
  result.stage="worker-initialization";
  await page.waitForFunction(()=>window.pocReady || window.workerFatal,{timeout:15000});
  const fatal=await page.evaluate(()=>window.workerFatal);
  if (fatal) throw new Error(fatal);
  result.stage="invalid-input";
  const invalid = await page.evaluate(async()=>{
    try { await window.compileBuffer(new Uint8Array([1,2,3]).buffer); return null; }
    catch(error) { return String(error); }
  });
  assert.match(invalid,/PDF byte buffers/);
  result.invalidInputRejected=invalid;
  assert.deepEqual(await fs.readFile(inputs.nested),await nestedPdf());
  for (const [name,paintedRules,tables] of [["text",false,false],["table",false,false],["table",true,false],["table",true,true],["nested",true,true],["real7",true,true],["real8",true,true]]) {
    const label=tables?`${name}-structured`:paintedRules?`${name}-painted`:name;
    result.stage=label;
    const evidence = await page.evaluate(({name,paintedRules,tables})=>window.runPdf(name,paintedRules,tables),{name,paintedRules,tables});
    const native = JSON.parse(await fs.readFile(path.join(artifacts,label+"-page/report.json"),"utf8"));
    assert.deepEqual(evidence.report,native,"Native/WASM IR reports differ");
    const detection=evidence.report.extraction.tableDetection;
    const recognition=evidence.report.extraction.recognition;
    assert.equal(recognition.algorithm,"hierarchical-efsm-v2");
    assert.equal(recognition.renderingIrChanged,false);
    const paragraphRegions=[...recognition.bodyRegions];
    for (const table of recognition.tables) {
      assert.equal(table.status,"GeometryConfirmed");
      assert.equal(table.semanticStatus,"unverified");
      assert.deepEqual(table.transitions.map(t=>t.event),["CandidateFound","ValidateGeometry","GeometryAccepted","BeginCells","EndCells"]);
      paragraphRegions.push(...table.cells.map(c=>c.recognition));
    }
    const paragraphs=paragraphRegions.flatMap(r=>r.paragraphs);
    const refs=paragraphs.flatMap(p=>p.spanIndices).sort((a,b)=>a-b);
    assert.deepEqual(refs,evidence.report.extraction.spans.map((_,i)=>i));
    if (!name.startsWith("real")) assert.ok(paragraphs.every(p=>p.status==="Confirmed"));
    if (name==="text") {
      assert.equal(paragraphs.length,1);
      assert.equal(paragraphs[0].lines.length,3);
    } else if (name==="table") {
      const pictureCell=recognition.tables[0].cells.find(c=>c.row===2 && c.column===2);
      assert.deepEqual(pictureCell.imageIndices,[0,1]);
      assert.equal(pictureCell.recognition.transitions.filter(t=>t.event==="AcceptBoundary" && t.reason==="intervening_image").length,2);
    }
    assert.equal(detection.tables.length,name.startsWith("real")?5:name==="nested"?3:name==="table"?1:0);
    const ir=evidence.report.documentIr.paragraphs[0].controls;
    const irTables=ir.filter(c=>c.Table).map(c=>c.Table);
    assert.equal(irTables.length,tables?1:0);
    if(tables && evidence.report.lowering.deferredTables.length===0) {
      const owned=new Set(detection.tables.flatMap(t=>t.sourcePathIndices));
      const rootRuleIds=new Set(ir.flatMap(c=>{
        const r=c.Shape?.Rectangle;
        return r && !r.drawing.text_box && r.drawing.fill.solid ? [r.common.instance_id]:[];
      }));
      for(const source of evidence.report.lowering.ruleSourceControls) {
        assert.equal(rootRuleIds.has(source.instanceId),!owned.has(source.path),
          "source boundary must have exactly one owner; unrelated rules must remain");
      }
    }
    if (name==="nested") {
      assert.deepEqual(detection.tables.map(t=>t.hierarchy.depth),[0,1,2]);
      assert.ok(detection.tables.every(t=>t.topology.boundarySquaredZero));
      assert.deepEqual(detection.tables.map(t=>t.topology.betti),[[1,0,0],[1,0,0],[1,0,0]]);
      assert.deepEqual(detection.ambiguousContent,[]);
      assert.equal(evidence.report.lowering.semanticTables,3);
      assert.equal(evidence.report.lowering.semanticNestedTables,2);
      assert.deepEqual(evidence.report.lowering.deferredTables,[]);
      assert.equal(evidence.report.lowering.paintedRuleRectangles,0);
      assert.equal(evidence.report.lowering.textBoxes,0);
      const descendants=t=>t.cells.flatMap(c=>c.paragraphs.flatMap(p=>p.controls.filter(c=>c.Table).map(c=>c.Table)));
      const child=descendants(irTables[0]);
      assert.equal(child.length,1);
      assert.equal(descendants(child[0]).length,1);
    }
    if (name.startsWith("real")) {
      assert.equal(evidence.report.lowering.semanticTables,5);
      assert.equal(evidence.report.lowering.semanticRootTables,1);
      assert.equal(evidence.report.lowering.semanticNestedTables,4);
      assert.deepEqual(evidence.report.lowering.deferredTables,[]);
    }
    if (name==="table") {
      assert.equal(detection.tables[0].rows,5);
      assert.equal(detection.tables[0].columns,3);
      assert.equal(detection.tables[0].cells.length,15);
      assert.deepEqual(detection.ambiguousContent,[]);
      const cell=detection.tables[0].cells.find(c=>c.row===2 && c.column===2);
      assert.deepEqual(cell.imageIndices,[0,1]);
      assert.equal(evidence.report.lowering.semanticTables,tables?1:0);
      assert.equal(evidence.report.lowering.straightRules,paintedRules?0:14);
      assert.equal(evidence.report.lowering.paintedRuleRectangles,paintedRules && !tables?14:0);
      assert.equal(evidence.report.lowering.textBoxes,tables?1:95);
      if(tables) {
        assert.equal(irTables[0].cells.length,15);
        assert.deepEqual([irTables[0].row_count,irTables[0].col_count],[5,3]);
        assert.deepEqual(evidence.report.lowering.deferredTables,[]);
      }
      const pictures=tables ? irTables[0].cells.find(c=>c.row===2 && c.col===2).paragraphs.flatMap(p=>p.controls).filter(c=>c.Picture).map(c=>c.Picture)
        :ir.filter(c=>c.Picture).map(c=>c.Picture);
      assert.equal(pictures.length,2);
      for(const pic of pictures) {
        assert.equal(pic.common.treat_as_char,tables);
        assert.deepEqual(pic.border_x,[0,0,pic.common.width,0]);
        assert.deepEqual(pic.border_y,[pic.common.width,pic.common.height,0,pic.common.height]);
        assert.deepEqual([pic.crop.right,pic.crop.bottom],pic.img_dim);
      }
    }
    assert.equal(evidence.pageCount,1);
    const textBoxCount=value=>{
      if(Array.isArray(value)) return value.reduce((sum,v)=>sum+textBoxCount(v),0);
      if(value && typeof value==="object") return Object.entries(value).reduce((sum,[key,v])=>
        sum+(key==="text_box" && v!==null?1:0)+textBoxCount(v),0);
      return 0;
    };
    assert.equal(evidence.report.lowering.explicitZeroTextMargins,textBoxCount(ir));
    assert.equal(evidence.report.lowering.singleLineParagraphs,"SQUEEZE");
    assert.ok(evidence.inkPixels>100);
    assert.equal(evidence.images,name==="table"?2:0);
    assert.ok(evidence.hwpx.length>1000);
    await fs.writeFile(path.join(out,label+"-wasm.hwpx"),new Uint8Array(evidence.hwpx));
    await fs.writeFile(path.join(out,label+"-wasm.svg"),evidence.svg);
    await fs.writeFile(path.join(out,label+"-wasm-report.json"),JSON.stringify(evidence.report,null,2));
    await (await page.$("canvas")).screenshot({path:path.join(out,label+"-wasm.png")});
    const nativeSvg=await fs.readFile(path.join(artifacts,label+"-page/native.svg"),"utf8");
    await page.evaluate(async svg=>{
      const canvas=document.querySelector("canvas");
      const image=new Image();
      const url=URL.createObjectURL(new Blob([svg],{type:"image/svg+xml"}));
      try {
        image.src=url;await image.decode();
        const ctx=canvas.getContext("2d");ctx.fillStyle="white";
        ctx.fillRect(0,0,canvas.width,canvas.height);ctx.drawImage(image,0,0,canvas.width,canvas.height);
      } finally { URL.revokeObjectURL(url); }
    },nativeSvg);
    await (await page.$("canvas")).screenshot({path:path.join(out,label+"-native.png")});
    result.cases.push({ name:label, elapsedMs:evidence.elapsedMs, inkPixels:evidence.inkPixels,
      canvasSize:evidence.canvasSize, images:evidence.images,pageCount:evidence.pageCount,
      lowering:evidence.report.lowering,detectedTables:detection.tables.length,
      paragraphCandidates:paragraphs.length,recognitionTraceEqual:true,allSpanRefsPreserved:true,nativeWasmIrEqual:true });
  }
  assert.deepEqual(result.consoleErrors,[]);
  assert.deepEqual(result.consoleMessages.filter(m=>m.type==="error"),[]);
  result.browser=await browser.version();
  result.passed=true;
  result.stage="complete";
  console.log(JSON.stringify(result,null,2));
} catch(error) { result.error=String(error); throw error; }
finally {
  await fs.mkdir(out,{recursive:true});
  await fs.writeFile(path.join(out,"browser-verification.json"),JSON.stringify(result,null,2));
  await browser?.close();
  await new Promise(resolve=>server.close(resolve));
}
