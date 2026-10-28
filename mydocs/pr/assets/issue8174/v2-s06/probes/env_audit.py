from pathlib import Path
import subprocess,json,os,time

root=Path.cwd();out=root/'output/pr-review/issue8174/v2-s06-20261025'
content=b'BT /F1 12 Tf 10 150 Td '+b'(A) Tj '*80000+b' ET'
objects=[b'<< /Type /Catalog /Pages 2 0 R >>',b'<< /Type /Pages /Count 1 /Kids [3 0 R] >>',b'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 240 200] /Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>',f'<< /Length {len(content)} >>\nstream\n'.encode()+content+b'\nendstream',b'<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>']
data=bytearray(b'%PDF-1.7\n');offsets=[0]
for i,obj in enumerate(objects,1):
    offsets.append(len(data));data.extend(f'{i} 0 obj\n'.encode()+obj+b'\nendobj\n')
start=len(data);data.extend(f'xref\n0 {len(offsets)}\n0000000000 65535 f \n'.encode())
for pos in offsets[1:]:data.extend(f'{pos:010d} 00000 n \n'.encode())
data.extend(f'trailer\n<< /Size {len(offsets)} /Root 1 0 R >>\nstartxref\n{start}\n%%EOF\n'.encode())
source=out/'environment-observation.pdf';source.write_bytes(data)
dest=out/'environment-observation.hwpx';dest.write_bytes(b'OWNED OUTPUT SENTINEL')
exe=root/'output/pr-review/issue8174/v2-s05-20261025/candidate-native/rhwp'
env=dict(os.environ);env['RHWP_AUDIT_OWN_SENTINEL']='owned-audit-value';env['RHWP_FONT_PATH']=str(root/'assets/fonts')
cmd=[str(exe),'import-pdf',str(source),str(dest),'--resource-profile','constrained','--json']
child=subprocess.Popen(cmd,stdout=subprocess.PIPE,stderr=subprocess.PIPE,env=env)
observation=None
for _ in range(1000):
    children=Path(f'/proc/{child.pid}/task/{child.pid}/children')
    if children.exists():
        for pid in children.read_text().split():
            proc=Path('/proc')/pid
            try:
                if b'__pdf-import-worker-v1' not in (proc/'cmdline').read_bytes():continue
                variables=dict(part.split(b'=',1) for part in (proc/'environ').read_bytes().split(b'\0') if b'=' in part)
                status=dict(line.split(':',1) for line in (proc/'status').read_text().splitlines() if ':' in line)
                limits=(proc/'limits').read_text().splitlines()
                observation={'own_sentinel_present':b'RHWP_AUDIT_OWN_SENTINEL' in variables,'own_font_path_forwarded':variables.get(b'RHWP_FONT_PATH')==env['RHWP_FONT_PATH'].encode(),'worker_nonce_present':b'RHWP_PDF_WORKER_NONCE' in variables,'no_new_privs':status.get('NoNewPrivs','').strip(),'limits':[line for line in limits if any(s in line for s in ['Max cpu time','Max address space','Max core file size'])],'scope':'Only presence of owned sentinels and selected OS fields read; no ambient environment values captured'}
            except (FileNotFoundError,ProcessLookupError):pass
    if observation or child.poll() is not None:break
    time.sleep(.001)
stdout,stderr=child.communicate(timeout=40)
(out/'environment-observation.stdout').write_bytes(stdout);(out/'environment-observation.stderr').write_bytes(stderr)
assert observation is not None,'own worker not observed'
assert not observation['own_sentinel_present'] and observation['own_font_path_forwarded'] and observation['worker_nonce_present']
assert observation['no_new_privs']=='1'
(out/'cli-environment.json').write_text(json.dumps({'command':cmd,'observation':observation,'exit':child.returncode,'output_sentinel_preserved':dest.read_bytes()==b'OWNED OUTPUT SENTINEL'},indent=2)+'\n')
print(json.dumps({'observation':observation,'exit':child.returncode},indent=2))
