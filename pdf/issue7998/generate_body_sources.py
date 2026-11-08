from pathlib import Path
import hashlib,json
b=Path(__file__).resolve().parent
def text(s,x,y,size=10,font='F1'):
 return f'BT /{font} {size} Tf 1 0 0 1 {x} {y} Tm ({s}) Tj ET'
basic=[text('ALPHA BETA',20,210),text('GAMMA DELTA',20,196),text('SECOND',20,160)]
columns=[text(s,x,y) for x,tag in [(20,'LEFT'),(220,'RIGHT')] for s,y in [(f'ALPHA {tag}',210),(f'GAMMA {tag}',196),(f'SECOND {tag}',160)]]
cases={
 'body-close-title':([text('CLOSE TITLE',20,210,14,'F2'),text('ALPHA BETA',20,195),text('GAMMA DELTA',20,181)],{'paragraphs':[['CLOSE TITLE','ALPHA BETA GAMMA DELTA']]}),
 'body-center':([text('ALPHA BETA',169.155,210),text('GAMMA',181.11,196),text('SECOND',20,160)],{'paragraphs':[['ALPHA BETA GAMMA','SECOND']],'alignment':['CENTER','LEFT'],'centerXPt':200}),
 'body-unrepresentable':([text('A',20,210),text('B',38,210),text('GAMMA',20,196),text('NORMAL BODY',20,150),text('NORMAL TAIL',20,136)],{'paragraphs':[['NORMAL BODY NORMAL TAIL']],'fallbackText':['A','B','GAMMA'],'reason':'inferred word-space width exceeds ordinary HWP character ratio range; unsupported paragraph must not consume the next supported one'}),
 'body-basic':(basic,{'paragraphs':[['ALPHA BETA GAMMA DELTA','SECOND']]}),
 'body-reordered':(list(reversed(basic)),{'paragraphs':[['ALPHA BETA GAMMA DELTA','SECOND']]}),
 'body-tj':(['BT /F1 10 Tf 20 210 Td [(ALPHA) -278 (BETA)] TJ ET',*basic[1:]],{'paragraphs':[['ALPHA BETA GAMMA DELTA','SECOND']]}),
 'body-columns':(columns,{'paragraphs':[['ALPHA LEFT GAMMA LEFT','SECOND LEFT'],['ALPHA RIGHT GAMMA RIGHT','SECOND RIGHT']]}),
 'body-title-columns':([text('HEADING ACROSS BOTH TEXT COLUMNS',20,242,14,'F2'),*columns],{'paragraphs':[['HEADING ACROSS BOTH TEXT COLUMNS'],['ALPHA LEFT GAMMA LEFT','SECOND LEFT'],['ALPHA RIGHT GAMMA RIGHT','SECOND RIGHT']]}),
 'body-indent':([text('ALPHA BETA',32,210),text('GAMMA DELTA',20,196),text('SECOND',20,160)],{'paragraphs':[['ALPHA BETA GAMMA DELTA','SECOND']],'firstIndentPt':12}),
 'body-list':([text('* LIST ENTRY',20,210),text('CONTINUED ITEM',35,196),text('NEXT ITEM',20,160)],{'paragraphs':[['* LIST ENTRY CONTINUED ITEM','NEXT ITEM']],'firstIndentPt':-15}),
 'body-ambiguous':([text('LEFT',20,210),text('RIGHT',220,210),text('NORMAL BODY',20,150),text('NORMAL TAIL',20,136)],{'paragraphs':[['NORMAL BODY NORMAL TAIL']],'fallbackText':['LEFT','RIGHT']}),
 'body-picture':([text('BEFORE PICTURE',20,210),'q 60 0 0 20 20 160 cm /Im1 Do Q',text('AFTER PICTURE',20,130)],{'paragraphs':[['BEFORE PICTURE'],['AFTER PICTURE']],'images':1,'relations':['BEFORE PICTURE above image above AFTER PICTURE']}),
 'body-occlusion':([text('VISIBLE BEFORE',20,210),'q 1 0 0 rg 20 204 90 10 re f Q',text('VISIBLE AFTER',20,196)],{'paragraphs':[],'fallbackText':['VISIBLE BEFORE','VISIBLE AFTER'],'reason':'overlapping retained paint interleaves source text orders'}),
 'body-baseline':([text('H',20,210),text('2',27.22,214,6),text('O',30.556,210),text('GAMMA',20,196),text('SECOND',20,160)],{'paragraphs':[['H2O GAMMA','SECOND']],'ascii2SizePt':6,'ascii2RisePt':4}),
}
for key,(ops,annotation) in cases.items():
 stream='\n'.join(ops).encode();objects=[b'<< /Type /Catalog /Pages 2 0 R >>',b'<< /Type /Pages /Count 1 /Kids [3 0 R] >>',b'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 260] /Resources << /Font << /F1 5 0 R /F2 6 0 R >> /XObject << /Im1 7 0 R >> >> /Contents 4 0 R >>',f'<< /Length {len(stream)} >>\nstream\n'.encode()+stream+b'\nendstream',b'<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>',b'<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold >>',b'<< /Type /XObject /Subtype /Image /Width 2 /Height 2 /ColorSpace /DeviceRGB /BitsPerComponent 8 /Length 12 >>\nstream\n'+bytes([0,120,220,220,120,0,220,120,0,0,120,220])+b'\nendstream']
 data=bytearray(b'%PDF-1.4\n');offsets=[]
 for i,obj in enumerate(objects,1):offsets.append(len(data));data.extend(f'{i} 0 obj\n'.encode()+obj+b'\nendobj\n')
 xref=len(data);data.extend(f'xref\n0 {len(objects)+1}\n0000000000 65535 f \n'.encode())
 for off in offsets:data.extend(f'{off:010} 00000 n \n'.encode())
 data.extend(f'trailer\n<< /Size {len(objects)+1} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n'.encode())
 (b/(key+'.pdf')).write_bytes(data);(b/(key+'-source.json')).write_text(json.dumps({'sha256':hashlib.sha256(data).hexdigest(),'source':'independently authored untagged PDF operators; expected grouping from generation annotations, not inferred output',**annotation},indent=2)+'\n')
