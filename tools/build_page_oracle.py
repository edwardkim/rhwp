"""#1658 대규모 페이지수 오라클 빌더.

hwpdocs 코퍼스에서 N개를 결정론적 랜덤 샘플 → 한글(OLE) PageCount 를 정답지로 수집해
render_page_gate.py fixture 호환 TSV(rel<TAB>hangul_pages)를 산출한다. 92건 controlset 과적합을
방지하고, 페이지네이션 엔진 개선을 대규모로 측정하기 위한 견고한 기준선.

사용:
    python tools/build_page_oracle.py --root C:/Users/planet/hwpdocs --sample 600 --seed 1658 \
        -o tests/fixtures/render_page_oracle_1658.tsv

요구: Windows + 한컴오피스 + pyhwpx.
"""
from __future__ import annotations

import argparse
import csv
import random
import subprocess
import sys
from pathlib import Path


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", type=Path, default=Path("C:/Users/planet/hwpdocs"))
    ap.add_argument("--sample", type=int, default=600)
    ap.add_argument("--seed", type=int, default=1658)
    ap.add_argument("--restart-every", type=int, default=300)
    ap.add_argument("-o", "--out", type=Path, required=True)
    args = ap.parse_args()

    try:
        from pyhwpx import Hwp
    except ImportError:
        print("오류: pyhwpx 미설치", file=sys.stderr)
        return 2

    files = [
        p
        for p in sorted(args.root.rglob("*"))
        if p.suffix.lower() in (".hwpx", ".hwp")
    ]
    if args.sample and len(files) > args.sample:
        rng = random.Random(args.seed)
        files = sorted(rng.sample(files, args.sample))
    print(f"대상 {len(files)} 파일 (seed={args.seed})")

    subprocess.run(["taskkill", "/F", "/IM", "Hwp.exe"], capture_output=True)
    hwp = Hwp(new=True, visible=False)

    args.out.parent.mkdir(parents=True, exist_ok=True)
    n_ok = n_err = 0
    with open(args.out, "w", encoding="utf-8", newline="") as fh:
        w = csv.writer(fh, delimiter="\t")
        w.writerow(["rel", "hangul_pages"])
        for i, f in enumerate(files):
            rel = str(f.relative_to(args.root)).replace("\\", "/")
            try:
                hwp.open(str(f))
                pages = int(hwp.PageCount)
                hwp.clear(option=1)
            except Exception as e:  # noqa: BLE001
                n_err += 1
                print(f"  ERR {rel}: {type(e).__name__}", file=sys.stderr)
                try:
                    subprocess.run(["taskkill", "/F", "/IM", "Hwp.exe"], capture_output=True)
                    hwp = Hwp(new=True, visible=False)
                except Exception:
                    pass
                continue
            if pages > 0:
                w.writerow([rel, pages])
                fh.flush()
                n_ok += 1
            if (i + 1) % args.restart_every == 0:
                try:
                    hwp.quit()
                    subprocess.run(["taskkill", "/F", "/IM", "Hwp.exe"], capture_output=True)
                    hwp = Hwp(new=True, visible=False)
                except Exception:
                    pass
    try:
        hwp.quit()
        subprocess.run(["taskkill", "/F", "/IM", "Hwp.exe"], capture_output=True)
    except Exception:
        pass
    print(f"\n완료: OK={n_ok} ERR={n_err} → {args.out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
