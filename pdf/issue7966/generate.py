#!/usr/bin/env python3
"""Generate 33 real paint placements per page via XObject, Form and inline image.

One 2x2 RGB source, independently specified CTMs, half-width clips and alpha.
No font or machine-dependent resource is required. Recreates the checked PDF.
"""
from pathlib import Path

objects = []


def obj(data):
    objects.append(data.encode() if isinstance(data, str) else data)
    return len(objects)


def stream(data, attrs=""):
    return f"<< /Length {len(data)} {attrs} >>\nstream\n".encode() + data + b"\nendstream"


obj("<< /Type /Catalog /Pages 2 0 R >>")
obj("")
image = obj(stream(bytes([255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 0]),
    "/Type /XObject /Subtype /Image /Width 2 /Height 2 /ColorSpace /DeviceRGB /BitsPerComponent 8 /Interpolate false"))
form = obj(stream(b"/Im Do", f"/Type /XObject /Subtype /Form /BBox [0 0 1 1] /Resources << /XObject << /Im {image} 0 R >> >>"))
kids = []
for kind in ["xobject", "form", "inline"]:
    ops = []
    for i in range(33):
        x, y = 8 + (i % 11) * 25, 8 + (i // 11) * 32
        content = f"q 18 0 0 18 {x} {y} cm ".encode()
        if i % 3 == 0:
            content += b"0 0 0.5 1 re W n "
        if i % 3 == 1:
            content += b"/Fade gs "
        if kind == "inline":
            content += b"BI /W 2 /H 2 /CS /RGB /BPC 8 /F /AHx ID ff000000ff000000ffffff00> EI "
        else:
            content += b"/Fm Do " if kind == "form" else b"/Im Do "
        ops.append(content + b"Q\n")
    content = obj(stream(b"".join(ops)))
    kids.append(obj(f"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 290 110] /Resources << /XObject << /Im {image} 0 R /Fm {form} 0 R >> /ExtGState << /Fade << /Type /ExtGState /ca 0.5 >> >> >> /Contents {content} 0 R >>"))
objects[1] = f"<< /Type /Pages /Count 3 /Kids [{' '.join(f'{p} 0 R' for p in kids)}] >>".encode()
body = bytearray(b"%PDF-1.4\n")
offsets = []
for i, data in enumerate(objects, 1):
    offsets.append(len(body))
    body += f"{i} 0 obj\n".encode() + data + b"\nendobj\n"
xref = len(body)
body += f"xref\n0 {len(objects) + 1}\n0000000000 65535 f \n".encode()
for off in offsets:
    body += f"{off:010} 00000 n \n".encode()
body += f"trailer\n<< /Size {len(objects) + 1} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n".encode()
dest = Path(__file__).with_name("image-placement-controls.pdf")
if dest.exists() and dest.read_bytes() != body:
    raise SystemExit("Existing fixture differs; preserve it and review the generation change.")
dest.write_bytes(body)
