#!/usr/bin/env python3
"""PR #7200 경계 재현: 빈 선행 문단이 실제 표 정렬에 없는 높이를 더하는지 검사."""
import argparse
import json
from pathlib import Path
import subprocess


def walk(node):
    yield node
    for child in node.get("children", []):
        yield from walk(child)


def table(tree, width):
    matches = [n["bbox"] for n in walk(tree)
               if n.get("type") == "Table" and abs(n["bbox"]["w"] - width) < 0.5]
    assert len(matches) == 1, (width, matches)
    return matches[0]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--rhwp-bin", type=Path)
    parser.add_argument("--trees", type=Path, required=True,
                        help="실행 출력 폴더. 바이너리 생략 시 기존 Native/WASM tree 검사")
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[4]
    trees = {}
    for align in ("top", "center", "bottom"):
        folder = args.trees / f"empty-first-{align}"
        if args.rhwp_bin:
            source = root / "tests/fixtures/pr7200_empty_leading_paragraph" / f"empty-first-{align}.hwpx"
            subprocess.run([str(args.rhwp_bin.resolve()), "export-render-tree", str(source),
                            "-p", "0", "-o", str(folder)], check=True,
                           stdout=subprocess.DEVNULL)
        files = list(folder.glob("render_tree_001.json")) + list(folder.glob("render_tree/render_tree_001.json"))
        assert len(files) == 1, files
        trees[align] = json.loads(files[0].read_text())
    outer = table(trees["top"], 320)
    nested = table(trees["top"], 160)
    slack = outer["y"] + outer["h"] - nested["y"] - nested["h"]
    failed = False
    for align, fraction in (("center", 0.5), ("bottom", 1)):
        current = table(trees[align], 160)
        assert table(trees[align], 320) == outer
        actual = current["y"] - nested["y"]
        expected = slack * fraction
        passed = abs(actual - expected) < 0.6
        failed |= not passed
        print(f"{align}: actual={actual:.2f}px expected={expected:.2f}px "
              f"delta={actual-expected:.2f}px {'PASS' if passed else 'FAIL'}")
    return int(failed)


if __name__ == "__main__":
    raise SystemExit(main())
