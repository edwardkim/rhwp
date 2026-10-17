#!/usr/bin/env python3
"""[#7092] 대체 글꼴이 가르는 `·`(U+00B7) 실현을 보는 HWPX 생성기.

  python3 gen_subst_font.py samples/issue1549_multipositive_float_tables.hwpx substitute-font-ttf.hwpx ttf
  python3 gen_subst_font.py ... substitute-font-hft.hwpx hft
  python3 gen_subst_font.py ... substitute-font-none.hwpx none

템플릿은 저장소 공개 샘플이며 header.xml 의 글꼴·글자모양 목록에 face 를 더하고
section0.xml 을 새로 만든다(모든 문단에 linesegarray 없음 — 한/글이 직접 조판). 쪽마다 사례 하나.

  S1  한양중고딕 TTF  + <hh:substFont face="HY중고딕" type="TTF"/>   → 한/글은 HY중고딕 TrueType
  S2  한양중고딕 TTF  + <hh:substFont face="한양중고딕" type="HFT"/> → 한/글은 자기 HFT 글리프
  S3  HY견고딕 TTF 직접 선언
  S4  휴먼명조 TTF 직접 선언 (대조군, 이미 전각)
  S5  한양중고딕 TTF, 대체 글꼴 없음
"""
import re
import sys
import zipfile

_id = [2000]


def nid():
    _id[0] += 1
    return _id[0]


# 한/글은 글꼴 목록을 이름으로 묶으므로 같은 이름(한양중고딕)의 두 선언을 한 파일에 넣을 수 없다.
# 변형(argv[3])마다 파일 하나: ttf = S1·S3·S4, hft = S2, none = 대체 글꼴 없는 한양중고딕 TTF.
VARIANTS = {
    "ttf": [
        ("S1 한양중고딕 + 대체 TTF HY중고딕", "한양중고딕", "TTF",
         '<hh:substFont face="HY중고딕" type="TTF" isEmbedded="0" binaryItemIDRef=""/>'),
        ("S3 HY견고딕", "HY견고딕", "TTF", ""),
        ("S4 휴먼명조", "휴먼명조", "TTF", ""),
    ],
    "hft": [
        ("S2 한양중고딕 + 대체 HFT 한양중고딕", "한양중고딕", "TTF",
         '<hh:substFont face="한양중고딕" type="HFT" isEmbedded="0" binaryItemIDRef=""/>'),
    ],
    "none": [
        ("S5 한양중고딕 (대체 글꼴 없음)", "한양중고딕", "TTF", ""),
    ],
}
FONTS = []
LANGS = ["HANGUL", "LATIN", "HANJA", "JAPANESE", "OTHER", "SYMBOL", "USER"]
LINES = [
    "가·나·다 라·마·바 사·아·자 차·카·타 파·하",
    "시·도조례 제정·개정 사업시행자·사업시행예정자",
    "ABC·DEF abc·def 123·456 A·B·C",
    "통지·공고·공개의 범위와 결정·변경 절차",
    "국내·외 주요 동향·전망 및 수립·집행·평가",
]


def para(text, char_pr, page_break=False):
    return (f'<hp:p id="{nid()}" paraPrIDRef="0" styleIDRef="0" pageBreak="{1 if page_break else 0}" '
            f'columnBreak="0" merged="0"><hp:run charPrIDRef="{char_pr}"><hp:t>{text}</hp:t></hp:run></hp:p>')


def patch_header(header, first_font_id, first_char_pr):
    """fontface 7개에 FONTS 를 같은 id 로 덧붙이고, 글자모양 0 을 복제해 face 별 charPr 을 만든다."""
    def add_fonts(m):
        lang = m.group(1)
        cnt = int(m.group(2))
        body = m.group(3)
        extra = "".join(
            f'<hh:font id="{first_font_id + i}" face="{face}" type="{ftype}" isEmbedded="0">{subst}</hh:font>'
            if subst else f'<hh:font id="{first_font_id + i}" face="{face}" type="{ftype}" isEmbedded="0"/>'
            for i, (_, face, ftype, subst) in enumerate(FONTS)
        )
        assert cnt == first_font_id, (lang, cnt)
        return f'<hh:fontface lang="{lang}" fontCnt="{cnt + len(FONTS)}">{body}{extra}</hh:fontface>'

    header = re.sub(r'<hh:fontface lang="(\w+)" fontCnt="(\d+)">(.*?)</hh:fontface>', add_fonts, header, flags=re.S)

    base = re.search(r'<hh:charPr id="0" .*?</hh:charPr>', header, re.S).group(0)
    extra = []
    for i in range(len(FONTS)):
        cp = base.replace('<hh:charPr id="0" ', f'<hh:charPr id="{first_char_pr + i}" ', 1)
        cp = re.sub(r'<hh:fontRef [^/]*/>',
                    "<hh:fontRef " + " ".join(f'{l.lower()}="{first_font_id + i}"' for l in LANGS) + "/>", cp)
        cp = cp.replace('height="1000"', 'height="1200"')
        extra.append(cp)

    def add_char_prs(m):
        cnt = int(m.group(1))
        assert cnt == first_char_pr, cnt
        return f'<hh:charProperties itemCnt="{cnt + len(FONTS)}">{m.group(2)}{"".join(extra)}</hh:charProperties>'

    return re.sub(r'<hh:charProperties itemCnt="(\d+)">(.*?)</hh:charProperties>', add_char_prs, header, flags=re.S)


def main():
    template, out, variant = sys.argv[1], sys.argv[2], sys.argv[3]
    FONTS.extend(VARIANTS[variant])
    zin = zipfile.ZipFile(template)
    header = zin.read("Contents/header.xml").decode("utf-8")
    first_font_id = int(re.search(r'<hh:fontface lang="HANGUL" fontCnt="(\d+)"', header).group(1))
    first_char_pr = int(re.search(r'<hh:charProperties itemCnt="(\d+)"', header).group(1))
    header = patch_header(header, first_font_id, first_char_pr)

    sec = zin.read("Contents/section0.xml").decode("utf-8")
    sec_head = sec[:sec.index("<hp:p ")]
    sec_run = re.search(r'<hp:p [^>]*>(<hp:run charPrIDRef="0"><hp:secPr.*?</hp:run>)', sec, re.S).group(1)
    body = [f'<hp:p id="{nid()}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
            f'{sec_run}<hp:run charPrIDRef="0"><hp:t>#7092 대체 글꼴 실험</hp:t></hp:run></hp:p>']
    for i, (label, _, _, _) in enumerate(FONTS):
        body.append(para(f"사례 {label}", 0, page_break=i > 0))
        for line in LINES:
            body.append(para(line, first_char_pr + i))
    new_sec = sec_head + "".join(body) + "</hs:sec>"
    with zipfile.ZipFile(out, "w") as zout:
        for item in zin.infolist():
            if item.filename.startswith("Preview/"):
                continue
            if item.filename == "Contents/section0.xml":
                data = new_sec.encode("utf-8")
            elif item.filename == "Contents/header.xml":
                data = header.encode("utf-8")
            else:
                data = zin.read(item.filename)
            ct = zipfile.ZIP_STORED if item.filename == "mimetype" else zipfile.ZIP_DEFLATED
            zout.writestr(item, data, compress_type=ct)


if __name__ == "__main__":
    main()
