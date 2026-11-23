from pathlib import Path
import zipfile,json,hashlib,time,os
R=Path('/home/tsjang/rhwp');O=R/'output/pr-review/issue7754-nine';A=Path('/home/tsjang/Downloads/hft2otf-deploy-20261117/previous-installed-otf.zip');D=O/'private-canonical-otf-recovery';D.mkdir(exist_ok=True,mode=0o700)
audit=json.load(open(O/'source36-previous-installed-font-archive-audit.json'));wanted={x['sha256'] for x in audit['matched']};rows=[]
with zipfile.ZipFile(A) as z:
 for i in z.infolist():
  if i.is_dir():continue
  b=z.read(i);h=hashlib.sha256(b).hexdigest()
  if h not in wanted:continue
  p=D/(h+'.otf')
  if not p.exists():p.write_bytes(b);p.chmod(0o600)
  assert hashlib.sha256(p.read_bytes()).hexdigest()==h;rows.append({'archive_entry':i.filename,'path':str(p),'sha256':h,'program_id_face0':hashlib.sha256(b+bytes(4)).hexdigest(),'bytes':len(b)})
r={'recorded':time.time(),'archive':str(A),'archive_sha256':audit['archive_sha256'],'scope':'Private exact original OTF recovery for existing canonical protected contexts only. No installation, no repository font bytes, no product priority change.','files':rows,'unique_files':len(wanted)};(O/'source36-canonical-font-archive-recovery.json').write_text(json.dumps(r,indent=2)+'\n');print('recovered exact files',len(wanted),'entries',len(rows))
