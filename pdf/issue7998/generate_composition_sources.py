from pathlib import Path
import hashlib,json,re,argparse
parser=argparse.ArgumentParser(description='Independent PDF table composition sources')
parser.add_argument('--output',type=Path,required=True)
parser.add_argument('--include-compound-hold',action='store_true')
args=parser.parse_args();out=args.output;out.mkdir(parents=True,exist_ok=True)

def text(s,x,y):return f'BT /F1 10 Tf 1 0 0 1 {x} {y} Tm ({s}) Tj ET'
def grid(x,y,w,h,dash=False):
 ops=['q 0 G 0.5 w'+(' [2 2] 0 d' if dash else '')]
 for a,b,c,d in [(x,y,x+w,y),(x,y+h,x+w,y+h),(x,y,x,y+h),(x+w,y,x+w,y+h),(x+w/2,y,x+w/2,y+h),(x,y+h/2,x+w,y+h/2)]:ops.append(f'{a} {b} m {c} {d} l S')
 return ops+['Q']
base=[text('OUTSIDE ABOVE',20,345),*grid(20,30,360,300),text('UPPER RIGHT',210,315),text('LOWER LEFT',30,165),text('LOWER RIGHT',210,165),text('OUTSIDE AFTER',20,12)]
flow=[text('ALPHA BETA',30,315),text('GAMMA DELTA',30,301),'q 60 0 0 20 30 260 cm /Im1 Do Q',text('AFTER PICTURE',30,245),text('CONTINUED TEXT',30,231)]
child=[*grid(40,220,140,60),text('A',48,265),text('B',118,265),text('C',48,235),text('D',118,235)]
cases={
 'cell-picture':(base+flow,{'tables':1,'firstCellParagraphs':['ALPHA BETA GAMMA DELTA','','AFTER PICTURE CONTINUED TEXT'],'firstCellImages':1}),
 'cell-picture-first':(base+['q 60 0 0 20 30 300 cm /Im1 Do Q',text('AFTER PICTURE',30,285),text('CONTINUED TEXT',30,271)],{'tables':1,'firstCellParagraphs':['','AFTER PICTURE CONTINUED TEXT'],'firstCellImages':1}),
 'cell-two-pictures':(base+[text('BEFORE',30,315),'q 60 0 0 20 30 280 cm /Im1 Do Q','q 45 0 0 15 30 250 cm /Im1 Do Q',text('AFTER',30,235)],{'tables':1,'firstCellImages':2,'order':['BEFORE','picture','picture','AFTER']}),
 'cell-nested':(base+[text('BEFORE CHILD',30,315),*child,text('AFTER CHILD',30,200)],{'tables':2,'nestedTables':1,'childCells':['A','B','C','D'],'order':['BEFORE CHILD','child table','AFTER CHILD']}),
 'cell-picture-crossing':(base+[text('BEFORE',30,315),'q 60 0 0 20 175 260 cm /Im1 Do Q',text('AFTER',30,235)],{'tables':0,'reason':'picture crosses unrelated cell owners; retain source owners'}),
 'cell-picture-overlap':(base+[text('BEFORE',30,315),'q 60 0 0 20 30 280 cm /Im1 Do Q',text('OVERLAPPING',30,290)],{'tables':0,'reason':'picture and text physically overlap; retain positional source paint'}),
}
# Hierarchy is independently drawn as a strictly enclosed grid in the first cell.
for key,parent_dash,child_dash in [('nested-dash-child',False,True),('nested-dash-parent',True,False),('nested-dash-both',True,True)]:
 parent=[text('OUTSIDE ABOVE',20,345),*grid(20,30,360,300,parent_dash),text('UPPER RIGHT',210,315),text('LOWER LEFT',30,165),text('LOWER RIGHT',210,165),text('OUTSIDE AFTER',20,12)]
 nested=[*grid(40,220,140,60,child_dash),text('A',48,265),text('B',118,265),text('C',48,235),text('D',118,235)]
 cases[key]=(parent+[text('BEFORE CHILD',30,315),*nested,text('AFTER CHILD',30,200)],{'tables':2,'nestedTables':1,'childCells':['A','B','C','D'],'order':['BEFORE CHILD','child table','AFTER CHILD'],'parentDashed':parent_dash,'childDashed':child_dash})
cases['nested-dash-first']=(base+[*grid(40,240,140,60,True),text('A',48,285),text('B',118,285),text('C',48,255),text('D',118,255),text('AFTER CHILD',30,220)],{'tables':2,'nestedTables':1,'childCells':['A','B','C','D'],'order':['child table','AFTER CHILD']})
cases['nested-dash-image']=(base+[text('BEFORE CHILD',30,315),*grid(40,220,140,60,True),'q 50 0 0 12 45 260 cm /Im1 Do Q',text('B',118,265),text('C',48,235),text('D',118,235),text('AFTER CHILD',30,200)],{'tables':2,'nestedTables':1,'childCells':['','B','C','D'],'childImages':1})
siblings=[text('BEFORE CHILD',30,315)]
for y,tags in [(280,'ABCD'),(220,'EFGH')]:
 siblings+=grid(40,y,140,30,True)
 siblings += [text(tag,x,at) for tag,x,at in [(tags[0],48,y+20),(tags[1],118,y+20),(tags[2],48,y+5),(tags[3],118,y+5)]]
cases['nested-dash-siblings']=(base+siblings+[text('AFTER CHILD',30,200)],{'tables':3,'nestedTables':2,'childCells':[['A','B','C','D'],['E','F','G','H']]})
# A real third level tests physical ownership, not just a flat list of two targets.
deep=[text('BEFORE CHILD',30,315),*grid(35,200,150,100,False),*grid(40,260,60,30,True),text('A',43,280),text('B',73,280),text('C',43,265),text('D',73,265),text('RIGHT',118,285),text('BOTTOM',43,225),text('END',118,225),text('AFTER CHILD',30,190)]
cases['nested-dash-deep']=(base+deep,{'tables':3,'nestedTables':2,'depth':2,'deepCells':['A','B','C','D']})
# Separate independent inputs: solid outer edges, native short internal segments.
# PDF dash operators above are preserved as compound-boundary follow-up inputs.
for old_key,(original,annotation) in list(cases.items()):
 if not old_key.startswith('nested-dash-'):continue
 ops=[];periodic=False;edge=0
 for op in original:
  if '[2 2] 0 d' in op:periodic=True;edge=0;ops.append('q 0 G 0.5 w');continue
  if op=='Q':periodic=False
  if periodic and re.fullmatch(r'[0-9.]+ [0-9.]+ m [0-9.]+ [0-9.]+ l S',op):
   edge+=1
   if edge<=4:ops.append(op);continue
   x,y,_,xx,yy,_,_=op.split();x,y,xx,yy=map(float,[x,y,xx,yy]);length=abs(xx-x)+abs(yy-y)
   for at in range(0,int(length),2):
    end=min(at+1,length)
    if xx!=x:ops.append(f'{x+at:g} {y:g} m {x+end:g} {y:g} l S')
    else:ops.append(f'{x:g} {y+at:g} m {x:g} {y+end:g} l S')
  else:ops.append(op)
 key=old_key.replace('nested-dash-','nested-periodic-');length=0.
 for op in ops:
  m=re.fullmatch(r'([0-9.]+) ([0-9.]+) m ([0-9.]+) ([0-9.]+) l S',op)
  if m:
   x,y,xx,yy=map(float,m.groups());length+=abs(xx-x)+abs(yy-y)
 cases[key]=(ops,{**annotation,'representation':'solid outer borders and explicit 1pt internal segments every 2pt','sourceLineLengthPt':length})
base_ops,base_annotation=cases['nested-periodic-child']
cases['nested-periodic-background']=([base_ops[0],'q 0.9 0.95 1 rg 25 185 170 140 re f Q',*base_ops[1:]],{**base_annotation,'sourceBackgroundAreaPt2':23800})
for key,(ops,annotation) in cases.items():
 if key.startswith('nested-dash-') and not args.include_compound_hold:continue
 stream='\n'.join(ops).encode();objects=[b'<< /Type /Catalog /Pages 2 0 R >>',b'<< /Type /Pages /Count 1 /Kids [3 0 R] >>',b'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 360] /Resources << /Font << /F1 5 0 R >> /XObject << /Im1 6 0 R >> >> /Contents 4 0 R >>',f'<< /Length {len(stream)} >>\nstream\n'.encode()+stream+b'\nendstream',b'<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>',b'<< /Type /XObject /Subtype /Image /Width 2 /Height 2 /ColorSpace /DeviceRGB /BitsPerComponent 8 /Length 12 >>\nstream\n'+bytes([0,120,220,220,120,0,220,120,0,0,120,220])+b'\nendstream']
 data=bytearray(b'%PDF-1.4\n');offsets=[]
 for i,obj in enumerate(objects,1):offsets.append(len(data));data.extend(f'{i} 0 obj\n'.encode()+obj+b'\nendobj\n')
 xref=len(data);data.extend(f'xref\n0 {len(objects)+1}\n0000000000 65535 f \n'.encode())
 for offset in offsets:data.extend(f'{offset:010} 00000 n \n'.encode())
 data.extend(f'trailer\n<< /Size {len(objects)+1} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n'.encode());(out/(key+'.pdf')).write_bytes(data)
 (out/(key+'-source.json')).write_text(json.dumps({'source':'independently authored untagged PDF operators; geometry and annotations define expected ownership, not inferred output','sha256':hashlib.sha256(data).hexdigest(),**annotation},indent=2)+'\n')
