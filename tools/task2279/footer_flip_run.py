# -*- coding: utf-8 -*-
"""#2279 선결1 — footer 흡수/분할 전환점 실측 러너.

문서 목록의 각 문서에 대해 footer_flip_worker(단일 인스턴스 이분탐색)를 돌리고,
rhwp dump-pages 슬랙(body/used/frame)과 결합해 전환점의 rhwp-slack 좌표
S* = slack0 - delta_px 를 산출한다. 2026-07-15 실측: S* 는 ~36.9px 에 균일
군집하며, 이탈치 = rhwp 슬랙 측정오차 (상세 #2279 이슈 코멘트).

usage: python tools/task2279/footer_flip_run.py --list docs.txt --tmpdir <dir> -o out.tsv
       [--exe target/debug/rhwp.exe]
docs.txt: 줄당 HWPX 경로 (탭 이후 컬럼은 라벨로 무시; tools/task2279/footer_cohort.tsv
= 리포 동봉 25건 코호트). 두 번째 문서가 warm-up 더미로 쓰임.
기준값: tools/task2279/flip_results_20260715.tsv (S* ~36.9px 균일 군집).

요구: Windows + 한컴오피스 + pyhwpx. stall-watchdog: heartbeat 90s 초과 시
Hwp.exe 강제 종료(외부 스레드) 후 재생성.
"""
import argparse
import json
import re
import subprocess
import sys
import threading
import time
from pathlib import Path

sys.stdout.reconfigure(encoding="utf-8")

PRECISION_HU = 10
HI_EXTEND_HU = 16000
STALL_SEC = 90

BODY = re.compile(r"body_area: x=[\d.]+ y=[\d.]+ w=[\d.]+ h=([\d.]+)")
USED = re.compile(r"단 0 \(items=\d+, used=([\d.]+)px")
FRAME = re.compile(r"Table\s+pi=\d+ ci=\d+\s+1x1\s+[\d.]+x([\d.]+)px\s+wrap=TopAndBottom")
NPAGE = re.compile(r"=== 페이지 ")
MARGIN_RE = re.compile(r'(<hp:margin[^>]*bottom=")(\d+)(")')

heartbeat = {"t": time.time(), "stop": False}


def watchdog():
    while not heartbeat["stop"]:
        if time.time() - heartbeat["t"] > STALL_SEC:
            subprocess.run(["taskkill", "/IM", "Hwp.exe", "/F"], capture_output=True)
            heartbeat["t"] = time.time()
        time.sleep(5)


class Oracle:
    def __init__(self, dummy: str):
        self.dummy = dummy
        self.hwp = None

    def ensure(self):
        if self.hwp is not None:
            return
        from pyhwpx import Hwp
        for d in (0, 15, 30, 60, 120):
            if d:
                time.sleep(d)
            try:
                heartbeat["t"] = time.time()
                self.hwp = Hwp(visible=False)
                heartbeat["t"] = time.time()
                self.hwp.open(self.dummy)  # warm-up
                return
            except Exception as e:  # noqa: BLE001
                print(f"  [oracle] create fail ({e!r}), backoff", flush=True)
                subprocess.run(["taskkill", "/IM", "Hwp.exe", "/F"], capture_output=True)
                self.hwp = None
        raise RuntimeError("Hwp instance creation failed after backoff")

    def drop(self):
        try:
            if self.hwp is not None:
                self.hwp.quit()
        except Exception:  # noqa: BLE001
            pass
        self.hwp = None
        subprocess.run(["taskkill", "/IM", "Hwp.exe", "/F"], capture_output=True)

    def page_count(self, path: str) -> int:
        for attempt in range(3):
            self.ensure()
            try:
                heartbeat["t"] = time.time()
                self.hwp.open(path)
                n = self.hwp.PageCount
                heartbeat["t"] = time.time()
                return n
            except Exception as e:  # noqa: BLE001
                print(f"  [oracle] open fail attempt{attempt} ({e!r})", flush=True)
                self.drop()
        raise RuntimeError(f"open failed x3: {path}")


def rhwp_probe(exe: str, path: str):
    out = subprocess.run(
        [exe, "dump-pages", path],
        capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=300,
    )
    body = used = frame = None
    n_pages = len(NPAGE.findall(out.stdout))
    for ln in out.stdout.splitlines():
        mb = BODY.search(ln)
        if mb and body is None:
            body = float(mb.group(1))
        mu = USED.search(ln)
        if mu and used is None:
            used = float(mu.group(1))
        mf = FRAME.search(ln)
        if mf:
            frame = float(mf.group(1))
    return n_pages, body, used, frame


def measure_doc(oracle: "Oracle", doc: str, tmpdir: Path):
    import zipfile

    doc_id = Path(doc).name.split("_")[0]
    sec = "Contents/section0.xml"
    with zipfile.ZipFile(doc) as zin:
        infos = zin.infolist()
        data = {i.filename: zin.read(i.filename) for i in infos}
    xml = data[sec].decode("utf-8")
    m = MARGIN_RE.search(xml)
    if not m:
        return {"doc": doc_id, "error": "margin not found"}
    margin0 = int(m.group(2))

    def variant(mval: int) -> str:
        out = tmpdir / f"{doc_id}_m{mval}.hwpx"
        if not out.exists():
            new_xml = xml[: m.start(2)] + str(mval) + xml[m.end(2):]
            with zipfile.ZipFile(out, "w") as zo:
                for zi in infos:
                    payload = new_xml.encode("utf-8") if zi.filename == sec else data[zi.filename]
                    zo.writestr(zi, payload, zi.compress_type)
        return str(out)

    cache: dict[int, int] = {}

    def pc(mval: int) -> int:
        if mval not in cache:
            cache[mval] = oracle.page_count(variant(mval))
        return cache[mval]

    res = {"doc": doc_id, "margin0": margin0}
    p0 = pc(margin0)
    res["p0_warm"] = p0
    if pc(0) < p0:
        split_pages, lo, hi = p0, 0, margin0
    elif pc(margin0 + HI_EXTEND_HU) > p0:
        split_pages, lo, hi = p0 + 1, margin0, margin0 + HI_EXTEND_HU
    else:
        res.update(flip="NOFLIP", pc_at_0=cache.get(0),
                   pc_at_hi=cache.get(margin0 + HI_EXTEND_HU), opens=len(cache))
        return res
    while hi - lo > PRECISION_HU:
        mid = (lo + hi) // 2
        if pc(mid) >= split_pages:
            hi = mid
        else:
            lo = mid
    m_star = (lo + hi) / 2.0
    res.update(flip="OK", split_pages=split_pages, m_star=m_star,
               delta_px=(m_star - margin0) / 75.0, opens=len(cache))
    return res


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--list", required=True, help="줄당 HWPX 절대경로")
    ap.add_argument("--tmpdir", required=True, help="변형본 생성 폴더")
    ap.add_argument("-o", "--out", required=True)
    ap.add_argument("--exe", default="target/debug/rhwp.exe")
    a = ap.parse_args()

    docs = [ln.split("\t")[0].strip()
            for ln in Path(a.list).read_text(encoding="utf-8-sig").splitlines()
            if ln.strip()]
    tmpdir = Path(a.tmpdir)
    tmpdir.mkdir(parents=True, exist_ok=True)
    out_path = Path(a.out)

    done: set[str] = set()
    cols = ["doc", "p0_warm", "rhwp_pages", "flip", "margin0", "m_star",
            "delta_px", "body", "used", "frame", "slack0", "s_star", "opens", "note"]
    if out_path.exists():
        for ln in out_path.read_text(encoding="utf-8").splitlines()[1:]:
            f = ln.split("\t")
            if len(f) > 3 and f[3] in ("OK", "NOFLIP"):
                done.add(f[0])
        fh = out_path.open("a", encoding="utf-8", newline="")
    else:
        fh = out_path.open("w", encoding="utf-8", newline="")
        fh.write("\t".join(cols) + "\n")
    fh.flush()

    threading.Thread(target=watchdog, daemon=True).start()
    dummy = docs[1] if len(docs) > 1 else docs[0]
    oracle = Oracle(dummy)

    for i, doc in enumerate(docs):
        did = Path(doc).name.split("_")[0]
        if did in done:
            continue
        try:
            n_pages, body, used, frame = rhwp_probe(a.exe, doc)
        except Exception:  # noqa: BLE001
            n_pages = body = used = frame = None
        slack0 = (body - frame) - used if None not in (body, used, frame) else None
        row = {"doc": did, "rhwp_pages": n_pages, "body": body, "used": used,
               "frame": frame, "slack0": slack0}
        try:
            row.update(measure_doc(oracle, doc, tmpdir))
        except Exception as e:  # noqa: BLE001
            row["note"] = f"ERR {e!r}"[:150]
            oracle.drop()
        if row.get("slack0") is not None and row.get("delta_px") is not None:
            row["s_star"] = row["slack0"] - row["delta_px"]
        fh.write("\t".join(str(row.get(c, "")) for c in cols) + "\n")
        fh.flush()
        print(f"[{i+1}/{len(docs)}] {did} p0={row.get('p0_warm')} rhwp={n_pages} "
              f"flip={row.get('flip')} slack0={slack0} s*={row.get('s_star')} {row.get('note','')}",
              flush=True)

    heartbeat["stop"] = True
    oracle.drop()
    fh.close()
    print(f"done -> {out_path}", flush=True)


if __name__ == "__main__":
    main()
