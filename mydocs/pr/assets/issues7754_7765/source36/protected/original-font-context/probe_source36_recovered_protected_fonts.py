from pathlib import Path
import json,subprocess,hashlib,time,os,traceback
R=Path('/home/tsjang/rhwp');O=R/'output/pr-review/issue7754-nine';S=O/'source36-recovered-protected-font-probe-state.json';sha=lambda p:hashlib.file_digest(Path(p).open('rb'),'sha256').hexdigest();s={'started':time.time(),'pid':os.getpid(),'results':[]}
def save():S.write_text(json.dumps(s,indent=2)+'\n')
try:
 end=time.monotonic()+600
 while not (O/'source36-canonical-font-archive-recovery.json').exists():assert time.monotonic()<end;time.sleep(2)
 recovered=json.load(open(O/'source36-canonical-font-archive-recovery.json'));idx=json.load(open(O/'protected-local-font-sha-index.json'));available={h:ps[0] for h,ps in idx.items()}
 for r in recovered['files']:available[r['sha256']]=r['path']
 programs={r['program_id_face0']:r['path'] for r in recovered['files']}
 manifest=json.load(open(R/'mydocs/pr/assets/visual-protected-baseline/manifest.json'));resolved=json.load(open(O/'source36-canonical-environment-resolution.json'))['datasets']
 for key,page in [('sample16-hwp2022',51),('nested',16),('issue8040-insert',1)]:
  d=next(x for x in manifest['datasets'] if x['logical_key']==key and x['backend']=='native');condition=d['source'];raw=(R/condition['condition_evidence']).read_bytes();assert hashlib.sha256(raw).hexdigest()==condition['condition_evidence_sha256'];report=json.loads(raw);rows=report.get('conditions') or report.get('datasets');row=next(x for x in rows if x.get('backend')=='native' and (x.get('document_id')==d['document_id'] or (x.get('key') or x.get('document'))==key));sup=row.get('provenance',{}).get('font_supply',{}).get('files',[])
  paths=[];missing=[]
  for f in sup:
   if f['sha256'] in available:paths.append(available[f['sha256']])
   else:missing.append(f)
  rr=next(x for x in resolved if x['key']==key and x['backend']=='native');env=R/rr['matching_files'][0] if rr['matching_files'] else None
  if env:
   e=json.load(open(env));entries=list(e.get('selectedPrograms',{}).values())+e.get('documentSubstitutions',[])
   for x in entries:
    if x.get('programId') in programs:paths.append(programs[x['programId']])
  paths=list(dict.fromkeys(paths));folder=O/'source36-recovered-protected-font-probe';cmd=[str(R/'venv/bin/python'),'scripts/visual_sweep.py','--silhouette-only','--hwp',d['input']['path'],'--pdf',d['reference_pdf']['path'],'--key',key+f'-p{page:03}-recovered-originals','--pages',str(page),'--rhwp-bin',str(O/'bin/rhwp-source36-optimized'),'--compat',d['identity']['compat'],'--dpi','96','--page-workers','1','--out',str(folder)]
  if env:cmd+=['--font-environment',str(env)]
  for p in paths:cmd+=['--font-path',p]
  s['running']={'key':key,'page':page,'exact_available_files':len(paths),'missing_declared_files':len(missing)};save();log=O/'logs'/f'source36-recovered-{key}-p{page}.log';t=time.time()
  with log.open('w') as f:r=subprocess.run(cmd,cwd=R,stdout=f,stderr=subprocess.STDOUT,timeout=1200)
  s['results'].append({'key':key,'page':page,'command':cmd,'exit_code':r.returncode,'seconds':time.time()-t,'missing_declared_files':missing,'explicit_files':[{'path':p,'sha256':sha(p)} for p in paths],'scope':'Focused cause probe using exact declared archive font bytes; not complete canonical condition reproduction'});save()
 s['complete']=True;s['running']=None
except BaseException as e:s['error']=str(e);s['traceback']=traceback.format_exc()
finally:s['ended']=time.time();save()
