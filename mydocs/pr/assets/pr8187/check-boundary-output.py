import hashlib,json,pathlib
root=pathlib.Path.cwd()
work=root/'output/pr-review/pr8187'
rows=[]
def walk(n):
 yield n
 for c in n.get('children',[]): yield from walk(c)
def ruling(table,cell,top):
 b=cell['bbox']; y=b['y']+(0 if top else b['h'])
 return any(n['type']=='Line' and n['bbox']['h']<2 and abs(n['bbox']['y']-y)<=2 and n['bbox']['x']<=b['x']+2 and n['bbox']['x']+n['bbox']['w']>=b['x']+b['w']-2 for n in table.get('children',[]))
for producer in ['base','head']:
 for backend in ['native','wasm']:
  cells=[]; checks={}; hashes={}
  for page in [65,66]:
   path=work/f'{producer}-{backend}-final/2022/kps-ai/render_tree/render_tree_{page:03}.json'
   hashes[str(path.relative_to(root))]=hashlib.sha256(path.read_bytes()).hexdigest()
   tree=json.loads(path.read_text())
   inner=next(n for n in walk(tree) if n.get('type')=='Table' and n.get('rows')==29)
   outer=next(n for n in walk(tree) if n.get('type')=='Table' and n.get('pi')==674 and n.get('ci')==0)
   label=next(n for n in inner['children'] if n.get('type')=='Cell' and n.get('row')==18 and n.get('col')==0)
   parent=next(n for n in outer['children'] if n.get('type')=='Cell' and n.get('row')==2 and n.get('col')==0)
   cells.append(label)
   for name,t,c in [('label',inner,label),('parent',outer,parent)]:
    for top in [True,False]:checks[f'p{page}_{name}_{"top" if top else "bottom"}_frame']=ruling(t,c,top)
   if page==65:checks['source_first_frame_height']=abs(inner['bbox']['h']-63299/75)<=0.1
   else:
    following=next(n for n in inner['children'] if n.get('type')=='Cell' and n.get('row')==24 and n.get('col')==0)
    checks['next_content_starts_at_label_end']=abs(label['bbox']['y']+label['bbox']['h']-following['bbox']['y'])<=0.2
  checks['source_rowspan_height_preserved']=abs(sum(c['bbox']['h'] for c in cells)-19259/75)<=0.2
  rows.append({'producer':producer,'backend':backend,'checks':checks,'all_pass':all(checks.values()),'render_tree_sha256':hashes})
assert all(not r['all_pass'] for r in rows if r['producer']=='base')
assert all(r['all_pass'] for r in rows if r['producer']=='head')
p=root/'mydocs/pr/assets/pr8187/boundary-execution.json'
p.write_text(json.dumps({'input':'rhwp-studio/public/samples/kps-ai.hwp','source_first_height_hu':63299,'source_label_height_hu':19259,'tree_rounding_tolerance_px':0.2,'results':rows},ensure_ascii=False,indent=2)+'\n')
print(json.dumps(rows,ensure_ascii=False,indent=2))
