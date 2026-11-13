"""Fresh product renders and metrics; reuse only hash-verified independent PDF rasters."""
import sys,json,hashlib,shutil,subprocess,atexit
from pathlib import Path
root=Path.cwd();sys.path.insert(0,str(root))
from scripts import visual_sweep as vs
from scripts.visual_sweep_pdf import pdf_rasterizer_provenance
s=root/'output/issue8946/stage9';registry=json.loads((s/'reference-raster-cache.json').read_text());identity=pdf_rasterizer_provenance();original_run=vs.run
pdf_hashes={};receipts={}
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def cached_run(cmd,**kwargs):
 args=[str(x) for x in cmd]
 if len(args)>1 and Path(args[1]).resolve()==(root/'scripts/visual_sweep_pdf.py').resolve():
  def arg(k):return args[args.index(k)+1]
  pdf=Path(arg('--pdf')).resolve();digest=pdf_hashes.setdefault(str(pdf),None)
  if digest is None:digest=pdf_hashes[str(pdf)]=sha(pdf)
  entry=registry['pdfs'].get(digest)
  if entry and float(arg('--dpi'))==entry['dpi'] and identity==entry['rasterizer']:
   prefix=Path(arg('--prefix'));pages=[int(x) for x in arg('--pages').split(',')] if '--pages' in args else list(range(1,entry['pages']+1));prefix.parent.mkdir(parents=True,exist_ok=True)
   rec=receipts.setdefault(str(prefix.parent),{'pdf_sha256':digest,'dpi':entry['dpi'],'rasterizer':identity,'source_manifest':entry['manifest'],'source_manifest_sha256':entry['manifest_sha256'],'pages':{}})
   for page in pages:
    source=entry['images'][str(page)];p=Path(source['path']);assert sha(p)==source['sha256'],f'Cached reference PNG changed: {p}'
    q=prefix.parent/f'{prefix.name}-{page:03d}.png';shutil.copyfile(p,q);assert sha(q)==source['sha256'];rec['pages'][str(page)]=source['sha256']
   print(f'reference cache: verified PDF {digest[:12]}, {len(pages)} pages; fresh product render and metric calculation unchanged',flush=True)
   return subprocess.CompletedProcess(cmd,0,'verified independent reference raster cache\n','')
 return original_run(cmd,**kwargs)
def save_receipts():
 grouped={}
 for folder,record in receipts.items():
  base=Path(folder).parent
  if 'silhouette-parts' in base.parts:base=Path(*base.parts[:base.parts.index('silhouette-parts')])
  grouped.setdefault(base,[]).append(record)
 for base,records in grouped.items():
  base.mkdir(parents=True,exist_ok=True)
  (base/'reference-raster-reuse.json').write_text(json.dumps(records,ensure_ascii=False,indent=2)+'\n')
original_provenance=vs.sweep_provenance
def cached_provenance(*args,**kwargs):
 value=original_provenance(*args,**kwargs)
 value['independent_reference_cache']={'policy':registry['policy'],'wrapper_sha256':sha(Path(__file__)),'registry_sha256':sha(s/'reference-raster-cache.json'),'actual_reuse_receipts':'reference-raster-reuse.json beside the final silhouette manifest'}
 return value
vs.sweep_provenance=cached_provenance
vs.run=cached_run
atexit.register(save_receipts)
if __name__ == "__main__":
 vs.main()
