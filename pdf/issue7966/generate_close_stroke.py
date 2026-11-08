#!/usr/bin/env python3
"""Generate independent s / h S / open S controls at page and nested Form scopes."""
from pathlib import Path

objects = []


def obj(data):
    objects.append(data.encode() if isinstance(data, str) else data)
    return len(objects)


def stream(data, attrs=""):
    return f"<< /Length {len(data)} {attrs} >>\nstream\n".encode() + data + b"\nendstream"


obj("<< /Type /Catalog /Pages 2 0 R >>")
obj("")
# s is exactly h S; the open S control must remain open. ISO 32000-1 Table 60.
# All three are painted before a filled square, so path consumption/order matters.
body_ops = b"2 w 0.8 0.05 0.15 RG 12 16 m 42 68 l 72 16 l s 0.1 0.3 0.8 RG 90 16 m 120 68 l 150 16 l h S 0 0.5 0 RG 168 16 m 198 68 l 228 16 l S 0.4 0.4 0.4 rg 240 16 16 16 re f"
inner = obj(stream(body_ops, "/Type /XObject /Subtype /Form /BBox [0 0 280 80] /Resources << >>"))
outer = obj(stream(f"/Inner Do".encode(), f"/Type /XObject /Subtype /Form /BBox [0 0 280 80] /Resources << /XObject << /Inner {inner} 0 R >> >>"))
kids=[]
for use_form in [False, True]:
    content=obj(stream(b"/Outer Do" if use_form else body_ops))
    kids.append(obj(f"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 280 80] /Resources << /XObject << /Outer {outer} 0 R >> >> /Contents {content} 0 R >>"))
objects[1] = f"<< /Type /Pages /Count 2 /Kids [{' '.join(f'{p} 0 R' for p in kids)}] >>".encode()
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
dest = Path(__file__).with_name("close-stroke-controls.pdf")
if dest.exists() and dest.read_bytes() != body:
    raise SystemExit("Existing fixture differs; preserve it and review the generation change.")
dest.write_bytes(body)
