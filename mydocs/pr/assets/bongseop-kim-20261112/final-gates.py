from pathlib import Path
import subprocess,time,os,json,sys
R=Path('/home/edward/mygithub/rhwp');W=Path('/home/edward/mygithub/rhwp-worktrees/review-integration-8942-bongseop-html-clipboard');O=R/'output/pr-review/bongseop-kim-20261112';env=os.environ.copy();env.update(CARGO_BUILD_JOBS='12',CARGO_TARGET_DIR=str(R/'target/pr-review'));records=[];source=subprocess.check_output(['git','rev-parse','HEAD'],cwd=W,text=True).strip();prefix=[str(O/'minimal-font/run.sh')]
commands=[('format',['cargo','fmt','--all','--','--check']),('manifest',['node','scripts/rust-test-suite-manifest.mjs','--check','--base-ref','80b1c354c29b7a02fb2b3c14a274da7f5b653ada'])]
for key in ['issue_8942_paste_html_table_border_attr','issue_8943_paste_html_leading_plain_run','issue_7516_html_paste_loose_inline_format']:
 commands.append(('focused-'+key,['node','scripts/run-rust-test.mjs',key,'--','--cargo-profile','release-test']))
commands += [('clippy-native',['cargo','clippy','--locked','--','-D','warnings']),('clippy-wasm',['cargo','clippy','--locked','-p','rhwp','--lib','--target','wasm32-unknown-unknown','--','-D','warnings']),('workspace-build',['cargo','build','--locked','--workspace']),('clippy-workspace',['cargo','clippy','--locked','--workspace','--all-targets','--','-D','warnings']),('full-regression',['cargo','nextest','run','--locked','--cargo-profile','release-test','--tests','--test-threads','8','--no-fail-fast']),('native-lib',['cargo','test','--locked','--profile','release-test','--features','native-skia','--lib','--','--test-threads','8']),('native-placeholder',['node','scripts/run-rust-test.mjs','issue_2225_missing_picture_placeholder','--','--cargo-profile','release-test','--features','native-skia']),('native-direct-pdf',['node','scripts/run-rust-test.mjs','render_p37_direct_pdf_export','--','--cargo-profile','release-test','--features','native-skia'])]
(O/'gate-logs').mkdir(exist_ok=True)
for name,cmd in commands:
 t=time.monotonic()
 with (O/'gate-logs'/(name+'.log')).open('w')as f:p=subprocess.run(prefix+cmd,cwd=W,env=env,stdout=f,stderr=subprocess.STDOUT)
 rec={'name':name,'source':source,'command':prefix+cmd,'returncode':p.returncode,'seconds':round(time.monotonic()-t,3)};records.append(rec);(O/'final-gate-receipts.json').write_text(json.dumps(records,indent=2));print(json.dumps(rec),flush=True)
 if p.returncode:raise SystemExit(p.returncode)
(O/'final-gates-passed.json').write_text(json.dumps({'source':source,'all_passed':True,'gates':len(records)},indent=2))
