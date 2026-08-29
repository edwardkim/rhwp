#!/usr/bin/env python3
"""`samples/` 문서와 `pdf/` 한컴 정답지를 자동으로 짝지어 목록을 낸다.

`fidelity_compare.py` 는 비교할 쌍을 `REG` 에 손으로 등록하거나 `--source`·`--reference-pdf`
로 직접 지정해야 한다. 등록된 것은 6 개인데 `pdf/` 에는 정답지가 **573 장** 있다. 나머지를
쓰려면 매번 짝을 찾아 손으로 적어야 한다.

이 도구는 그 짝짓기를 자동화한다. 산출은 `fidelity_compare.py` 에 그대로 넣을 수 있는
`--source`/`--reference-pdf` 인자쌍이거나 TSV 목록이다.

    python tools/fidelity_compare/oracle_pair_index.py --list
    python tools/fidelity_compare/oracle_pair_index.py --args "samples/basic/sungeo.hwp"

## 짝짓기 규칙 — 디렉터리까지 본다

자동 선택하는 정답지 파일명은 `<이름>-<hwp|hwpx>-<2020|2024>.pdf`다. 이름만 맞추면
**같은 이름의 다른 문서**를 집고, 형식·엔진이 미기록된 PDF는 원본과의 관계도 증명할 수 없다.

    samples/KTX.hwp        27쪽  「AI-반도체 해외실증 지원 사업 공모 안내서」
    samples/basic/KTX.hwp   1쪽  실제 KTX 노선도

이 둘은 `pdf/KTX-2022.pdf`(27 쪽)와 `pdf/basic/KTX-2022.pdf`(1 쪽)를 함께 후보로 갖는다.
잘못 짝지으면 대조 결과 전체가 무의미해진다 — 실제로 이 함정에 걸려 "글자 93.8% 손실" 이라는
가짜 결함을 만들 뻔했다. 저장소에는 같은 이름의 서로 다른 문서가 **44 종** 있다.

그래서 원본 형식이 같은 canonical PDF만 남기고, 같은 디렉터리의 후보를 우선한다. canonical
후보가 없거나 2020·2024 후보가 함께 있으면 자동 비교 인자를 출력하지 않는다. 후자는
`--engine`으로 출력 조건을 명시해야 한다.

## 모아 찍기 문서는 표시한다

`print_method` 가 모아 찍기(4·5)면 한글이 한 장에 여러 쪽을 실어 뽑으므로 쪽수·용지 방향이
rhwp 와 다르다(`model::document::print_method_implies_nup`). 쪽 단위로 견주면 오판하므로
`--list` 에 `nup` 으로 표시한다. `--rhwp` 를 주면 그 판정을 채운다.
"""
import argparse
import json
import os
import subprocess
import sys

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
TOOLS_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if TOOLS_DIR not in sys.path:
    sys.path.insert(0, TOOLS_DIR)

from oracle_pdf_selection import canonical_candidates, canonical_engine, choose_canonical, stem


def git_pdfs():
    """`pdf/` 는 sparse-checkout 대상이 아니라 작업 트리에 없을 수 있다 — git 으로 읽는다."""
    out = subprocess.run(
        ['git', '-c', 'core.quotePath=false', 'ls-tree', '-r', 'HEAD', '--name-only', 'pdf/'],
        capture_output=True, text=True, encoding='utf-8', errors='replace', cwd=REPO).stdout
    return [p.strip() for p in out.split('\n') if p.strip().lower().endswith('.pdf')]


def samples():
    found = []
    root = os.path.join(REPO, 'samples')
    for dirpath, _, files in os.walk(root):
        for f in files:
            if f.lower().endswith(('.hwp', '.hwpx')):
                p = os.path.join(dirpath, f).replace(os.sep, '/')
                found.append('samples/' + p[len(root.replace(os.sep, '/')) + 1:])
    return sorted(found)


def build_index():
    by_name = {}
    for p in git_pdfs():
        by_name.setdefault(stem(p), []).append(p)

    index = {}
    unverified = {}
    for s in samples():
        cands = by_name.get(stem(s))
        if not cands:
            continue
        chosen = canonical_candidates(s, cands)
        if chosen:
            index[s] = (chosen, len(cands))
        else:
            unverified[s] = sorted(cands)
    return index, unverified


def nup_flag(rhwp, sample):
    r = subprocess.run([rhwp, 'info', os.path.join(REPO, sample), '--json'],
                       capture_output=True, text=True, encoding='utf-8', errors='replace')
    if r.returncode != 0:
        return None
    try:
        return bool(json.loads(r.stdout).get('printMethodImpliesNup'))
    except Exception:
        return None


def main():
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('--list', action='store_true', help='짝지어진 목록을 TSV 로 낸다')
    ap.add_argument('--args', metavar='SAMPLE',
                    help='그 문서의 fidelity_compare 인자쌍을 낸다')
    ap.add_argument('--engine', choices=('2020', '2024'),
                    help='여러 canonical PDF가 있을 때 선택할 한컴 출력 엔진')
    ap.add_argument('--rhwp', help='모아 찍기 판정을 채울 rhwp 바이너리 경로')
    args = ap.parse_args()

    index, unverified = build_index()

    if args.args:
        key = args.args.replace(os.sep, '/')
        if key not in index:
            if key in unverified:
                print(f'형식·엔진이 확인된 canonical PDF가 없어 자동 비교하지 않는다: {key}',
                      file=sys.stderr)
            else:
                print(f'짝지어진 정답지가 없다: {key}', file=sys.stderr)
            return 1
        chosen, _ = index[key]
        try:
            reference = choose_canonical(key, chosen, args.engine)
        except ValueError as e:
            print(str(e), file=sys.stderr)
            return 1
        print('--source "%s" --reference-pdf "%s" --label %s'
              % (key, reference, ''.join(ch if ch.isalnum() else '-' for ch in stem(key)).strip('-') or 'doc'))
        return 0

    if not args.list:
        ap.print_help()
        return 2

    print('# 문서\tcanonical 정답지(쉼표구분)\t같은-stem 후보수\t모아찍기')
    nup_count = 0
    for s, (chosen, n_all) in sorted(index.items()):
        nup = ''
        if args.rhwp:
            f = nup_flag(args.rhwp, s)
            if f:
                nup = 'nup'
                nup_count += 1
        print('%s\t%s\t%d\t%s' % (s, ','.join(chosen), n_all, nup))
    narrowed = sum(1 for _, (c, n) in index.items() if len(c) < n)
    print('# 자동 비교 가능 문서 %d개 / canonical 미확인 제외 %d개 / 디렉터리로 좁혀진 것 %d개%s'
          % (len(index), len(unverified), narrowed,
             ' / 모아찍기 %d개' % nup_count if args.rhwp else ''),
          file=sys.stderr)
    return 0


if __name__ == '__main__':
    sys.exit(main())
