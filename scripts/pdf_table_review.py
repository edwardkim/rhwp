#!/usr/bin/env python3
"""Inspect product PDF table diagnostics; rasterisation is for review only."""
import argparse
import hashlib
import html
import json
from pathlib import Path
import subprocess


def esc(value):
    return html.escape(str(value), quote=True)


def rect(bounds, css, title):
    if not bounds:
        return ""
    return (f'<rect class="{css}" x="{bounds["x"]}" y="{bounds["y"]}" '
            f'width="{bounds["width"]}" height="{bounds["height"]}">'
            f'<title>{esc(title)}</title></rect>')


def overlay(page, image_name):
    ledger = page["tableInference"]
    left, bottom, right, top = page["mediaBox"]
    parts = [f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {right-left} {top-bottom}">',
             f'<image href="{esc(image_name)}" width="{right-left}" height="{top-bottom}"/>']
    parts += [rect(g["bounds"], "glyph", f'{g["id"]}: {g["text"]}; {g["geometryQuality"]}')
              for g in ledger["glyphs"]]
    parts += [rect(b["bounds"], "block", b["id"]) for b in ledger["initialBlocks"]]
    parts += [rect(r["bounds"], "region", f'{r["id"]}: clip, not paint')
              for r in ledger["regions"]]
    raw = ledger["tableDetection"].get("boundaryObservations", [])
    for edge in raw:
        horizontal = edge["horizontal"]
        x1, y1 = (edge["from"], edge["at"]) if horizontal else (edge["at"], edge["from"])
        x2, y2 = (edge["to"], edge["at"]) if horizontal else (edge["at"], edge["to"])
        parts.append(f'<line class="boundary" x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}">'
                     f'<title>{esc(edge["id"])}: {esc(edge["paintKind"])}</title></line>')
    for i, candidate in enumerate(ledger["candidates"]):
        bounds = candidate["bounds"]
        parts.append(f'<g class="grid" data-candidate="{i}">')
        parts.append(rect(bounds, "frame", f'{candidate["id"]}: {candidate["generator"]}'))
        for x in candidate["xBoundaries"]:
            parts.append(f'<line x1="{x}" x2="{x}" y1="{bounds["y"]}" y2="{bounds["y"]+bounds["height"]}"/>')
        for y in candidate["yBoundaries"]:
            parts.append(f'<line y1="{y}" y2="{y}" x1="{bounds["x"]}" x2="{bounds["x"]+bounds["width"]}"/>')
        parts.append("</g>")
    parts.append("</svg>")
    return "\n".join(parts)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--diagnostics", type=Path, required=True)
    parser.add_argument("--reference-pdf", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    args = parser.parse_args()
    data = json.loads(args.diagnostics.read_text(encoding="utf-8"))
    metadata = data.get("diagnostics", data)
    pages = metadata["pages"]
    if any("glyphs" not in page["tableInference"] for page in pages):
        parser.error("상세 진단이 필요합니다: import-pdf ... --json --table-inference")
    args.output_dir.mkdir(parents=True, exist_ok=True)
    provenance = {"referencePdf": str(args.reference_pdf.resolve()),
                  "referenceSha256": hashlib.sha256(args.reference_pdf.read_bytes()).hexdigest(),
                  "diagnosticsSha256": hashlib.sha256(args.diagnostics.read_bytes()).hexdigest(),
                  "scope": "source observation and hypotheses; not a Native/WASM fidelity verdict"}
    (args.output_dir / "source.json").write_text(json.dumps(provenance, ensure_ascii=False, indent=2)+"\n")
    sections = []
    # Raw source text stays in this local artifact; escape it before HTML embedding.
    for i, page in enumerate(pages):
        number = page["sourcePage"]+1
        prefix = args.output_dir / f"source-{number}"
        subprocess.run(["pdftoppm", "-f", str(number), "-l", str(number),
                        "-singlefile", "-r", "96", "-png", str(args.reference_pdf), str(prefix)],
                       check=True, stdout=subprocess.DEVNULL)
        ledger = page["tableInference"]
        options = '<option value="-1">후보 숨기기</option>'+"".join(
            f'<option value="{j}">{esc(c["id"])} · {esc(c["generator"])} · '
            f'{len(c["yBoundaries"])-1}×{len(c["xBoundaries"])-1}</option>'
            for j, c in enumerate(ledger["candidates"]))
        details = esc(json.dumps({k: ledger.get(k) for k in (
            "status", "incompleteReasons", "detectionDeferredReason", "rejectedComponents",
            "loweringDeferred", "compositions", "candidates")}, ensure_ascii=False, indent=2))
        sections.append(f'<section><h2>원본 {number}쪽</h2><p>문자 {ledger["glyphCount"]} · '
                        f'T0 {ledger["initialBlockCount"]} · 클립 {ledger["clipCount"]} · '
                        f'후보 {ledger["candidateCount"]} · 의미 표 {ledger["semanticTables"]}</p>'
                        f'<label>격자 후보 <select>{options}</select></label>'
                        f'<div class="sheet">{overlay(page, prefix.name+".png")}</div>'
                        f'<details><summary>보류 사유·내용 소속·경쟁 가설</summary><pre>{details}</pre></details></section>')
    output = '''<!doctype html><html lang="ko"><meta charset="utf-8">
<title>PDF 표 추론 관측·가설 검토</title><style>
body{font-family:system-ui,sans-serif;margin:24px;background:#f5f7fa;color:#18212c}
nav{position:sticky;top:0;background:#fff;padding:12px;z-index:2;border:1px solid #ccd}
label{margin-right:16px}section{background:white;padding:16px;margin-top:20px}
.sheet{max-width:1000px;border:1px solid #aaa}svg{display:block;width:100%}
rect,line{fill:none;vector-effect:non-scaling-stroke;pointer-events:stroke}
.glyph{stroke:#41b3ba;stroke-width:.4;display:none}.block{stroke:#00897b;stroke-width:1;display:none}
.region{stroke:#7957c1;stroke-width:1;stroke-dasharray:3 3;display:none}
.boundary{stroke:#ed4141;stroke-width:1;display:none}.grid{stroke:#dd8700;stroke-width:1;stroke-dasharray:5 3;display:none}
.show-glyph .glyph,.show-block .block,.show-region .region,.show-boundary .boundary{display:block}
pre{white-space:pre-wrap;overflow-wrap:anywhere;max-height:600px;overflow:auto}
</style><body><h1>PDF 표 추론: 관측과 구조 가설</h1>
<p>격자는 작성 당시 셀 구조의 정답이 아닙니다. 배경은 원본 PDF이며 Native/WASM 출력의 시각 gate와 구분합니다.
원본 문자가 포함된 로컬 검토 자료입니다.</p><nav>
<label><input type="checkbox" data-layer="glyph">문자 상자</label>
<label><input type="checkbox" data-layer="block">초기 내용 T0</label>
<label><input type="checkbox" data-layer="region">클립 S</label>
<label><input type="checkbox" data-layer="boundary">원본 stroke·fill 띠</label>
</nav>'''+"\n".join(sections)+'''
<script>
document.querySelectorAll('[data-layer]').forEach(box=>box.addEventListener('change',()=>
 document.body.classList.toggle('show-'+box.dataset.layer,box.checked)));
document.querySelectorAll('section').forEach(section=>{
 const select=section.querySelector('select');
 const update=()=>section.querySelectorAll('[data-candidate]').forEach(grid=>
  grid.style.display=grid.dataset.candidate===select.value?'block':'none');
 select.addEventListener('change',update);
});
</script></body></html>'''
    (args.output_dir / "index.html").write_text(output, encoding="utf-8")
    print(args.output_dir / "index.html")


if __name__ == "__main__":
    main()
