"""Deterministic public PDF pen/structure fixtures; no rhwp-derived geometry."""
from pathlib import Path
import re

HERE = Path(__file__).resolve().parent
source = (HERE.parent / "compound-borders/compound-solid.pdf").read_bytes()
ops = re.search(rb"4 0 obj\n.*?stream\n(.*?)\nendstream", source, re.S).group(1)
base = ops.replace(b"1 0 0 1 20 12 Tm", b"1 0 0 1 20 -300 Tm")
blue = base.replace(b"q 0 G", b"q 0.2 0.4 0.6 RG").replace(b"0.5 w", b"0.51 w")
mixed = base.replace(b"20 330 m 380 330 l\n20 30 m", b"20 330 m 380 330 l\nS 0.7 w\n20 30 m")
mixed = mixed.replace(b"20 180.0 m", b"S 0.5 w\n20 180.0 m")

nonuniform = mixed.replace(b"S 0.7 w\n20 30 m 20 330 l", b"S 0.9 w\n20 30 m 20 330 l S 0.7 w")

horizontal = mixed.replace(b"20 30 m 380 30 l\n20 330 m 380 330 l", b"20 30 m 380 30 l S 0.9 w 20 330 m 380 330 l")
interval = nonuniform.replace(b"20 30 m 20 330 l S 0.7 w", b"20 30 m 20 180 l S 0.7 w 20 180 m 20 330 l S 0.7 w")

h_interval = horizontal.replace(b"20 330 m 380 330 l", b"20 330 m 200 330 l S 0.5 w 200 330 m 380 330 l")
v_shared = mixed.replace(b"200.0 30 m 200.0 330 l", b"200.0 30 m 200.0 180 l S 0.9 w 200.0 180 m 200.0 330 l S 0.7 w")
h_shared = mixed.replace(b"20 180.0 m 380 180.0 l", b"20 180.0 m 200 180.0 l S 0.9 w 200 180.0 m 380 180.0 l S 0.5 w")
v_partial = nonuniform.replace(b"20 30 m 20 330 l S 0.7 w", b"20 30 m 20 130 l S 0.7 w 20 130 m 20 330 l S 0.7 w")
h_partial = horizontal.replace(b"20 330 m 380 330 l", b"20 330 m 130 330 l S 0.5 w 130 330 m 380 330 l")

for name, content in [("horizontal-interval", h_interval), ("shared-vertical-interval", v_shared), ("shared-horizontal-interval", h_shared), ("within-cell-vertical", v_partial), ("within-cell-horizontal", h_partial), ("nonuniform-horizontal", horizontal), ("nonuniform-interval", interval), ("retained-pen", base), ("odd-blue", blue), ("mixed-pen", mixed), ("nonuniform-axis", nonuniform)]:
    body = b"q 1 0 0 1 0 360 cm\n" + content + b"\nQ"
    font = b"Arial" if name == "odd-blue" else b"Helvetica"
    objects = [
        b"<< /Type /Catalog /Pages 2 0 R >>",
        b"<< /Type /Pages /Count 1 /Kids [3 0 R] >>",
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 600 720] /Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>",
        f"<< /Length {len(body)} >>\nstream\n".encode() + body + b"\nendstream",
        b"<< /Type /Font /Subtype /Type1 /BaseFont /" + font + b" >>",
    ]
    data = bytearray(b"%PDF-1.4\n")
    offsets = []
    for i, obj in enumerate(objects, 1):
        offsets.append(len(data))
        data.extend(f"{i} 0 obj\n".encode() + obj + b"\nendobj\n")
    xref = len(data)
    data.extend(f"xref\n0 {len(objects)+1}\n0000000000 65535 f \n".encode())
    for offset in offsets:
        data.extend(f"{offset:010} 00000 n \n".encode())
    data.extend(f"trailer\n<< /Size {len(objects)+1} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n".encode())
    (HERE / f"{name}.pdf").write_bytes(data)
