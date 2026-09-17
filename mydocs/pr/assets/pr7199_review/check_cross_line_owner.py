#!/usr/bin/env python3
"""PR #7199: 이전 줄 여백만 바꾸어 다음 줄 TAC 표의 y 불변성을 검사한다.

한컴 PDF 오라클 시험이 아니라 줄 소속 계약을 확인하는 합성 진단이다.
원본/생성 내역은 tests/fixtures/issue7150_cross_line_owner/README.md 참조.
"""
import argparse
import json
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[4]
FIXTURES = ROOT / 'tests/fixtures/issue7150_cross_line_owner'


def tables(node):
    if node.get('type') == 'Table':
        yield node
    for child in node.get('children', []):
        yield from tables(child)


def measure(binary, source, output):
    result = subprocess.run(
        [str(binary), 'export-render-tree', str(source), '-o', str(output)],
        capture_output=True, text=True,
    )
    if result.returncode:
        raise RuntimeError(result.stderr)
    tree = json.loads((output / 'render_tree_001.json').read_text())
    matches = [t for t in tables(tree) if t.get('ci') == 2
               and abs(t['bbox']['w'] - 252.3) < 0.2
               and abs(t['bbox']['h'] - 109.0) < 0.2]
    if len(matches) != 1:
        raise AssertionError(f'다음 줄 표 하나를 기대함: {matches}')
    return matches[0]['bbox']['y']


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('binary', type=Path)
    args = parser.parse_args()
    binary = args.binary.resolve()
    with tempfile.TemporaryDirectory(prefix='pr7199-line-owner-') as tmp:
        ys = [measure(binary, FIXTURES / f'previous_line_margin_{mt}.hwpx',
                      Path(tmp) / str(mt)) for mt in (140, 240)]
    delta = abs(ys[1] - ys[0])
    print(f'previous top margin 140/240 HU: next-line y={ys[0]:.1f}/{ys[1]:.1f}px; delta={delta:.1f}px')
    if delta > 0.1:
        print('FAIL: 이전 줄 여백이 다음 줄 표의 기준선에 유입됨')
        return 1
    print('PASS: 다음 줄 표의 y 불변')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
