from pathlib import Path
import zipfile,xml.etree.ElementTree as E,copy,json,hashlib,sys
ROOT=Path(__file__).resolve().parents[3];D=Path(sys.argv[1]);D.mkdir(parents=True,exist_ok=True)
NS='http://www.hancom.co.kr/hwpml/2011/paragraph';q=lambda n:'{'+NS+'}'+n
for k,u in [('hp',NS),('hs','http://www.hancom.co.kr/hwpml/2011/section'),('hc','http://www.hancom.co.kr/hwpml/2011/core')]:E.register_namespace(k,u)
def load(p):
 with zipfile.ZipFile(p) as z:return {n:z.read(n) for n in z.namelist() if not n.startswith('Preview/')}
def write(name,entries,sec):
 p=D/(name+'-generated.hwpx')
 with zipfile.ZipFile(p,'w') as z:
  for n,b in entries.items():z.writestr(n,E.tostring(sec,encoding='utf-8',xml_declaration=True) if n=='Contents/section0.xml' else b,compress_type=zipfile.ZIP_STORED if n=='mimetype' else zipfile.ZIP_DEFLATED)
 return {'key':name,'state':'prepared','input':str(p),'input_sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'engine':'2020','output_filename':name+'-saved.hwp','output_dir':str(D/'hwp'/name)}
def para(parent,ident,text=''):
 p=E.SubElement(parent,q('p'),{'id':str(ident),'paraPrIDRef':'0','styleIDRef':'0','pageBreak':'0','columnBreak':'0','merged':'0'})
 r=E.SubElement(p,q('run'),{'charPrIDRef':'0'});E.SubElement(r,q('t')).text=text;return p,r
entries=load('samples/issue5818/cell_square_logo_text_wrap.hwpx');entries['BinData/image1.png']=(ROOT/'rhwp-studio/public/icons/icon-128.png').read_bytes();sec=E.fromstring(entries['Contents/section0.xml']);first=sec.find(q('p'))
base_run=copy.deepcopy(first.find(q('run')));outer=copy.deepcopy(first.find('.//'+q('tbl')));tc=copy.deepcopy(outer.find('.//'+q('tc')));pic=copy.deepcopy(tc.find('.//'+q('pic')))
for c in list(sec):sec.remove(c)
host,_=para(sec,0);host.remove(host.find(q('run')));host.append(base_run)
outer.set('colCnt','1');outer.set('rowCnt','1');outer.set('id','8101');outer.set('noAdjust','0');outer.find(q('sz')).set('width','42000');outer.find(q('sz')).set('height','18000')
for c in list(outer):
 if c.tag==q('tr'):outer.remove(c)
tr=E.SubElement(outer,q('tr'));tr.append(tc);tc.set('name','');tc.find(q('cellAddr')).set('colAddr','0');tc.find(q('cellSz')).set('width','42000');tc.find(q('cellSz')).set('height','18000')
sub=tc.find(q('subList'));sub.set('vertAlign','CENTER')
for c in list(sub):sub.remove(c)
p0,r0=para(sub,10);r0.insert(0,pic);pos=pic.find(q('pos'));pos.set('vertOffset',str((1<<32)-1200));pos.set('horzOffset','600');pic.find(q('sz')).set('width','3200');pic.find(q('sz')).set('height','4800')
nested=copy.deepcopy(outer);nested.set('id','8102');nested.set('pageBreak','NONE');nested.find(q('sz')).set('height','5000');nested.find(q('sz')).set('width','40000');inner=nested.find('.//'+q('tc'));inner.find(q('cellSz')).set('height','5000');inner.find(q('cellSz')).set('width','40000');inner.set('name','');ins=inner.find(q('subList'))
for c in list(ins):ins.remove(c)
para(ins,20,'Centered title');p1,r1=para(sub,11);r1.insert(0,nested);r=E.SubElement(host,q('run'),{'charPrIDRef':'0'});r.append(outer)
items=[]
for name,offset,wrap in [('cell-negative',str((1<<32)-1200),'SQUARE'),('cell-zero','0','SQUARE'),('cell-positive','1200','SQUARE'),('cell-overlay',str((1<<32)-1200),'BEHIND_TEXT')]:
 pos.set('vertOffset',offset);pic.set('textWrap',wrap);items.append(write(name,entries,sec))
pos.set('vertOffset',str((1<<32)-1200));pos.set('horzOffset','2000');pic.set('textWrap','SQUARE');items.append(write('cell-fragments',entries,sec))
entries=load('samples/issue2527_empty_linesegs.hwpx');sec=E.fromstring(entries['Contents/section0.xml']);first=sec.find(q('p'));run=copy.deepcopy(first.find(q('run')))
for c in list(run):
 if c.tag not in [q('secPr'),q('ctrl')]:run.remove(c)
for c in list(sec):sec.remove(c)
p,r=para(sec,0,'Original flow');p.insert(0,run)
p,r=para(sec,1);t=r.find(q('t'));t.text='Filler line 1'
for i in range(40):E.SubElement(t,q('lineBreak')).tail='Filler line '+str(i+2)
para(sec,2,' ');para(sec,3,' ');para(sec,4,'Automatic boundary');para(sec,5,'Tail preserved');items.append(write('body-original',entries,sec))
(D/'requests.json').write_text(json.dumps(items,indent=2))
