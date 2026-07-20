# -*- coding: utf-8 -*-
"""#2373 잔여 트랙 — tac-host 가산/무가산 판별자용 한글 재저장 오라클.

각 문서를 한글 COM 으로 열어 HWPX 재저장(=한글 fresh 좌표가 lineseg 에 기록됨).
원본/재저장 lineseg vpos ladder 를 문단 단위로 대조해, tac-host 발동 문단(pi)의
스텝(다음 문단 vpos − 현 문단 vpos)이 한글 fresh 에서 얼마나 전진하는지 실측.

주의(resave-anchor-oracle 기법 함정): 쪽 경계에서 vpos 리셋 — 음수 스텝은 무효(SKIP).
"""
import os, re, sys, time, zipfile

sys.stdout.reconfigure(encoding="utf-8")

OUT_DIR = r"C:\Users\planet\rhwp\output\poc\task2373\resaved"
DOCS = [
    r"C:\Users\planet\hwpdocs\samples\노원소방서 현장대응단\36399374_결재문서본문_노원소방서 사고조사팀 간소화 운영 결과.hwpx",
    r"C:\Users\planet\hwpdocs\samples\미래공간기획관 도시활력담당관\36392557_결재문서본문_창동역·가산디지털단지역 펀스테이션기본 및 실시설계 용역 추진계획.hwpx",
]
SAMPLE_LIST = r"C:\Users\planet\rhwp\output\poc\survey10k_r16_20260719\sample10000.txt"
CAUSAL_PREFIXES = [
    "156534231", "156586235", "156602253", "156603956", "156620256",
    "156639641", "156676971", "156731730", "156768311", "82948_",
]


def collect_docs():
    docs = list(DOCS)
    with open(SAMPLE_LIST, encoding="utf-8") as fh:
        for l in fh:
            l = l.strip()
            name = os.path.basename(l)
            if any(name.startswith(p) for p in CAUSAL_PREFIXES):
                docs.append(l)
    return docs


def resave_all(docs):
    from pyhwpx import Hwp
    os.makedirs(OUT_DIR, exist_ok=True)
    hwp = Hwp(visible=False)
    out = {}
    try:
        for i, src in enumerate(docs):
            dst = os.path.join(OUT_DIR, f"{i:02d}_" + os.path.basename(src))
            if not dst.lower().endswith(".hwpx"):
                dst += ".hwpx"
            if os.path.exists(dst):
                out[src] = dst
                print(f"[{i}] cached {os.path.basename(dst)}")
                continue
            try:
                hwp.open(src, arg="versionwarning:false;forceopen:true")
                time.sleep(0.5)
                hwp.save_as(dst, format="HWPX")
                hwp.clear()
                out[src] = dst
                print(f"[{i}] resaved {os.path.basename(dst)}")
            except Exception as e:
                print(f"[{i}] ERR {os.path.basename(src)}: {e}")
    finally:
        try:
            hwp.quit()
        except Exception:
            pass
    return out


P_RE = re.compile(rb"<hp:p [^>]*?paraPrIDRef=\"(\d+)\"[^>]*>")
SEG_RE = re.compile(rb"<hp:lineseg [^>]*?vertpos=\"(-?\d+)\"[^>]*?textheight=\"(\d+)\"[^>]*?lineheight=\"(\d+)\"")


def ladder(hwpx_path):
    """구역별 문단 첫 lineseg (vertpos, textheight, lineheight) 리스트."""
    secs = []
    with zipfile.ZipFile(hwpx_path) as z:
        names = sorted(n for n in z.namelist() if re.match(r"Contents/section\d+\.xml", n))
        for n in names:
            data = z.read(n)
            paras = []
            # 문단 경계로 분할: <hp:p ...> 태그 위치 기준
            positions = [m.start() for m in re.finditer(rb"<hp:p [^>]*>", data)]
            positions.append(len(data))
            for a, b in zip(positions, positions[1:]):
                m = SEG_RE.search(data[a:b])
                if m:
                    paras.append((int(m.group(1)), int(m.group(2)), int(m.group(3))))
                else:
                    paras.append(None)
            secs.append(paras)
    return secs


def steps(secs, sec_idx, pi):
    """pi→다음 유효 문단 스텝(HWPUNIT). 음수(쪽 리셋)는 None."""
    try:
        paras = secs[sec_idx]
    except IndexError:
        return None
    if pi >= len(paras) or paras[pi] is None:
        return None
    cur = paras[pi][0]
    for j in range(pi + 1, len(paras)):
        if paras[j] is not None:
            nxt = paras[j][0]
            if nxt <= cur:
                return None  # 쪽 리셋/역행
            return nxt - cur
    return None


def main():
    docs = collect_docs()
    print(f"resave {len(docs)}건")
    saved = resave_all(docs)
    print("\n=== ladder 대조 (HWPUNIT; step = 다음 문단 vpos − 현 문단 vpos) ===")
    for src, dst in saved.items():
        try:
            lo, lr = ladder(src), ladder(dst)
        except Exception as e:
            print(f"!! {os.path.basename(src)}: {e}")
            continue
        name = os.path.basename(src)[:44]
        n_para_o = sum(len(s) for s in lo)
        n_para_r = sum(len(s) for s in lr)
        print(f"\n-- {name} (sec o={len(lo)}/r={len(lr)}, para o={n_para_o}/r={n_para_r})")
        # 전 문단 스텝 대조(구역0), 차이 있는 곳만 출력
        for pi in range(min(len(lo[0]), len(lr[0]))):
            so, sr = steps(lo, 0, pi), steps(lr, 0, pi)
            if so is None or sr is None:
                continue
            if abs(so - sr) > 60:  # ≈0.8px 이상 차이만
                th = lo[0][pi][2] if lo[0][pi] else -1
                print(f"   pi={pi} orig_step={so} fresh_step={sr} d={sr-so} lh={th}")

if __name__ == "__main__":
    main()
