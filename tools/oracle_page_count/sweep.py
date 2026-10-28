#!/usr/bin/env python3
"""저장소가 **알고 있는 한글 쪽수 전부**에 대해 rhwp 를 재고, 두 실행의 차이를 낸다.

## 왜 필요한가

쪽수 원장(`tests/fixtures/oracle_page_count_baseline.tsv`)은 `pdf/` 에 정답지가 있는
문서만 덮는다. 그런데 저장소가 아는 한글 쪽수는 그것만이 아니다 — `tests/` 안에
"한글 2020 정본은 60쪽이다" 같은 assert 로 박혀 있는 문서가 따로 있다.

이 차이는 실제로 사람을 오도한다. `recalculate_section_vpos` 의 vpos=0 리셋 규칙을
넓혀 보고 원장 553 문서로 재니 **개선 2 · 회귀 0** 이었다. 그래서 안전하다고 판단했는데,
전체 테스트를 돌리자 `samples/issue6031/3249937_asset_management_rules.hwpx` 가
**60 → 61 쪽**으로 깨졌다. 그 문서는 `pdf/` 에 정답지가 없어 원장에 없었다.

전체 통합 테스트는 90 분 넘게 걸린다. 조판을 건드리는 변경마다 그것을 돌릴 수는 없다.
이 스윕은 **아는 정답 전부**를 몇 분 안에 재서 같은 질문에 답한다.

## 쓰는 법

```bash
cargo build --profile release-test
python tools/oracle_page_count/sweep.py --rhwp target/release-test/rhwp.exe --out before.tsv
# ... 코드 수정 ...
cargo build --profile release-test
python tools/oracle_page_count/sweep.py --rhwp target/release-test/rhwp.exe --base before.tsv
```

`--base` 를 주면 개선·회귀를 문서별로 낸다. 회귀가 하나라도 있으면 종료 코드 1 이다.

## 진실의 출처

| 출처 | 무엇 | 신뢰 |
| --- | --- | --- |
| `pdf` | `pdf/` 의 한글 출력 PDF 쪽 수 | 한글이 직접 뽑은 값 |
| `test` | `tests/` 의 `assert_eq!(…page_count…, N, "…한글…")` | 그 시험이 기록한 한글 값 |

`test` 출처는 **파일에 `samples/…` 경로 문자열이 하나뿐일 때만** 귀속한다. 여럿이면
어느 문서의 값인지 알 수 없으므로 버린다. 원장 픽스처는 건드리지 않는다 — 이 스윕은
읽기만 한다.
"""
import argparse
import collections
import io
import json
import os
import re
import subprocess
import sys
import tempfile

FIXTURE = 'tests/fixtures/oracle_page_count_baseline.tsv'
SUFFIX = re.compile(r'-(20\d\d|hwp|hwpx|kopub|no-ttf|current)+$', re.I)
TRUTH_WORD = re.compile(r'(한글|오라클|정본)')
PAGE_ASSERT = re.compile(
    r'assert_eq!\s*\(\s*([^,]*?page_count[^,]*?)\s*,\s*(\d+)\s*,\s*(.*?)\)\s*;', re.S)
SAMPLE_LITERAL = re.compile(r'"(samples/[^"]+\.(?:hwpx|hwp))"', re.I)


def read_fixture(root):
    """`pdf` 출처: 원장 픽스처의 (경로 → 정답지 쪽수 집합)."""
    out = {}
    path = os.path.join(root, FIXTURE)
    if not os.path.isfile(path):
        return out
    with io.open(path, encoding='utf-8') as fh:
        for line in fh:
            line = line.rstrip('\n')
            if not line or line.startswith('#'):
                continue
            cols = line.split('\t')
            if len(cols) < 3:
                continue
            pages = {int(x) for x in cols[1].split(',') if x.strip().isdigit()}
            if pages:
                out[cols[0]] = pages
    return out


def harvest_tests(root):
    """`test` 출처: 시험에 상수로 박힌 한글 쪽수."""
    out = collections.defaultdict(set)
    base = os.path.join(root, 'tests')
    for cur, _dirs, names in os.walk(base):
        if 'generated' in cur.replace(os.sep, '/').split('/'):
            continue
        for name in names:
            if not name.endswith('.rs'):
                continue
            try:
                with io.open(os.path.join(cur, name), encoding='utf-8',
                             errors='replace') as fh:
                    src = fh.read()
            except OSError:
                continue
            samples = set(SAMPLE_LITERAL.findall(src))
            if len(samples) != 1:
                # 파일이 여러 문서를 다루면 어느 쪽 값인지 확정할 수 없다.
                continue
            sample = next(iter(samples))
            for m in PAGE_ASSERT.finditer(src):
                if TRUTH_WORD.search(m.group(3)):
                    out[sample].add(int(m.group(2)))
    return out


def measure(rhwp, root, rel):
    r = subprocess.run([rhwp, 'info', rel, '--json'], capture_output=True,
                       text=True, encoding='utf-8', errors='replace', cwd=root)
    if r.returncode != 0:
        return None, False
    try:
        d = json.loads(r.stdout)
        return d.get('pageCount'), bool(d.get('printMethodImpliesNup'))
    except Exception:
        return None, False


def load_run(path):
    out = {}
    with io.open(path, encoding='utf-8') as fh:
        for line in fh:
            line = line.rstrip('\n')
            if not line or line.startswith('#'):
                continue
            cols = line.split('\t')
            if len(cols) >= 2 and cols[1].strip().isdigit():
                out[cols[0]] = int(cols[1])
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--rhwp', default='target/release-test/rhwp.exe')
    ap.add_argument('--root', default='.', help='저장소 루트')
    ap.add_argument('--out', help='이번 실행의 쪽수를 이 TSV 로 저장')
    ap.add_argument('--base', help='이 TSV 와 비교해 개선·회귀를 낸다')
    args = ap.parse_args()

    root = os.path.abspath(args.root)
    rhwp = args.rhwp
    if not os.path.isabs(rhwp):
        rhwp = os.path.join(root, rhwp)

    truth = {}
    source = {}
    for rel, pages in read_fixture(root).items():
        truth[rel] = set(pages)
        source[rel] = 'pdf'
    for rel, pages in harvest_tests(root).items():
        if rel in truth:
            truth[rel] |= pages
        else:
            truth[rel] = set(pages)
            source[rel] = 'test'

    rows = []
    by_source = collections.Counter()
    match = mismatch = skipped = 0
    for rel in sorted(truth):
        if not os.path.isfile(os.path.join(root, rel)):
            skipped += 1
            continue
        got, nup = measure(rhwp, root, rel)
        if got is None or nup:
            skipped += 1
            continue
        ok = got in truth[rel]
        rows.append((rel, got, sorted(truth[rel]), source[rel], ok))
        by_source[source[rel]] += 1
        if ok:
            match += 1
        else:
            mismatch += 1

    total = match + mismatch
    print('아는 한글 쪽수 %d문서 (pdf %d · test %d) / 일치 %d (%.1f%%) / 불일치 %d / 건너뜀 %d'
          % (total, by_source['pdf'], by_source['test'], match,
             100.0 * match / total if total else 0.0, mismatch, skipped))

    if args.out:
        with io.open(args.out, 'w', encoding='utf-8', newline='\n') as fh:
            fh.write('# rhwp 쪽수 스윕 — 경로 <TAB> rhwp쪽수 <TAB> 한글쪽수 <TAB> 출처\n')
            for rel, got, want, src, _ok in rows:
                fh.write('%s\t%d\t%s\t%s\n'
                         % (rel, got, ','.join(str(w) for w in want), src))
        print('기록: %s' % args.out)

    if not args.base:
        return 0

    before = load_run(args.base)
    improved, regressed = [], []
    for rel, got, want, src, ok in rows:
        if rel not in before or before[rel] == got:
            continue
        was_ok = before[rel] in set(want)
        if not was_ok and ok:
            improved.append((rel, before[rel], got, want, src))
        elif was_ok and not ok:
            regressed.append((rel, before[rel], got, want, src))
    print('\n기준 대비: 개선 %d / 회귀 %d' % (len(improved), len(regressed)))
    for tag, lst in (('개선', improved), ('회귀', regressed)):
        for rel, b, g, want, src in lst:
            print('  [%s][%s] %s: %d -> %d (한글 %s)' % (tag, src, rel, b, g, want))
    return 1 if regressed else 0


if __name__ == '__main__':
    sys.exit(main())
