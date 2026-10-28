from pathlib import Path
import subprocess,json,hashlib
R=Path.cwd();O=R/'output/pr-review/issue8174/v2-s07-20261025';sha=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip();names={
'allocation.rs':['fn entries','fn vec','fn map','fn insert','fn string_capacity'],
'cff_encoding.rs':['fn parse_index_limited','fn parse_dict_real_limited','fn parse_cff_font_matrix_scale_with_budget','fn parse_cff_cid_to_gid_with_budget','fn parse_charset_limited','fn parse_cff_encoding_with_budget','fn parse_cff_gid_mapping_with_pdf_encoding_with_budget','fn resolve_bytes_via_pdf_encoding_limited'],
'font_dict.rs':['fn embedded_glyph_name','parse_type1_encoding_with_budget(','parse_cff_cid_to_gid_with_budget(','fn wrap_cff_in_opentype_with_budget','fn char_to_unicode(','fn char_to_unicode_uncached','fn shift_jis_to_unicode','fn normalize_cjk_radical_forms_owned','fn decode_legacy_cjk'],
'cmap.rs':['fn get_with_budget'],
'character_mapper.rs':['fn glyph_name_to_unicode_char'],
 'type1_encoding.rs':['fn parse_type1_encoding_with_budget','fn parse_dup_entry']}
rows=[]
for name,needles in names.items():
 path=Path('vendor/rhwp-pdf-import-backend/src/fonts')/name;data=(R/path).read_bytes();lines=data.decode().splitlines();blob=subprocess.check_output(['git','rev-parse',sha+':'+str(path)],text=True).strip();actual=subprocess.check_output(['git','hash-object',str(path)],text=True).strip();assert blob==actual
 rows.append({'path':str(path),'gitBlob':blob,'sha256':hashlib.sha256(data).hexdigest(),'boundaries':[{'needle':needle,'line':next(i+1 for i,line in enumerate(lines) if needle in line)} for needle in needles]})
(O/'source-boundaries.json').write_text(json.dumps({'source':sha,'base':'9418ed0c7cf289c5c4ac2b3bb8cdc18a1b637a9a','before':'4e2149dc65bdbc15413bf09d710c85882260e2ae','files':rows,'note':'Only the font expansion/Unicode boundary is corrected. FontInfo payload clone/index, other parser/frontend/Core allocations and host confinement remain tracked separately.'},indent=2)+'\n')
print('source',sha,'files',len(rows),'boundaries',sum(len(x['boundaries']) for x in rows))
