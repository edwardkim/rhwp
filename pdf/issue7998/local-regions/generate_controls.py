"""Explicit PDF graphics-state controls from an independent normal table Print."""
from pathlib import Path
from decimal import Decimal
import pikepdf, hashlib, json, argparse
parser=argparse.ArgumentParser();parser.add_argument("--only",action="append");parser.add_argument("--print-dir",type=Path,required=True);parser.add_argument("--output",type=Path,required=True);args=parser.parse_args()
b=args.output;b.mkdir(parents=True,exist_ok=True)
records=[]
for key in ['local-missing-clip','local-restored-clip','local-partial-clip','local-open-exterior','local-form','space-cell']:
 if args.only and key not in args.only:continue
 pdf=pikepdf.open(args.print_dir/('merged-blank.pdf' if key=='space-cell' else 'local-outer-only.pdf'));page=pdf.pages[0];ops=pikepdf.parse_content_stream(page)
 assert str(ops[15].operator)=='W*'
 if key=='space-cell':
  assert b'<0003> <0020>' in page.Resources.Font.F1.ToUnicode.read_bytes()
  ops+=pikepdf.parse_content_stream(pikepdf.Stream(pdf,b'q 2.873 331.56 176.695 12.84 re W n BT /F1 10 Tf 1 0 0 1 8 334.44 Tm <0003> Tj ET Q'))
 elif key=='local-missing-clip':del ops[15]
 elif key=='local-restored-clip':
  ops[17:17]=[pikepdf.ContentStreamInstruction([],pikepdf.Operator(k)) for k in ['Q','q']]
 elif key=='local-partial-clip':
  for i in [11,12]:
   operands=list(ops[i].operands);operands[0]=Decimal('178.568');ops[i]=pikepdf.ContentStreamInstruction(operands,ops[i].operator)
 elif key=='local-open-exterior':
  # Both duplicate left strokes are removed; duplicate paint is not new evidence.
  removed=0
  for i,op in enumerate(ops):
   if str(op.operator)=='S' and i>=2 and str(ops[i-1].operator)=='l' and str(ops[i-2].operator)=='m':
    start,end=ops[i-2].operands,ops[i-1].operands
    if start[0]==end[0]==Decimal('2.873000'):
     ops[i]=pikepdf.ContentStreamInstruction([],pikepdf.Operator('n'));removed+=1
  assert removed==2
 if key=='local-form':
  form=pikepdf.Stream(pdf,pikepdf.unparse_content_stream(ops));form.Type=pikepdf.Name.XObject;form.Subtype=pikepdf.Name.Form;form.BBox=page.MediaBox;form.Resources=page.Resources
  page.Resources=pikepdf.Dictionary(XObject=pikepdf.Dictionary(Fm=form));page.Contents=pikepdf.Stream(pdf,b'q 1 0 0 1 15 -35 cm /Fm Do Q')
 else:page.Contents=pikepdf.Stream(pdf,pikepdf.unparse_content_stream(ops))
 pdf.save(b/(key+'.pdf'));pdf.close()
