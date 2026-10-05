// 저장소 루트에서 fresh WASM으로 누락 단 정의와 편집 입력을 재현한다.
import fs from 'node:fs';
import http from 'node:http';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
const require=createRequire(process.cwd()+'/rhwp-studio/package.json');
const {findChrome}=await import(process.cwd()+'/scripts/rasterize-svg-webfonts.mjs');
const root='output/pr-review/semanticist21-20261005';
const files=new Map([['/rhwp.js',['text/javascript',fs.readFileSync('pkg/rhwp.js')]],['/rhwp_bg.wasm',['application/wasm',fs.readFileSync('pkg/rhwp_bg.wasm')]],['/input',['application/octet-stream',fs.readFileSync('tests/fixtures/issue7491/center_align_first_line_indent_missing_column.hwp')]]]);
const server=http.createServer((req,res)=>{if(req.url==='/'){res.end('<!doctype html>');return;}const f=files.get(req.url);if(!f){res.writeHead(404);res.end();return;}res.setHeader('Content-Type',f[0]);res.end(f[1]);});
await new Promise(r=>server.listen(0,'127.0.0.1',r));
let browser;
try {browser=await require('puppeteer-core').launch({executablePath:findChrome(),headless:true});const page=await browser.newPage();await page.goto('http://127.0.0.1:'+server.address().port);
const d=await page.evaluate(async()=>{const m=await import('/rhwp.js');await m.default({module_or_path:'/rhwp_bg.wasm'});const d=new m.HwpDocument(new Uint8Array(await(await fetch('/input')).arrayBuffer()));d.setColumnDef(0,1,0,true,0);const restored=Array.from(d.exportHwp());d.insertText(0,4,7,'가');d.insertText(0,7,0,'가');const result={restored,bytes:Array.from(d.exportHwp()),tree:JSON.parse(d.getPageRenderTree(0)),pageCount:d.pageCount()};d.free();return result;});fs.writeFileSync(root+'/issue6190-fresh-restored.hwp',Buffer.from(d.restored));delete d.restored;fs.writeFileSync(root+'/issue6190-fresh-wasm-edited.hwp',Buffer.from(d.bytes));delete d.bytes;fs.writeFileSync(root+'/fresh-edit-tree.json',JSON.stringify(d,null,2));console.log(JSON.stringify({pageCount:d.pageCount,sha256:createHash('sha256').update(fs.readFileSync(root+'/issue6190-fresh-wasm-edited.hwp')).digest('hex')}));
}finally{if(browser)await browser.close();await new Promise(r=>server.close(r));}
