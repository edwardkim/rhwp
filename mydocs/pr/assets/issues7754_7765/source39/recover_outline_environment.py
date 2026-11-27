from pathlib import Path
import json,hashlib,itertools,time,math
O=Path('/home/tsjang/rhwp/output/pr-review/issue7754-nine');P=O/'source39-outline-known-program-replay.json';expected='4ade9f383e2554fddcfa517cf8de6d1dadf51c379836a615795afabe9eeaf41d'
def encode(programs, resources=None, hidden=None):
 programs=dict(programs)
 if hidden:programs['#신명조']=hidden
 digest=hashlib.sha256(b'preserve-source-hft-metrics\0')
 for source,p in sorted(programs.items()):
  for v in [source,p['family'],p['programId'],p['query']]:digest.update(v.encode()+b'\0')
  digest.update(bytes([p['trueType'],p['openTypeCff']]))
 env={'id':'declared-program-'+digest.hexdigest(),'preserveSourceMetrics':True,'substitutions':{s:p['family'] for s,p in programs.items()}}
 if programs:env['selectedPrograms']=programs
 tt=sorted({p['family'] for p in programs.values() if p['trueType']})
 if tt:env['trueTypeFaces']=tt
 if not programs:env={'id':'native-webfont-fallback','preserveSourceMetrics':True,'substitutions':{}}
 env['substitutions']['#신명조']='NotoSansKR-Regular'
 if 'selectedPrograms' in env:env['selectedPrograms'].pop('#신명조',None)
 # Native CLI merges the mapping and hashes compact serde JSON before Python prepares it.
 identity=json.dumps([json.loads(json.dumps(env,ensure_ascii=False,sort_keys=True)),resources or []],ensure_ascii=False,separators=(',',':')).encode()
 env['id']='native-webfont-'+hashlib.sha256(identity).hexdigest()[:24]
 # serde deserialization omits maps that became empty after CLI removed a mapped selection.
 if not env.get('selectedPrograms'):env.pop('selectedPrograms',None)
 env['documentSubstitutions']=[]
 env['id']='native-webfont-'+hashlib.sha256(json.dumps(env,ensure_ascii=False,sort_keys=True).encode()).hexdigest()[:24]
 return (json.dumps(env,ensure_ascii=False,sort_keys=True)+'\n').encode()
known=Path('/home/tsjang/rhwp/mydocs/pr/assets/issue8856/final-20261110/font-environments/cce7acdac8ae6caa24767d1c22ee4bcc3aac5dabbcf0bc8dcefb88d2727cd745.json');assert encode(json.loads(known.read_text())['selectedPrograms'])==known.read_bytes(),'constructor self-check'
v=json.loads((O/'source39-outline-original-supply-program-variants.json').read_text());names=sorted(n for n in v if n!='#신명조');options=[[None]+[x['program'] for x in v[n] if not x['program']['programId'].startswith('sweep-')] for n in names]
r={'started':time.time(),'method':'Original Rust constructor and Python webfont serializer over independently stored Native program variants; exact pinned SHA is sole acceptance. Independently stored 2026-11-10 canonical environment reproduced byte for byte before search.','expected_sha256':expected,'total_combinations':math.prod(map(len,options)),'tested':0,'match':None}
P.write_text(json.dumps(r,indent=2)+'\n')
resource=json.loads(Path('/home/tsjang/.cache/rhwp/webfonts/0a1ecc38c31a1d6cd9d4ca00aa95cb0939428ac1e7635efb7335c8ae98a87747/resource.json').read_text())
resource_options=[[],[resource],[dict(resource,nativeSha256='af6268b76ab738e0d8a7ffaa149480d949898443dc11560f9262c3d3c1893834',nativeBytes=2519996)]]
hidden_options=[None]+json.loads((O/'source39-outline-original-hidden-programs.json').read_text())
full_options=[[p for p in opts if p is not None] for opts in options]
r['method']+=' Full originally supplied programs first; original CFF Sinmyeongjo program selected under # source can remain in constructor digest after Native CLI removes the mapping-owned program record.'
r['total_combinations']=len(hidden_options)*len(resource_options)*math.prod(map(len,full_options));r['tested']=0
for i,(hidden,resources,comb) in enumerate(itertools.product(hidden_options,resource_options,itertools.product(*full_options)),1):
 programs={n:p for n,p in zip(names,comb)};b=encode(programs,resources,hidden)
 if hashlib.sha256(b).hexdigest()==expected:
  dest=Path('/home/tsjang/rhwp/output/pr-review/native-webfont-cache/environments')/(expected+'.json');dest.write_bytes(b);r['match']=str(dest);r['tested']=i;r['hidden_selection']=hidden;r['native_cli_resources']=resources;break
 if i%1000==0:r['tested']=i;P.write_text(json.dumps(r,indent=2)+'\n')
else:r['tested']=i
r['ended']=time.time();P.write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(r),flush=True)
