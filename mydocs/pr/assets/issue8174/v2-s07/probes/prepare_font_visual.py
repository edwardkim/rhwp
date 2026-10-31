from pathlib import Path
import pymupdf as fitz, json, hashlib, subprocess
R=Path.cwd();O=R/'output/pr-review/issue8174/v2-s07-20261025';D=O/'font-visual-inputs';D.mkdir(exist_ok=True)
font=Path('/usr/share/fonts/opentype/urw-base35/NimbusRoman-Regular.otf')
d=fitz.open();p=d.new_page(width=360,height=240);p.insert_font(fontname='ReviewCFF',fontfile=str(font))
for y,size,text in [(45,20,'CFF: ABa office fi fl'),(85,16,'0123456789 + - = (A, B).'),(125,14,'Unicode: ÁÉÖ αβγ – —')]:
 p.insert_text((24,y),text,fontname='ReviewCFF',fontsize=size)
d.save(D/'cff-cid.pdf');d.close()
rows=[]
for f in [D/'cff-cid.pdf',R/'pdf/issue5875/nested_table_text_caption-2020.pdf']:
 rows.append({'input':str(f),'reference':str(f),'sha256':hashlib.sha256(f.read_bytes()).hexdigest(),'pages':len(fitz.open(f))})
(O/'font-visual-inputs.json').write_text(json.dumps({'font':str(font),'fontSha256':hashlib.sha256(font.read_bytes()).hexdigest(),'fixtures':rows},indent=2)+'\n')
print(subprocess.check_output(['pdffonts',str(D/'cff-cid.pdf')],text=True))
