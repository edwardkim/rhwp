from pathlib import Path
import subprocess,os,json,time,hashlib,traceback,re,shutil
R=Path('/home/tsjang/rhwp');W=Path('/home/tsjang/rhwp-worktrees/task-7754-rebase-5665');O=R/'output/pr-review/issue7754-nine';P=O/'source36-rust-gates-state.json';BASE='1a27faacff48ea7329e76ce8eca15aa9ca8aa877';HEAD='c48c2b895d31ba0fd4969c837bfc7a6d8a2c65ac';T=str(R/'target/pr-review');wrapper=O/'minimal-font-review/run.sh'
env=dict(os.environ,CARGO_BUILD_JOBS='3',CARGO_TARGET_DIR=T,CARGO_PROFILE_RELEASE_CODEGEN_UNITS='16',CARGO_PROFILE_DEV_DEBUG='0',CARGO_PROFILE_TEST_DEBUG='0');s={'started':time.time(),'pid':os.getpid(),'base':BASE,'candidate':HEAD,'results':[],'environment':{k:env[k] for k in ['CARGO_BUILD_JOBS','CARGO_TARGET_DIR','CARGO_PROFILE_RELEASE_CODEGEN_UNITS','CARGO_PROFILE_DEV_DEBUG','CARGO_PROFILE_TEST_DEBUG']}}
def save():P.write_text(json.dumps(s,indent=2)+'\n')
def git(*a):return subprocess.check_output(['git',*a],cwd=W,text=True).strip()
def sha(p):return hashlib.file_digest(Path(p).open('rb'),'sha256').hexdigest()
def run(label,cmd,minimum=True):
 assert git('rev-parse','HEAD')==(BASE if label.startswith('base-') else HEAD)
 s['running']={'label':label,'command':cmd,'cwd':str(W),'minimum_fonts':minimum};save();log=O/'logs'/('source36-rust-'+label+'.log');t=time.monotonic()
 actual=[str(wrapper),*cmd] if minimum else cmd
 with log.open('w') as out:p=subprocess.run(actual,cwd=W,env=env,stdout=out,stderr=subprocess.STDOUT,timeout=10800)
 row={'label':label,'command':actual,'cwd':str(W),'exit_code':p.returncode,'seconds':time.monotonic()-t,'log':str(log),'source_head':git('rev-parse','HEAD')};s['results'].append(row);save();print(label,p.returncode,row['seconds'],flush=True);assert p.returncode==0,(label,p.returncode,str(log))
try:
 s['running']='wait base optimized production and exclusive Cargo availability';save();deadline=time.monotonic()+21600
 while True:
  b=json.loads((O/'source36-base-build-state.json').read_text())
  if b.get('completed'):break
  if b.get('ended'):raise RuntimeError(b.get('error','base production failed'))
  if time.monotonic()>deadline:raise TimeoutError('base production wait')
  time.sleep(10)
 s['base_producer_receipt_sha256']=sha(O/'source36-base-build-state.json');assert git('rev-parse','HEAD')==BASE;assert not git('status','--porcelain=v1')
 # Bind mounts supply unchanged input bytes instead of duplicating corpus/PDF storage.
 for name in ['samples','pdf','rhwp-studio','npm']:(W/name).mkdir(exist_ok=True)
 # CLI embeds these tracked documents. They are identical between the frozen base and head.
 embedded=['mydocs/manual/agent_knowledge_map.md','mydocs/manual/agent_troubleshooting_guide.md','mydocs/manual/gym_optional_tool.md','mydocs/tech/agent_roadmap/atlas_r1_r200.md']+[f'mydocs/manual/recipes/{x}.md' for x in ['01_fill_form_and_submit','02_table_csv_roundtrip','03_redact_before_sharing','04_safety_check_untrusted_doc','05_mail_merge_batch_fill','06_visual_regression_before_after']]
 for p in embedded:assert subprocess.check_output(['git','show',BASE+':'+p],cwd=R)==(R/p).read_bytes(),p
 s['shared_embedded_documents']={p:sha(R/p) for p in embedded};save()
 run('base-prepare',['/usr/local/bin/node','scripts/rust-test-suite-manifest.mjs','--prepare'],False)
 run('base-ctrl-data',['/usr/local/bin/node','scripts/run-rust-test.mjs','issue_8225_group_child_ctrl_data','--','--cargo-profile','release-test','--target-dir',T,'--features','native-skia','--test-threads','12','--no-fail-fast'])
 changed=git('diff','--name-only').splitlines();assert all(x=='tests/suites/manifest.json' or x.startswith('tests/generated/') for x in changed),changed
 if changed:subprocess.run(['git','restore','--source=HEAD','--',*changed],cwd=W,check=True)
 assert not git('status','--porcelain=v1')
 subprocess.run(['git','switch','--detach',HEAD],cwd=W,check=True)
 subprocess.run(['git','diff','--exit-code','436f8629b7f2b9aee520962297b79cdf15c17743',HEAD,'--','src','crates','build_scripts','.cargo','Cargo.toml','Cargo.lock','build.rs'],cwd=W,check=True)
 run('candidate-prepare',['/usr/local/bin/node','scripts/rust-test-suite-manifest.mjs','--prepare'],False)
 run('manifest-policy',['/usr/local/bin/node','scripts/rust-test-suite-manifest.mjs','--check','--base-ref',BASE],False)
 run('unit-tier-policy',['/usr/local/bin/node','scripts/rust-unit-test-tiers.mjs','--check','--base-ref',BASE],False)
 run('format',['cargo','fmt','--all','--','--check'],False)
 run('native-unfiltered-lib',['cargo','test','--locked','--profile','release-test','--target-dir',T,'--features','native-skia','--lib','--','--test-threads','12'])
 # Execute existing Native CI logical cases, once each, through the existing selector.
 # The root lib filter is intentionally replaced by the stronger unfiltered run above.
 workflow=(W/'.github/workflows/ci.yml').read_text() if (W/'.github/workflows/ci.yml').exists() else subprocess.check_output(['git','show',HEAD+':.github/workflows/ci.yml'],cwd=W,text=True)
 start=workflow.index('      - name: Native Skia tests\n');end=workflow.index('            release)\n',start);block=workflow[start:end]
 cases=list(dict.fromkeys(re.findall(r'node scripts/run-rust-test\.mjs --cargo-test ([a-z0-9_]+) --',block)));assert 'issue_8153_print_glyph_geometry' in cases and 'issue_2225_missing_picture_placeholder' in cases and 'render_p37_direct_pdf_export' in cases;s['native_existing_ci_cases']=cases;save()
 for case in cases:run('native-'+case,['/usr/local/bin/node','scripts/run-rust-test.mjs',case,'--','--cargo-profile','release-test','--target-dir',T,'--features','native-skia','--test-threads','12','--no-fail-fast'])
 for case in ['issue_7754_original_frame_and_glyph_origins','issue_8418_hft_outline_tracking','issue_8225_group_child_ctrl_data']:
  run('focused-'+case,['/usr/local/bin/node','scripts/run-rust-test.mjs',case,'--','--cargo-profile','release-test','--target-dir',T,'--features','native-skia','--test-threads','12','--no-fail-fast'])
 for label,cmd in [('workspace-clippy',['cargo','clippy','--locked','--workspace','--all-targets','--target-dir',T,'--','-D','warnings']),('wasm-clippy',['cargo','clippy','--locked','--target','wasm32-unknown-unknown','--lib','--target-dir',T,'--','-D','warnings']),('workspace-build',['cargo','build','--locked','--workspace','--target-dir',T]),('all-rust',['cargo','nextest','run','--locked','--cargo-profile','release-test','--target-dir',T,'--tests','--test-threads','12','--no-fail-fast'])]:run(label,cmd)
 s.update(passed=True,complete_coverage=True,running=None)
except BaseException as e:s['error']=str(e);s['traceback']=traceback.format_exc();print(s['traceback'],flush=True)
finally:s['ended']=time.time();save()
