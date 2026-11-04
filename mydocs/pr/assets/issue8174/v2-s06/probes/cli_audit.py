from pathlib import Path
import hashlib, json, subprocess, time

root=Path.cwd()
out=root/'output/pr-review/issue8174/v2-s06-20261025'
exe=root/'output/pr-review/issue8174/v2-s05-20261025/candidate-native/rhwp'
rows=[]
for item in json.loads((out/'inputs.json').read_text()):
    name=item['name']
    dest=out/(name+'.hwpx')
    dest.write_bytes(b'OWNED OUTPUT SENTINEL')
    cmd=[str(exe),'import-pdf',str(out/'inputs'/name),str(dest),'--resource-profile','constrained','--resource-limits',str(out/'tiny-font-policy.json'),'--json']
    started=time.monotonic()
    result=subprocess.run(cmd,capture_output=True,timeout=40)
    (out/(name+'.cli.stdout')).write_bytes(result.stdout)
    (out/(name+'.cli.stderr')).write_bytes(result.stderr)
    try:
        payload=json.loads(result.stdout or result.stderr)
    except ValueError:
        payload={'stdout':result.stdout.decode(errors='replace')[:400],'stderr':result.stderr.decode(errors='replace')[:400]}
    row={'name':name,'command':cmd,'exit':result.returncode,'elapsed_seconds':time.monotonic()-started,'output_bytes':dest.stat().st_size,'sentinel_preserved':dest.read_bytes()==b'OWNED OUTPUT SENTINEL','payload':payload}
    rows.append(row)
    print(name,'exit',result.returncode,'preserved',row['sentinel_preserved'])
    if name=='cid-width-control.pdf':
        assert result.returncode==1 and row['sentinel_preserved']
    else:
        assert result.returncode==0 and not row['sentinel_preserved']
(out/'cli-audit.json').write_text(json.dumps({'executable':str(exe),'sha256':hashlib.sha256(exe.read_bytes()).hexdigest(),'rows':rows},indent=2)+'\n')
