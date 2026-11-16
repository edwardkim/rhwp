"""합성본: 쪽 너비를 거의 채운 글자처럼 표 + 표 뒤 20pt 굵은 공백 N개 + 13pt 공백 M개.

공개 샘플만 조합: samples/test-image.hwpx(구역·글꼴), samples/issue3738/tac_sibling_shape_line_advance.hwpx(표).
문단 0 "Q" / 문단 1 host(16pt 런의 글자처럼 표 + 20pt 굵은 공백 N + 13pt 공백 M) / 문단 2 빈 문단 / 문단 3 "TEXT"
사용: gen_tac_space_tail.py <repo> <out.hwpx> --n N [--m 40] [--rest 572] [--table-h 3000]
--rest: 단 폭 - (표 폭 + 바깥 여백) (HU). 572 는 20pt 공백 하나도 들지 않는 폭이다.
HWP 로: `rhwp convert out.hwpx out.hwp` 뒤 OLE 스트림 이름 RhwpHwpxOrigin 을 같은 길이의
XhwpHwpxOrigin 으로 바꾼다(UTF-16LE 치환). convert 결과에는 PARA_LINE_SEG 가 없다(저장 줄 없음).
이 폴더 파일: at_table_nN = --n N (rest 572), after_3fit_nN = --n N --rest 3572,
after_1fit_nN = --n N --rest 1572. 웹한글기안기 인쇄 PDF 는 pdf/tac_space_tail/<이름>-webhwp-print.pdf.
"""
import argparse
import re
import zipfile

ap = argparse.ArgumentParser()
ap.add_argument("repo")
ap.add_argument("out")
ap.add_argument("--n", type=int, required=True)
ap.add_argument("--m", type=int, default=40)
ap.add_argument("--rest", type=int, default=572)
ap.add_argument("--table-h", type=int, default=3000)
ap.add_argument("--no-float", action="store_true", help="문단 끝 글앞으로 표(실문서의 공람 표 자리)를 두지 않는다")
a = ap.parse_args()

COL = 59528 - 8504 * 2
z = zipfile.ZipFile(f"{a.repo}/samples/test-image.hwpx")
sec = z.read("Contents/section0.xml").decode()
header = z.read("Contents/header.xml").decode()
cp0 = re.search(r'<hh:charPr id="0".*?</hh:charPr>', header, re.S).group(0)


def char_pr(cid, height, bold):
    c = cp0.replace('id="0"', f'id="{cid}"', 1).replace('height="1000"', f'height="{height}"', 1)
    if bold:
        c = c.replace("<hh:underline", "<hh:bold/><hh:underline", 1)
    return c


n_items = int(re.search(r'<hh:charProperties itemCnt="(\d+)"', header).group(1))
C16, C20B, C13 = n_items, n_items + 1, n_items + 2
header = header.replace(
    "</hh:charProperties>",
    char_pr(C16, 1600, False) + char_pr(C20B, 2000, True) + char_pr(C13, 1300, False) + "</hh:charProperties>",
)
header = re.sub(r'<hh:charProperties itemCnt="\d+"', f'<hh:charProperties itemCnt="{n_items + 3}"', header)

tsec = zipfile.ZipFile(f"{a.repo}/samples/issue3738/tac_sibling_shape_line_advance.hwpx").read("Contents/section0.xml").decode()
tbl = re.search(r"<hp:tbl\b.*?</hp:tbl>", tsec, re.S).group(0)
width = COL - 283 * 2 - a.rest
tbl = tbl.replace('width="40000"', f'width="{width}"').replace('height="30034"', f'height="{a.table_h}"')
tbl = tbl.replace('horzsize="39000"', f'horzsize="{width - 1000}"')

head = sec[: sec.index("<hp:p ")]
first = sec[sec.index("<hp:p "):]
prefix = first[: first.index("</hp:run>") + len("</hp:run>")]
pid = [10]


def para(runs):
    pid[0] += 1
    return f'<hp:p id="{pid[0]}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">{runs}</hp:p>'


def run(cid, body):
    return f'<hp:run charPrIDRef="{cid}">{body}</hp:run>'


ftbl = re.search(r"<hp:tbl\b.*?</hp:tbl>", tsec, re.S).group(0)
ftbl = ftbl.replace('id="100"', 'id="101"').replace('width="40000"', 'width="4000"').replace('height="30034"', 'height="1000"')
ftbl = ftbl.replace('textWrap="TOP_AND_BOTTOM"', 'textWrap="IN_FRONT_OF_TEXT"')
ftbl = ftbl.replace('treatAsChar="1"', 'treatAsChar="0"').replace('horzRelTo="PARA"', 'horzRelTo="COLUMN"')
ftbl = ftbl.replace('horzOffset="0"', 'horzOffset="38000"').replace('horzsize="39000"', 'horzsize="3000"')
host = run(C16, tbl)
if a.n:
    host += run(C20B, f"<hp:t>{' ' * a.n}</hp:t>")
if a.m:
    host += run(C13, f"<hp:t>{' ' * a.m}</hp:t>")
if not a.no_float:
    host += run(C13, ftbl)
parts = [f'{prefix}{run(0, "<hp:t>Q</hp:t>")}</hp:p>', para(host), para(run(C13, "")), para(run(C13, "<hp:t>TEXT</hp:t>"))]
body = head + "".join(parts) + "</hs:sec>"
body = re.sub(r"<hp:linesegarray>.*?</hp:linesegarray>", "", body, flags=re.S)
with zipfile.ZipFile(a.out, "w") as zout:
    for info in z.infolist():
        if info.filename in ("META-INF/rhwp-hwp5-origin", "Preview/PrvImage.png"):
            continue
        data = {"Contents/section0.xml": body, "Contents/header.xml": header}.get(info.filename)
        data = data.encode("utf-8") if data is not None else z.read(info.filename)
        if info.filename == "Preview/PrvText.txt":
            data = b""
        zout.writestr(info, data, compress_type=info.compress_type)
print(a.out, "table_width", width)
