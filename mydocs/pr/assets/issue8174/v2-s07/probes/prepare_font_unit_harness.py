from pathlib import Path
import shutil,subprocess,json,hashlib,re
R=Path.cwd();O=R/'output/pr-review/issue8174/v2-s07-20261025';D=O/'font-unit-backend'
assert not D.exists()
shutil.copytree(R/'vendor/rhwp-pdf-import-backend',D,ignore=shutil.ignore_patterns('target','.git'))
patches=[]
for name in ['objstm.rs','xref.rs']:
 p=D/'src'/name;s=p.read_text();start=s.index('#[cfg(test)]\nmod tests');prefix=s[:start];body=s[start:]
 original=subprocess.check_output(['git','show','4e2149dc65bdbc15413bf09d710c85882260e2ae:vendor/rhwp-pdf-import-backend/src/'+name]);assert original==s.encode()
 if name=='objstm.rs':
  assert body.count('Bytes::')==6;body=body.replace('Bytes::','StreamData::')
 else:
  old='parse_xref_iterative(&mut cursor, off_classic as u64).unwrap()';assert body.count(old)==1;body=body.replace(old,'parse_xref_iterative(&mut cursor, off_classic as u64, None).unwrap()')
 p.write_text(prefix+body);assert p.read_text().split('#[cfg(test)]\nmod tests')[0]==prefix
 patches.append({'file':name,'unchangedSinceSliceStart':True,'originalSolutionCommit':'4662c069e557e928ed7968d061121fde90b1f027','changes':'Only legacy test wiring inside the existing cfg(test) module; no expectation or production code changes','productionPrefixSha256':hashlib.sha256(prefix.encode()).hexdigest()})
font=Path('/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf');dest=D/'tests/fixtures/fonts/DejaVuSans.ttf';dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(font,dest)
manifest=(D/'Cargo.toml').read_text();manifest=re.sub(r'path = "\.\./([^\"]+)"',lambda m:'path = "'+str(R/'vendor'/m[1])+'"',manifest);(D/'Cargo.toml').write_text(manifest)
fonts=[]
for p in sorted((R/'vendor/rhwp-pdf-import-backend/src/fonts').rglob('*.rs')):
 q=D/'src/fonts'/p.relative_to(R/'vendor/rhwp-pdf-import-backend/src/fonts');assert p.read_bytes()==q.read_bytes();fonts.append({'path':str(p.relative_to(R)),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()})
(O/'font-unit-harness.json').write_text(json.dumps({'source':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),'fontSourcesExact':fonts,'contextTestWiringOnly':patches,'compileOnlyFixture':{'source':str(font),'sha256':hashlib.sha256(font.read_bytes()).hexdigest(),'note':'Required to compile unrelated html_css tests; font suite selection does not run those tests.'},'originalFullBackendHarnessCompiles':False,'claim':'Exact unchanged product font source and its existing unit tests, with test-only adapters for pre-existing context compilation defects. This is not a passing whole-backend test run.'},indent=2)+'\n')
print('prepared exact font source harness; seven unrelated test calls adapted; one compile-only upstream font fixture supplied')
