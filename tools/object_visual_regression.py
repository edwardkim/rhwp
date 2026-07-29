"""개체 단위 시각/geometry 회귀 하니스 — rhwp 개체 배치를 한글(OLE) 권위 출력과 대조.

page/PI 레벨(`verify_pi_page_vs_hangul.py`)로는 잡히지 않는 **개체(표·그림) 단위** 배치 차이를
검출한다. rhwp 의 render-tree 에서 표 개체(pi/ci/rows/cols/bbox)를 추출하고, 한글은 COM→PDF→
fitz 로 페이지 래스터 + 이미지 bbox 를 얻어, 개체를 읽기순으로 매칭해:

  1) geometry delta (페이지·bbox·크기) TSV
  2) 개체별 rhwp↔한글 side-by-side 크롭 HTML 갤러리 (작업지시자 시각 판정)
  3) baseline 저장/비교 — rhwp 버전 간 개체 이동/크기변경 회귀 검출

좌표계: render-tree bbox 는 96 DPI px. 한글 PDF 는 pt(72 DPI) → 96 DPI 로 래스터하여 정합.

사용:
    # 한글 대조 + 시각 갤러리 + baseline 저장
    python tools/object_visual_regression.py <file.hwp> -o out/ovr --save-baseline
    # rhwp 버전 간 회귀 비교 (한글 불필요, 빠름)
    python tools/object_visual_regression.py <file.hwp> -o out/ovr --baseline out/ovr/baseline.json --no-hwp
    # rhwp 래스터 크롭까지 (native-skia 빌드 필요)
    python tools/object_visual_regression.py <file.hwp> -o out/ovr --rhwp-png
    # 원커맨드 before/after: 지정 ref 를 worktree 빌드해 baseline 자동 생성 → 현 트리와 대조
    python tools/object_visual_regression.py --preset ovr5 -o out/ovr --diff-against devel

요구: rhwp release 바이너리(+ --rhwp-png 시 native-skia). --no-hwp 아니면 Windows+한컴+pyhwpx+PyMuPDF.
--diff-against 는 cargo 빌드 2회(기준 ref + 현 트리)를 자동 수행 — 한컴 불필요, geometry 무회귀 전용.
"""
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RHWP = ROOT / "target" / "release" / ("rhwp.exe" if sys.platform == "win32" else "rhwp")
RHWP_SKIA = RHWP  # export-png 은 동일 바이너리(native-skia feature 빌드)
DPI = 96.0  # render-tree px 기준
PT2PX = DPI / 72.0

# OVR 관례 샘플 세트 — CONTRIBUTING "렌더링 PR 자가 검증" 기본 프리셋 (samples/ 기준 상대경로)
PRESETS = {
    "ovr5": ["KTX.hwp", "exam_math.hwp", "21_언어_기출_편집가능본.hwp", "aift.hwp", "biz_plan.hwp"],
}


def _ngrams(text: str, n: int = 3):
    """공백·구두점 제거 후 문자 n-gram 집합 — 내용 서명(언어 무관, 무공백 한글 대응)."""
    import re
    s = re.sub(r"\s+", "", text or "")
    s = re.sub(r"[·,()\[\]{}<>/\\|:;.\-—–_=+*~`\"'…]", "", s)
    if len(s) < n:
        return {s} if s else set()
    return {s[i:i + n] for i in range(len(s) - n + 1)}


def _jaccard(a: set, b: set) -> float:
    if not a or not b:
        return 0.0
    inter = len(a & b)
    return inter / len(a | b) if inter else 0.0


def git_head() -> str:
    try:
        return subprocess.run(["git", "rev-parse", "--short", "HEAD"],
                              capture_output=True, text=True, timeout=10).stdout.strip() or "?"
    except Exception:
        return "?"


# ---------------------------------------------------------------------------
# --diff-against: 기준 ref worktree 빌드 + baseline 자동 생성
# ---------------------------------------------------------------------------
def resolve_ref(ref: str) -> str | None:
    """ref → short sha. 로컬 ref 우선, 없으면 origin/<ref> 폴백."""
    for cand in (ref, f"origin/{ref}"):
        r = subprocess.run(["git", "rev-parse", "--verify", "--short", f"{cand}^{{commit}}"],
                           cwd=ROOT, capture_output=True, text=True, timeout=10)
        if r.returncode == 0:
            return r.stdout.strip()
    return None


def build_ref_binary(sha: str, build_root: Path):
    """sha 를 임시 worktree 로 체크아웃해 release 빌드. 반환 (binary, err).

    바이너리는 sha 별 캐시(build_root/target_<sha>)에 남아 재실행 시 빌드 생략.
    worktree 는 성공/실패 무관 항상 정리(잔재 없음).
    """
    import shutil
    import tempfile
    # cargo 는 CARGO_TARGET_DIR 상대경로를 자기 cwd(worktree) 기준으로 해석 — 절대경로로 고정
    target = build_root.resolve() / f"target_{sha}"
    binary = target / "release" / ("rhwp.exe" if sys.platform == "win32" else "rhwp")
    if binary.exists():
        return binary, None
    wt = Path(tempfile.mkdtemp(prefix=f"rhwp_ovr_{sha}_"))
    try:
        r = subprocess.run(["git", "worktree", "add", "--detach", str(wt), sha],
                           cwd=ROOT, capture_output=True, text=True, timeout=120)
        if r.returncode != 0:
            return None, f"worktree add 실패: {(r.stderr or '').strip()[:200]}"
        env = dict(os.environ, CARGO_TARGET_DIR=str(target))
        r = subprocess.run(["cargo", "build", "--release", "--bin", "rhwp"],
                           cwd=wt, env=env, capture_output=True, text=True, timeout=5400)
        if r.returncode != 0:
            return None, f"기준 ref cargo build 실패(rc={r.returncode}): {(r.stderr or '').strip()[-400:]}"
        return (binary, None) if binary.exists() else (None, "빌드 후 바이너리 없음")
    finally:
        subprocess.run(["git", "worktree", "remove", "--force", str(wt)],
                       cwd=ROOT, capture_output=True, timeout=60)
        shutil.rmtree(wt, ignore_errors=True)
        subprocess.run(["git", "worktree", "prune"], cwd=ROOT, capture_output=True, timeout=60)


def compare_objects(robj, bobj, tol):
    """현재 개체 vs baseline 개체 — 읽기순 인덱스 대응으로 page 이동/크기변경 검출."""
    regressions = []
    for i, o in enumerate(robj):
        b = bobj[i] if i < len(bobj) else None
        if b is None:
            regressions.append((i, "신규 개체", "", ""))
            continue
        dp = o["page"] - b["page"]
        dw = o["w"] - b["w"]
        dh = o["h"] - b["h"]
        if dp != 0 or abs(dw) > tol or abs(dh) > tol:
            regressions.append((i, f"page{dp:+d}", f"w{dw:+.1f}", f"h{dh:+.1f}"))
    for i in range(len(robj), len(bobj)):
        regressions.append((i, "개체 소실", "", ""))
    return regressions


def write_md_summary(path: Path, rows, cur_head: str, base_label: str, tol: float) -> str:
    """PR 본문에 붙이기 좋은 markdown 요약표. 반환값 = 파일에 쓴 내용."""
    lines = [f"### OVR 개체 무회귀 — 현재({cur_head}) vs {base_label} · tol ±{tol:g}px", "",
             "| 샘플 | 페이지 | 개체 | 회귀 | 상세 |", "|---|---|---|---|---|"]
    for r in rows:
        det = "; ".join(f"obj{reg[0]} " + " ".join(x for x in reg[1:] if x) for reg in r["regs"][:5]) or "-"
        if len(r["regs"]) > 5:
            det += f" 외 {len(r['regs']) - 5}건"
        lines.append(f"| {r['name']} | {r['bpages']}→{r['pages']} | {r['bn']}→{r['n']} "
                     f"| {len(r['regs'])} | {det} |")
    total = sum(len(r["regs"]) for r in rows)
    lines += ["", ("**합계: 회귀 0건 — 변경 범위 밖 개체 무이동**" if total == 0 else
                   f"**합계: 회귀 {total}건 — 개체 이동/크기변경 검출**")]
    text = "\n".join(lines)
    path.write_text(text + "\n", encoding="utf-8")
    return text


# ---------------------------------------------------------------------------
# rhwp 개체 추출 (render-tree)
# ---------------------------------------------------------------------------
def rhwp_objects(path: Path, outdir: Path, reuse: bool = False, rhwp: Path | None = None,
                 rtree: str = "rtree"):
    """rhwp export-render-tree → 표 개체 리스트(읽기순). 반환: (objects, page_count, err)."""
    rtdir = outdir / rtree
    rtdir.mkdir(parents=True, exist_ok=True)
    if not (reuse and any(rtdir.glob("render_tree_*.json"))):
        try:
            # export-render-tree 는 -o 를 디렉터리로 취급해 그 안에 render_tree_NNN.json 을 쓴다.
            r = subprocess.run([str(rhwp or RHWP), "export-render-tree", str(path), "-o", str(rtdir)],
                               capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=900)
        except Exception as e:  # noqa: BLE001
            return None, None, f"render-tree:{e}"
        if r.returncode != 0:
            return None, None, f"render-tree:rc={r.returncode}"
    files = sorted(rtdir.glob("render_tree_*.json"))
    if not files:
        return None, None, "render-tree:no-pages"

    objects = []

    def collect_text(node):
        """서브트리의 모든 TextRun 텍스트 연결 — 개체 내용 서명용."""
        out = []

        def w(n):
            if not isinstance(n, dict):
                return
            if n.get("type") == "TextRun" and n.get("text"):
                out.append(n["text"])
            for c in n.get("children", []) or []:
                w(c)
        w(node)
        return "".join(out)

    def walk(node, page, depth):
        if not isinstance(node, dict):
            return
        if node.get("type") == "Table":
            b = node.get("bbox", {})
            rows, cols = node.get("rows"), node.get("cols")
            # depth 0 = 외곽 RowBreak 컨테이너(매 페이지 반복) → 개체 아님, 스킵.
            # depth>=1 중첩만 추적. 1×1 = 그림/도형 프레임(image), 그 외 = 중첩표(table).
            if depth >= 1:
                kind = "image" if (rows == 1 and cols == 1) else "table"
                objects.append({
                    "kind": kind, "pi": node.get("pi"), "ci": node.get("ci"),
                    "rows": rows, "cols": cols, "depth": depth, "page": page,
                    "x": round(b.get("x", 0), 1), "y": round(b.get("y", 0), 1),
                    "w": round(b.get("w", 0), 1), "h": round(b.get("h", 0), 1),
                    "text": collect_text(node),
                })
            depth += 1  # 중첩표 깊이
        for c in node.get("children", []) or []:
            walk(c, page, depth)

    for i, f in enumerate(files):
        try:
            j = json.load(open(f, encoding="utf-8"))
        except Exception:  # noqa: BLE001
            continue
        walk(j, i + 1, 0)

    # 읽기순: page, y, x
    objects.sort(key=lambda o: (o["page"], o["y"], o["x"]))
    for idx, o in enumerate(objects):
        o["id"] = idx
    return objects, len(files), None


def rhwp_render_png(path: Path, outdir: Path, reuse: bool = False):
    """rhwp export-png → 페이지별 PNG. 반환 {page: Path} 또는 None."""
    pdir = outdir / "rhwp_png"
    pdir.mkdir(parents=True, exist_ok=True)
    if not (reuse and any(pdir.glob("*.png"))):
        try:
            r = subprocess.run([str(RHWP_SKIA), "export-png", str(path), "-o", str(pdir)],
                               capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=900)
        except Exception as e:  # noqa: BLE001
            return None, f"export-png:{e}"
        if r.returncode != 0:
            return None, f"export-png:rc ({(r.stderr or r.stdout)[:120].strip()})"
    pages = {}
    import re
    for p in sorted(pdir.glob("*.png")):
        # export-png 은 {stem}_{NNN}.png 로 저장 — 파일명의 "마지막" 숫자가 페이지 번호.
        # (파일명 stem 에 숫자가 있어도 접미 페이지 번호를 잡도록 last-match 사용)
        nums = re.findall(r"(\d+)", p.stem)
        if nums:
            pages[int(nums[-1])] = p
    return pages, None


# ---------------------------------------------------------------------------
# 한글 개체 추출 + 래스터 (COM→PDF→fitz)
# ---------------------------------------------------------------------------
def hwp_pdf_and_objects(path: Path, outdir: Path, reuse: bool = False):
    """한글 COM 으로 PDF 저장 후 fitz 로 페이지 래스터 + 이미지 bbox. 반환 (pages_png, objects, n, err)."""
    try:
        import fitz  # PyMuPDF
    except ImportError:
        return None, None, None, "fitz 미설치"
    pdf = outdir / "hwp_ref.pdf"
    if not (reuse and pdf.exists()):
        try:
            from pyhwpx import Hwp
            subprocess.run(["taskkill", "/F", "/IM", "Hwp.exe"], capture_output=True)
            hwp = Hwp(new=True, visible=False)
            hwp.open(str(path))
            hwp.SaveAs(str(pdf), "PDF")
            hwp.quit()
            subprocess.run(["taskkill", "/F", "/IM", "Hwp.exe"], capture_output=True)
        except Exception as e:  # noqa: BLE001
            return None, None, None, f"한글:{type(e).__name__}:{e}"
    if not pdf.exists():
        return None, None, None, "한글:PDF 미생성"

    doc = fitz.open(str(pdf))
    pdir = outdir / "hwp_png"
    pdir.mkdir(parents=True, exist_ok=True)
    mat = fitz.Matrix(PT2PX, PT2PX)  # 96 DPI 로 래스터 → render-tree px 정합
    pages_png = {}
    objects = []
    for i in range(doc.page_count):
        pg = doc[i]
        out = pdir / f"page_{i + 1:03d}.png"
        if not (reuse and out.exists()):
            pg.get_pixmap(matrix=mat).save(str(out))
        pages_png[i + 1] = out
        # 이미지 개체 bbox (pt → px)
        for img in pg.get_images(full=True):
            try:
                rects = pg.get_image_rects(img[0])
            except Exception:  # noqa: BLE001
                rects = []
            for rc in rects:
                objects.append({
                    "kind": "image", "page": i + 1,
                    "x": round(rc.x0 * PT2PX, 1), "y": round(rc.y0 * PT2PX, 1),
                    "w": round((rc.x1 - rc.x0) * PT2PX, 1), "h": round((rc.y1 - rc.y0) * PT2PX, 1),
                })
        # 표 영역 검출(PyMuPDF find_tables) — 한글 PDF 는 표 구조를 선으로 그리므로 검출 가능
        try:
            for tb in pg.find_tables().tables:
                x0, y0, x1, y1 = tb.bbox
                try:
                    cells = tb.extract()
                    txt = "".join(str(c) for row in cells for c in row if c)
                except Exception:  # noqa: BLE001
                    txt = ""
                objects.append({
                    "kind": "table", "page": i + 1, "rows": tb.row_count, "cols": tb.col_count,
                    "x": round(x0 * PT2PX, 1), "y": round(y0 * PT2PX, 1),
                    "w": round((x1 - x0) * PT2PX, 1), "h": round((y1 - y0) * PT2PX, 1),
                    "text": txt,
                })
        except Exception:  # noqa: BLE001
            pass
    objects.sort(key=lambda o: (o["page"], o["y"], o["x"]))
    return pages_png, objects, doc.page_count, None


# ---------------------------------------------------------------------------
# 크롭 + HTML 갤러리
# ---------------------------------------------------------------------------
def crop(png_by_page, obj, outdir, tag):
    try:
        from PIL import Image
    except ImportError:
        return None
    p = png_by_page.get(obj["page"])
    if not p or not Path(p).exists():
        return None
    try:
        im = Image.open(p)
        pad = 6
        box = (max(0, int(obj["x"] - pad)), max(0, int(obj["y"] - pad)),
               min(im.width, int(obj["x"] + obj["w"] + pad)),
               min(im.height, int(obj["y"] + obj["h"] + pad)))
        crop_dir = outdir / "crops"
        crop_dir.mkdir(parents=True, exist_ok=True)
        out = crop_dir / f"{tag}.png"
        im.crop(box).save(out)
        return out.relative_to(outdir).as_posix()
    except Exception:  # noqa: BLE001
        return None


def write_gallery(outdir, rows, head, meta):
    html = [f"<html><head><meta charset='utf-8'><title>개체 시각 회귀 {head}</title>",
            "<style>body{font-family:sans-serif}table{border-collapse:collapse}",
            "td,th{border:1px solid #ccc;padding:6px;vertical-align:top;font-size:12px}",
            "img{max-width:360px;border:1px solid #eee}.d{color:#c00;font-weight:bold}</style></head><body>",
            f"<h2>개체 단위 시각 회귀 — {head}</h2><p>{meta}</p>",
            "<table><tr><th>id</th><th>kind</th><th>rhwp (page/bbox)</th><th>한글 (page/bbox)</th>",
            "<th>Δ</th><th>rhwp crop</th><th>한글 crop</th></tr>"]
    for r in rows:
        html.append("<tr>" + "".join(f"<td>{c}</td>" for c in [
            r["id"], r["kind"], r["rhwp"], r["hwp"], f"<span class='d'>{r['delta']}</span>",
            f"<img src='{r['rc']}'>" if r["rc"] else "-",
            f"<img src='{r['hc']}'>" if r["hc"] else "-",
        ]) + "</tr>")
    html.append("</table></body></html>")
    (outdir / "gallery.html").write_text("\n".join(html), encoding="utf-8")


# ---------------------------------------------------------------------------
def run_diff_against(args, files, head) -> int:
    """원커맨드 before/after: ref worktree 빌드 → baseline 자동 생성 → 현 트리 대조 → md 요약."""
    sha = resolve_ref(args.diff_against)
    if not sha:
        print(f"오류: ref 해석 실패 — {args.diff_against} (git fetch origin {args.diff_against} 필요할 수 있음)",
              file=sys.stderr)
        return 2
    base_label = f"{args.diff_against}@{sha}"
    print(f"[base] {base_label} worktree 빌드 (최초 1회, 이후 캐시)…")
    # 캐시는 -o 와 무관한 고정 위치(/target/ 은 gitignore) — sha 단위로 재사용
    base_bin, err = build_ref_binary(sha, ROOT / "target" / "ovr-baseline")
    if err:
        print(f"오류: {err}", file=sys.stderr)
        return 2
    print("[cur] 현 트리 cargo build --release…")
    r = subprocess.run(["cargo", "build", "--release", "--bin", "rhwp"],
                       cwd=ROOT, capture_output=True, text=True, timeout=5400)
    if r.returncode != 0:
        print(f"오류: 현 트리 cargo build 실패(rc={r.returncode}): {(r.stderr or '').strip()[-400:]}",
              file=sys.stderr)
        return 2

    rows = []
    for f in files:
        sub = args.out if len(files) == 1 else args.out / f.stem
        sub.mkdir(parents=True, exist_ok=True)
        bobj, bpages, berr = rhwp_objects(f, sub, reuse=args.reuse, rhwp=base_bin, rtree="rtree_base")
        if berr:
            print(f"오류: {f.name} 기준측 {berr}", file=sys.stderr)
            return 2
        (sub / "baseline.json").write_text(
            json.dumps({"head": base_label, "pages": bpages, "objects": bobj},
                       ensure_ascii=False, indent=1), encoding="utf-8")
        robj, rpages, rerr = rhwp_objects(f, sub, reuse=args.reuse)
        if rerr:
            print(f"오류: {f.name} 현재측 {rerr}", file=sys.stderr)
            return 2
        regs = compare_objects(robj, bobj, args.tol)
        print(f"[{f.name}] 기준 {bpages}쪽/{len(bobj)}개체 → 현재 {rpages}쪽/{len(robj)}개체, 회귀 {len(regs)}건")
        rows.append({"name": f.name, "bpages": bpages, "pages": rpages,
                     "bn": len(bobj), "n": len(robj), "regs": regs})

    md = write_md_summary(args.out / "ovr_diff.md", rows, head, base_label, args.tol)
    print("\n" + md)
    print(f"\n[out] {args.out / 'ovr_diff.md'} (PR 본문에 붙여넣기용)")
    return 1 if any(r["regs"] for r in rows) else 0


def run_one(f: Path, outdir: Path, args, head) -> int:
    """단일 파일 legacy 흐름: baseline 저장/비교 + (옵션) 한글 대조·갤러리."""
    outdir.mkdir(parents=True, exist_ok=True)
    robj, rpages, rerr = rhwp_objects(f, outdir, reuse=args.reuse)
    if rerr:
        print(f"오류: {rerr}", file=sys.stderr)
        return 2
    print(f"[rhwp] {rpages}쪽, 개체(표) {len(robj)}개")

    # baseline 저장/비교 (rhwp-vs-rhwp 회귀)
    if args.save_baseline:
        (outdir / "baseline.json").write_text(
            json.dumps({"head": head, "pages": rpages, "objects": robj}, ensure_ascii=False, indent=1),
            encoding="utf-8")
        print(f"[baseline] 저장 → {outdir / 'baseline.json'}")
    regressions = []
    if args.baseline and args.baseline.exists():
        base = json.loads(args.baseline.read_text(encoding="utf-8"))
        bobj = base.get("objects", [])
        regressions = compare_objects(robj, bobj, args.tol)
        print(f"[regression] baseline({base.get('head')}) 대비 개체 회귀 {len(regressions)}건")
        for r in regressions[:30]:
            print("   obj", *r)

    # 한글 대조 + 시각 갤러리
    if not args.no_hwp:
        hpages_png, hobj, hn, herr = hwp_pdf_and_objects(f, outdir, reuse=args.reuse)
        if herr:
            print(f"경고: 한글 대조 실패 — {herr} (rhwp-only 진행)", file=sys.stderr)
            hobj, hpages_png, hn = [], {}, None
        else:
            print(f"[한글] {hn}쪽, 이미지 개체 {len(hobj)}개")

        rpng = {}
        if args.rhwp_png:
            rpng, perr = rhwp_render_png(f, outdir, reuse=args.reuse)
            if perr:
                print(f"경고: rhwp export-png 실패 — {perr}", file=sys.stderr)
                rpng = {}

        # 개체 매칭: 내용(문자 3-gram Jaccard) 우선, 텍스트 없으면 크기 기반 폴백.
        # 전폭 표들이 크기가 우연히 겹쳐도 셀 텍스트로 정확히 구분된다.
        for o in robj + hobj:
            o["_ng"] = _ngrams(o.get("text", ""))
        rows = []
        ci = 0
        for kind in ("table", "image"):
            rlist = [o for o in robj if o["kind"] == kind]
            hlist = list(enumerate(o for o in hobj if o["kind"] == kind))
            used = set()
            for ro in rlist:
                best, bestscore, bymethod = None, None, ""
                rtext = len(ro["_ng"]) >= 3
                for hi, ho in hlist:
                    if hi in used:
                        continue
                    if rtext and len(ho["_ng"]) >= 3:
                        # 내용 매칭: Jaccard 높을수록 좋음 → score(높을수록 좋음)
                        score = _jaccard(ro["_ng"], ho["_ng"])
                        method = "text"
                    else:
                        # 크기 폴백: -(면적차+종횡비차) (높을수록 좋음)
                        ra, ha = ro["w"] * ro["h"], ho["w"] * ho["h"]
                        acost = abs(ra - ha) / max(ra, ha, 1)
                        cost = acost + 0.3 * abs(ro["w"] / max(ro["h"], 1) - ho["w"] / max(ho["h"], 1))
                        score = -cost
                        method = "size"
                    if bestscore is None or score > bestscore:
                        best, bestscore, bymethod = (hi, ho), score, method
                ho = None
                # 임계: 내용 Jaccard>=0.12, 크기 폴백 cost<0.6(즉 score>-0.6)
                ok = best is not None and (
                    (bymethod == "text" and bestscore >= 0.12) or
                    (bymethod == "size" and bestscore > -0.6))
                label = ""
                if ok:
                    used.add(best[0]); ho = best[1]
                    label = f"J={bestscore:.2f}" if bymethod == "text" else f"size cost={-bestscore:.2f}"
                rc = crop(rpng, ro, outdir, f"rhwp_{kind}_{ci}") if rpng else None
                hc = crop(hpages_png, ho, outdir, f"hwp_{kind}_{ci}") if ho else None
                if ho:
                    delta = f"page {ro['page']}→{ho['page']} ({ho['page']-ro['page']:+d}), w{ho['w']-ro['w']:+.0f} h{ho['h']-ro['h']:+.0f} [{label}]"
                else:
                    delta = "rhwp에만(매칭 없음)"
                rows.append({
                    "id": ro["id"], "kind": kind,
                    "rhwp": f"p{ro['page']} ({ro['x']},{ro['y']}) {ro['w']}×{ro['h']}" + (f" {ro['rows']}×{ro['cols']}" if ro.get('rows') else ""),
                    "hwp": f"p{ho['page']} ({ho['x']},{ho['y']}) {ho['w']}×{ho['h']}" + (f" {ho.get('rows')}×{ho.get('cols')}" if ho and ho.get('rows') else "") if ho else "-",
                    "delta": delta, "rc": rc, "hc": hc,
                })
                ci += 1
            # 한글에만 있는 개체
            for hi, ho in hlist:
                if hi in used:
                    continue
                hc = crop(hpages_png, ho, outdir, f"hwp_{kind}_only_{hi}")
                rows.append({
                    "id": f"{kind}#{hi}", "kind": kind, "rhwp": "-",
                    "hwp": f"p{ho['page']} ({ho['x']},{ho['y']}) {ho['w']}×{ho['h']}" + (f" {ho.get('rows')}×{ho.get('cols')}" if ho.get('rows') else ""),
                    "delta": "한글에만(매칭 없음)", "rc": None, "hc": hc,
                })
        write_gallery(outdir, rows, head, f"{f.name} | rhwp {rpages}쪽 vs 한글 {hn}쪽")
        # TSV
        with open(outdir / "objects.tsv", "w", encoding="utf-8") as tf:
            tf.write("id\tkind\trhwp_page\trhwp_wxh\thwp_page\thwp_wxh\tdelta\n")
            for r in rows:
                tf.write(f"{r['id']}\t{r['kind']}\t{r['rhwp']}\t\t{r['hwp']}\t\t{r['delta']}\n")
        print(f"[out] gallery.html + objects.tsv → {outdir}")

    return 1 if regressions else 0


def main() -> int:
    ap = argparse.ArgumentParser(description="개체 단위 시각/geometry 회귀 (rhwp vs 한글)")
    ap.add_argument("files", type=Path, nargs="*", help="대상 HWP (여러 개 가능, --preset 과 병용 가능)")
    ap.add_argument("-o", "--out", type=Path, required=True)
    ap.add_argument("--preset", choices=sorted(PRESETS),
                    help="관례 샘플 세트 추가 — ovr5: KTX/exam_math/21_언어/aift/biz_plan")
    ap.add_argument("--diff-against", metavar="REF",
                    help="지정 ref(예: devel)를 임시 worktree 로 빌드해 baseline 자동 생성 후 "
                         "현 트리와 대조 — 원커맨드, 한컴 불필요, md 요약 출력")
    ap.add_argument("--baseline", type=Path, help="이전 rhwp 개체 geometry JSON 과 회귀 비교")
    ap.add_argument("--save-baseline", action="store_true", help="현재 rhwp 개체 geometry 를 baseline.json 으로 저장")
    ap.add_argument("--no-hwp", action="store_true", help="한글 대조 생략(rhwp-vs-baseline 회귀만)")
    ap.add_argument("--rhwp-png", action="store_true", help="rhwp export-png 래스터 크롭(native-skia 필요)")
    ap.add_argument("--reuse", action="store_true", help="기존 산출물(render-tree/PNG/PDF) 재사용 — 재렌더 생략")
    ap.add_argument("--tol", type=float, default=2.0, help="geometry 회귀 허용 오차(px)")
    args = ap.parse_args()

    files = list(args.files)
    if args.preset:
        files += [ROOT / "samples" / n for n in PRESETS[args.preset]]
    if not files:
        ap.error("대상 파일 또는 --preset 필요")
    missing = [f for f in files if not f.exists()]
    if missing:
        print("오류: 파일 없음 — " + ", ".join(str(m) for m in missing), file=sys.stderr)
        return 2
    if args.baseline and len(files) > 1:
        ap.error("--baseline 은 단일 파일 전용 — 다중 샘플 before/after 는 --diff-against 사용")

    args.out.mkdir(parents=True, exist_ok=True)
    head = git_head()

    if args.diff_against:
        return run_diff_against(args, files, head)

    if not RHWP.exists():
        print(f"오류: rhwp 바이너리 없음 {RHWP}", file=sys.stderr)
        return 2
    rc = 0
    for f in files:
        outdir = args.out if len(files) == 1 else args.out / f.stem
        if len(files) > 1:
            print(f"=== {f.name} ===")
        rc = max(rc, run_one(f, outdir, args, head))
    return rc


if __name__ == "__main__":
    sys.exit(main())
