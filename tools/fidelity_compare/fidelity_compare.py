# 한컴 공식 PDF(정답지) vs rhwp 렌더 — 페이지별 대규모 비교 하네스
# 사용: python fidelity_harness.py <이름> <원본.hwp(x)> <정답지.pdf> <시작쪽> <끝쪽(포함)>
# 산출: work/<이름>/ 에 페이지별 비교 시트 + diff 점수 랭킹(report.tsv)
import os, sys, io, subprocess, base64, glob, re
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", errors="replace")
import pypdfium2 as pdfium
from PIL import Image, ImageChops

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
RHWP = os.environ.get("RHWP_BIN") or os.path.join(REPO, "target", "release-test", "rhwp.exe")
CH = os.environ.get("CHROME_BIN") or r"C:\Program Files\Google\Chrome\Application\chrome.exe"
# 배경 셸의 cp949 argv 인코딩이 한글 경로를 깨뜨리므로 ASCII 키 레지스트리를 쓴다.
BASE_SAMPLES = os.path.join(REPO, "samples")
# 한글 리터럴·NFC/NFD 정규화 문제를 피하려고 ASCII 글롭 패턴으로 해석한다.
REG = {
    "plan":   ("plan",    "2022* *.hwp",  "2022* *.pdf"),
    "manual": ("manual",  "2025 *.hwpx",  "2025 *.pdf"),
    "bunjang":("bunjang", "21868765*.hwp", "21868765*.pdf"),
    "korexam":("korexam", "21_*.hwp",       "21_*.pdf"),
    "math":   ("math",    "exam_math.hwp",  "exam_math.pdf"),
    "eng":    ("eng",     "exam_eng.hwp",   "exam_eng.pdf"),
}
key, P0, P1 = sys.argv[1], int(sys.argv[2]), int(sys.argv[3])
NAME, src_pat, pdf_pat = REG[key]
def resolve(pat):
    hits = sorted(glob.glob(os.path.join(BASE_SAMPLES, pat)), key=lambda h: (len(h), h))
    if not hits:
        raise SystemExit(f"글롭 미해결: {pat}")
    return hits[0]  # 최단 이름 우선 — 변형본(-2010 등)보다 원본을 고른다
SRC = resolve(src_pat)
PDF = resolve(pdf_pat)
W = os.path.join(REPO, "output", "fidelity", NAME)
os.makedirs(W, exist_ok=True)
SVGD = os.path.join(W, "svg")
PNG_W = 700  # 비교용 페이지 폭(px)

# ── 1) rhwp SVG 렌더 (요청 구간만) ──
os.makedirs(SVGD, exist_ok=True)
for p in range(P0, P1 + 1):
    if glob.glob(os.path.join(SVGD, f"*_{p+1:03}.svg")):
        continue
    cmd = [RHWP, "export-svg", SRC, "-p", str(p), "-o", SVGD]
    fp = os.environ.get("RHWP_FONT_PATH_DIR")
    if fp:
        cmd += ["--font-path", fp]
    subprocess.run(cmd, capture_output=True)

# ── 2) SVG → PNG (Chrome headless) — 창 크기를 SVG 실제 판형에 맞춘다 (B4 크롭 방지) ──
import math as _math
def svg_to_png(svg_path, out_png):
    if os.path.exists(out_png):
        return
    head = open(svg_path, encoding="utf-8", errors="ignore").read(600)
    mw = re.search(r'width="([0-9.]+)"', head)
    mh = re.search(r'height="([0-9.]+)"', head)
    w = _math.ceil(float(mw.group(1))) + 2 if mw else 810
    h = _math.ceil(float(mh.group(1))) + 2 if mh else 1140
    subprocess.run([CH, "--headless=new", "--disable-gpu", f"--screenshot={out_png}",
                    f"--window-size={w},{h}", "--hide-scrollbars",
                    f"file:///{svg_path}"], capture_output=True)

# ── 3) PDF 페이지 → PNG ──
pdf = pdfium.PdfDocument(PDF)
def pdf_to_png(p, out_png):
    if os.path.exists(out_png):
        return
    page = pdf[p]
    scale = PNG_W / page.get_size()[0]
    bmp = page.render(scale=scale * 72 / 72)
    img = bmp.to_pil()
    img.save(out_png)

# ── 4) 비교 + diff 점수 ──
def diff_score(a_png, b_png):
    a = Image.open(a_png).convert("L")
    b = Image.open(b_png).convert("L")
    h = min(a.height, b.height)
    a = a.resize((PNG_W, h)); b = b.resize((PNG_W, h))
    d = ImageChops.difference(a, b)
    hist = d.histogram()
    total = sum(hist)
    changed = sum(hist[16:])          # 미세 노이즈(<16) 무시
    return round(100.0 * changed / total, 2)

def sheet(title, left, right, out_png):
    def b64(p):
        return base64.b64encode(open(p, "rb").read()).decode()
    html = (f'<!doctype html><meta charset="utf-8"><style>body{{margin:0;background:#eee;'
            f'font-family:Malgun Gothic}}.t{{text-align:center;font-weight:700;padding:6px;font-size:15px}}'
            f'.r{{display:flex;gap:8px;padding:0 8px}}.c{{flex:1}}'
            f'.l{{text-align:center;font-size:12px;font-weight:600;padding:2px}}'
            f'img{{width:100%;border:1px solid #aaa;background:#fff}}</style>'
            f'<div class="t">{title}</div><div class="r">'
            f'<div class="c"><div class="l" style="color:#1a56db">한컴 공식 PDF (정답지)</div><img src="data:image/png;base64,{b64(left)}"></div>'
            f'<div class="c"><div class="l" style="color:#0e9f6e">rhwp 렌더</div><img src="data:image/png;base64,{b64(right)}"></div></div>')
    hp = os.path.join(W, "_s.html")
    open(hp, "w", encoding="utf-8").write(html)
    subprocess.run([CH, "--headless=new", "--disable-gpu", f"--screenshot={out_png}",
                    "--window-size=1440,1040", "--hide-scrollbars", f"file:///{hp}"],
                   capture_output=True)

rows = []
for p in range(P0, P1 + 1):
    svgs = glob.glob(os.path.join(SVGD, f"*_{p+1:03}.svg"))
    r_png = os.path.join(W, f"r{p:03}.png")
    g_png = os.path.join(W, f"g{p:03}.png")
    c_png = os.path.join(W, f"cmp-p{p:03}.png")
    if not svgs:
        rows.append((p, -1.0, "rhwp SVG 없음"))
        continue
    svg_to_png(svgs[0], r_png)
    pdf_to_png(p, g_png)
    if not (os.path.exists(r_png) and os.path.exists(g_png)):
        rows.append((p, -1.0, "PNG 실패"))
        continue
    score = diff_score(g_png, r_png)
    if not os.path.exists(c_png):
        sheet(f"{NAME} — p{p+1} (diff {score}%)", g_png, r_png, c_png)
    rows.append((p, score, ""))

rows.sort(key=lambda r: -r[1])
rep = os.path.join(W, "report.tsv")
with open(rep, "w", encoding="utf-8") as f:
    f.write("page\tdiff%\tnote\n")
    for p, sc, note in rows:
        f.write(f"{p+1}\t{sc}\t{note}\n")
print(f"완료: {P1-P0+1}쪽. diff 랭킹(top 8):")
for p, sc, note in rows[:8]:
    print(f"  p{p+1}: {sc}% {note}")
print("report:", rep)
