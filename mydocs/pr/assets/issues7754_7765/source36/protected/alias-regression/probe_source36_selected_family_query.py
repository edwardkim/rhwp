from pathlib import Path
import json,subprocess,copy,time,os,traceback
R=Path('/home/tsjang/rhwp');O=R/'output/pr-review/issue7754-nine';D=O/'source36-selected-family-query';D.mkdir(exist_ok=True);P=O/'source36-selected-family-query-state.json';s={'started':time.time(),'pid':os.getpid(),'results':[],'scope':'Diagnostic only: query the already selected exact SFNT family instead of adding the unqualified logical HFT alias. Program ID, metric preservation and original bytes retained.'}
def save():P.write_text(json.dumps(s,ensure_ascii=False,indent=2)+'\n')
try:
 guide=json.load(open(O/'source36-guide-default-font-metadata.json'))['fontEnvironment'];nested=json.load(open(R/'mydocs/pr/assets/planet6897_9208_9236_20261121/final-protection/nested-font-environment.json'));m=json.load(open(O/'source36-spsmj-process-diagnostic-execution.json'))['command'];mounts=m[:m.index('--')]
 for key,env in [('guide',guide),('nested',nested)]:
  before=copy.deepcopy(env['selectedPrograms']['한양신명조']);env['selectedPrograms']['한양신명조']['query']=before['family'];env['id']+='-selected-family-query-diagnostic';f=D/(key+'-environment.json');f.write_text(json.dumps(env,ensure_ascii=False,indent=2)+'\n')
  if key=='guide':cmd=[str(R/'venv/bin/python'),'scripts/visual_sweep.py','--silhouette-only','--hwp','samples/HWP5-nopassword-123456.hwpx','--pdf','pdf/HWP5-nopassword-123456-hwpx-2024.pdf','--key','guide-p002-selected-family-query','--pages','2','--rhwp-bin',str(O/'bin/rhwp-source36-optimized'),'--compat','2024','--dpi','96','--page-workers','1','--font-environment',str(f),'--out',str(D)];actual=cmd
  else:
   src=json.load(open(O/'source36-spsmj-other-native-process-state.json'))['results'][0]['command'];cmd=src[src.index('--')+1:];cmd[cmd.index('--key')+1]='nested-p016-selected-family-query';cmd[cmd.index('--out')+1]=str(D);cmd[cmd.index('--font-environment')+1]=str(f);actual=[*mounts,'--',*cmd]
  s['running']={'key':key,'command':actual};save();t=time.time()
  with (O/'logs'/f'source36-{key}-selected-family-query.log').open('w') as log:p=subprocess.run(actual,cwd=R,stdout=log,stderr=subprocess.STDOUT,timeout=1200)
  s['results'].append({'key':key,'before_selected_program':before,'after_selected_program':env['selectedPrograms']['한양신명조'],'command':actual,'exit_code':p.returncode,'seconds':time.time()-t});save()
 s.update(complete=True,running=None)
except BaseException as e:s['error']=str(e);s['traceback']=traceback.format_exc()
finally:s['ended']=time.time();save()
