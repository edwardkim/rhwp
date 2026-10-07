from pathlib import Path
import importlib.util, sys, subprocess, os, json, hashlib
root=Path.cwd(); out=root/'output/pr-review/pr7518-20261004'
spec=importlib.util.spec_from_file_location('p39_visual_sweep',root/'scripts/visual_sweep.py'); sweep=importlib.util.module_from_spec(spec); sys.modules[spec.name]=sweep; spec.loader.exec_module(sweep)
for page in [22,38]:
 num=f"{page:03}"
 base=out/f'p22-p38-frame-wasm/control-{page}'; native=out/'p22-p38-frame-native/regulatory-p22-p38-frame'
 for name in ['svg','rhwp_png','compare','overlay','review']:(base/name).mkdir(parents=True,exist_ok=True)
 raw=out/f'cdp-p39-frame/wasm_{num}.svg'; svg=base/f'svg/wasm_{num}.svg'
 svg.write_text(sweep.apply_svg_font_policy(raw.read_text(),sweep.svg_font_face_rules(native/f'svg/76076_regulatory_analysis_{num}.svg')))
 sweep.check_sweep_embedded_fonts([svg],base/'embedded-font-check.json')
 chrome='/home/edward/.cache/puppeteer/chrome/linux-152.0.7977.54/chrome-linux64/chrome'
 png=base/f'rhwp_png/rhwp_{num}.png'; pdf=native/f'pdf_png/pdf-{page}.png'
 cmd=['node','scripts/rasterize-svg-webfonts.mjs','--input',str(svg),'--output',str(png),'--zoom','1','--chrome',chrome]
 subprocess.run(cmd,check=True,env={**os.environ,'VISUAL_SWEEP_CHROME':chrome})
 compares=sweep.make_compares([png],[pdf],base/'compare',f'regulatory-p{page}-frame fresh WASM')
 overlay=base/f'overlay/overlay_{num}.png'
 metrics=sweep.make_overlay_page(png,pdf,overlay,f'regulatory-p{page}-frame fresh WASM',page-1,pixel_diff_threshold=32)
 sweep.make_review_panels(compares,[overlay],[metrics],base/'review')
 def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
 manifest={'source_sha':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),'method':'selected portable WASM print SVG via CDP, canonical visual_sweep font-policy/raster/compare/overlay/review helpers','pages':[page],'document_page_count':82,'dpi':96,'input_sha256':sha(root/'samples/76076_regulatory_analysis.hwp'),'reference_pdf_sha256':sha(root/'samples/issue1891/76076_regulatory_analysis-2024.pdf'),'wasm_sha256':sha(root/'pkg/rhwp_bg.wasm'),'raw_wasm_svg_sha256':sha(raw),'font_recovery_manifest_sha256':sha(out/'p39-font-supply-additions.json'),'raster_command':cmd,'metrics':[metrics],'pr_review_gate':sweep.pr_review_gate([metrics],expected_pages=[page]),'native_png_sha256':sha(native/f'rhwp_png/rhwp_{num}.png'),'wasm_png_sha256':sha(png)}
 (base/'run_manifest.json').write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+'\n')
 print(json.dumps({'score':metrics.get('tolerant_content_match_percent'),'gate':manifest['pr_review_gate'],'native_wasm_png_identical':manifest['native_png_sha256']==manifest['wasm_png_sha256']},ensure_ascii=False))
