#!/usr/bin/env python3
"""Independent #8174 PDF feature inputs, not RHWP-rendered goldens.

Run from the repository root with venv/bin/python. Requires Pillow/LittleCMS,
OpenJPEG opj_compress and Ghostscript. Re-run both product visual gates after
regeneration. PDF reference metadata/timestamps can change between runs.
"""
from pathlib import Path
import hashlib
import importlib.util
import io
import json
import struct
import subprocess
import tempfile
import zlib
from PIL import Image, ImageCms

ROOT = Path(__file__).resolve().parents[2]
PDF = ROOT / "pdf/issue8174/features"
spec = importlib.util.spec_from_file_location("codec_inputs", ROOT / "tools/fixture_generators/pdf_codecs_issue8183.py")
g = importlib.util.module_from_spec(spec)
spec.loader.exec_module(g)


def objects_pdf(objects, trailer=b"", xref=True):
    data = b"%PDF-1.7\n"
    offsets = [0]
    for index, obj in enumerate(objects, 1):
        offsets.append(len(data))
        data += f"{index} 0 obj\n".encode() + obj + b"\nendobj\n"
    if not xref:
        return data + b"%%EOF\n"
    at = len(data)
    data += f"xref\n0 {len(offsets)}\n0000000000 65535 f \n".encode()
    data += b"".join(f"{o:010} 00000 n \n".encode() for o in offsets[1:])
    return data + f"trailer\n<< /Size {len(offsets)} /Root 1 0 R ".encode() + trailer + f">>\nstartxref\n{at}\n%%EOF\n".encode()


def normalize(source, destination):
    subprocess.run(["gs", "-q", "-dSAFER", "-dBATCH", "-dNOPAUSE", "-sDEVICE=pdfwrite",
                    "-dCompatibilityLevel=1.4", "-dAutoFilterColorImages=false",
                    "-dColorImageFilter=/FlateEncode", "-dPassThroughJPEGImages=false",
                    f"-sOutputFile={destination}", str(source)], check=True)


def main():
    PDF.mkdir(parents=True, exist_ok=True)
    (PDF / "reference").mkdir(exist_ok=True)
    g.PDF = PDF
    profiles = {
        "rgb": ROOT / "vendor/rhwp-hayro-jpeg2000/assets/ProPhoto-v2-micro.icc",
        "cmyk": ROOT / "vendor/rhwp-hayro-jpeg2000/assets/CGATS001Compat-v2-micro.icc",
        "gray": ROOT / "vendor/svg2pdf/src/icc/sGrey-v4.icc",
    }
    colours = [(128, 64, 32), (32, 128, 64), (64, 32, 128), (192, 192, 192)] * 4
    for kind, mode, samples in [
        ("rgb", "RGB", colours),
        ("gray", "L", [32, 96, 160, 224] * 4),
        ("cmyk", "CMYK", [(0, 255, 255, 0), (255, 0, 255, 0), (255, 255, 0, 0), (0, 0, 0, 128)] * 4),
    ]:
        image = Image.new(mode, (4, 4))
        image.putdata(samples)
        n = len(image.getbands())
        profile = profiles[kind].read_bytes()
        extras = [g.stream(f"/N {n}", profile)]
        g.write_pdf("icc-" + kind, image.tobytes(), 4, 4, "/ColorSpace [/ICCBased 6 0 R] /BitsPerComponent 8", extras, size=(160, 160))
        if kind != "gray":
            jpeg = io.BytesIO()
            image.save(jpeg, format="JPEG", **({"quality": 90, "subsampling": 0} if kind == "rgb" else {"quality": 100}))
            name = "icc-jpeg" if kind == "rgb" else "icc-cmyk-jpeg"
            g.write_pdf(name, jpeg.getvalue(), 4, 4, "/Filter /DCTDecode /ColorSpace [/ICCBased 6 0 R] /BitsPerComponent 8", extras, size=(160, 160))
        if kind == "rgb":
            g.write_pdf("icc-indirect-filter", image.tobytes(), 4, 4,
                        "/ColorSpace [/ICCBased 6 0 R] /BitsPerComponent 8",
                        [g.stream("/N 3 /Filter 7 0 R", zlib.compress(profile)), b"/FlateDecode"], size=(160, 160))
            g.write_pdf("icc-indexed", bytes([0, 1, 2, 3] * 4), 4, 4,
                        "/ColorSpace [/Indexed [/ICCBased 6 0 R] 3 <" + image.tobytes()[:12].hex() + ">] /BitsPerComponent 8", extras, size=(160, 160))
            ImageCms.profileToProfile(image, ImageCms.ImageCmsProfile(io.BytesIO(profile)),
                                      ImageCms.createProfile("sRGB"), renderingIntent=1).save(PDF / "reference/icc-lcms-expected.png")
    # Raw J2K and boxed JP2 carry the same independent encoder samples.
    palette = [(0, 0, 0)] * 256
    for i, rgb in [(0, (0, 0, 255)), (64, (0, 255, 0)), (128, (255, 0, 0))]:
        palette[i] = rgb
    contracts = [
        ("jpx-indexed-alpha", [(0, 32), (1, 128), (2, 192), (3, 255)] * 4,
         [(255, 0, 0), (0, 255, 0), (0, 0, 255), (0, 255, 255)], 1),
        ("jpx-indexed-premult", [(64, 128), (128, 255), (0, 64), (0, 0)] * 4, palette, 2),
        ("jpx-indexed-palette-premult", [(0, 128), (1, 255), (2, 64), (3, 0)] * 4,
         [(128, 0, 0), (255, 0, 0), (0, 0, 64), (0, 0, 0)], 2),
        ("jpx-indexed-icc-premult", [(0, 128), (1, 255), (2, 64), (3, 0)] * 4,
         [(64, 32, 16), (32, 128, 64), (16, 8, 32), (0, 0, 0)], 2),
        ("jpx-cmyk-alpha", [(0, 255, 255, 0, 32), (255, 0, 255, 0, 128), (255, 255, 0, 0, 192), (0, 0, 0, 128, 255)] * 4, None, 1),
        ("jpx-cmyk-premult", [(0, 128, 128, 0, 128), (255, 0, 255, 0, 255), (64, 64, 0, 0, 64), (0, 0, 0, 0, 0)] * 4, None, 2),
    ]
    for name, samples, lookup, opacity in contracts:
        n = len(samples[0])
        base_space = "[/ICCBased 6 0 R]" if name == "jpx-indexed-icc-premult" else "/DeviceRGB"
        colour_space = (f"[/Indexed {base_space} {len(lookup)-1} <{bytes(c for rgb in lookup for c in rgb).hex()}>]" if lookup else "/DeviceCMYK")
        with tempfile.TemporaryDirectory() as temp:
            raw, encoded = Path(temp) / "samples.raw", Path(temp) / "samples.j2k"
            raw.write_bytes(bytes(p[c] for c in range(n) for p in samples))
            subprocess.run(["opj_compress", "-i", str(raw), "-o", str(encoded), "-F", f"4,4,{n},8,u", "-n", "1", "-mct", "0"], check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            j2k = encoded.read_bytes()
        fields = f"/Filter /JPXDecode /ColorSpace {colour_space} /SMaskInData {opacity}"
        extras = [g.stream("/N 3", profiles["rgb"].read_bytes())] if name == "jpx-indexed-icc-premult" else []
        g.write_pdf(name, j2k, 4, 4, fields, extras, size=(160, 160))
        header = g.box(b"ihdr", struct.pack(">IIHBBBB", 4, 4, n, 7, 7, 0, 0))
        header += g.box(b"colr", struct.pack(">BBBI", 1, 0, 0, 17 if lookup else 12))
        header += g.box(b"cdef", struct.pack(">H", n) + b"".join(struct.pack(">HHH", i, 1 if i == n-1 else 0, 0 if i == n-1 else i+1) for i in range(n)))
        jp2 = g.box(b"jP  ", b"\r\n\x87\n") + g.box(b"ftyp", b"jp2 " + struct.pack(">I", 0) + b"jp2 ") + g.box(b"jp2h", header) + g.box(b"jp2c", j2k)
        g.write_pdf(name + "-boxed", jp2, 4, 4, fields, extras, size=(160, 160))
        # Independent reference: reconstruct straight samples from ENCODER input.
        # ISO 32000-1 11.6.5.3: invert palette COLOURS, never indices.
        colours = [lookup[p[0]] if lookup else p[:-1] for p in samples]
        straight = [tuple((min(255, round(c*255/p[-1])) if p[-1] else 0) if opacity == 2 else c for c in colour) for p, colour in zip(samples, colours)]
        expanded = bytes(c for colour in straight for c in colour)
        mask = g.stream("/Type /XObject /Subtype /Image /Width 4 /Height 4 /ColorSpace /DeviceGray /BitsPerComponent 8", bytes(p[-1] for p in samples))
        g.PDF = PDF / "reference"
        reference_space = base_space if lookup else "/DeviceCMYK"
        reference_extras = extras + [mask]
        mask_id = 6 + len(extras)
        g.write_pdf(name + "-source", expanded, 4, 4, f"/ColorSpace {reference_space} /BitsPerComponent 8 /SMask {mask_id} 0 R", reference_extras, size=(160, 160))
        normalize(g.PDF / (name + "-source.pdf"), g.PDF / (name + "-independent.pdf"))
        g.PDF = PDF
    base = [b"<< /Type /Catalog /Pages 2 0 R >>", b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
            b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 160 160] /Resources << >> /Contents 4 0 R >>",
            g.stream("", b"% 999 0 obj << /Type /Catalog >> endobj\n1 0 0 rg 0 0 160 160 re f"),
            b"<< /Title (998 0 obj << /Type /Catalog >> endobj) >>"]
    (PDF / "recovery-fake-header.pdf").write_bytes(objects_pdf(base, xref=False))
    (PDF / "reference/recovery-fake-header-original.pdf").write_bytes(objects_pdf(base, b"/Info 5 0 R "))
    updated = objects_pdf(base, b"/Info 5 0 R ") + b"4 0 obj\n" + g.stream("", b"0 0 1 rg 0 0 160 160 re f") + b"\nendobj\ntrailer\n<< /Size 6 /Root 1 0 R /Info 5 0 R >>\nstartxref\n0\n%%EOF\n"
    (PDF / "recovery-incremental.pdf").write_bytes(updated)
    base[3] = g.stream("", b"0 0 1 rg 0 0 160 160 re f")
    (PDF / "reference/recovery-incremental-original.pdf").write_bytes(objects_pdf(base, b"/Info 5 0 R "))
    original = (ROOT / "pdf/issue8183/jpx-indexed.pdf").read_bytes()
    (PDF / "recovery-missing-xref.pdf").write_bytes(original[:original.rfind(b"\nxref\n")] + b"\n%%EOF\n")
    (PDF / "reference/recovery-original.pdf").write_bytes(original)
    for kind, n, values in [("rgb", 3, b"0.5 0.25 0.125"), ("cmyk", 4, b"0.2 0.4 0.1 0.3")]:
        objects = [base[0], base[1], b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 160 160] /Resources << /ColorSpace << /C1 [/ICCBased 5 0 R] >> >> /Contents 4 0 R >>",
                   g.stream("", b"/C1 cs " + values + b" scn 0 0 160 160 re f"), g.stream(f"/N {n}", profiles[kind].read_bytes())]
        (PDF / ("icc-path-" + kind + ".pdf")).write_bytes(objects_pdf(objects))
    for f in sorted(PDF.glob("icc-*.pdf")):
        normalize(f, PDF / "reference" / (f.stem + "-gs.pdf"))
    manifest = {str(f.relative_to(PDF)): hashlib.sha256(f.read_bytes()).hexdigest() for f in sorted(PDF.rglob("*")) if f.is_file() and f.suffix in (".pdf", ".png")}
    (PDF / "provenance.json").write_text(json.dumps({"source": "independent writer/Pillow/LittleCMS/OpenJPEG; no RHWP output", "profiles": {k: {"path": str(v.relative_to(ROOT)), "sha256": hashlib.sha256(v.read_bytes()).hexdigest()} for k, v in profiles.items()}, "files": manifest}, indent=2) + "\n")


if __name__ == "__main__":
    main()
