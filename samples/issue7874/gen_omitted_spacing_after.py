#!/usr/bin/env python3
"""[#7874] 저장 줄 위치가 문단 아래 간격을 빼먹은 HWPX — 공개 합성 입력.

  python3 gen_omitted_spacing_after.py samples/issue2527_empty_linesegs.hwpx omitted-spacing-after.hwpx

문단 아래 간격 568HU(HWPX `hc:next` 568) 인 한 줄 문단 30개를 두 쪽에 둔다.
  1쪽  저장 줄 위치가 아래 간격을 **뺀** 사다리(문단마다 1600HU = 줄 높이 1000 + 줄간격 600)
  2쪽  같은 문단, 저장 줄 위치가 아래 간격을 **담은** 사다리(문단마다 2168HU) — 대조군
568HU = 7.57px 는 rhwp 의 후방 스냅 허용(8px) 안이다. 한/글은 두 쪽을 같은 간격으로 그린다.
"""
import re
import sys
import zipfile

src = zipfile.ZipFile(sys.argv[1])
out = sys.argv[2]
hdr = src.read('Contents/header.xml').decode('utf8')
sec = src.read('Contents/section0.xml').decode('utf8')
pp0 = re.search(r'<hh:paraPr id="0".*?</hh:paraPr>', hdr, re.S).group(0)
cnt = int(re.search(r'<hh:paraProperties itemCnt="(\d+)">', hdr).group(1))
pp_sa = pp0.replace('id="0"', f'id="{cnt}"').replace('<hc:next value="0"', '<hc:next value="568"')
hdr = hdr.replace(f'<hh:paraProperties itemCnt="{cnt}">' + pp0,
                  f'<hh:paraProperties itemCnt="{cnt + 1}">' + pp0 + pp_sa)
PP_SA = cnt
head = sec[:sec.find('<hp:bookmark')]

_id = [500]


def nid():
    _id[0] += 1
    return _id[0]


def lineseg(vpos):
    return (f'<hp:linesegarray><hp:lineseg textpos="0" vertpos="{vpos}" vertsize="1000" textheight="1000" '
            f'baseline="850" spacing="600" horzpos="0" horzsize="48188" flags="393216"/></hp:linesegarray>')


def para(text, vpos, page_break=False):
    return (f'<hp:p id="{nid()}" paraPrIDRef="{PP_SA}" styleIDRef="0" pageBreak="{1 if page_break else 0}" '
            f'columnBreak="0" merged="0"><hp:run charPrIDRef="0"><hp:t>{text}</hp:t></hp:run>'
            + lineseg(vpos) + '</hp:p>\n')


body = [head + '</hp:run><hp:run charPrIDRef="0"><hp:t>#7874 아래 간격을 뺀 저장 사다리</hp:t></hp:run>'
        + lineseg(0) + '</hp:p>\n']
N = 30
# 1쪽: 제목 줄(0) 뒤 문단 k 의 저장 위치 = 1600·k (아래 간격 568 을 뺌).
for k in range(1, N + 1):
    body.append(para(f'1쪽 문단 {k} — 저장 위치가 아래 간격을 뺐다', 1600 * k))
# 2쪽: 저장 위치 = 2168·(k-1) (아래 간격을 담음).
for k in range(1, N + 1):
    body.append(para(f'2쪽 문단 {k} — 저장 위치가 아래 간격을 담았다', 2168 * (k - 1), page_break=k == 1))
sec_new = ''.join(body) + '</hs:sec>'


def put(z, name, data, stored=False):
    info = zipfile.ZipInfo(name, date_time=(2026, 10, 18, 0, 0, 0))
    info.compress_type = zipfile.ZIP_STORED if stored else zipfile.ZIP_DEFLATED
    z.writestr(info, data)


with zipfile.ZipFile(out, 'w') as z:
    put(z, 'mimetype', src.read('mimetype'), stored=True)
    for n in ('version.xml', 'Contents/content.hpf', 'META-INF/container.xml', 'META-INF/manifest.xml'):
        put(z, n, src.read(n))
    put(z, 'Contents/header.xml', hdr.encode('utf8'))
    put(z, 'Contents/section0.xml', sec_new.encode('utf8'))
