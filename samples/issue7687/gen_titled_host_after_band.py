#!/usr/bin/env python3
"""빈 host 의 양수 오프셋 자리차지 표 띠 뒤에 오는 **글 있는 host** 의 자리차지 표 위치를 보는
HWPX 생성기. 모든 문단에 linesegarray 가 없다(한/글이 직접 조판). 쪽마다 사례 하나.

  python3 gen_titled_host_after_band.py samples/issue1549_multipositive_float_tables.hwpx titled-host.hwpx

빈 host(T1 오프셋 0 / T2 오프셋 15000HU) 뒤 빈 문단이 T2 띠에 걸려 띠 아래로 가고, 그 다음
글 있는 host(T3 오프셋 0 / T4 오프셋 9000HU)가 온다. T4 의 위끝은 host 문단 원점 + 오프셋 +
바깥 위 여백이다. 그 원점은 앞 띠의 끝에서 정해지므로, 띠 끝을 잘못 세면 T3 는 그대로인데
T4·제목·뒤 글만 어긋난다(한글문서파일형식 20쪽 pi=133, #7687).
"""
import re
import sys
import zipfile

from gen_empty_host import empty, host, nid, para, table

T3 = (0, 0, 6000, "T3")
T4 = (9000, 20000, 6000, "T4")


def titled_host(text, tables=(T3, T4)):
    ctrls = "".join(table(*t) for t in tables)
    return (f'<hp:p id="{nid()}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
            f'<hp:run charPrIDRef="0">{ctrls}<hp:t>{text}</hp:t></hp:run></hp:p>')


CASES = [
    ("K1 빈 host·빈 문단 둘 뒤 제목 host",
     lambda: [host(None), empty(), empty(), titled_host("K1 제목")] + [para(f"K1 글 {i}") for i in range(1, 4)]),
    ("K2 빈 host·빈 문단 넷 뒤 제목 host",
     lambda: [host(None)] + [empty() for _ in range(4)] + [titled_host("K2 제목")]
     + [para(f"K2 글 {i}") for i in range(1, 4)]),
    ("K3 빈 host 바로 뒤 제목 host",
     lambda: [host(None), titled_host("K3 제목")] + [para(f"K3 글 {i}") for i in range(1, 4)]),
    ("K4 대조군 — 앞 빈 host 없음",
     lambda: [para("K4 앞 글"), titled_host("K4 제목")] + [para(f"K4 글 {i}") for i in range(1, 4)]),
]


def main():
    template, out = sys.argv[1], sys.argv[2]
    zin = zipfile.ZipFile(template)
    sec = zin.read("Contents/section0.xml").decode("utf-8")
    sec_head = sec[:sec.index("<hp:p ")]
    sec_run = re.search(r'<hp:p [^>]*>(<hp:run charPrIDRef="0"><hp:secPr.*?</hp:run>)', sec, re.S).group(1)
    body = [f'<hp:p id="{nid()}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
            f'{sec_run}<hp:run charPrIDRef="0"><hp:t>#7687 띠 뒤 제목 host 실험</hp:t></hp:run></hp:p>']
    for i, (name, build) in enumerate(CASES):
        body.append(para(f"사례 {name}", page_break=i > 0))
        body.extend(build())
    new_sec = sec_head + "".join(body) + "</hs:sec>"
    with zipfile.ZipFile(out, "w") as zout:
        for item in zin.infolist():
            if item.filename.startswith("Preview/"):
                continue
            data = new_sec.encode("utf-8") if item.filename == "Contents/section0.xml" else zin.read(item.filename)
            ct = zipfile.ZIP_STORED if item.filename == "mimetype" else zipfile.ZIP_DEFLATED
            zout.writestr(item, data, compress_type=ct)


if __name__ == "__main__":
    main()
