"""#7418 condense 규칙 재현 프로브 — 한/글이 저장한 줄을 규칙 후보로 다시 짜 본다.

입력은 `samples/issue7418/condense_break_synthetic-hancom-2024.hwpx`(한/글 2024 저장본).
맑은 고딕 10pt 는 한글 1000 · 공백 500 HWPUNIT 이고(한/글 PDF 원점 간격으로 확인),
가용 폭은 저장 `horzsize` 42520 이다. 정수만 쓰므로 측정 잡음이 없다.

모형
  R2      공백은 condense% 까지 줄인다. 새 낱말은 그 앞 줄이 자연폭 안일 때만 시작한다.
  simple  새 낱말도 이어지는 글자와 같이 condense 한도까지 받는다.
  gate25  종전 rhwp: condense 가 필요하면 줄에 2.5em 이상 틈이 남았을 때만 받는다.

usage: python condense_rule_replay.py [hwpx]
"""
import html
import re
import sys
import zipfile

SRC = sys.argv[1] if len(sys.argv) > 1 else 'samples/issue7418/condense_break_synthetic-hancom-2024.hwpx'
HANGUL, SPACE, EM = 1000, 500, 1000

z = zipfile.ZipFile(SRC)
header = z.read('Contents/header.xml').decode('utf-8')
section = z.read('Contents/section0.xml').decode('utf-8')
para_pr = {}
for m in re.finditer(r'<hh:paraPr id="(\d+)"[^>]*?condense="(\d+)"[^>]*>(.*?)</hh:paraPr>', header, re.S):
    unit = re.search(r'breakNonLatinWord="(\w+)"', m.group(3)).group(1)
    para_pr[m.group(1)] = (int(m.group(2)), unit)


def replay(text, width, condense, char_unit, model):
    """줄 시작 위치(글자 인덱스) 목록."""
    f = condense / 100
    starts, nat, nsp = [0], 0, 0
    i = 0
    words = []
    for w in text.split(' '):
        words.append((i, w))
        i += len(w) + 1
    for wi, (pos, w) in enumerate(words):
        units = [w[k] for k in range(len(w))] if char_unit else [w]
        for ui, u in enumerate(units):
            u_w = len(u) * HANGUL
            new_word = ui == 0 and nat > 0
            gap = SPACE if new_word else 0
            cand = nat + gap + u_w
            cand_sp = nsp + (1 if new_word else 0)
            condensed = cand - cand_sp * SPACE * f
            fits = condensed <= width
            needs = cand > width
            if fits and needs:
                if model == 'R2' and new_word:
                    fits = nat <= width
                elif model == 'gate25':
                    fits = width - (nat - nsp * SPACE * f) >= 2.5 * EM
            if nat == 0 or fits:
                nat, nsp = cand, cand_sp
            else:
                starts.append(pos + (ui if char_unit else 0))
                nat, nsp = u_w, 0
    return starts


total = {'R2': 0, 'simple': 0, 'gate25': 0}
lines = 0
for p in re.findall(r'<hp:p [^>]*>.*?</hp:p>', section, re.S):
    text = ''.join(html.unescape(s) for s in re.findall(r'<hp:t>(.*?)</hp:t>', p, re.S))
    if len(text) < 50:
        continue
    pid = re.search(r'paraPrIDRef="(\d+)"', p).group(1)
    condense, unit = para_pr[pid]
    saved = [int(v) for v in re.findall(r'<hp:lineseg textpos="(\d+)"', p)]
    width = int(re.search(r'horzsize="(\d+)"', p).group(1))
    char_unit = unit == 'KEEP_WORD'  # #2185: KEEP_WORD 가 글자 단위
    row = []
    for model in total:
        got = replay(text, width, condense, char_unit, model)
        same = sum(1 for a, b in zip(got, saved) if a == b)
        prefix = next((k for k, (a, b) in enumerate(zip(got, saved)) if a != b), min(len(got), len(saved)))
        total[model] += prefix
        row.append(f'{model}={prefix}/{len(saved)}')
    lines += len(saved)
    print(f'condense={condense:2d} {"글자" if char_unit else "낱말"}  ' + '  '.join(row))
print('줄 시작 일치(첫 불일치 전까지):', {k: f'{v}/{lines}' for k, v in total.items()})
