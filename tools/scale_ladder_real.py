# -*- coding: utf-8 -*-
"""R55 2차 — 실물 코퍼스 규모 사다리 + 반복·분산 + 실물 증폭 한계 탐침.

1차([`tools/scale_ladder.py`](scale_ladder.py) + `mydocs/report/scale_ladder_r1_20260808.md`)
는 **합성** HWPX 사다리를 쟀고, 스스로 "합성 문서는 실물 공공문서가 아니다"·"각 단
1회 실행이다"를 미측정 항목으로 남겼다. 트랙 F R56 의 착수 게이트도 같은 구멍을
지목한다 — *"실물(합성 아님) 대형 문서에서 벽이 관측될 것"*.

이 2차 러너가 그 구멍 세 개를 메운다:

1. **실물 사다리** — `samples/` 의 진짜 HWP/HWPX 를 파일 크기순으로 세워 로그
   간격으로 뽑고(`--rungs`), 명령별 시간·최대 RSS·산출 봉투 크기·종료 코드를 잰다.
2. **반복·분산** — 단마다 `--repeats` 회 반복하고 원시값을 전부 남긴다. 표에는
   중앙값과 min~max 폭을 같이 낸다(단발 수치를 일반화하지 않기 위해).
3. **실물 증폭 탐침** — 실물 HWPX 의 본문(문단·표·개체 그대로)을 K 배로 복제해
   한계 근처를 민다(`--amplify`). 합성 문장 반복이 아니라 **실물 본문의 반복**이라
   1차의 균질 합성보다 실물 밀도에 가깝다. `id="N"` 만 사본별로 재번호한다.

결과 판정은 네 가지로 갈린다 — 서로 다른 실패 모드를 한 칸에 뭉개지 않는다:

* ``ok``      — exit 0, 임계시간 이내
* ``slow``    — exit 0 이지만 `--slow-threshold` 초 초과 (비현실적 시간)
* ``error``   — 0 이 아닌 정상 종료 코드 (도구가 스스로 판정해서 실패했다)
* ``crash``   — Windows 예외 종료 코드 (0xC0000005 등) 또는 음수 신호 종료
* ``timeout`` — `--timeout` 초 안에 안 끝남

산출물은 **NDJSON 원시 로그**(한 줄 = 한 번의 실행) + **사람이 읽는 마크다운 표**.
첫 줄은 항상 측정 환경 레코드(`record="env"`)라 원시 로그만으로 재현 맥락이 산다.

사용법 (저장소 루트에서):

    # 실물 사다리 8단 × 명령 4종 × 3회 반복
    python tools/scale_ladder_real.py --bin target/release-test/rhwp.exe \
        --ndjson output/scale_ladder_real/runs.ndjson \
        --md output/scale_ladder_real/table.md

    # 축소 재현 (빠름)
    python tools/scale_ladder_real.py --bin <경로> --rungs 4 --repeats 1 \
        --commands info,export-text

    # 실물 증폭 탐침 (한 문서를 2·4·8·16 배로 불려 한계를 민다)
    python tools/scale_ladder_real.py --bin <경로> --amplify samples/<파일>.hwpx \
        --amplify-factors 2,4,8,16 --skip-ladder

**측정 위생 (실측으로 배운 것)** — 이 러너를 **다른 빌드·측정과 동시에 돌리지
마라. 그리고 무거운 빌드 직후에 돌리지 마라.** 2026-08-16 실측에서, 같은 문서·같은
바이너리·같은 러너인데 다른 작업이 CPU 를 쓰던 시점의 수치가 유휴 시점보다
**2.6~3.0배 느리게** 나왔다(13.0MB 문서 `export-text` 3,850ms vs 1,730ms). 반복
3회의 ±폭은 이런 **구간 간 드리프트를 못 잡는다** — 한 삼각형 안에서는 셋 다 똑같이
느리기 때문이다. 반복 분산은 "이 순간의 잡음"만 재고, 머신 상태 변화는 못 잰다.
그래서 러너를 돌리기 전에 빌드·백그라운드 작업을 끝내고, 무거운 빌드 뒤에는 CPU 가
식을 시간을 준 뒤 시작하라.

바이너리는 `--bin` → 환경변수 `RHWP_BIN` → `target/release-test/rhwp(.exe)` →
`target/release/rhwp(.exe)` → `target/debug/rhwp(.exe)` 순으로 찾는다. **어느
프로필로 빌드된 바이너리인지는 러너가 알 수 없으므로 `--profile-label` 로
직접 적어 넣는다** — 이 값이 NDJSON 환경 레코드에 그대로 실린다.

표준 라이브러리만 쓴다(저장소의 다른 측정 스크립트 관례와 동일).
"""
from __future__ import annotations

import argparse
import hashlib
import io
import json
import os
import platform
import re
import statistics
import subprocess
import sys
import time
import zipfile

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SAMPLES = os.path.join(REPO_ROOT, "samples")
DEFAULT_OUT_DIR = os.path.join(REPO_ROOT, "output", "scale_ladder_real")

DEFAULT_COMMANDS = ["info", "export-text", "export-structure", "export-tables"]
DEFAULT_TIMEOUT = 180.0
DEFAULT_SLOW = 60.0
DEFAULT_RUNGS = 8
DEFAULT_REPEATS = 3

# Windows NTSTATUS 예외 종료 코드(자주 보는 것) — 크래시로 분류한다.
WIN_CRASH_CODES = {
    0xC0000005: "ACCESS_VIOLATION",
    0xC00000FD: "STACK_OVERFLOW",
    0xC0000409: "STACK_BUFFER_OVERRUN",
    0xC000001D: "ILLEGAL_INSTRUCTION",
    0xC0000374: "HEAP_CORRUPTION",
    0xC0000017: "NO_MEMORY",
    0xC00000FF: "SEGMENT_NOTIFICATION",
}


# ---------------------------------------------------------------------------
# 환경 채집
# ---------------------------------------------------------------------------


def _cpu_name() -> str:
    if os.name == "nt":
        try:
            import winreg

            key = winreg.OpenKey(
                winreg.HKEY_LOCAL_MACHINE,
                r"HARDWARE\DESCRIPTION\System\CentralProcessor\0",
            )
            return winreg.QueryValueEx(key, "ProcessorNameString")[0].strip()
        except Exception:
            pass
    return platform.processor() or platform.machine()


def _total_ram_mb() -> float | None:
    if os.name == "nt":
        try:
            import ctypes
            from ctypes import wintypes

            class MEMORYSTATUSEX(ctypes.Structure):
                _fields_ = [
                    ("dwLength", wintypes.DWORD),
                    ("dwMemoryLoad", wintypes.DWORD),
                    ("ullTotalPhys", ctypes.c_ulonglong),
                    ("ullAvailPhys", ctypes.c_ulonglong),
                    ("ullTotalPageFile", ctypes.c_ulonglong),
                    ("ullAvailPageFile", ctypes.c_ulonglong),
                    ("ullTotalVirtual", ctypes.c_ulonglong),
                    ("ullAvailVirtual", ctypes.c_ulonglong),
                    ("ullAvailExtendedVirtual", ctypes.c_ulonglong),
                ]

            st = MEMORYSTATUSEX()
            st.dwLength = ctypes.sizeof(st)
            if ctypes.windll.kernel32.GlobalMemoryStatusEx(ctypes.byref(st)):
                return st.ullTotalPhys / (1024.0 * 1024.0)
        except Exception:
            return None
    try:
        return os.sysconf("SC_PAGE_SIZE") * os.sysconf("SC_PHYS_PAGES") / (1024.0**2)
    except Exception:
        return None


def _sha256(path: str) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def _git(*args: str) -> str:
    try:
        out = subprocess.run(
            ["git"] + list(args),
            cwd=REPO_ROOT,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            text=True,
            timeout=30,
        )
        return out.stdout.strip()
    except Exception:
        return ""


def probe_env(rhwp: str, profile_label: str) -> dict:
    try:
        ver = subprocess.run(
            [rhwp, "--version"],
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            timeout=60,
        ).stdout.strip()
    except Exception as exc:  # pragma: no cover - 환경 사고
        ver = "<version 조회 실패: %s>" % exc
    return {
        "record": "env",
        "timestamp": time.strftime("%Y-%m-%dT%H:%M:%S%z"),
        "os": "%s %s" % (platform.system(), platform.version()),
        "cpu": _cpu_name(),
        "cpu_logical": os.cpu_count(),
        "ram_mb": _total_ram_mb(),
        "python": platform.python_version(),
        "rhwp_bin": os.path.abspath(rhwp),
        "rhwp_version": ver,
        "rhwp_bin_sha256": _sha256(rhwp),
        "rhwp_bin_bytes": os.path.getsize(rhwp),
        "build_profile": profile_label,
        "git_commit": _git("rev-parse", "HEAD"),
        "git_describe": _git("describe", "--always", "--dirty"),
        "git_dirty": bool(_git("status", "--porcelain")),
        "argv": sys.argv[1:],
    }


# ---------------------------------------------------------------------------
# 실행 + 계측
# ---------------------------------------------------------------------------


def _peak_rss_mb_windows(proc: subprocess.Popen) -> float | None:
    """종료된 자식의 PeakWorkingSetSize(MB). Popen 이 핸들을 쥔 동안만 유효.

    1차 러너(`tools/scale_ladder.py`)와 같은 방식 — 값의 뜻이 두 보고서에서
    같아야 비교가 되므로 의도적으로 동일하게 유지한다.
    """
    try:
        import ctypes
        from ctypes import wintypes

        class PROCESS_MEMORY_COUNTERS(ctypes.Structure):
            _fields_ = [
                ("cb", wintypes.DWORD),
                ("PageFaultCount", wintypes.DWORD),
                ("PeakWorkingSetSize", ctypes.c_size_t),
                ("WorkingSetSize", ctypes.c_size_t),
                ("QuotaPeakPagedPoolUsage", ctypes.c_size_t),
                ("QuotaPagedPoolUsage", ctypes.c_size_t),
                ("QuotaPeakNonPagedPoolUsage", ctypes.c_size_t),
                ("QuotaNonPagedPoolUsage", ctypes.c_size_t),
                ("PagefileUsage", ctypes.c_size_t),
                ("PeakPagefileUsage", ctypes.c_size_t),
            ]

        counters = PROCESS_MEMORY_COUNTERS()
        counters.cb = ctypes.sizeof(counters)
        handle = int(proc._handle)  # noqa: SLF001 - stdlib 가 쥔 프로세스 핸들
        psapi = ctypes.WinDLL("psapi")
        if not psapi.GetProcessMemoryInfo(handle, ctypes.byref(counters), counters.cb):
            return None
        return counters.PeakWorkingSetSize / (1024.0 * 1024.0)
    except Exception:
        return None


def classify(exit_code: int | None, elapsed_s: float, slow_threshold: float) -> tuple:
    """(verdict, detail) — 실패 모드를 뭉개지 않고 가른다."""
    if exit_code is None:
        return "timeout", ""
    if exit_code == 0:
        if elapsed_s > slow_threshold:
            return "slow", ">%.0fs" % slow_threshold
        return "ok", ""
    if os.name == "nt":
        unsigned = exit_code & 0xFFFFFFFF
        if unsigned in WIN_CRASH_CODES:
            return "crash", WIN_CRASH_CODES[unsigned]
        # 0xC0000000 대역 = NTSTATUS 예외. 목록에 없어도 크래시로 본다.
        if 0xC0000000 <= unsigned <= 0xCFFFFFFF:
            return "crash", "NTSTATUS 0x%08X" % unsigned
    elif exit_code < 0:
        return "crash", "signal %d" % (-exit_code)
    return "error", "exit %d" % exit_code


def _cmd_args(command: str, doc: str, out_path: str) -> list:
    """명령별 인자. 산출은 stdout 으로 받아 파일로 흘린다(봉투 크기 계측)."""
    if command == "info":
        return ["info", doc, "--json"]
    if command == "export-text":
        return ["export-text", doc, "--json"]
    if command == "export-structure":
        return ["export-structure", doc, "--json"]
    if command == "export-tables":
        return ["export-tables", doc, "--json"]
    if command == "search":
        return ["search", doc, "의", "--json"]
    if command == "digest":
        return ["digest", doc, "--json"]
    if command == "export-pdf":
        return ["export-pdf", doc, "-o", out_path]
    raise SystemExit("지원하지 않는 명령: %s" % command)


def run_once(
    rhwp: str, command: str, doc: str, out_dir: str, timeout: float, slow: float
) -> dict:
    tag = "%s__%s" % (
        re.sub(r"[^0-9A-Za-z_.-]", "_", os.path.basename(doc))[:60],
        command,
    )
    stdout_path = os.path.join(out_dir, tag + ".out")
    pdf_path = os.path.join(out_dir, tag + ".pdf")
    args = _cmd_args(command, doc, pdf_path)
    # 이전 반복이 남긴 산출물을 지운다 — 안 지우면 타임아웃 실행이 앞 회차의
    # 파일 크기를 자기 out_bytes 로 보고한다.
    for stale in (stdout_path, pdf_path):
        try:
            os.remove(stale)
        except OSError:
            pass
    with open(stdout_path, "wb") as sink:
        t0 = time.perf_counter()
        proc = subprocess.Popen(
            [rhwp] + args,
            stdout=sink,
            stderr=subprocess.DEVNULL,
            cwd=REPO_ROOT,
        )
        try:
            proc.wait(timeout=timeout)
            code: int | None = proc.returncode
        except subprocess.TimeoutExpired:
            proc.kill()
            proc.wait()
            code = None
        elapsed = time.perf_counter() - t0
        rss = _peak_rss_mb_windows(proc) if os.name == "nt" else None
    verdict, detail = classify(code, elapsed, slow)
    out_bytes = os.path.getsize(stdout_path) if os.path.isfile(stdout_path) else 0
    if command == "export-pdf" and os.path.isfile(pdf_path):
        out_bytes = os.path.getsize(pdf_path)
    try:
        os.remove(stdout_path)
    except OSError:
        pass
    return {
        "elapsed_ms": round(elapsed * 1000.0, 1),
        "peak_rss_mb": round(rss, 1) if rss is not None else None,
        "exit": code,
        "verdict": verdict,
        "detail": detail,
        "out_bytes": out_bytes,
    }


# ---------------------------------------------------------------------------
# 코퍼스 선정
# ---------------------------------------------------------------------------


def corpus(samples_dir: str, rungs: int, min_bytes: int) -> list:
    """samples/ 의 실물 문서를 크기순으로 세우고 로그 간격으로 rungs 개 뽑는다."""
    docs = []
    for name in sorted(os.listdir(samples_dir)):
        path = os.path.join(samples_dir, name)
        if not os.path.isfile(path):
            continue
        ext = os.path.splitext(name)[1].lower()
        if ext not in (".hwp", ".hwpx"):
            continue
        size = os.path.getsize(path)
        if size < min_bytes:
            continue
        docs.append((size, path, ext.lstrip(".")))
    docs.sort()
    if not docs:
        return []
    if rungs >= len(docs):
        return docs
    # 로그 크기 축에서 균등 — 작은 문서에 표본이 몰리지 않게.
    import math

    lo, hi = math.log(docs[0][0]), math.log(docs[-1][0])
    picked: dict = {}
    for i in range(rungs):
        target = lo + (hi - lo) * i / max(1, rungs - 1)
        best = min(docs, key=lambda d: abs(math.log(d[0]) - target))
        picked[best[1]] = best
    return sorted(picked.values())


# ---------------------------------------------------------------------------
# 실물 증폭 (HWPX 본문 K 배 복제)
# ---------------------------------------------------------------------------

_ID_ATTR = re.compile(r'(?<![A-Za-z])id="(\d+)"')


def _renumber(xml: str, offset: int) -> str:
    """`id="N"` 만 offset 만큼 민다. `*IDRef` 는 (?<![A-Za-z]) 로 걸러진다."""
    return _ID_ATTR.sub(lambda m: 'id="%d"' % (int(m.group(1)) + offset), xml)


_P_TAG = re.compile(r"<hp:p\b[^>]*?(/?)>|</hp:p\s*>")


def split_first_para(body: str) -> tuple:
    """`<hs:sec>` 안쪽 본문에서 **최상위** 첫 문단을 떼어 (첫문단, 나머지) 로 준다.

    첫 문단은 구역 설정(`<hp:secPr>`)을 품고 있어 복제하면 안 된다. 단순히
    처음 나오는 `</hp:p>` 로 자르면 표 셀(`<hp:subList>`) 안의 **중첩 문단**
    끝에서 잘려 XML 이 깨진다 — 그래서 깊이를 세며 짝을 맞춘다.
    """
    first = _P_TAG.search(body)
    if not first:
        return "", body
    depth = 0
    for tok in _P_TAG.finditer(body, first.start()):
        text = tok.group(0)
        if text.startswith("</"):
            depth -= 1
            if depth <= 0:
                return body[: tok.end()], body[tok.end() :]
        elif tok.group(1) == "/":  # 자기닫힘 <hp:p .../>
            if depth == 0:
                return body[: tok.end()], body[tok.end() :]
        else:
            depth += 1
    return body, ""


def amplify_hwpx(src: str, factor: int, out_path: str) -> dict:
    """실물 HWPX 의 section 본문을 factor 배로 복제한 문서를 만든다.

    첫 문단(secPr 를 품은 구역 설정 문단)은 한 번만 두고, 그 뒤 본문 전체를
    복제한다. 사본마다 `id="N"` 을 큰 오프셋으로 밀어 충돌을 없앤다.
    """
    with zipfile.ZipFile(src) as z:
        names = z.namelist()
        files = {n: z.read(n) for n in names}
    sec_names = [n for n in names if re.match(r"Contents/section\d+\.xml$", n)]
    if not sec_names:
        raise SystemExit("HWPX 에 Contents/sectionN.xml 이 없다: %s" % src)
    grown = 0
    for sec in sec_names:
        xml = files[sec].decode("utf-8")
        m = re.search(r"(<hs:sec\b[^>]*>)(.*)(</hs:sec>)", xml, re.S)
        if not m:
            raise SystemExit("section 루트를 못 찾았다: %s / %s" % (src, sec))
        head, body, tail = m.group(1), m.group(2), m.group(3)
        # 첫 문단(secPr 보유)은 복제 대상에서 뺀다 — 구역 설정이 중복되면 안 된다.
        first, rest = split_first_para(body)
        buf = io.StringIO()
        buf.write(xml[: m.start()])
        buf.write(head)
        buf.write(first)
        for k in range(factor):
            buf.write(_renumber(rest, k * 10_000_000))
        buf.write(tail)
        buf.write(xml[m.end() :])
        files[sec] = buf.getvalue().encode("utf-8")
        grown += 1
    with zipfile.ZipFile(out_path, "w", zipfile.ZIP_DEFLATED) as z:
        if "mimetype" in files:
            z.writestr("mimetype", files["mimetype"], compress_type=zipfile.ZIP_STORED)
        for name in names:
            if name == "mimetype":
                continue
            z.writestr(name, files[name])
    return {
        "sections_grown": grown,
        "zip_bytes": os.path.getsize(out_path),
        "section_xml_bytes": sum(len(files[n]) for n in sec_names),
    }


# ---------------------------------------------------------------------------
# 코퍼스 인벤토리 — 입력 바이트 vs 실제 작업량
# ---------------------------------------------------------------------------


def inventory_hwpx(path: str) -> dict | None:
    """HWPX 한 건의 zip 바이트 대 압축 해제 바이트·section XML 바이트.

    입력 크기 상한(예: 서비스 계층 `DEFAULT_MAX_BYTES`)은 **zip 바이트**에
    걸리는데 파서가 실제로 감당하는 양은 압축 해제 후 XML 이다. 그 배율이
    얼마나 벌어지는지가 "바이트 상한이 작업량을 묶는가"의 근거가 된다.
    """
    try:
        with zipfile.ZipFile(path) as z:
            infos = z.infolist()
    except Exception:
        return None
    total_raw = sum(i.file_size for i in infos)
    sec_raw = sum(
        i.file_size for i in infos if re.match(r"Contents/section\d+\.xml$", i.filename)
    )
    zip_bytes = os.path.getsize(path)
    return {
        "record": "inventory",
        "doc": os.path.basename(path),
        "zip_bytes": zip_bytes,
        "uncompressed_bytes": total_raw,
        "section_xml_bytes": sec_raw,
        "expansion_x": round(total_raw / zip_bytes, 2) if zip_bytes else None,
        "section_expansion_x": round(sec_raw / zip_bytes, 2) if zip_bytes else None,
        "entries": len(infos),
    }


# ---------------------------------------------------------------------------
# 표 만들기
# ---------------------------------------------------------------------------


def _human_bytes(n: int) -> str:
    for unit, div in (("GB", 1 << 30), ("MB", 1 << 20), ("KB", 1 << 10)):
        if n >= div:
            return "%.1f %s" % (n / div, unit)
    return "%d B" % n


def _cell(runs: list) -> str:
    """중앙값 + min~max 폭 + 판정. 실패는 판정만."""
    verdicts = {r["verdict"] for r in runs}
    if verdicts != {"ok"} and verdicts != {"ok", "slow"} and verdicts != {"slow"}:
        bad = sorted(v for v in verdicts if v not in ("ok", "slow"))
        detail = next((r["detail"] for r in runs if r["verdict"] in bad), "")
        return "**%s**%s" % (bad[0], (" (%s)" % detail if detail else ""))
    ms = [r["elapsed_ms"] for r in runs]
    rss = [r["peak_rss_mb"] for r in runs if r["peak_rss_mb"] is not None]
    med = statistics.median(ms)
    span = ""
    if len(ms) > 1 and med > 0:
        span = " ±%.0f%%" % (100.0 * (max(ms) - min(ms)) / 2.0 / med)
    txt = "%.0f%s" % (med, span)
    if rss:
        txt += " / %.0f" % statistics.median(rss)
    if "slow" in verdicts:
        txt += " **slow**"
    return txt


def build_table(results: dict, commands: list, rows: list, row_label: str) -> str:
    """results[(row_key, command)] = [run, ...]"""
    out = io.StringIO()
    out.write("| %s | 크기 | %s |\n" % (row_label, " | ".join(commands)))
    out.write("|---|---:|%s\n" % ("---:|" * len(commands)))
    for key, size in rows:
        cells = []
        for c in commands:
            runs = results.get((key, c))
            cells.append(_cell(runs) if runs else "-")
        out.write("| %s | %s | %s |\n" % (key, _human_bytes(size), " | ".join(cells)))
    return out.getvalue()


# ---------------------------------------------------------------------------
# main
# ---------------------------------------------------------------------------


def _find_bin(cli_bin: str | None) -> str:
    # 자식 프로세스를 cwd=REPO_ROOT 로 띄우므로 상대 경로 바이너리는 해석이
    # 엇갈린다(Windows CreateProcess). 항상 절대 경로로 못 박는다.
    if cli_bin:
        return os.path.abspath(cli_bin)
    env = os.environ.get("RHWP_BIN")
    if env:
        return os.path.abspath(env)
    exe = ".exe" if os.name == "nt" else ""
    for rel in ("target/release-test/rhwp", "target/release/rhwp", "target/debug/rhwp"):
        p = os.path.join(REPO_ROOT, *rel.split("/")) + exe
        if os.path.isfile(p):
            return p
    raise SystemExit("rhwp 바이너리를 못 찾았다 — --bin 또는 RHWP_BIN 으로 지정하라")


def main() -> None:
    # Windows 기본 콘솔은 cp949 라 한글 도움말·표의 기호에서 UnicodeEncodeError 가
    # 난다. 측정 도구가 인코딩으로 죽으면 안 되므로 UTF-8 로 못 박는다.
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")
        except Exception:
            pass
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--bin", help="rhwp 바이너리 (기본: RHWP_BIN → target/*)")
    ap.add_argument(
        "--profile-label",
        default="unknown",
        help="바이너리 빌드 프로필 표기 (예: release-test, debug). 러너가 알 수 "
        "없으므로 직접 적는다 — 결과 해석이 달라진다.",
    )
    ap.add_argument("--samples", default=SAMPLES, help="실물 코퍼스 폴더")
    ap.add_argument("--rungs", type=int, default=DEFAULT_RUNGS, help="실물 사다리 단 수")
    ap.add_argument(
        "--min-bytes", type=int, default=64 * 1024, help="이보다 작은 문서는 뺀다"
    )
    ap.add_argument(
        "--commands",
        default=",".join(DEFAULT_COMMANDS),
        help="측정할 명령 (기본: %(default)s)",
    )
    ap.add_argument("--repeats", type=int, default=DEFAULT_REPEATS, help="단·명령당 반복")
    ap.add_argument("--timeout", type=float, default=DEFAULT_TIMEOUT)
    ap.add_argument(
        "--slow-threshold",
        type=float,
        default=DEFAULT_SLOW,
        help="exit 0 이어도 이 초를 넘으면 slow 로 판정 (기본: %(default)s)",
    )
    ap.add_argument("--out-dir", default=DEFAULT_OUT_DIR)
    ap.add_argument("--ndjson", help="원시 실행 로그 NDJSON 경로")
    ap.add_argument("--md", help="사람이 읽는 마크다운 표 경로")
    ap.add_argument("--skip-ladder", action="store_true", help="실물 사다리를 건너뛴다")
    ap.add_argument(
        "--inventory",
        action="store_true",
        help="samples/ 의 HWPX 전건에 대해 zip 바이트 대 압축 해제 바이트 배율을 "
        "집계한다 (입력 바이트 상한이 작업량을 묶는지의 근거)",
    )
    ap.add_argument("--amplify", help="증폭 탐침의 씨앗 HWPX 경로")
    ap.add_argument(
        "--amplify-factors",
        default="2,4,8,16",
        help="증폭 배수 (쉼표 구분, 기본: %(default)s)",
    )
    opts = ap.parse_args()

    rhwp = _find_bin(opts.bin)
    if not os.path.isfile(rhwp):
        raise SystemExit("rhwp 바이너리가 없다: %s" % rhwp)
    os.makedirs(opts.out_dir, exist_ok=True)
    commands = [c.strip() for c in opts.commands.split(",") if c.strip()]

    sink = open(opts.ndjson, "w", encoding="utf-8", newline="\n") if opts.ndjson else None

    def emit(rec: dict) -> None:
        if sink:
            sink.write(json.dumps(rec, ensure_ascii=False) + "\n")
            sink.flush()

    env = probe_env(rhwp, opts.profile_label)
    emit(env)
    print("# 환경: %s / %s / %s / profile=%s"
          % (env["os"], env["cpu"], env["rhwp_version"], env["build_profile"]))

    md = io.StringIO()

    # --- 0) 코퍼스 인벤토리 --------------------------------------------------
    if opts.inventory:
        invs = []
        for name in sorted(os.listdir(opts.samples)):
            path = os.path.join(opts.samples, name)
            if os.path.isfile(path) and name.lower().endswith(".hwpx"):
                inv = inventory_hwpx(path)
                if inv:
                    emit(inv)
                    invs.append(inv)
        if invs:
            ratios = [i["expansion_x"] for i in invs if i["expansion_x"]]
            top = sorted(invs, key=lambda i: -(i["expansion_x"] or 0))[:5]
            biggest = max(invs, key=lambda i: i["zip_bytes"])
            md.write("### 코퍼스 인벤토리 — 입력 바이트 대 압축 해제 바이트 (HWPX %d건)\n\n"
                     % len(invs))
            md.write("| 지표 | 값 |\n|---|---|\n")
            md.write("| 최대 입력 파일 | `%s` — %s |\n"
                     % (biggest["doc"], _human_bytes(biggest["zip_bytes"])))
            md.write("| 팽창 배율 중앙값 | ×%.1f |\n" % statistics.median(ratios))
            md.write("| 팽창 배율 최대 | ×%.1f (`%s`) |\n" % (top[0]["expansion_x"], top[0]["doc"]))
            md.write("| 팽창 배율 최소 | ×%.1f |\n" % min(ratios))
            md.write("\n팽창 상위 5건:\n\n| 문서 | zip | 압축 해제 | 배율 |\n|---|---:|---:|---:|\n")
            for i in top:
                md.write("| `%s` | %s | %s | ×%.1f |\n"
                         % (i["doc"], _human_bytes(i["zip_bytes"]),
                            _human_bytes(i["uncompressed_bytes"]), i["expansion_x"]))
            md.write("\n")

    # --- 1) 실물 사다리 -----------------------------------------------------
    if not opts.skip_ladder:
        docs = corpus(opts.samples, opts.rungs, opts.min_bytes)
        if not docs:
            print("경고: 실물 코퍼스가 비었다 (%s)" % opts.samples, file=sys.stderr)
        results: dict = {}
        rows = []
        dead: set = set()
        for size, path, fmt in docs:
            key = os.path.basename(path)
            rows.append((key, size))
            for command in commands:
                if (fmt, command) in dead:
                    emit({"record": "run", "axis": "real", "doc": key, "doc_bytes": size,
                          "format": fmt, "command": command, "verdict": "skipped"})
                    continue
                runs = []
                for rep in range(opts.repeats):
                    r = run_once(rhwp, command, path, opts.out_dir, opts.timeout,
                                 opts.slow_threshold)
                    rec = {"record": "run", "axis": "real", "doc": key,
                           "doc_bytes": size, "format": fmt, "command": command,
                           "repeat": rep}
                    rec.update(r)
                    emit(rec)
                    runs.append(r)
                    print("real\t%s\t%s\t%s\trep%d\t%s\t%sms\t%s"
                          % (key, fmt, command, rep, r["verdict"], r["elapsed_ms"],
                             r["peak_rss_mb"]))
                    if r["verdict"] in ("timeout", "crash"):
                        break
                results[(key, command)] = runs
                if any(r["verdict"] in ("timeout", "crash") for r in runs):
                    dead.add((fmt, command))
        md.write("### 실물 코퍼스 사다리 (중앙값 ms ±폭 / 최대 RSS MB)\n\n")
        md.write(build_table(results, commands, rows, "문서"))
        md.write("\n")

    # --- 2) 실물 증폭 탐침 ---------------------------------------------------
    if opts.amplify:
        factors = [int(x) for x in opts.amplify_factors.split(",") if x.strip()]
        amp_results: dict = {}
        amp_rows = []
        dead2: set = set()
        base = os.path.abspath(opts.amplify)
        for f in sorted(factors):
            out_doc = os.path.join(opts.out_dir, "amp_x%d.hwpx" % f)
            meta = amplify_hwpx(base, f, out_doc)
            key = "×%d" % f
            size = meta["zip_bytes"]
            amp_rows.append((key, size))
            emit({"record": "amplify", "seed": os.path.basename(base), "factor": f,
                  "path": out_doc, **meta})
            print("# 증폭 %s: zip %s / section XML %s"
                  % (key, _human_bytes(size), _human_bytes(meta["section_xml_bytes"])))
            for command in commands:
                if command in dead2:
                    emit({"record": "run", "axis": "amplified", "doc": key,
                          "doc_bytes": size, "format": "hwpx", "command": command,
                          "verdict": "skipped"})
                    continue
                runs = []
                for rep in range(opts.repeats):
                    r = run_once(rhwp, command, out_doc, opts.out_dir, opts.timeout,
                                 opts.slow_threshold)
                    rec = {"record": "run", "axis": "amplified", "doc": key,
                           "doc_bytes": size,
                           "section_xml_bytes": meta["section_xml_bytes"],
                           "format": "hwpx", "command": command, "repeat": rep}
                    rec.update(r)
                    emit(rec)
                    runs.append(r)
                    print("amp\t%s\t%s\trep%d\t%s\t%sms\t%s"
                          % (key, command, rep, r["verdict"], r["elapsed_ms"],
                             r["peak_rss_mb"]))
                    if r["verdict"] in ("timeout", "crash", "slow"):
                        break
                amp_results[(key, command)] = runs
                if any(r["verdict"] in ("timeout", "crash") for r in runs):
                    dead2.add(command)
        md.write("### 실물 증폭 탐침 — 씨앗 `%s`\n\n" % os.path.basename(base))
        md.write(build_table(amp_results, commands, amp_rows, "배수"))
        md.write("\n")

    text = md.getvalue()
    print()
    print(text)
    if opts.md:
        os.makedirs(os.path.dirname(os.path.abspath(opts.md)), exist_ok=True)
        with open(opts.md, "w", encoding="utf-8", newline="\n") as f:
            f.write(text)
        print("표 저장: %s" % opts.md, file=sys.stderr)
    if sink:
        sink.close()
        print("NDJSON 저장: %s" % opts.ndjson, file=sys.stderr)


if __name__ == "__main__":
    main()
