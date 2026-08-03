#!/usr/bin/env python
"""#3834 — 한글이 HWPX→HWP 변환에서 표 `flowWithText` 를 정규화하는가.

rhwp 의 HWPX→HWP5 변환기는 표 공통 속성 bit 13 을 무조건 켠다. 그것이 한컴의 저장
관례를 따른 것인지, 아니면 원본 `0` 을 파괴하는 결함인지는 **한글이 같은 변환을 했을 때
비트를 어떻게 두는지**로만 갈린다. 이 도구가 그 한 가지를 잰다.

한글 COM 으로 원본 HWPX 를 열어 HWP 로 저장한 뒤, 그 저장본을 `rhwp dump` 로 읽어
표 `attr` 의 bit 13 을 원본과 대조한다. 한글 COM 은 실패 뒤 인스턴스가 오염되므로
문서 1건당 자식 프로세스 하나로 격리한다.

사용:
  python tools/hangul_flowwithtext_oracle.py --exe target/debug/rhwp.exe --list repro.txt
  python tools/hangul_flowwithtext_oracle.py --child <입력.hwpx> <출력.hwp>   (내부용)
"""
from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
import tempfile

sys.stdout.reconfigure(encoding="utf-8", errors="replace")

FLOW_WITH_TEXT_BIT = 0x0000_2000
ATTR = re.compile(r"표:.*?attr=0x([0-9a-fA-F]{8})")


def child(src: str, dst: str) -> int:
    """한글로 src 를 열어 dst(HWP) 로 저장한다."""
    from pyhwpx import Hwp

    hwp = Hwp(new=True, visible=False)
    try:
        if not hwp.open(src):
            print("RESULT\t0")
            return 1
        hwp.save_as(dst, format="HWP")
        print(f"RESULT\t{int(os.path.exists(dst))}")
        hwp.clear(option=1)
    finally:
        try:
            hwp.quit()
        except Exception:  # noqa: BLE001
            pass
    return 0


def table_flow_bits(exe: str, path: str) -> list[bool]:
    """`rhwp dump` 에서 표마다 bit 13 을 읽는다."""
    out = subprocess.run([exe, "dump", path], capture_output=True, text=True,
                         encoding="utf-8", errors="replace", timeout=300)
    return [bool(int(m, 16) & FLOW_WITH_TEXT_BIT) for m in ATTR.findall(out.stdout)]


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--child", nargs=2, metavar=("SRC", "DST"))
    ap.add_argument("--exe", default="target/debug/rhwp.exe")
    ap.add_argument("--list", dest="lst")
    ap.add_argument("--timeout", type=int, default=180)
    a = ap.parse_args()

    if a.child:
        return child(*a.child)
    if not a.lst:
        ap.error("--list 또는 --child 가 필요하다")

    paths = [ln.strip() for ln in open(a.lst, encoding="utf-8") if ln.strip()]
    print("원본bit13\t한글저장bit13\t판정\t파일")
    for src in paths:
        before = table_flow_bits(a.exe, src)
        with tempfile.TemporaryDirectory() as td:
            dst = os.path.join(td, "hangul_saved.hwp")
            try:
                subprocess.run([sys.executable, __file__, "--child", src, dst],
                               capture_output=True, text=True, timeout=a.timeout)
            except subprocess.TimeoutExpired:
                print(f"{before}\t-\tTIMEOUT\t{os.path.basename(src)}")
                continue
            if not os.path.exists(dst):
                print(f"{before}\t-\tSAVE_FAIL\t{os.path.basename(src)}")
                continue
            after = table_flow_bits(a.exe, dst)
        if len(before) != len(after):
            verdict = "표수불일치"
        elif before == after:
            verdict = "보존"
        else:
            verdict = "정규화" if all(b for b in after) else "변동"
        print(f"{before}\t{after}\t{verdict}\t{os.path.basename(src)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
