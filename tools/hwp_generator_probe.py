#!/usr/bin/env python
"""#2148 문서 생성기 마커 추출 — NO_LS 라벨 셀 선언-신뢰 vs 성장 판별 신호 후보.

HWP5 는 `\\x05HwpSummaryInformation`(OLE property set)의 last-author(PID 8)와
설명 문자열에, HWPX 는 `Contents/content.hpf` 의 `lastsaveby` 메타에 생성기 흔적이
남는다. 특정 생성기(opendoc=온나라 전자결재, 법제처 정부입법지원센터 법령안편집기,
한국법령정보원)는 선언-신뢰형 NO_LS 셀을, 국가법령정보센터·수동 편집(사람이름/User)
은 성장형을 만든다(#2148 코멘트 대조표 참조).

사용:
    python tools/hwp_generator_probe.py <file.hwp|file.hwpx> [...]
요구: olefile (HWP5), 표준 zipfile (HWPX).
"""
import re
import sys
import zipfile
from pathlib import Path

sys.stdout.reconfigure(encoding="utf-8", errors="replace")

# 알려진 선언-신뢰형 생성기 시그니처(부분 문자열)
DECLARED_TRUST_MARKERS = ("opendoc", "법령안편집기", "정부입법지원센터", "한국법령정보원")
GROWTH_MARKERS = ("국가법령정보센터",)


def hwpx_lastsaveby(path: str) -> str:
    try:
        with zipfile.ZipFile(path) as z:
            data = z.read("Contents/content.hpf").decode("utf-8", "replace")
        m = re.search(r'lastsaveby"\s+content="text">([^<]*)', data)
        return m.group(1) if m else ""
    except Exception:  # noqa: BLE001
        return ""


def hwp5_generator(path: str) -> str:
    try:
        import olefile
    except ImportError:
        return "(olefile 미설치)"
    try:
        ole = olefile.OleFileIO(path)
        raw = ole.openstream("\x05HwpSummaryInformation").read()
        ole.close()
        txt = raw.decode("utf-16-le", errors="replace")
        # 생성기 설명 문자열 우선, 없으면 last-author 근사 추출
        for mk in DECLARED_TRUST_MARKERS + GROWTH_MARKERS:
            if mk in txt:
                return mk
        strs = [s.strip() for s in re.findall(r"[가-힣A-Za-z0-9_.\- ]{3,}", txt) if s.strip()]
        return strs[0] if strs else ""
    except Exception as e:  # noqa: BLE001
        return f"(ERR {e})"


def classify(marker: str) -> str:
    ml = marker.lower()
    if any(m.lower() in ml for m in DECLARED_TRUST_MARKERS):
        return "선언신뢰?"
    if any(m in marker for m in GROWTH_MARKERS):
        return "성장?"
    return "미상(수동/plain)"


def main() -> int:
    if len(sys.argv) < 2:
        print(__doc__, file=sys.stderr)
        return 2
    for f in sys.argv[1:]:
        ext = Path(f).suffix.lower()
        marker = hwpx_lastsaveby(f) if ext == ".hwpx" else hwp5_generator(f)
        print(f"  [{classify(marker):14}] marker={marker!r:40}  {Path(f).name[:44]}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
