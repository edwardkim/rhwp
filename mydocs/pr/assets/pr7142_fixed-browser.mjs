import puppeteer from '/Users/tsjang/rhwp/rhwp-studio/node_modules/puppeteer-core/lib/puppeteer/puppeteer-core.js';
import fs from 'node:fs';import assert from 'node:assert/strict';
const out='/private/tmp/rhwp-pr7142-review';const browser=await puppeteer.launch({executablePath:'/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',headless:true,args:['--no-first-run']});const results=[];
try{
for(const kind of ['chrome','firefox']){
 const p=await browser.newPage();await p.setRequestInterception(true);p.on('request',req=>{const url=new URL(req.url());if(url.pathname==='/__probe')return req.respond({contentType:'text/html',body:'<!doctype html><html lang="ko"><head><script src="/packaged-locale-init.js"></script></head><body></body></html>'});if(url.pathname==='/packaged-locale-init.js')return req.respond({contentType:'text/javascript',body:fs.readFileSync(`/private/tmp/rhwp-pr7142-fix-sandbox/rhwp-${kind}/dist/locale-init.js`,'utf8')});req.continue();});
 await p.goto('http://127.0.0.1:7797/__probe?lang=en',{waitUntil:'load'});
 const result=await p.evaluate(async()=>{const bootstrapLang=document.documentElement.lang;const core=await import('/src/i18n/core.ts');const dom=await import('/src/i18n/dom.ts');core.registerCatalog('ko',{'label':'열기'});core.registerCatalog('en',{'label':'Open'});core.setLocale('en');const button=document.createElement('button');button.dataset.i18n='label';const icon=document.createElement('span');icon.textContent='★';let clicks=0;icon.addEventListener('click',()=>clicks++);button.append(icon);dom.applyI18nToElement(button);dom.applyI18nToElement(button);icon.click();return{bootstrapLang,iconPreserved:button.contains(icon),clicks,text:button.textContent,children:button.children.length};});
 assert.equal(result.bootstrapLang,'en');assert.equal(result.iconPreserved,true);assert.equal(result.clicks,1);assert.equal(result.text,'★Open');assert.equal(result.children,1);results.push({kind,...result});await p.close();
}
fs.writeFileSync(out+'/fixed-browser.json',JSON.stringify({browser:await browser.version(),scope:'Packaged bootstrap script in real Chrome before any application bundle; real DOM helper identity/listener/idempotence. Firefox package bytes tested in Chrome, not Firefox runtime.',results},null,2));console.log(JSON.stringify(results));
}finally{await browser.close()}
