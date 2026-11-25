from pathlib import Path
import subprocess,json,time,csv,hashlib,traceback
R=Path('/home/tsjang/rhwp');O=R/'output/pr-review/issue7754-nine';D=O/'source36-nested-live';S=D/'state.json';s={'started':time.time(),'base_source':'1a27faacff48ea7329e76ce8eca15aa9ca8aa877','candidate_source':'c48c2b895d31ba0fd4969c837bfc7a6d8a2c65ac','results':[]}
sha=lambda p:hashlib.file_digest(Path(p).open('rb'),'sha256').hexdigest()
def save():S.write_text(json.dumps(s,indent=2)+'\n')
try:
 if S.exists():
  prior=json.loads(S.read_text());assert prior['base_source']==s['base_source'] and prior['candidate_source']==s['candidate_source'];s=prior
  for field in ['error','traceback','ended','complete','comparisons']:s.pop(field,None)
  s['resumed_after_rust_input_preparation_repair']=time.time()
 save()
 for role in ['base','candidate']:
  if role=='candidate':
   s['running']='wait final candidate Native regular library from existing CI case';save();deadline=time.monotonic()+43200
   while True:
    gates=json.loads((O/'source36-rust-gates-state.json').read_text())
    if any(r['label'].startswith('native-') and r['label']!='native-unfiltered-lib' and r['exit_code']==0 for r in gates['results']):break
    if gates.get('ended'):raise RuntimeError(gates.get('error','candidate Native producer incomplete'))
    assert time.monotonic()<deadline;time.sleep(10)
   W=Path('/home/tsjang/rhwp-worktrees/task-7754-rebase-5665');head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=W,text=True).strip();assert head==s['candidate_source']
   deps=R/'target/pr-review/release-test/deps';lib=deps/'librhwp.rlib';frozen=D/'librhwp-candidate.rlib';import shutil;shutil.copy2(lib,frozen)
   cmd=['rustc','--edition=2021',str(O/'source33_nested_live.rs'),'-o',str(D/'candidate-runner'),'--extern',f'rhwp={frozen}','-L',f'dependency={deps}','--extern',f"serde_json={deps/'libserde_json-589e12259f7a4912.rlib'}"]
   for name in ['skia-bindings-079888cb8874cccd/out/skia','ring-62411559129b98e5/out']:cmd+=['-L',f'native={R}/target/pr-review/release-test/build/{name}']
   attempts=[]
   for serde in sorted(deps.glob('libserde_json-*.rlib')):
    idx=next(i for i,x in enumerate(cmd) if x.startswith('serde_json='));cmd[idx]=f'serde_json={serde}'
    logpath=D/('candidate-build-'+serde.stem+'.log')
    with logpath.open('w') as log:built=subprocess.run(cmd,cwd=R,stdout=log,stderr=subprocess.STDOUT,timeout=600)
    attempts.append({'serde_json':str(serde),'exit_code':built.returncode,'log':str(logpath)})
    if built.returncode==0:break
   (D/'candidate-link-attempts.json').write_text(json.dumps(attempts,indent=2)+'\n')
   rec={'source_head':head,'library_sha256':sha(frozen),'command':cmd,'exit_code':built.returncode};(D/'candidate-build.json').write_text(json.dumps(rec,indent=2)+'\n');assert built.returncode==0,rec
  for key,src in [('origin','samples/inner-table-01.hwp'),('hancom','tests/fixtures/nested-row-split/inner-table-01-2020.hwpx')]:
   for lines in [30,70]:
    existing=next((r for r in s['results'] if r['role']==role and r['key']==key and r['lines']==lines),None)
    if existing:
     assert existing['input_sha256']==sha(R/src) and existing['exit_code']==0 and len(existing['states'])==3
     for entry in existing['states']:
      folder=Path(entry['document_json']).parent;doc=json.loads(Path(entry['document_json']).read_text());assert [x['pageIndex'] for x in doc['carets']]==[1 if n<=21 else 2 if n<=58 else 3 for n in range(1,lines+1)]
      assert {p.name:sha(p) for p in (folder/'rhwp_png').glob('*.png')}==entry['images']
     print('reused verified existing',role,key,lines,flush=True);continue
    out=D/role/f'{key}-{lines}';out.mkdir(parents=True,exist_ok=True);c=[str(D/(role+'-runner')),src,str(lines),str(out)];s['running']={'role':role,'key':key,'lines':lines};save();t=time.monotonic()
    with (out/'export.log').open('w') as f:r=subprocess.run(c,cwd=R,stdout=f,stderr=subprocess.STDOUT,timeout=1800)
    record={'role':role,'key':key,'source':src,'input_sha256':sha(R/src),'lines':lines,'command':c,'exit_code':r.returncode,'seconds':time.monotonic()-t,'states':[]}
    assert r.returncode==0,(role,key,lines)
    for state in ['live','reopen-hwp','reopen-hwpx']:
     folder=out/state;doc=json.loads((folder/'document.json').read_text());images=list((folder/'rhwp_png').glob('*.png'));assert doc['page_count']==len(images)
     owners=[x['pageIndex'] for x in doc['carets']];expected=[1 if n<=21 else 2 if n<=58 else 3 for n in range(1,lines+1)];assert owners==expected,(role,key,lines,state,owners)
     record['states'].append({'state':state,'page_count':doc['page_count'],'carets_match_independent_print_page_ownership':True,'images':{p.name:sha(p) for p in images},'document_json':str(folder/'document.json')})
    s['results'].append(record);save();print(role,key,lines,'exported',flush=True)
 comparisons=[]
 for a in [x for x in s['results'] if x['role']=='base']:
  b=next(x for x in s['results'] if x['role']=='candidate' and x['key']==a['key'] and x['lines']==a['lines'])
  for x,y in zip(a['states'],b['states']):comparisons.append({'key':a['key'],'lines':a['lines'],'state':x['state'],'same_page_count':x['page_count']==y['page_count'],'byte_identical_pngs':x['images']==y['images']})
 s['comparisons']=comparisons;s['complete']=len(s['results'])==8;s['running']=None
except BaseException as e:s['error']=str(e);s['traceback']=traceback.format_exc();print(s['traceback'],flush=True)
finally:s['ended']=time.time();save()
