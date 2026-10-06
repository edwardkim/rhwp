import {build,loadConfigFromFile} from '/Users/tsjang/rhwp/rhwp-studio/node_modules/vite/dist/node/index.js';
import fs from 'node:fs';
const root='/Users/tsjang/rhwp';const out='/private/tmp/rhwp-pr7142-review';const summary=[];
for(const kind of ['chrome','firefox']){
 const loaded=await loadConfigFromFile({command:'build',mode:'production'},`${root}/rhwp-${kind}/vite.config.ts`);const config=loaded.config;
 config.resolve.alias['@wasm/rhwp.js']=out+'/pkg/rhwp.js';config.resolve.alias['@wasm']=out+'/pkg';
 await build({...config,configFile:false,root:root+'/rhwp-studio',build:{...config.build,outDir:out+'/'+kind+'-dist',emptyOutDir:true}});
 const html=fs.readFileSync(out+'/'+kind+'-dist/index.html','utf8');const script=fs.readFileSync(`${root}/rhwp-${kind}/build.mjs`,'utf8');
 summary.push({kind,htmlRequestsLocaleInit:html.includes('locale-init.js'),localeInitBundled:fs.existsSync(out+'/'+kind+'-dist/locale-init.js'),copyScriptMentionsLocaleInit:script.includes('locale-init.js'),publicDir:config.publicDir});
 fs.writeFileSync(out+'/extension-builds.json',JSON.stringify(summary,null,2));
}
console.log(JSON.stringify(summary));
