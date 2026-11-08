#!/usr/bin/env python3
"""Generate text-before-opaque-path/image controls, including repeated nested Forms."""
from pathlib import Path

objects = []


def obj(data):
    objects.append(data.encode() if isinstance(data, str) else data)
    return len(objects)


def stream(data, attrs=""):
    return f"<< /Length {len(data)} {attrs} >>\nstream\n".encode() + data + b"\nendstream"


obj("<< /Type /Catalog /Pages 2 0 R >>")
obj("")
font=obj("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")
image=obj(stream(bytes([0,128,0]), "/Type /XObject /Subtype /Image /Width 1 /Height 1 /ColorSpace /DeviceRGB /BitsPerComponent 8"))
# Opaque later marks cover text. The text must remain in IR, behind its actual cover.
ops=b"BT /F1 18 Tf 20 25 Td (FIRST) Tj ET 1 0 0 rg 10 10 100 45 re f BT /F1 18 Tf 140 25 Td (SECOND) Tj ET q 100 0 0 45 130 10 cm /Im Do Q"
resources=f"<< /Font << /F1 {font} 0 R >> /XObject << /Im {image} 0 R >> >>"
inner=obj(stream(ops,f"/Type /XObject /Subtype /Form /BBox [0 0 240 65] /Resources {resources}"))
outer=obj(stream(b"/Inner Do",f"/Type /XObject /Subtype /Form /BBox [0 0 240 65] /Resources << /XObject << /Inner {inner} 0 R >> >>"))
kids=[]
for content,height in [(ops,65),(b"/Outer Do",65),(b"/Outer Do q 1 0 0 1 0 65 cm /Outer Do Q",130)]:
    content=obj(stream(content))
    kids.append(obj(f"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 240 {height}] /Resources << /Font << /F1 {font} 0 R >> /XObject << /Im {image} 0 R /Outer {outer} 0 R >> >> /Contents {content} 0 R >>"))
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
dest = Path(__file__).with_name("text-paint-order-controls.pdf")
if dest.exists() and dest.read_bytes() != body:
    raise SystemExit("Existing fixture differs; preserve it and review the generation change.")
dest.write_bytes(body)
