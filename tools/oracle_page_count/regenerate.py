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
import collections
import io
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile

SUFFIX = re.compile(r'-(20\d\d|hwp|hwpx|kopub|no-ttf|current)+$', re.I)
FIXTURE = 'tests/fixtures/oracle_page_count_baseline.tsv'


def stem(path):
    name = re.sub(r'\.(pdf|hwp|hwpx)$', '', os.path.basename(path), flags=re.I)
    prev = None
    while prev != name:
        prev = name
        name = SUFFIX.sub('', name)
    return name


def subdir(path, root):
    """`samples/`·`pdf/` 아래의 상대 디렉터리."""
    d = os.path.dirname(path).replace(os.sep, '/')
    return d[len(root):].lstrip('/') if d.startswith(root) else d


def pick_oracles(sample, candidates):
    """이름이 같은 정답지 후보 중 이 샘플의 것만 고른다.

    **파일명만 보면 다른 문서의 정답지를 집어 온다.** 저장소에는 같은 이름의 서로 다른
    문서가 44 종 있다 — 예를 들어 `samples/KTX.hwp` 는 27 쪽짜리 AI-반도체 사업 공모
    안내서이고 `samples/basic/KTX.hwp` 는 1 쪽짜리 KTX 노선도인데, 이름이 같아서 서로의
    정답지를 공유했다. 그러면 각자 상대의 쪽수로도 "일치" 판정을 받아 **진짜 불일치가
    가려진다.**

    같은 디렉터리의 정답지가 있으면 그것만 쓴다. 없으면 이름 후보를 그대로 쓰되(정답지가
    `pdf/` 최상위에만 있는 문서가 많다), 후보가 여럿이면 그 사실이 픽스처의 쪽수 집합에
    드러난다.

    디렉터리 fallback 은 여기서 끝나지 않는다 — `owner_claims_oracle` 이 본문으로 확인한다.
    """
    same_dir = [p for p in candidates if subdir(p, 'pdf') == subdir(sample, 'samples')]
    return same_dir if same_dir else candidates


#: 디렉터리 주인이 후보보다 이만큼 더 맞으면 후보의 짝짓기를 버린다.
#:
#: **절대 임계로는 판별할 수 없다.** 정답지 PDF 의 텍스트 추출 품질이 문서마다 극과
#: 극이라(수식·기호 폰트 문서는 0% 도 나온다) 낮은 일치율 자체는 "다른 문서" 의 증거가
#: 아니다. 실제로 절대 55% 로 걸러 봤더니 6 건이 배제됐는데 그중 5 건은 같은 문서였다
#: (`form-01` 0.0%, `exam_social` 2.4% 등 — 추출이 안 된 것뿐).
#:
#: 판별력은 **상대 비교**에 있다. 진짜 오짝인
#: `samples/hwpx/hancom-hwp/hwpx-02.hwp` 는 14.1% 인데 그 정답지의 디렉터리 주인
#: `samples/hwpx/hwpx-02.hwpx` 는 78.7% 다 — 64.6%p 차. 같은 문서인 fallback 짝 9 쌍은
#: 주인과 후보의 차가 0%p 다(양쪽 다 92~100%). 그 사이라 30%p 로 둔다.
PAIRING_OWNER_MARGIN = 0.30

#: 주인 쪽이 이만큼은 맞아야 "주인이 더 맞는다" 는 비교가 의미를 갖는다.
#: 둘 다 추출이 안 된 문서에서 잡음으로 배제되는 것을 막는다.
PAIRING_OWNER_MIN_SHARE = 0.55


def _char_multiset(text):
    """공백과 사제 영역(PUA)을 뺀 문자 카운터.

    정답지 PDF 는 수식·기호를 PUA 코드포인트로 싣고 rhwp 는 실제 유니코드를 낸다.
    그 차이는 표현이지 문서가 다르다는 뜻이 아니므로 양쪽에서 뺀다.
    """
    return collections.Counter(
        c for c in re.sub(r'\s', '', text) if not 0xE000 <= ord(c) <= 0xF8FF)


def text_share(sample_text, oracle_text):
    """정답지와 렌더 텍스트의 문자 멀티셋 일치율."""
    a, b = _char_multiset(oracle_text), _char_multiset(sample_text)
    total = max(sum(a.values()), sum(b.values()))
    if total == 0:
        return 1.0
    return sum((a & b).values()) / total


def owner_claims_oracle(candidate_share, owner_share):
    """디렉터리 주인이 이 정답지를 더 잘 맞추는가.

    디렉터리가 다른데 이름만 같은 짝은 쪽수만 봐서는 오짝을 알아챌 수 없다. 실제로
    `samples/hwpx/hancom-hwp/hwpx-02.hwp`(10,204 자 보도자료)가
    `pdf/hwpx/hwpx-02-2022.pdf`(1,437 자 해외직접투자 요약, 진짜 주인은
    `samples/hwpx/hwpx-02.hwpx`)를 물어 "정답지 5 쪽 vs rhwp 9 쪽" 이라는 **없는 불일치**
    를 원장에 실었다. 거짓 불일치가 하나라도 있으면 원장 전체를 믿을 수 없다.

    반대로 멀쩡한 짝을 버려도 곤란하다 — 원장이 조용히 줄어드는 것은 통과처럼 보이는
    미검증이다. 그래서 "이 후보가 정답지와 덜 닮았다" 가 아니라 "**다른 샘플이 더
    닮았다**" 를 근거로 삼는다.
    """
    return (owner_share >= PAIRING_OWNER_MIN_SHARE
            and owner_share - candidate_share >= PAIRING_OWNER_MARGIN)


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


def oracle_pages_and_text(git_path, tmp):
    import pypdfium2 as pdfium
    with open(tmp, 'wb') as fh:
        if subprocess.run(['git', 'show', 'HEAD:' + git_path], stdout=fh).returncode != 0:
            return None, ''
    try:
        doc = pdfium.PdfDocument(tmp)
        n = len(doc)
        text = ''.join(doc[i].get_textpage().get_text_range() for i in range(n))
        doc.close()
        return n, text
    except Exception:
        return None, ''


def rhwp_text(rhwp, path, outdir):
    """`export-text` 로 뽑은 전체 본문. 짝짓기 확인에만 쓴다."""
    shutil.rmtree(outdir, ignore_errors=True)
    r = subprocess.run([rhwp, 'export-text', path, '-o', outdir],
                       capture_output=True, text=True, encoding='utf-8', errors='replace')
    if r.returncode != 0 or not os.path.isdir(outdir):
        return None
    parts = []
    for name in sorted(os.listdir(outdir)):
        if name.lower().endswith('.txt'):
            try:
                with io.open(os.path.join(outdir, name), encoding='utf-8',
                             errors='replace') as fh:
                    parts.append(fh.read())
            except OSError:
                pass
    return ''.join(parts)


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

    samples = sample_paths()
    # 각 정답지의 "디렉터리 주인" — 그 정답지와 같은 상대 디렉터리에 있는 같은 이름의
    # 샘플. 주인이 있으면 이름만 같은 다른 디렉터리의 샘플은 그 정답지를 넘볼 수 없다.
    owners = {}
    for sample in samples:
        for pdf in pmap.get(stem(sample), []):
            if subdir(pdf, 'pdf') == subdir(sample, 'samples'):
                owners.setdefault(pdf, sample)

    tmpdir = tempfile.mkdtemp(prefix='rhwp_oracle_regen_')
    tmp = os.path.join(tmpdir, 'oracle.pdf')
    textdir = os.path.join(tmpdir, 'text')
    cache = {}
    text_cache = {}
    rows = []
    skipped_nup = []
    skipped_pairing = []
    for sample in samples:
        key = stem(sample)
        if key not in pmap:
            continue
        picked = pick_oracles(sample, pmap[key])
        counts = set()
        rejected = []
        for pdf in picked:
            if pdf not in cache:
                cache[pdf] = oracle_pages_and_text(pdf, tmp)
            pages, otext = cache[pdf]
            if not pages:
                continue
            # 디렉터리로 고른 짝은 그대로 믿는다. 이름만 같은 fallback 짝은, 그 정답지의
            # 디렉터리 주인이 따로 있을 때만 본문으로 견준다 — 확인은 export-text 를
            # 두 번 부르므로 전건에 걸면 생성이 몇 배 느려진다.
            owner = owners.get(pdf)
            if owner is None or owner == sample:
                counts.add(pages)
                continue
            if sample not in text_cache:
                text_cache[sample] = rhwp_text(args.rhwp, sample, textdir)
            if owner not in text_cache:
                text_cache[owner] = rhwp_text(args.rhwp, owner, textdir)
            mine, theirs = text_cache[sample], text_cache[owner]
            if mine is None or theirs is None:
                counts.add(pages)
                continue
            mine_share = text_share(mine, otext)
            owner_share = text_share(theirs, otext)
            if owner_claims_oracle(mine_share, owner_share):
                rejected.append((pdf, mine_share, owner, owner_share))
                continue
            counts.add(pages)
        if rejected:
            skipped_pairing.extend(
                (sample, pdf, share, owner, owner_share)
                for pdf, share, owner, owner_share in rejected)
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
    for sample, pdf, share, owner, owner_share in skipped_pairing:
        print('  오짝 제외: %s ← %s (본문 %.1f%%, 주인 %s 는 %.1f%%)'
              % (sample, pdf, share * 100, owner, owner_share * 100))
    print('기록: %s' % FIXTURE)
    shutil.rmtree(tmpdir, ignore_errors=True)
    return 0


if __name__ == '__main__':
    sys.exit(main())
