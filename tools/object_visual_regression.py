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

요구: rhwp release 바이너리(+ --rhwp-png 시 native-skia). --no-hwp 아니면 Windows+한컴+pyhwpx+PyMuPDF.
"""
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RHWP = ROOT / "target" / "release" / ("rhwp.exe" if sys.platform == "win32" else "rhwp")
RHWP_SKIA = RHWP  # export-png 은 동일 바이너리(native-skia feature 빌드)
DPI = 96.0  # render-tree px 기준
PT2PX = DPI / 72.0


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
# rhwp 개체 추출 (render-tree)
# ---------------------------------------------------------------------------
def rhwp_objects(path: Path, outdir: Path, reuse: bool = False):
    """rhwp export-render-tree → 표 개체 리스트(읽기순). 반환: (objects, page_count, err)."""
    rtdir = outdir / "rtree"
    rtdir.mkdir(parents=True, exist_ok=True)
    if not (reuse and any(rtdir.glob("render_tree_*.json"))):
        try:
            # export-render-tree 는 -o 를 디렉터리로 취급해 그 안에 render_tree_NNN.json 을 쓴다.
            r = subprocess.run([str(RHWP), "export-render-tree", str(path), "-o", str(rtdir)],
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
def main() -> int:
    ap = argparse.ArgumentParser(description="개체 단위 시각/geometry 회귀 (rhwp vs 한글)")
    ap.add_argument("file", type=Path)
    ap.add_argument("-o", "--out", type=Path, required=True)
    ap.add_argument("--baseline", type=Path, help="이전 rhwp 개체 geometry JSON 과 회귀 비교")
    ap.add_argument("--save-baseline", action="store_true", help="현재 rhwp 개체 geometry 를 baseline.json 으로 저장")
    ap.add_argument("--no-hwp", action="store_true", help="한글 대조 생략(rhwp-vs-baseline 회귀만)")
    ap.add_argument("--rhwp-png", action="store_true", help="rhwp export-png 래스터 크롭(native-skia 필요)")
    ap.add_argument("--reuse", action="store_true", help="기존 산출물(render-tree/PNG/PDF) 재사용 — 재렌더 생략")
    ap.add_argument("--tol", type=float, default=2.0, help="geometry 회귀 허용 오차(px)")
    args = ap.parse_args()

    if not RHWP.exists():
        print(f"오류: rhwp 바이너리 없음 {RHWP}", file=sys.stderr)
        return 2
    args.out.mkdir(parents=True, exist_ok=True)
    head = git_head()

    robj, rpages, rerr = rhwp_objects(args.file, args.out, reuse=args.reuse)
    if rerr:
        print(f"오류: {rerr}", file=sys.stderr)
        return 2
    print(f"[rhwp] {rpages}쪽, 개체(표) {len(robj)}개")

    # baseline 저장/비교 (rhwp-vs-rhwp 회귀)
    if args.save_baseline:
        (args.out / "baseline.json").write_text(
            json.dumps({"head": head, "pages": rpages, "objects": robj}, ensure_ascii=False, indent=1),
            encoding="utf-8")
        print(f"[baseline] 저장 → {args.out / 'baseline.json'}")
    regressions = []
    if args.baseline and args.baseline.exists():
        base = json.loads(args.baseline.read_text(encoding="utf-8"))
        bobj = base.get("objects", [])
        for i, o in enumerate(robj):
            b = bobj[i] if i < len(bobj) else None
            if b is None:
                regressions.append((i, "신규 개체", "", ""))
                continue
            dp = o["page"] - b["page"]
            dw = o["w"] - b["w"]
            dh = o["h"] - b["h"]
            if dp != 0 or abs(dw) > args.tol or abs(dh) > args.tol:
                regressions.append((i, f"page{dp:+d}", f"w{dw:+.1f}", f"h{dh:+.1f}"))
        print(f"[regression] baseline({base.get('head')}) 대비 개체 회귀 {len(regressions)}건")
        for r in regressions[:30]:
            print("   obj", *r)

    # 한글 대조 + 시각 갤러리
    if not args.no_hwp:
        hpages_png, hobj, hn, herr = hwp_pdf_and_objects(args.file, args.out, reuse=args.reuse)
        if herr:
            print(f"경고: 한글 대조 실패 — {herr} (rhwp-only 진행)", file=sys.stderr)
            hobj, hpages_png, hn = [], {}, None
        else:
            print(f"[한글] {hn}쪽, 이미지 개체 {len(hobj)}개")

        rpng = {}
        if args.rhwp_png:
            rpng, perr = rhwp_render_png(args.file, args.out, reuse=args.reuse)
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
                rc = crop(rpng, ro, args.out, f"rhwp_{kind}_{ci}") if rpng else None
                hc = crop(hpages_png, ho, args.out, f"hwp_{kind}_{ci}") if ho else None
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
                hc = crop(hpages_png, ho, args.out, f"hwp_{kind}_only_{hi}")
                rows.append({
                    "id": f"{kind}#{hi}", "kind": kind, "rhwp": "-",
                    "hwp": f"p{ho['page']} ({ho['x']},{ho['y']}) {ho['w']}×{ho['h']}" + (f" {ho.get('rows')}×{ho.get('cols')}" if ho.get('rows') else ""),
                    "delta": "한글에만(매칭 없음)", "rc": None, "hc": hc,
                })
        write_gallery(args.out, rows, head, f"{args.file.name} | rhwp {rpages}쪽 vs 한글 {hn}쪽")
        # TSV
        with open(args.out / "objects.tsv", "w", encoding="utf-8") as f:
            f.write("id\tkind\trhwp_page\trhwp_wxh\thwp_page\thwp_wxh\tdelta\n")
            for r in rows:
                f.write(f"{r['id']}\t{r['kind']}\t{r['rhwp']}\t\t{r['hwp']}\t\t{r['delta']}\n")
        print(f"[out] gallery.html + objects.tsv → {args.out}")

    return 1 if regressions else 0


if __name__ == "__main__":
    sys.exit(main())
