from pathlib import Path
import subprocess,time,json,statistics
R=Path.cwd();O=R/'output/pr-review/issue8174/v2-s07-20261025';D=O/'performance';D.mkdir(exist_ok=True)
exe={'before':R/'output/pr-review/issue8174/v2-s05-20261025/candidate-native/rhwp','candidate':O/'candidate-native/rhwp'}
fixtures=json.loads((O/'font-visual-inputs.json').read_text())['fixtures'];rows=[]
for round in range(0 if (D/"native-import.json").exists() else 14):
 for v in (['before','candidate']if round%2==0 else ['candidate','before']):
  for i,x in enumerate(fixtures):
   dest=D/(v+'-'+str(i)+'.hwpx');start=time.perf_counter_ns();p=subprocess.run([str(exe[v]),'import-pdf',x['input'],str(dest),'--json'],capture_output=True);elapsed=time.perf_counter_ns()-start;assert p.returncode==0,p.stderr.decode();result=json.loads(p.stdout);assert result['pageCount']==x['pages'];rows.append({'round':round,'variant':v,'name':'type3-twelve-pages' if i else 'cff-cid','wallNs':elapsed,'execution':result['execution'],'bytes':result['bytes']})
if rows: (D/'native-import.json').write_text(json.dumps({'warmupRounds':2,'samples':rows},indent=2)+'\n')
else: rows=json.loads((D/'native-import.json').read_text())["samples"]
bench=[]
for round in range(4):
 for v in ['before','candidate']if round%2==0 else ['candidate','before']:
  p=subprocess.run([str(O/(v+'-probe-bin')),'--bench'],capture_output=True);assert p.returncode==0,p.stderr.decode();samples=json.loads(p.stdout);assert all(x['sum']==500000 for x in samples);bench.append({'round':round,'variant':v,'samples':samples})
(D/'native-memo.json').write_text(json.dumps(bench,indent=2)+'\n')
for name in ['cff-cid','type3-twelve-pages']:
 med={v:statistics.median(x['wallNs']/1e6 for x in rows if x['variant']==v and x['name']==name and x['round']>=2) for v in exe};print(name,med,'ratio',med['candidate']/med['before'])
