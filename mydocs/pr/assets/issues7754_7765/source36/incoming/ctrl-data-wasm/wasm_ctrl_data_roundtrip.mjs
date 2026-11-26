import fs from 'node:fs';import path from 'node:path';import crypto from 'node:crypto';import assert from 'node:assert/strict';
import {launchBrowser,createPage,loadApp,createNewDocument,closeBrowser} from '/home/tsjang/rhwp/rhwp-studio/e2e/helpers.mjs';
const R='/home/tsjang/rhwp',O=R+'/output/pr-review/issue7754-nine',role=process.env.RHWP_CTRL_ROLE??'candidate';assert.ok(['base','candidate'].includes(role));
const dir=O+'/source36-ctrl-data-wasm/'+role;fs.mkdirSync(dir,{recursive:true});const inputs=['samples/basic/sungeo.hwp','samples/20250130-hongbo.hwp','samples/task2137/156618554_petfood_press.hwp','samples/156636617_240617 2024년 5월 월간 수출입 현황(확정치).hwp'];
const sha=b=>crypto.createHash('sha256').update(b).digest('hex'),s={role,scope:'actual frozen WASM constructor, insertText, HWP export and reopen twice; physical CTRL_DATA owner/bytes checked independently in Python',started:new Date().toISOString(),results:[]};const save=()=>fs.writeFileSync(dir+'/state.json',JSON.stringify(s,null,2)+'\n');let browser;
try{browser=await launchBrowser();const page=await createPage(browser);await loadApp(page,'?renderer=canvas2d');await createNewDocument(page);
 const manifest=JSON.parse(fs.readFileSync(process.env.RHWP_WASM_BUILD_MANIFEST,'utf8'));s.source_head=manifest.source_sha;s.wasm_sha256=manifest.artifacts['rhwp_bg.wasm'].sha256;
 for(let i=0;i<inputs.length;i++){
  const source=inputs[i],bytes=fs.readFileSync(R+'/'+source);s.running=source;save();
  const result=await page.evaluate(data=>{
   const Doc=window.__wasm.borrowDocumentHandle().constructor,decode=base64=>Uint8Array.from(atob(base64),c=>c.charCodeAt(0));
   const encode=bytes=>{let text='';for(let i=0;i<bytes.length;i+=32768)text+=String.fromCharCode(...bytes.subarray(i,i+32768));return btoa(text);};
   let doc=new Doc(decode(data));let expected=doc.getTextRange(0,0,0,doc.getParagraphLength(0,0));const originalText=expected,rounds=[];
   try{for(let round=1;round<=2;round++){
    doc.insertText(0,0,0,'가');expected='가'+expected;const saved=doc.exportHwp();const reopened=new Doc(saved);const text=reopened.getTextRange(0,0,0,reopened.getParagraphLength(0,0));
    if(text!==expected)throw Error('public source text lost across reopen '+round);rounds.push({round,base64:encode(saved),text,bytes:saved.length});doc.free();doc=reopened;
   }}finally{doc.free();}return {originalText,rounds};
  },bytes.toString('base64'));
  const rec={input:source,input_sha256:sha(bytes),original_text:result.originalText,rounds:[]};
  for(const round of result.rounds){const data=Buffer.from(round.base64,'base64'),file=path.join(dir,`input${i+1}-round${round.round}.hwp`);fs.writeFileSync(file,data);rec.rounds.push({round:round.round,file,bytes:data.length,sha256:sha(data),text:round.text});}
  s.results.push(rec);save();console.log(role,source,'two actual HWP edit/save/reopen rounds');
 }
 s.passed=true;s.running=null;
}catch(e){s.error=String(e.stack??e);throw e;}finally{if(browser)await closeBrowser(browser);s.ended=new Date().toISOString();save();}
