#!/usr/bin/env python3
"""공개 합성 입력 생성기 (#7330 synth-fwd 재현용) — 2단계.

실물 형상: 결재문서를 외부 도구로 가림(***) 처리하면 가린 문단의 글이 짧아져 줄 수가 줄고,
도구는 그 문단의 LINE_SEG 에 구현 속성(bit31, flags 0x80000000)을 단다. 뒤 문단의 저장
vpos 는 옛 줄 수로 잰 값 그대로 남는다.

1) gen  <출력.hwpx>      : linesegarray 없는 원문 HWPX. pi=3 이 4줄짜리 긴 문단이다.
   뼈대는 공개 샘플 samples/issue2527_empty_linesegs.hwpx (rhwp 출처 표식 없음).
   → 한/글 2020(hwp2024Convert MCP engine 2020)으로 HWPX 저장해 저장 LineSeg 를 얻는다.
2) mask <한글저장.hwpx> <출력.hwpx> : 그 저장본의 pi=3 글을 '*' 12자로 바꾸고 linesegarray 를
   첫 줄 하나(flags |= 0x80000000)로 줄인다. 뒤 문단의 저장 vpos 는 손대지 않는다(옛 4줄 기준).
   정본 PDF 는 이 mask 출력을 한/글 2020 으로 PDF 출력한 것이다.

사용(저장소 루트에서):
  python3 samples/masked_stale_vpos/make_masked_stale_vpos.py gen out_gen.hwpx
  python3 samples/masked_stale_vpos/make_masked_stale_vpos.py mask hancom_saved.hwpx masked_stale_vpos.hwpx
"""
import re
import sys
import zipfile

LONG = ('3. 이 문단은 가림 처리 전 원문이다. ' * 9).strip()


def put(z, name, data, stored=False):
    info = zipfile.ZipInfo(name, date_time=(2026, 10, 7, 0, 0, 0))
    info.compress_type = zipfile.ZIP_STORED if stored else zipfile.ZIP_DEFLATED
    z.writestr(info, data)


def gen(out):
    src = zipfile.ZipFile('samples/issue2527_empty_linesegs.hwpx')
    sec = src.read('Contents/section0.xml').decode('utf8')
    head = sec[:sec.find('<hp:bookmark')]

    def para(pid, text):
        return (f'<hp:p id="{pid}" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">'
                f'<hp:run charPrIDRef="0"><hp:t>{text}</hp:t></hp:run></hp:p>\n')

    first = head + '</hp:run><hp:run charPrIDRef="0"><hp:t>가림 처리 문단 뒤 저장 vpos 합성 입력 (#7330)</hp:t></hp:run></hp:p>\n'
    paras = [para(10, '1. 가림 앞 첫째 문단이다.'), para(11, '2. 가림 앞 둘째 문단이다.'), para(12, LONG)]
    paras += [para(20 + i, s) for i, s in enumerate([
        '4. 가림 문단 바로 뒤 문단 — 저장 vpos 는 옛 4줄 기준이다.',
        '5. 그 뒤 문단이다.', '6. 그 뒤 문단이다.', '7. 끝.'])]
    with zipfile.ZipFile(out, 'w') as z:
        put(z, 'mimetype', src.read('mimetype'), stored=True)
        for n in ('version.xml', 'Contents/content.hpf', 'META-INF/container.xml', 'META-INF/manifest.xml',
                  'Contents/header.xml'):
            put(z, n, src.read(n))
        put(z, 'Contents/section0.xml', (first + ''.join(paras) + '</hs:sec>').encode('utf8'))


def mask(src_path, out):
    src = zipfile.ZipFile(src_path)
    sec = src.read('Contents/section0.xml').decode('utf8')
    i = sec.find('3. 이 문단은')
    assert i > 0, '원문 pi=3 을 찾지 못했다'
    p0 = sec.rfind('<hp:p ', 0, i)
    p1 = sec.find('</hp:p>', i) + len('</hp:p>')
    p = sec[p0:p1]
    p = re.sub(r'<hp:t>[^<]*</hp:t>', '<hp:t>' + '*' * 12 + '</hp:t>', p, count=1)
    segs = re.findall(r'<hp:lineseg [^>]*/>', p)
    assert len(segs) >= 3, f'원문 pi=3 은 여러 줄이어야 한다: {len(segs)}'
    seg0 = segs[0]
    flags = int(re.search(r'flags="(\d+)"', seg0).group(1)) | 0x80000000
    seg0 = re.sub(r'flags="\d+"', f'flags="{flags}"', seg0)
    p = re.sub(r'<hp:linesegarray>.*?</hp:linesegarray>',
               f'<hp:linesegarray>{seg0}</hp:linesegarray>', p, flags=re.S)
    sec = sec[:p0] + p + sec[p1:]
    with zipfile.ZipFile(out, 'w') as z:
        for info in src.infolist():
            data = sec.encode('utf8') if info.filename == 'Contents/section0.xml' else src.read(info.filename)
            put(z, info.filename, data, stored=info.filename == 'mimetype')


if __name__ == '__main__':
    if sys.argv[1] == 'gen':
        gen(sys.argv[2])
    else:
        mask(sys.argv[2], sys.argv[3])
