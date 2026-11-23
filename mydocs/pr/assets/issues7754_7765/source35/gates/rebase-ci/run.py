from pathlib import Path
import subprocess,json,time
R=Path('/home/tsjang/rhwp');D=R/'output/pr-review/issue7754-nine/source35-rebase-ci';P=D/'state.json';s={'started':time.time(),'results':[]}
for role,cwd in [('base',D/'base'),('candidate',R)]:
 for label,cmd in [('python-render-report',[str(R/'venv/bin/python'),'-m','unittest','discover','-s','scripts/tests','-p','test_render_diff_workflow.py']),('node-ci-summary',['/usr/local/bin/node','--test','scripts/tests/pr-ci-summary.test.cjs'])]:
  log=D/f'{role}-{label}.log';t=time.monotonic()
  with log.open('w') as f:r=subprocess.run(cmd,cwd=cwd,stdout=f,stderr=subprocess.STDOUT,timeout=600)
  s['results'].append({'role':role,'cwd':str(cwd),'label':label,'command':cmd,'exit_code':r.returncode,'seconds':time.monotonic()-t,'log':str(log)});P.write_text(json.dumps(s,indent=2)+'\n')
s.update(ended=time.time(),complete=True,passed=all(x['exit_code']==0 for x in s['results']));P.write_text(json.dumps(s,indent=2)+'\n')
