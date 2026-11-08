import fs from 'node:fs';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
const reviewRoot=process.env.RHWP_REVIEW_ROOT??process.cwd();
const {launchBrowser,createPage,loadApp,loadHwpFile,closePage,closeBrowser}=await import(pathToFileURL(path.join(reviewRoot,'rhwp-studio/e2e/helpers.mjs')).href);
const stage=process.env.PR8296_STAGE??'head';
const out=process.env.RHWP_REVIEW_OUTPUT??path.join(reviewRoot,'output/pr8296-canvaskit-probe');
fs.mkdirSync(out,{recursive:true});
const browser=await launchBrowser();const page=await createPage(browser);
const fontResponses=[],fontReads=[];
page.on('response',response=>{if(/\/fonts\/.*\.(woff2?|ttf|otf)(?:\?|$)/i.test(response.url()))fontReads.push((async()=>{try{const bytes=await response.buffer();fontResponses.push({url:response.url(),status:response.status(),bytes:bytes.length,sha256:createHash('sha256').update(bytes).digest('hex')});}catch(error){fontResponses.push({url:response.url(),error:String(error)});}})());});
try{
 await page.setViewport({width:1150,height:1000,deviceScaleFactor:1});
 await loadApp(page,'?renderer=canvas2d');
 await loadHwpFile(page,'issue8201/seal-delegation-form.hwp');
 const result=await page.evaluate(async stage=>{
  const {CanvasKitLayerRenderer}=await import('/src/view/canvaskit-renderer.ts');
  const {parseCssFontFamilyList}=await import('/src/core/font-substitution.ts');
  const tree=window.__wasm.withPortableMetrics(()=>window.__wasm.getPageLayerTreeObject(0,'print'));
  const ops=[];const visit=n=>{if(n.kind==='group')n.children.forEach(visit);else if(n.child)visit(n.child);else for(const op of n.ops??[])if(op.type==='textRun'&&op.text.includes('\u{F012B}'))ops.push(op);};visit(tree.root);
  if(ops.length!==1)throw Error('expected one original seal textRun');
  const source=ops[0],renderer=await CanvasKitLayerRenderer.create('default','software',{requirePreparedFontFamilies:true});
  const aliases=[...new Set(['Probe Body',...parseCssFontFamilyList(source.style.fontFamily)])];
  const supply=[{url:'fonts/NotoSansKR-Regular.woff2',aliases},{url:'fonts/NotoSansSymbols2-Regular.ttf',aliases:['Probe Missing Hangul']},{url:'fonts/RHWPPublicBatang-Regular.woff2',aliases:['RHWP Public Batang']}];
  await renderer.prepareBundledFonts(supply);
  const panel=document.createElement('div');panel.id='pr8296-canvaskit-probe';Object.assign(panel.style,{position:'fixed',left:'0',top:'0',zIndex:'2147483647',background:'white',width:'1100px',padding:'12px',font:'18px sans-serif',color:'black'});document.body.append(panel);
  const results=[]; const innerDraws=[]; const drawPrototype=renderer.canvasKit.Canvas.prototype; const realDrawText=drawPrototype.drawText; drawPrototype.drawText=function(text,x,y,paint,font){ if(text==='인')innerDraws.push({text,glyphIds:[...font.getGlyphIDs(text,1)]}); return realDrawText.call(this,text,x,y,paint,font); };
  const add=(label,canvas)=>{const row=document.createElement('div');Object.assign(row.style,{display:'flex',alignItems:'center',gap:'18px',marginBottom:'8px'});const title=document.createElement('div');title.textContent=label;title.style.width='420px';row.append(title,canvas);panel.append(row);};
  try{
   for(const [sourceLabel,sourceOp] of [['Original seal textRun; explicit Noto alias',source],['Original op; body has no Hangul, Public ready',{...source,style:{...source.style,fontFamily:'Probe Missing Hangul'}}]]){
   const original={...tree,root:{kind:'leaf',bounds:{x:0,y:0,width:tree.pageWidth,height:tree.pageHeight},ops:[sourceOp]}};
   const originalCanvas=document.createElement('canvas');originalCanvas.width=Math.ceil(tree.pageWidth*3);originalCanvas.height=Math.ceil(tree.pageHeight*3);
   const rendered=renderer.renderPage(original,originalCanvas,3);
   const data=rendered.getContext('2d',{willReadFrequently:true}).getImageData(0,0,rendered.width,rendered.height).data;
   let left=rendered.width,top=rendered.height,right=-1,bottom=-1;
   for(let y=0;y<rendered.height;y++)for(let x=0;x<rendered.width;x++){const i=(y*rendered.width+x)*4;if(data[i+3]>20&&Math.min(data[i],data[i+1],data[i+2])<220){left=Math.min(left,x);top=Math.min(top,y);right=Math.max(right,x);bottom=Math.max(bottom,y);}}
   if(right<left)throw Error('original textRun produced no visible ink');
   left=Math.max(0,left-10);top=Math.max(0,top-10);right=Math.min(rendered.width,right+11);bottom=Math.min(rendered.height,bottom+11);
   const crop=document.createElement('canvas');crop.width=right-left;crop.height=bottom-top;
   crop.getContext('2d').drawImage(rendered,left,top,crop.width,crop.height,0,0,crop.width,crop.height);
   add(sourceLabel,crop);
   results.push({case:sourceLabel,text:sourceOp.text,displayText:sourceOp.displayText,bbox:sourceOp.bbox,style:sourceOp.style,positions:sourceOp.positions,displayPositions:sourceOp.displayPositions,placement:sourceOp.placement,renderedSize:[rendered.width,rendered.height],diagnosticInkCrop:[left,top,right,bottom],diagnostics:renderer.diagnostics(),png:crop.toDataURL('image/png')});
   }
   for(const [family,ratio]of [['Probe Body',.5],['Probe Body',1],['Probe Body',1.5],['Probe Missing Hangul',1],['RHWP Public Batang',1]]){
    const size=64,symbol=stage==='base'?'\u{F02B2}':'\u{F012B}',op={type:'textRun',text:symbol,bbox:{x:20,y:12,width:size*ratio,height:80},baseline:64,positions:[0,size*ratio],style:{fontFamily:family,fontSize:size,ratio,color:'#000000'}};
    const input={pageWidth:240,pageHeight:100,root:{kind:'leaf',bounds:{x:0,y:0,width:240,height:100},ops:[op]}};
    const targetCanvas=document.createElement('canvas');targetCanvas.width=input.pageWidth;targetCanvas.height=input.pageHeight;
    const canvas=renderer.renderPage(input,targetCanvas,1);
    const diag=renderer.diagnostics();const face=renderer.findPreparedTypeface(family)?.typeface;
    const innerText=stage==='base'?'2':'인';const font=new renderer.canvasKit.Font(face,size);const glyphIds=[...font.getGlyphIDs(innerText)];font.delete();
    const ctx=canvas.getContext('2d');ctx.strokeStyle='#e02020';ctx.strokeRect(20,12,size*ratio,80);
    add(family+' | '+(stage==='base'?'boxed 2':'seal')+' | ratio='+ratio,canvas);
    results.push({case:'controlled-legacy-textRun',symbol,innerText,family,ratio,advance:size*ratio,bodyInnerGlyphIds:glyphIds,diagnostics:diag,png:canvas.toDataURL('image/png')});
   }
   return{browser:navigator.userAgent,devicePixelRatio,stage,sourceDocument:'samples/issue8201/seal-delegation-form.hwp',sourceSha:stage==='head'?'f3e2128edca7b188080b2a94f25e0988e26c9699':'da39c536613a4bd543dc72b63a556183efc0383f',surface:'software',innerDraws,scope:'Actual browser CanvasKitLayerRenderer and real supplied font bytes; original produced seal textRun and a body-font coverage counterexample, plus controlled legacy PaintOps (head: seal, base: preexisting boxed digit).',supply,results};
  }finally{renderer.dispose();}
 },stage);
 await Promise.allSettled(fontReads);result.fontResponses=fontResponses; result.sourceSha=execFileSync('git',['rev-parse','HEAD'],{cwd:reviewRoot,encoding:'utf8'}).trim(); result.rendererSha256=createHash('sha256').update(fs.readFileSync(path.join(reviewRoot,'rhwp-studio/src/view/canvaskit-renderer.ts'))).digest('hex');
 for(const [index,row]of result.results.entries()){fs.writeFileSync(path.join(out,`case-${index}.png`),Buffer.from(row.png.split(',')[1],'base64'));delete row.png;}
 await (await page.$('#pr8296-canvaskit-probe')).screenshot({path:path.join(out,'browser-canvaskit-probe.png')});
 fs.writeFileSync(path.join(out,'result.json'),JSON.stringify(result,null,2));
 console.log(JSON.stringify({scope:result.scope,surface:result.surface,cases:result.results.map(r=>({case:r.case,family:r.family,ratio:r.ratio,bodyInnerGlyphIds:r.bodyInnerGlyphIds,unsupported:r.diagnostics?.lastUnsupportedOps,actualSurface:r.diagnostics?.surfaceBackend,completed:r.diagnostics?.lastRenderCompleted}))},null,2));
}finally{await closePage(page);await closeBrowser(browser);}
