#!/usr/bin/env python3
"""쪽 맨 위 글자처럼 취급 표 host 의 문단 위 간격 — 자연 쪽 넘김·큰 표·작은 표.

  python3 gen_top_tac_cases.py samples/issue1549_multipositive_float_tables.hwpx out.hwpx

N: 채움 40줄 뒤 자연 쪽 넘김으로 쪽 맨 위에 오는 위 간격 1000HU(원본 2000) 표(1x1, 3000HU).
T: 새 구역 첫 문단이 위 간격 1200HU(원본 2400)의 큰 표(1x1, 60000HU).
U: 새 구역 첫 문단이 위 간격 1200HU 의 작은 표(1x1, 3000HU).
모든 문단에 linesegarray 가 없다(한/글이 직접 조판).
"""
import re
import sys
import zipfile

_id = [1000]
PREV = {21: 1000, 22: 2000, 23: 3000, 24: 2400}


def nid():
    _id[0] += 1
    return _id[0]


def run_text(text):
    return f'<hp:run charPrIDRef="0"><hp:t>{text}</hp:t></hp:run>'


def para(text, pr=0, page_break=False, lead=""):
    return (f'<hp:p id="{nid()}" paraPrIDRef="{pr}" styleIDRef="0" pageBreak="{1 if page_break else 0}" '
            f'columnBreak="0" merged="0">{lead}{run_text(text)}</hp:p>')


def tac_table(label, height=3000):
    cell = (f'<hp:tc name="" header="0" hasMargin="0" protect="0" editable="0" dirty="0" borderFillIDRef="3">'
            f'<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="CENTER" linkListIDRef="0" '
            f'linkListNextIDRef="0" textWidth="0" textHeight="0" hasTextRef="0" hasNumRef="0">'
            f'{para(label)}</hp:subList><hp:cellAddr colAddr="0" rowAddr="0"/>'
            f'<hp:cellSpan colSpan="1" rowSpan="1"/><hp:cellSz width="30000" height="{height}"/>'
            f'<hp:cellMargin left="141" right="141" top="141" bottom="141"/></hp:tc>')
    return (f'<hp:tbl id="{nid()}" zOrder="0" numberingType="TABLE" textWrap="TOP_AND_BOTTOM" textFlow="BOTH_SIDES" '
            f'lock="0" dropcapstyle="None" pageBreak="NONE" repeatHeader="0" rowCnt="1" colCnt="1" '
            f'cellSpacing="0" borderFillIDRef="3" noAdjust="0"><hp:sz width="30000" widthRelTo="ABSOLUTE" '
            f'height="{height}" heightRelTo="ABSOLUTE" protect="0"/><hp:pos treatAsChar="1" affectLSpacing="0" '
            f'flowWithText="1" allowOverlap="0" holdAnchorAndSO="0" vertRelTo="PARA" horzRelTo="COLUMN" '
            f'vertAlign="TOP" horzAlign="LEFT" vertOffset="0" horzOffset="0"/>'
            f'<hp:outMargin left="140" right="140" top="140" bottom="140"/>'
            f'<hp:inMargin left="141" right="141" top="141" bottom="141"/><hp:tr>{cell}</hp:tr></hp:tbl>')


def header_with_spacing(h):
    base = re.search(r'<hh:paraPr id="0".*?</hh:paraPr>', h, re.S).group(0)
    extra = ""
    for pid, prev in PREV.items():
        p = base.replace('<hh:paraPr id="0"', f'<hh:paraPr id="{pid}"', 1)
        p = p.replace('<hc:prev value="0" unit="HWPUNIT"/>', f'<hc:prev value="{prev}" unit="HWPUNIT"/>')
        extra += p
    count = int(re.search(r'<hh:paraProperties itemCnt="(\d+)">', h).group(1))
    h = h.replace(f'<hh:paraProperties itemCnt="{count}">',
                  f'<hh:paraProperties itemCnt="{count + len(PREV)}">', 1)
    h = h.replace("</hh:paraProperties>", extra + "</hh:paraProperties>", 1)
    return h


def main():
    template, out = sys.argv[1], sys.argv[2]
    zin = zipfile.ZipFile(template)
    files = [(i.filename, zin.read(i.filename)) for i in zin.infolist()]
    data = dict(files)
    sec = data["Contents/section0.xml"].decode("utf-8")
    sec_head = sec[:sec.index("<hp:p ")]
    sec_run_inner = re.search(r'<hp:p [^>]*><hp:run charPrIDRef="0">(<hp:secPr.*?</hp:secPr>.*?)</hp:run>',
                              sec, re.S).group(1)
    sec_lead = f'<hp:run charPrIDRef="0">{sec_run_inner}</hp:run>'

    def tac_para(pid, label, height, lead=""):
        return (f'<hp:p id="{nid()}" paraPrIDRef="{pid}" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
                f'{lead}<hp:run charPrIDRef="0">{tac_table(label, height)}</hp:run></hp:p>')
    sections = []
    s0 = [para("N 머리", 0, lead=sec_lead)]
    s0.extend(para(f"N 채움 {n}", 0) for n in range(1, 41))
    s0.append(tac_para(22, "N 표 위간격2000 자연쪽넘김", 3000))
    s0.append(para("N 끝"))
    sections.append(s0)
    sections.append([tac_para(24, "T 큰표 위간격2400", 60000, lead=sec_lead), para("T 끝")])
    sections.append([tac_para(24, "U 작은표 위간격2400", 3000, lead=sec_lead), para("U 끝")])
    out_files = {}
    for i, paras in enumerate(sections):
        out_files[f"Contents/section{i}.xml"] = (sec_head + "".join(paras) + "</hs:sec>").encode("utf-8")
    header = header_with_spacing(data["Contents/header.xml"].decode("utf-8"))
    header = re.sub(r'secCnt="\d+"', f'secCnt="{len(sections)}"', header, count=1)
    hpf = data["Contents/content.hpf"].decode("utf-8")
    items = "".join(f'<opf:item id="section{i}" href="Contents/section{i}.xml" media-type="application/xml"/>'
                    for i in range(1, len(sections)))
    refs = "".join(f'<opf:itemref idref="section{i}" linear="yes"/>' for i in range(1, len(sections)))
    hpf = hpf.replace('<opf:item id="section0" href="Contents/section0.xml" media-type="application/xml"/>',
                      '<opf:item id="section0" href="Contents/section0.xml" media-type="application/xml"/>' + items, 1)
    hpf = hpf.replace('<opf:itemref idref="section0" linear="yes"/>',
                      '<opf:itemref idref="section0" linear="yes"/>' + refs, 1)
    with zipfile.ZipFile(out, "w") as zout:
        for name, raw in files:
            if name.startswith("Preview/"):
                continue
            if name == "Contents/section0.xml":
                continue
            if name == "Contents/header.xml":
                raw = header.encode("utf-8")
            elif name == "Contents/content.hpf":
                raw = hpf.encode("utf-8")
            ct = zipfile.ZIP_STORED if name == "mimetype" else zipfile.ZIP_DEFLATED
            zout.writestr(name, raw, compress_type=ct)
            if name == "Contents/header.xml":
                for sname, sraw in out_files.items():
                    zout.writestr(sname, sraw, compress_type=zipfile.ZIP_DEFLATED)


if __name__ == "__main__":
    main()
