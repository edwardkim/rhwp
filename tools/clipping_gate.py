"""#1658 시각 회귀 게이트 — controlset 전체 클리핑 baseline 대조.

render_page_gate(페이지수)가 못 잡는 **시각 회귀(본문 클리핑=데이터 손실)** 를 잡는다.
render 좌표/높이 변경(예: rowspan cut↔render 정합)이 byeolpyo4 클리핑을 고치려다 다른 문서에
클리핑을 유발하지 않는지 검증한다(한글-break 스냅이 18190781 에 33px 유발한 사례 방지).

원리: controlset(render_page_controlset.tsv)의 각 문서를 detect_table_clipping 으로 검사하여
(rel, clip_pages, max_overflow_px) baseline 을 만들고, 변경 후 재검하여 **클리핑 증가**(신규 클리핑
문서, 페이지 증가, overflow 증가)를 회귀로 판정한다.

사용:
  python tools/clipping_gate.py --save tests/fixtures/clipping_baseline.tsv   # baseline 생성
  python tools/clipping_gate.py --check tests/fixtures/clipping_baseline.tsv   # 회귀 검사(증가 시 exit 1)
  python tools/clipping_gate.py --check <baseline> --fixture <other.tsv> --exe <rhwp>
종료코드: 회귀(클리핑 증가) 1건↑ → 1.
"""
from __future__ import annotations

import argparse
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from detect_table_clipping import check_file, norm_path  # noqa: E402

REPO = Path(__file__).resolve().parent.parent
DEFAULT_FIXTURE = REPO / "tests" / "fixtures" / "render_page_controlset.tsv"
EPS = 1.0
OVERFLOW_TOL = 0.5  # px, baseline 대비 overflow 증가 허용오차(렌더 미세 변동)


def load_rels(fixture: Path) -> list[str]:
    """fixture 의 rel 열(1열) 목록 (헤더 제외)."""
    rels: list[str] = []
    for i, line in enumerate(fixture.read_text(encoding="utf-8").splitlines()):
        if i == 0 or not line.strip():
            continue
        rels.append(line.split("\t")[0])
    return rels


def measure(fixture: Path, exe: str) -> dict[str, tuple[int, float]]:
    """rel -> (clip_pages, max_overflow_px). 문서 부재/ERR 은 건너뛴다."""
    out: dict[str, tuple[int, float]] = {}
    rels = load_rels(fixture)
    for n, rel in enumerate(rels, 1):
        # 한글 파일명 NFC/NFD 정규화 차이로 Path.exists() 가 오탐하므로 pre-check 없이
        # rhwp(OS)가 직접 열게 한다. 진짜 부재면 check_file 이 ERR(-1) 반환.
        src = norm_path(REPO / rel)
        clipped, _pages, max_ov = check_file(src, exe, EPS)
        if clipped < 0:
            print(f"  ERR/없음 {rel}", file=sys.stderr)
            continue
        out[rel] = (clipped, round(max_ov, 1))
        if clipped > 0:
            print(f"  CLIP {clipped}p {max_ov:.1f}px  {rel}")
        if n % 20 == 0:
            print(f"  ...{n}/{len(rels)}", file=sys.stderr)
    return out


def save(baseline: dict[str, tuple[int, float]], path: Path) -> None:
    lines = ["rel\tclip_pages\tmax_overflow_px"]
    for rel in sorted(baseline):
        cp, ov = baseline[rel]
        lines.append(f"{rel}\t{cp}\t{ov}")
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def load_baseline(path: Path) -> dict[str, tuple[int, float]]:
    out: dict[str, tuple[int, float]] = {}
    for i, line in enumerate(path.read_text(encoding="utf-8").splitlines()):
        if i == 0 or not line.strip():
            continue
        rel, cp, ov = line.split("\t")
        out[rel] = (int(cp), float(ov))
    return out


def main() -> int:
    ap = argparse.ArgumentParser()
    g = ap.add_mutually_exclusive_group(required=True)
    g.add_argument("--save", type=Path, help="baseline 생성 경로")
    g.add_argument("--check", type=Path, help="baseline 대조(회귀 시 exit 1)")
    ap.add_argument("--fixture", type=Path, default=DEFAULT_FIXTURE)
    ap.add_argument("--exe", default="target/release/rhwp.exe"
                    if sys.platform == "win32" else "target/release/rhwp")
    a = ap.parse_args()

    cur = measure(a.fixture, a.exe)

    if a.save:
        save(cur, a.save)
        n_clip = sum(1 for cp, _ in cur.values() if cp > 0)
        print(f"\n[baseline 저장] 문서={len(cur)} 클리핑문서={n_clip} → {a.save}")
        return 0

    base = load_baseline(a.check)
    regressions: list[str] = []
    for rel, (cp, ov) in cur.items():
        bcp, bov = base.get(rel, (0, 0.0))
        if cp > bcp or ov > bov + OVERFLOW_TOL:
            regressions.append(f"{rel}: clip {bcp}p/{bov:.1f}px → {cp}p/{ov:.1f}px")
    improvements = sum(
        1 for rel, (cp, ov) in cur.items()
        if (cp, ov) < base.get(rel, (0, 0.0))
    )
    print(f"\n[clipping-gate] 문서={len(cur)} 개선={improvements} 회귀={len(regressions)}")
    for r in regressions:
        print(f"  회귀 ▲ {r}")
    return 1 if regressions else 0


if __name__ == "__main__":
    sys.exit(main())
