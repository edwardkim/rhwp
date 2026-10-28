from pathlib import Path
import importlib.util, sys, subprocess, os, json, hashlib
root=Path.cwd(); out=root/'output/pr-review/pr7518-20261004'
spec=importlib.util.spec_from_file_location('p39_visual_sweep',root/'scripts/visual_sweep.py'); sweep=importlib.util.module_from_spec(spec); sys.modules[spec.name]=sweep; spec.loader.exec_module(sweep)
base=out/'p39-frame-wasm/regulatory-p39-frame'; native=out/'p39-frame-native/regulatory-p39-frame'
for name in ['svg','rhwp_png','compare','overlay','review']:(base/name).mkdir(parents=True,exist_ok=True)
raw=out/'cdp-p39-frame/wasm_039.svg'; svg=base/'svg/wasm_039.svg'
svg.write_text(sweep.apply_svg_font_policy(raw.read_text(),sweep.svg_font_face_rules(native/'svg/76076_regulatory_analysis_039.svg')))
sweep.check_sweep_embedded_fonts([svg],base/'embedded-font-check.json')
chrome='/home/edward/.cache/puppeteer/chrome/linux-152.0.7977.54/chrome-linux64/chrome'
png=base/'rhwp_png/rhwp_039.png'; pdf=native/'pdf_png/pdf-39.png'
cmd=['node','scripts/rasterize-svg-webfonts.mjs','--input',str(svg),'--output',str(png),'--zoom','1','--chrome',chrome]
subprocess.run(cmd,check=True,env={**os.environ,'VISUAL_SWEEP_CHROME':chrome})
compares=sweep.make_compares([png],[pdf],base/'compare','regulatory-p39-frame fresh WASM')
overlay=base/'overlay/overlay_039.png'
metrics=sweep.make_overlay_page(png,pdf,overlay,'regulatory-p39-frame fresh WASM',38,pixel_diff_threshold=32)
sweep.make_review_panels(compares,[overlay],[metrics],base/'review')
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
manifest={'source_sha':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),'method':'selected portable WASM print SVG via CDP, canonical visual_sweep font-policy/raster/compare/overlay/review helpers','pages':[39],'document_page_count':82,'dpi':96,'input_sha256':sha(root/'samples/76076_regulatory_analysis.hwp'),'reference_pdf_sha256':sha(root/'samples/issue1891/76076_regulatory_analysis-2024.pdf'),'wasm_sha256':sha(root/'pkg/rhwp_bg.wasm'),'raw_wasm_svg_sha256':sha(raw),'font_recovery_manifest_sha256':sha(out/'p39-font-supply-additions.json'),'raster_command':cmd,'metrics':[metrics],'pr_review_gate':sweep.pr_review_gate([metrics],expected_pages=[39]),'native_png_sha256':sha(native/'rhwp_png/rhwp_039.png'),'wasm_png_sha256':sha(png)}
(base/'run_manifest.json').write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+'\n')
print(json.dumps({'score':metrics.get('tolerant_content_match_percent'),'gate':manifest['pr_review_gate'],'native_wasm_png_identical':manifest['native_png_sha256']==manifest['wasm_png_sha256']},ensure_ascii=False))
