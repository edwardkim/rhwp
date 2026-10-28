import { createRequire } from 'node:module';
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { loadApp } from '/tmp/rhwp-pr7518-review-20261004/rhwp-studio/e2e/helpers.mjs';
const root='/tmp/rhwp-pr7518-review-20261004';
const out=root+'/output/pr-review/pr7518-20261004/cdp-p39-frame';
mkdirSync(out,{recursive:true});
const sha=b=>createHash('sha256').update(b).digest('hex');
const require=createRequire(root+'/rhwp-studio/package.json');
const browser=await require('puppeteer-core').connect({browserURL:'http://localhost:19222',defaultViewport:null});
const page=await browser.newPage();
await page.setCacheEnabled(false);
const checks=[],errors=[],reads=[],served=[];
const check=(name,passed,detail)=>checks.push({name,passed:!!passed,detail});
const nodes=n=>[n,...(n.children||[]).flatMap(nodes)];
page.on('pageerror',e=>errors.push(e.message));
page.on('response',r=>{if(r.status()===200 && new URL(r.url()).pathname.endsWith('rhwp_bg.wasm')) reads.push(r.buffer().then(b=>served.push(sha(b))));});
const input='samples/76076_regulatory_analysis.hwp';
const bytes=readFileSync(root+'/'+input);
const pkgHash=sha(readFileSync(root+'/pkg/rhwp_bg.wasm'));
const publicHash=sha(readFileSync(root+'/rhwp-studio/public/rhwp_bg.wasm'));
try {
 await page.setViewport({width:1440,height:1100,deviceScaleFactor:1});
 await loadApp(page); await Promise.all(reads);
 check('fresh compiled WASM served with cache disabled',pkgHash===publicHash && served.includes(pkgHash),{pkgHash,publicHash,served});
 const result=await page.evaluate(async data=>{
  const info=window.__wasm.loadDocument(new Uint8Array(data),'76076_regulatory_analysis.hwp');
  const {loadWebFonts}=await import('/src/core/font-loader.ts');
  await loadWebFonts(info.fontsUsed||[]); await document.fonts.ready;
  await window.__canvasView.loadDocument();
  const d=window.__wasm.doc;
  const module=await import('/rhwp.js');
  await module.default({module_or_path:'/rhwp_bg.wasm'});
  const portable=new module.HwpDocument(new Uint8Array(data));
  const result={count:d.pageCount(),tree:JSON.parse(d.getPageRenderTree(38)),portableCount:portable.pageCount(),portableTree:JSON.parse(portable.getPageRenderTree(38)),svg:portable.renderPageSvgWithProfile(38,'print'),controls:[21,37].map(index=>({index,tree:JSON.parse(portable.getPageRenderTree(index)),svg:portable.renderPageSvgWithProfile(index,'print')}))};
  portable.free();return result;
 },Array.from(bytes));
 writeFileSync(out+'/tree.json',JSON.stringify({count:result.count,tree:result.tree,portableCount:result.portableCount,portableTree:result.portableTree}));
 writeFileSync(out+'/wasm_039.svg',result.svg);
 for(const control of result.controls){const num=String(control.index+1).padStart(3,'0');writeFileSync(out+'/wasm_'+num+'.svg',control.svg);writeFileSync(out+'/tree_'+num+'.json',JSON.stringify(control.tree));}
 const body=[];
 const visit=n=>{body.push(n);if(n.type!=='Table')(n.children||[]).forEach(visit);};
 visit(result.tree);
 const find=(type,pi)=>body.find(n=>n.type===type&&n.pi===pi);
 const title=find('TextLine',373), spacer=find('TextLine',374), table=find('Table',375), tail=find('TextLine',376);
 const bottom=n=>n.bbox.y+n.bbox.h;
 check('title and intentional blank paragraph preserved',title&&spacer&&bottom(title)<=spacer.bbox.y,{title:title?.bbox,spacer:spacer?.bbox});
 check('table follows intentional blank line',spacer&&table&&bottom(spacer)<=table.bbox.y,{spacer:spacer?.bbox,table:table?.bbox});
 check('following paragraph follows table',tail&&table&&bottom(table)<=tail.bbox.y,{tail:tail?.bbox,table:table?.bbox});
 const cell=nodes(result.portableTree).find(n=>n.type==='Cell'&&n.row===1&&n.col===2&&Math.abs(n.bbox.w-115.2266667)<.1);
 const cellLines=cell?nodes(cell).filter(n=>n.type==='TextLine'):[];
 check('opinion cell retains padding and four wrapped lines',cellLines.length===4&&cellLines.every(n=>Math.abs(n.bbox.x-cell.bbox.x-6.8)<.11),{cell:cell?.bbox,lines:cellLines.map(n=>n.bbox)});
 check('original 82 pages preserved',result.count===82,{pageCount:result.count});
 const native=JSON.parse(readFileSync(root+'/output/pr-review/pr7518-20261004/p39-frame-native/regulatory-p39-frame/render_tree/render_tree_039.json'));
 const a=nodes(native).filter(n=>['TextLine','Table'].includes(n.type));
 const b=nodes(result.portableTree).filter(n=>['TextLine','Table'].includes(n.type));
 check('p39 Native and portable WASM final geometry agrees',a.length===b.length&&a.every((n,i)=>n.type===b[i].type&&n.pi===b[i].pi&&['x','y','w','h'].every(k=>Math.abs(n.bbox[k]-b[i].bbox[k])<.2)),{nativeCount:a.length,wasmCount:b.length});
 for(const control of result.controls){
  const num=String(control.index+1).padStart(3,'0');
  const native=JSON.parse(readFileSync(root+'/output/pr-review/pr7518-20261004/p22-p38-frame-native/regulatory-p22-p38-frame/render_tree/render_tree_'+num+'.json'));
  const a=nodes(native).filter(n=>['TextLine','Table'].includes(n.type)),b=nodes(control.tree).filter(n=>['TextLine','Table'].includes(n.type));
  check('page '+(control.index+1)+' Native/portable WASM geometry agrees',a.length===b.length&&a.every((n,i)=>n.type===b[i].type&&n.pi===b[i].pi&&['x','y','w','h'].every(k=>Math.abs(n.bbox[k]-b[i].bbox[k])<.2)),{nativeCount:a.length,wasmCount:b.length});
 }
 await page.evaluate(()=>window.__canvasView.gotoPage(38));
 await new Promise(r=>setTimeout(r,500));
 await page.screenshot({path:out+'/studio-page-39.png'});
 check('no browser errors',errors.length===0,errors);
} finally {
 const result={sourceHead:execFileSync('git',['rev-parse','HEAD'],{cwd:root,encoding:'utf8'}).trim(),input,inputSha256:sha(bytes),wasmSha256:pkgHash,checks,errors};
 writeFileSync(out+'/result.json',JSON.stringify(result,null,2));
 console.log(JSON.stringify({passed:checks.filter(c=>c.passed).length,total:checks.length,failed:checks.filter(c=>!c.passed)}));
 await page.close();browser.disconnect();
}
if(checks.some(c=>!c.passed))process.exitCode=1;
