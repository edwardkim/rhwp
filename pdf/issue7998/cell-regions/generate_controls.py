"""Build PDF graphics-state controls from the three independent Hancom Prints."""
import argparse
import hashlib
import json
from decimal import Decimal
from pathlib import Path

import pikepdf

parser = argparse.ArgumentParser()
parser.add_argument("--print-dir", type=Path, required=True)
parser.add_argument("--output", type=Path, required=True)
options = parser.parse_args()
out = options.output
out.mkdir(parents=True, exist_ok=True)
records = []
for name in ("colspan", "rowspan", "hidden-separate"):
    source = options.print_dir / f"{name}.pdf"
    (out / source.name).write_bytes(source.read_bytes())
    records.append({"key": name, "origin": str(source)})

controls = (
    "restored-clip", "missing-clip", "partial-clip", "nonrectangular-clip",
    "form-transformed", "competing-scopes", "painted-separator-conflict",
    "unwitnessed-cell", "form-shadowed-font",
)
for key in controls:
    pdf = pikepdf.open(options.print_dir / "colspan.pdf")
    page = pdf.pages[0]
    ops = pikepdf.parse_content_stream(page)
    # These ordinals belong to this fixed independent Print, never the importer.
    # Fail if another producer's stream is supplied instead of guessing its scope.
    for index, operator in ((15, "W*"), (16, "n"), (24, "TJ"), (62, "TJ"), (72, "TJ")):
        assert str(ops[index].operator) == operator, (index, operator)

    def replace_operand(index, operand, value):
        operands = list(ops[index].operands)
        operands[operand] = Decimal(value)
        ops[index] = pikepdf.ContentStreamInstruction(operands, ops[index].operator)

    if key == "restored-clip":
        # The rectangle still exists in observations but is restored BEFORE text.
        ops[17:17] = [
            pikepdf.ContentStreamInstruction([], pikepdf.Operator("Q")),
            pikepdf.ContentStreamInstruction([], pikepdf.Operator("q")),
        ]
    elif key == "missing-clip":
        del ops[15]
    elif key == "partial-clip":
        for index in (10, 13):
            replace_operand(index, 0, "3.873")
    elif key == "nonrectangular-clip":
        # A bevel in an empty corner preserves all visible text.
        replace_operand(12, 1, "345.4")
        ops.insert(13, pikepdf.ContentStreamInstruction(
            [Decimal("355.263"), Decimal("344.4")], pikepdf.Operator("l")))
    elif key == "competing-scopes":
        extra = pikepdf.parse_content_stream(pikepdf.Stream(
            pdf, b"2.873 344.4 176.695 12.72 re W n"))
        ops[28:28] = extra
    elif key == "painted-separator-conflict":
        for index in (12, 13):
            replace_operand(index, 1, "331.56")
    elif key == "unwitnessed-cell":
        ops = [op for index, op in enumerate(ops) if index not in (62, 72)]

    if key in ("form-transformed", "form-shadowed-font"):
        form = pikepdf.Stream(pdf, pikepdf.unparse_content_stream(ops))
        form.Type = pikepdf.Name.XObject
        form.Subtype = pikepdf.Name.Form
        form.BBox = page.MediaBox
        form.Resources = page.Resources
        page.Resources = pikepdf.Dictionary(XObject=pikepdf.Dictionary(Fm=form))
        page.Contents = pikepdf.Stream(pdf, b"q 1 0 0 1 15 -35 cm /Fm Do Q")
        if key == "form-shadowed-font":
            page.Resources.Font = pikepdf.Dictionary(F1=pikepdf.Dictionary(
                Type=pikepdf.Name.Font, Subtype=pikepdf.Name.Type1,
                BaseFont=pikepdf.Name("/Times-Roman")))
    else:
        page.Contents = pikepdf.Stream(pdf, pikepdf.unparse_content_stream(ops))
    pdf.save(out / f"{key}.pdf")
    records.append({"key": key, "origin": "independent colspan Print; explicit PDF scope control"})

for record in records:
    data = (out / (record["key"] + ".pdf")).read_bytes()
    record.update(bytes=len(data), sha256=hashlib.sha256(data).hexdigest())
(out / "inputs.json").write_text(json.dumps(records, indent=2) + "\n")
print([(record["key"], record["bytes"]) for record in records])
