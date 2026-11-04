import fs from 'node:fs';
import {createHash} from 'node:crypto';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import http from 'node:http';
import {createRequire} from 'node:module';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../../..');
const label=process.argv[2];
const out=path.join(root,'output/pr-review/chrisryugj-20261031',label+'-edits');
const pkg=path.resolve(process.argv[3]);
const require=createRequire(root+'/rhwp-studio/package.json');
const {findChrome}=await import(root+'/scripts/rasterize-svg-webfonts.mjs');
const sources={tab:'samples/lseg-05-tab.hwp',group:'samples/basic/sungeo.hwp',child:'samples/issue6542/156678235_mid_para_vpos_rewind.hwp',multi:'samples/hwp-multi-001.hwp'};
const files=new Map([['/rhwp.js',fs.readFileSync(pkg+'/rhwp.js')],['/rhwp_bg.wasm',fs.readFileSync(pkg+'/rhwp_bg.wasm')],...Object.entries(sources).map(([k,v])=>['/'+k,fs.readFileSync(root+'/'+v)])]);
const server=http.createServer((req,res)=>{if(req.url==='/'){res.end('<!doctype html>');return;}if(!files.has(req.url)){res.writeHead(404);res.end();return;}res.setHeader('Content-Type',req.url.endsWith('.js')?'application/javascript':req.url.endsWith('.wasm')?'application/wasm':'application/octet-stream');res.end(files.get(req.url));});
await new Promise(r=>server.listen(0,'127.0.0.1',r));
let browser;
try {
 browser=await require('puppeteer-core').launch({executablePath:findChrome(process.env.VISUAL_SWEEP_CHROME),headless:true,protocolTimeout:180000});
 const page=await browser.newPage();await page.goto('http://127.0.0.1:'+server.address().port);
 const results=await page.evaluate(async()=>{
  const m=await import('/rhwp.js');await m.default({module_or_path:'/rhwp_bg.wasm'});
  const outputs=[];
  for(const [key,operation] of [['tab','original'],['tab','insert'],['tab','delete'],['tab','split'],['tab','split-merge'],['group','touch'],['child','touch'],['multi','touch']]){
   const d=new m.HwpDocument(new Uint8Array(await(await fetch('/'+key)).arrayBuffer()));d.setHangul2024Compat(false);
   const before=d.getTextFileText();const calls=[];
   if(operation==='insert')calls.push(d.insertText(0,2,0,'새\t'));
   if(operation==='delete'){const t=d.getTextRange(0,2,0,10000);const at=Array.from(t).indexOf('\t');if(at<0)throw Error('missing tab');calls.push(d.deleteText(0,2,at,1));}
   if(operation==='split'||operation==='split-merge'){const t=d.getTextRange(0,2,0,10000);const at=Array.from(t).indexOf('\t')+1;calls.push(d.splitParagraph(0,2,at));if(operation==='split-merge')calls.push(d.mergeParagraph(0,3));}
   if(operation==='touch'){calls.push(d.insertText(0,0,0,'X'));calls.push(d.deleteText(0,0,0,1));}
   const data=Array.from(d.exportHwp());const reopen=new m.HwpDocument(new Uint8Array(data));
   const pages=[],trees=[];for(let p=0;p<d.pageCount();p++){trees.push(d.getPageRenderTree(p));if(key==='tab')pages.push(d.renderPageSvgWithProfile(p,'print'));}
   outputs.push({key,operation,before,text:d.getTextFileText(),reopenedText:reopen.getTextFileText(),pageCount:d.pageCount(),reopenedPages:reopen.pageCount(),calls,data,pages,trees});
   reopen.free();d.free();
  }
  return outputs;
 });
 fs.mkdirSync(out,{recursive:true});
 for(const r of results){const stem=r.key+'-'+r.operation;fs.writeFileSync(out+'/'+stem+'.hwp',Buffer.from(r.data));for(let p=0;p<r.pages.length;p++)fs.writeFileSync(out+'/'+stem+'-'+(p+1)+'.svg',r.pages[p]);r.treeHashes=r.trees.map(t=>createHash('sha256').update(t).digest('hex'));delete r.trees;delete r.data;delete r.pages;}
 fs.writeFileSync(out+'/results.json',JSON.stringify(results,null,2));console.log(JSON.stringify(results.map(r=>({key:r.key,operation:r.operation,pages:r.pageCount,reopenedPages:r.reopenedPages,textRoundtrip:r.text===r.reopenedText}))));
}finally{if(browser)await browser.close();await new Promise(r=>server.close(r));}
