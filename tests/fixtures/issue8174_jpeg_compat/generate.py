"""Reproduce the public codec inputs in a new explicit directory."""
from pathlib import Path
import argparse
import json
import subprocess
from PIL import Image

parser = argparse.ArgumentParser()
parser.add_argument("--out", type=Path, required=True)
args = parser.parse_args()
manifest = json.loads((Path(__file__).parent / "manifest.json").read_text())
args.out.mkdir(parents=True, exist_ok=True)
for item in manifest["inputs"]:
    if (args.out / item["name"]).exists():
        raise SystemExit("Choose a new output directory; fixtures are preserved")

def pdf(jpg,w,h,alpha=False):
 objs=[b'<< /Type /Catalog /Pages 2 0 R >>',b'<< /Type /Pages /Kids [3 0 R] /Count 1 >>',b'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 200] /Resources << /XObject << /Im 5 0 R >> >> /Contents 4 0 R >>']
 content=b'q 160 0 0 160 20 20 cm /Im Do Q'
 objs+=[b'<< /Length '+str(len(content)).encode()+b' >>\nstream\n'+content+b'\nendstream']
 extra=b' /SMask 6 0 R'if alpha else b''
 objs+=[f'<< /Type /XObject /Subtype /Image /Width {w} /Height {h} /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /DCTDecode /Length {len(jpg)}'.encode()+extra+b' >>\nstream\n'+jpg+b'\nendstream']
 if alpha:
  mask=bytes([128])*w*h;objs+=[f'<< /Type /XObject /Subtype /Image /Width {w} /Height {h} /ColorSpace /DeviceGray /BitsPerComponent 8 /Length {len(mask)} >>\nstream\n'.encode()+mask+b'\nendstream']
 data=bytearray(b'%PDF-1.4\n');offset=[0]
 for n,b in enumerate(objs,1):offset.append(len(data));data+=f'{n} 0 obj\n'.encode()+b+b'\nendobj\n'
 x=len(data);data+=f'xref\n0 {len(objs)+1}\n0000000000 65535 f \n'.encode()
 for v in offset[1:]:data+=f'{v:010d} 00000 n \n'.encode()
 data+=f'trailer\n<< /Root 1 0 R /Size {len(objs)+1} >>\nstartxref\n{x}\n%%EOF\n'.encode();return data

for item in manifest["inputs"]:
    w, h = item["width"], item["height"]
    image = Image.new("RGB", (w, h))
    image.putdata([((31*x) % 256, (19*y) % 256, (23*(x+y)) % 256)
                   for y in range(h) for x in range(w)])
    dest = args.out / item["name"]
    if "sampling" in item:
        ppm = dest.with_suffix(".ppm")
        image.save(ppm)
        cmd = ["cjpeg", "-quality", "90", "-sample", "1x2,2x2,2x2"]
        if item["progressive"]:
            cmd.append("-progressive")
        subprocess.run([*cmd, "-outfile", str(dest), str(ppm)], check=True)
        for alpha in ([False, True] if item["progressive"] and w == 17 else [False]):
            name = dest.stem + ("-alpha" if alpha else "") + ".pdf"
            (args.out / name).write_bytes(pdf(dest.read_bytes(), w, h, alpha))
    else:
        image.save(dest, quality=90, subsampling=item["subsampling"],
                   progressive=item["progressive"])
print("Generated", len(manifest["inputs"]), "JPEGs and 17 PDF sources")
