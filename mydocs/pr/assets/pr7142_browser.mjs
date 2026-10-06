import puppeteer from '/Users/tsjang/rhwp/rhwp-studio/node_modules/puppeteer-core/lib/puppeteer/puppeteer-core.js';
import fs from 'node:fs';import crypto from 'node:crypto';
const out='/private/tmp/rhwp-pr7142-review';const browser=await puppeteer.launch({executablePath:'/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',headless:true,args:['--no-first-run','--disable-gpu','--lang=en-US']});
const results=[];
try {
for(const lang of ['ko','en']){
 const context=await browser.createBrowserContext();const p=await context.newPage();const errors=[];p.on('pageerror',e=>errors.push(String(e)));p.on('dialog',d=>d.accept());await p.setViewport({width:1280,height:900,deviceScaleFactor:1});
 await p.goto('http://127.0.0.1:7797/'+(lang==='en'?'?lang=en':''),{waitUntil:'domcontentloaded'});await p.waitForFunction(()=>window.__wasm&&window.__inputHandler&&window.__wasm.pageCount>0&&!document.querySelector('#sb-message')?.textContent?.includes('파일 로딩'),{timeout:120000});
 const initial=await p.evaluate(async()=>{const m=await import('/src/i18n/index.ts');return {locale:m.getLocale(),htmlLang:document.documentElement.lang,page:document.querySelector('#sb-page')?.textContent,section:document.querySelector('#sb-section')?.textContent,message:document.querySelector('#sb-message')?.textContent,navigator:navigator.language};});
 const state=await p.evaluate(async()=>{window.__eventBus.emit('insert-mode-changed',false);window.__eventBus.emit('headerFooterModeChanged',{mode:'footer',sectionIdx:0,applyTo:2,previewPage:0});const m=await import('/src/i18n/index.ts');const before={mode:document.querySelector('#sb-mode').textContent,hf:document.querySelector('.tb-hf-label')?.textContent};const preference=m.setPreferredLocale(m.getLocale()==='ko'?'en':'ko');const after={locale:m.getLocale(),mode:document.querySelector('#sb-mode').textContent};return {before,preference,after,stored:localStorage.getItem('rhwp-locale')};});
 await p.screenshot({path:out+'/'+lang+'-studio.png'});
 const dynamic=await p.evaluate(async()=>{const {applyI18nToElement}=await import('/src/i18n/dom.ts');const el=document.createElement('button');el.setAttribute('data-i18n','ui.sbMode.label');const icon=document.createElement('span');icon.className='icon';el.append(icon);applyI18nToElement(el);return {html:el.outerHTML,iconPreserved:el.contains(icon)};});
 const next=await context.newPage();await next.goto('http://127.0.0.1:7797/',{waitUntil:'domcontentloaded'});await next.waitForFunction(()=>window.__inputHandler,{timeout:120000});const nextLocale=await next.evaluate(async()=> (await import('/src/i18n/index.ts')).getLocale());
 results.push({lang,initial,state,dynamic,nextLocale,errors});await context.close();
}
fs.writeFileSync(out+'/browser.json',JSON.stringify({browser:await browser.version(),results},null,2));console.log(JSON.stringify(results));
}finally{await browser.close()}
