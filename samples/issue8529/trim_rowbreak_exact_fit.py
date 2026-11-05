"""Trim the public regulatory-impact HWPX to its «규제 개요» table (#8529).

Input: 79596_(규제영향분석서) 먹는물관리법 시행규칙 일부개정령(안).hwpx (public
government opinion-download document). Keeps section 0's page properties
(secPr/colPr from the first paragraph) and the two top-level paragraphs
that start a stored page there: the «< 규제 개요 >» heading (vpos 0) and the
31x17 RowBreak table. Every kept element is copied verbatim, so the stored
line segments and the table's declared first-fragment height are Hancom's.

usage: python trim_rowbreak_exact_fit.py <source.hwpx> <out.hwpx> [heading_index]
"""
import copy
import sys
import zipfile

import lxml.etree as E

src, out = sys.argv[1], sys.argv[2]
first = int(sys.argv[3]) if len(sys.argv) > 3 else 571
SECTION = "Contents/section0.xml"

with zipfile.ZipFile(src) as zin:
    names = zin.namelist()
    data = {n: zin.read(n) for n in names}
    infos = {n: zin.getinfo(n) for n in names}

root = E.fromstring(data[SECTION])
paras = [c for c in root if E.QName(c).localname == "p"]
heading = copy.deepcopy(paras[first])
table = copy.deepcopy(paras[first + 1])
assert any(E.QName(x).localname == "tbl" for x in table.iter()), "second kept paragraph must host the table"

# Section and column definitions live in the first run of the first paragraph.
first_run = next(c for c in paras[0] if E.QName(c).localname == "run")
section_defs = [
    copy.deepcopy(c)
    for c in first_run
    if E.QName(c).localname == "secPr"
    or (E.QName(c).localname == "ctrl" and any(E.QName(x).localname == "colPr" for x in c))
]
heading_run = next(c for c in heading if E.QName(c).localname == "run")
for i, element in enumerate(section_defs):
    heading_run.insert(i, element)

for child in list(root):
    root.remove(child)
root.append(heading)
root.append(table)
data[SECTION] = E.tostring(root, xml_declaration=True, encoding="UTF-8", standalone=True)

with zipfile.ZipFile(out, "w") as zout:
    for name in names:
        method = zipfile.ZIP_STORED if name == "mimetype" else zipfile.ZIP_DEFLATED
        zout.writestr(infos[name], data[name], compress_type=method)
