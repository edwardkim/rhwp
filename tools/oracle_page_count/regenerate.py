#!/usr/bin/env python3
"""저장소 안의 한컴 정답지 PDF 로 `tests/fixtures/oracle_page_count_baseline.tsv` 를 만든다.

이 저장소는 `pdf/` 에 한글이 직접 뽑은 출력 573 장을 갖고 있다(sparse-checkout 대상이
아니라 작업 트리에는 안 보이지만 오브젝트에는 있다). 그 쪽수는 **한글이 이 문서를 몇 쪽으로
조판했는가**의 정답이므로, rhwp 의 `pageCount` 와 견주면 v1.0 의 "한컴과 같은 조판" 을
저장소 자산만으로 판정할 수 있다.

사용법:

    python tools/oracle_page_count/regenerate.py --rhwp target/release-test/rhwp.exe

`pypdfium2` 가 필요하다(`pip install pypdfium2`). 이 스크립트는 픽스처를 만들 때만 쓰고,
CI 와 회귀 시험은 만들어진 TSV 만 읽는다 — Rust 쪽에 PDF 파서 의존을 들이지 않기 위해서다.

## 짝짓기 규칙

정답지 파일명은 `<이름>[-접미사].pdf` 이고 접미사는 한글 버전·폰트 조건이다
(`-2022`, `-2020-kopub`, `-no-ttf` 등). 같은 문서를 여러 조건으로 뽑은 정답지가 있으면
**쪽수의 집합**을 만들어 그중 하나와 맞으면 일치로 본다 — 조건 차이를 결함으로 오인하지
않기 위한 보수적 판정이다.

## 모아 찍기 제외

`print_method` 가 모아 찍기(4·5)인 문서는 한글이 한 장에 여러 쪽을 실어 뽑으므로 장 수가
애초에 다르다(`model::document::print_method_implies_nup` 주석의 실측표). 그 문서는 이
대조에서 제외한다. **정답지의 용지 방향 같은 간접 신호로 추측하지 않는다** — 세로로 뽑힌
정답지를 2-up 으로 오인해 진짜 불일치를 삼킨 사례가 있었다(`hancom-hwp/hwpx-02.hwp`).
"""
import argparse
import json
import os
import re
import subprocess
import sys

SUFFIX = re.compile(r'-(20\d\d|hwp|hwpx|kopub|no-ttf|current)+$', re.I)
FIXTURE = 'tests/fixtures/oracle_page_count_baseline.tsv'


def stem(path):
    name = re.sub(r'\.(pdf|hwp|hwpx)$', '', os.path.basename(path), flags=re.I)
    prev = None
    while prev != name:
        prev = name
        name = SUFFIX.sub('', name)
    return name


def git_pdf_paths():
    out = subprocess.run(
        ['git', '-c', 'core.quotePath=false', 'ls-tree', '-r', 'HEAD', '--name-only', 'pdf/'],
        capture_output=True, text=True, encoding='utf-8', errors='replace').stdout
    return [p.strip() for p in out.split('\n') if p.strip().lower().endswith('.pdf')]


def sample_paths():
    found = []
    for root, _, files in os.walk('samples'):
        for f in files:
            if f.lower().endswith(('.hwp', '.hwpx')):
                found.append(os.path.join(root, f).replace(os.sep, '/'))
    return sorted(found)


def oracle_pages(git_path, tmp):
    import pypdfium2 as pdfium
    with open(tmp, 'wb') as fh:
        if subprocess.run(['git', 'show', 'HEAD:' + git_path], stdout=fh).returncode != 0:
            return None
    try:
        doc = pdfium.PdfDocument(tmp)
        n = len(doc)
        doc.close()
        return n
    except Exception:
        return None


def rhwp_info(rhwp, path):
    r = subprocess.run([rhwp, 'info', path, '--json'],
                       capture_output=True, text=True, encoding='utf-8', errors='replace')
    if r.returncode != 0:
        return None, False
    try:
        d = json.loads(r.stdout)
        return d.get('pageCount'), bool(d.get('printMethodImpliesNup'))
    except Exception:
        return None, False


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--rhwp', default='target/release-test/rhwp.exe')
    args = ap.parse_args()

    pmap = {}
    for p in git_pdf_paths():
        pmap.setdefault(stem(p), []).append(p)

    tmp = os.path.join(os.environ.get('TEMP', '.'), 'rhwp_oracle_regen.pdf')
    cache = {}
    rows = []
    skipped_nup = []
    for sample in sample_paths():
        key = stem(sample)
        if key not in pmap:
            continue
        counts = set()
        for pdf in pmap[key]:
            if pdf not in cache:
                cache[pdf] = oracle_pages(pdf, tmp)
            if cache[pdf]:
                counts.add(cache[pdf])
        if not counts:
            continue
        got, nup = rhwp_info(args.rhwp, sample)
        if got is None:
            continue
        if nup:
            skipped_nup.append(sample)
            continue
        rows.append((sample, sorted(counts), got))

    lines = [
        '# 한글 정답지 PDF 대비 rhwp pageCount 기준선.',
        '# 생성: python tools/oracle_page_count/regenerate.py',
        '# 열: 상대경로 <TAB> 정답지쪽수(쉼표구분) <TAB> 이 기준선의 rhwp쪽수',
        '# 모아 찍기(print_method 4·5) 문서는 장 수가 애초에 달라 제외한다.',
    ]
    for sample, counts, got in rows:
        lines.append('%s\t%s\t%d' % (sample, ','.join(str(c) for c in counts), got))
    with open(FIXTURE, 'w', encoding='utf-8', newline='\n') as fh:
        fh.write('\n'.join(lines) + '\n')

    match = sum(1 for _, c, g in rows if g in c)
    print('대조 대상 %d개 / 정답지와 일치 %d개 / 불일치 %d개 / 모아찍기 제외 %d개'
          % (len(rows), match, len(rows) - match, len(skipped_nup)))
    for s in skipped_nup:
        print('  모아찍기 제외: %s' % s)
    print('기록: %s' % FIXTURE)
    return 0


if __name__ == '__main__':
    sys.exit(main())
