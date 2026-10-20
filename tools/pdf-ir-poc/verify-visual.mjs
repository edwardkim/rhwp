import fs from "node:fs/promises";
import path from "node:path";
import {spawnSync} from "node:child_process";
import {createHash} from "node:crypto";

const root=path.dirname(new URL(import.meta.url).pathname);
const repo=process.argv[2];
if(!repo) throw new Error("usage: node verify-visual.mjs RHWP_REPO [OUTPUT]");
const output=path.resolve(process.argv[3] || path.join(repo,"output/poc/pdf-ir"));
const commands=[];
const cases=[];
const hash=async file=>createHash("sha256").update(await fs.readFile(file)).digest("hex");
try {
  for(const [name,input] of [["text","pdf/re-align-center-hancom-hwp-2020.pdf"],["table","pdf/issue5875/nested_table_text_caption-2020.pdf"],["table-painted","pdf/issue5875/nested_table_text_caption-2020.pdf"],["table-structured","pdf/issue5875/nested_table_text_caption-2020.pdf"],["nested-structured",path.join(output,"nested-input.pdf")],["real7-structured",path.join(output,"source/page-7.pdf")],["real8-structured",path.join(output,"source/page-8.pdf")]]) {
    const base=path.join(output,"visual",name);
    const referenceDir=path.join(base,"reference");
    await fs.mkdir(referenceDir,{recursive:true});
    const rasterArgs=["-f","1","-l","1","-r","96","-png","-singlefile",path.resolve(repo,input),path.join(referenceDir,"page_001")];
    const raster=spawnSync("pdftoppm",rasterArgs,{encoding:"utf8",timeout:120000});
    commands.push({name:`${name}-reference`,command:["pdftoppm",...rasterArgs],exitCode:raster.status,stdout:raster.stdout,stderr:raster.stderr});
    if(raster.status!==0) throw new Error(`independent PDF raster failed: ${raster.stderr}`);
    const metrics={name,scope:"source first page only; not whole-document or Hancom editor verification"};
    for(const backend of ["native","wasm"]) {
      const imageDir=path.join(base,backend);
      const image=path.join(imageDir,"page_001.png");
      await fs.mkdir(imageDir,{recursive:true});
      await fs.copyFile(path.join(output,"browser/evidence",`${name}-${backend}.png`),image);
      const scoreOut=path.join(base,`${backend}-score`);
      const args=[path.join(repo,"scripts/visual_sweep.py"),"--silhouette-only","--png-pair",imageDir,referenceDir,"--out",scoreOut];
      const r=spawnSync("python3",args,{encoding:"utf8",timeout:120000});
      commands.push({name:`${name}-${backend}`,command:["python3",...args],exitCode:r.status,stdout:r.stdout,stderr:r.stderr,error:r.error?String(r.error):undefined});
      if(r.status!==0) throw new Error(`${name}-${backend} sweep failed: ${r.stderr}`);
      const tsv=await fs.readFile(path.join(scoreOut,"silhouette.tsv"),"utf8");
      metrics[backend]={pngSha256:await hash(image),tolerantMatchPercent:Number(tsv.trim().split("\n")[1].split("\t")[1])};
    }
    cases.push(metrics);
  }
  await fs.writeFile(path.join(output,"verification/visual-summary.json"),JSON.stringify({cases,hancomCompatibility:"maintainer-retest-required"},null,2));
  console.log(JSON.stringify(cases,null,2));
} finally {
  await fs.writeFile(path.join(output,"verification/visual-commands.json"),JSON.stringify(commands,null,2));
}
