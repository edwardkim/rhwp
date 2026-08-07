# -*- coding: utf-8 -*-
"""R55 대형 문서 규모 사다리 — 합성 HWPX 생성기 + 대표 명령 실측 러너.

로드맵 트랙 F R55(`mydocs/tech/agent_roadmap/track_f_scale_perf.md`)의 선행
준비물인 "규모 사다리 코퍼스(대형 합성 문서 생성기)"와, 그 코퍼스 위에서
대표 명령을 단(rung)별로 실행해 소요 시간·최대 메모리·종료 코드를 TSV 로
기록하는 러너를 한 파일로 묶었다.

합성 문서 골격은 저장소 fixture 관례(`tools/make_issue3765_fixture.py` 등)와
같이 `samples/tac-host-spacing.hwpx` 를 껍데기로 쓰고 `Contents/section0.xml`
만 갈아 끼운다. HWPX 는 zip+xml 이라 표준 라이브러리만으로 생성 가능하다.

두 축의 사다리를 만든다:

* 문단 축 — 단순 텍스트 문단을 N 개 복제 (기본 1k·5k·20k·50k)
* 표 축   — 5열 × N행 단일 표 (기본 100·1,000·5,000 행)

대표 명령 4종(기본): ``info --json`` / ``export-text --json`` /
``export-structure --json`` / ``export-pdf``. 앞 3종은 stdout 봉투만 받고,
export-pdf 는 실제 산출 파일을 쓴다(레이아웃+렌더 경로 실측).

사용법 (저장소 루트에서):

    python tools/scale_ladder.py                      # 기본 사다리 전체
    python tools/scale_ladder.py --rungs 1000,5000    # 문단 축만 축소
    python tools/scale_ladder.py --table-rungs 100    # 표 축 축소
    python tools/scale_ladder.py --commands info,export-text
    python tools/scale_ladder.py --timeout 120 --tsv output/scale_ladder/result.tsv

실행 바이너리는 환경변수 ``RHWP_BIN`` 또는 ``--bin`` 으로 지정한다. 둘 다
없으면 ``target/release-test/rhwp(.exe)`` → ``target/release/rhwp(.exe)``
순으로 탐색한다.

기록 항목: 문서 축·단·파일 크기·명령·벽시계 ms·최대 RSS(MB)·exit code.
최대 RSS 는 Windows 에서 ctypes 로 PeakWorkingSetSize 를 읽고(정확),
POSIX 에서는 ``resource.getrusage(RUSAGE_CHILDREN)`` 의 증분으로 근사한다.
한 단에서 타임아웃/실패한 명령은 더 큰 단에서 건너뛴다(사다리 원칙:
작은 단에서 죽은 명령이 큰 단에서 살아날 리 없다 — 시간 절약).

산출물은 전부 ``output/scale_ladder/`` (git-ignore 영역) 아래에 둔다.
"""
from __future__ import annotations

import argparse
import io
import os
import re
import subprocess
import sys
import time
import zipfile

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SEED = os.path.join(REPO_ROOT, "samples", "tac-host-spacing.hwpx")
DEFAULT_OUT_DIR = os.path.join(REPO_ROOT, "output", "scale_ladder")

DEFAULT_PARA_RUNGS = [1000, 5000, 20000, 50000]
DEFAULT_TABLE_RUNGS = [100, 1000, 5000]
TABLE_COLS = 5
DEFAULT_COMMANDS = ["info", "export-text", "export-structure", "export-pdf"]
DEFAULT_TIMEOUT = 120.0

# ---------------------------------------------------------------------------
# 합성 HWPX 생성
# ---------------------------------------------------------------------------

SEC_OPEN = (
    '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n'
    '<hs:sec xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section" '
    'xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph" '
    'xmlns:hc="http://www.hancom.co.kr/hwpml/2011/core">\n'
)

PARA_TEXT = (
    "문단 {i}: 대형 문서 규모 사다리 실측용 합성 문단이다. 공공 문서의 평균적인 "
    "한 줄 반 분량을 흉내 내기 위해 한글과 영문을 섞어 길이를 맞춘다. "
    "synthetic paragraph {i} for the R55 scale ladder measurement."
)


def _load_seed() -> dict:
    with zipfile.ZipFile(SEED) as z:
        return {n: z.read(n) for n in z.namelist()}


def _secpr_head(seed_section: str) -> str:
    """seed 첫 문단에서 secPr 부분만 취해 새 첫 문단을 만든다 (표·북마크 제거)."""
    m = re.search(r'(<hp:p id="1".*?</hp:secPr>)', seed_section, re.S)
    if not m:
        raise RuntimeError("seed 에서 secPr 문단을 찾지 못했다: %s" % SEED)
    return (
        m.group(1)
        + '<hp:ctrl><hp:colPr id="" type="NEWSPAPER" layout="LEFT" colCount="1" '
        'sameSz="1" sameGap="0"/></hp:ctrl>'
        + "<hp:t>R55 scale-ladder synthetic document</hp:t></hp:run></hp:p>\n"
    )


def _para(pid: int, text: str) -> str:
    return (
        '<hp:p id="%d" paraPrIDRef="0" styleIDRef="0" pageBreak="0" '
        'columnBreak="0" merged="0"><hp:run charPrIDRef="0"><hp:t>%s</hp:t>'
        "</hp:run></hp:p>\n" % (pid, text)
    )


def _table(pid: int, rows: int, cols: int) -> str:
    """5열 × rows 행 단일 표. 셀 텍스트는 '행r 열c' 짧은 값."""
    cell_w = 48000 // cols
    row_h = 1200
    buf = io.StringIO()
    buf.write(
        '<hp:p id="%d" paraPrIDRef="0" styleIDRef="0" pageBreak="0" '
        'columnBreak="0" merged="0"><hp:run charPrIDRef="0">' % pid
    )
    buf.write(
        '<hp:tbl id="%d" zOrder="0" numberingType="TABLE" textWrap="TOP_AND_BOTTOM" '
        'textFlow="BOTH_SIDES" lock="0" dropcapstyle="None" pageBreak="CELL" '
        'repeatHeader="0" rowCnt="%d" colCnt="%d" cellSpacing="0" '
        'borderFillIDRef="2" noAdjust="0">' % (pid * 10, rows, cols)
    )
    buf.write(
        '<hp:sz width="48000" widthRelTo="ABSOLUTE" height="%d" '
        'heightRelTo="ABSOLUTE" protect="0"/>' % (rows * row_h)
    )
    buf.write(
        '<hp:pos treatAsChar="1" affectLSpacing="0" flowWithText="1" '
        'allowOverlap="0" holdAnchorAndSO="0" vertRelTo="PARA" horzRelTo="PARA" '
        'vertAlign="TOP" horzAlign="LEFT" vertOffset="0" horzOffset="0"/>'
    )
    buf.write('<hp:outMargin left="283" right="283" top="283" bottom="283"/>')
    buf.write('<hp:inMargin left="141" right="141" top="141" bottom="141"/>')
    next_pid = pid + 1
    for r in range(rows):
        buf.write("<hp:tr>")
        for c in range(cols):
            buf.write(
                '<hp:tc name="" header="0" hasMargin="0" protect="0" '
                'editable="0" dirty="0" borderFillIDRef="2">'
                '<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" '
                'vertAlign="CENTER" linkListIDRef="0" linkListNextIDRef="0" '
                'textWidth="0" textHeight="0" hasTextRef="0" hasNumRef="0">'
            )
            buf.write(_para(next_pid, "행%d 열%d" % (r + 1, c + 1)).rstrip("\n"))
            next_pid += 1
            buf.write("</hp:subList>")
            buf.write(
                '<hp:cellAddr colAddr="%d" rowAddr="%d"/>'
                '<hp:cellSpan colSpan="1" rowSpan="1"/>'
                '<hp:cellSz width="%d" height="%d"/>'
                '<hp:cellMargin left="141" right="141" top="141" bottom="141"/>'
                % (c, r, cell_w, row_h)
            )
            buf.write("</hp:tc>")
        buf.write("</hp:tr>\n")
    buf.write("</hp:tbl></hp:run></hp:p>\n")
    return buf.getvalue()


def _write_hwpx(out_path: str, section_xml: str, seed_files: dict) -> None:
    with zipfile.ZipFile(out_path, "w", zipfile.ZIP_DEFLATED) as z:
        # mimetype 은 관례상 무압축 선두
        z.writestr("mimetype", seed_files["mimetype"], compress_type=zipfile.ZIP_STORED)
        for name, data in seed_files.items():
            if name in ("mimetype", "Contents/section0.xml"):
                continue
            z.writestr(name, data)
        z.writestr("Contents/section0.xml", section_xml)


def gen_para_doc(out_path: str, n_paras: int, seed_files: dict) -> None:
    seed_section = seed_files["Contents/section0.xml"].decode("utf-8")
    buf = io.StringIO()
    buf.write(SEC_OPEN)
    buf.write(_secpr_head(seed_section))
    for i in range(n_paras):
        buf.write(_para(i + 2, PARA_TEXT.format(i=i + 1)))
    buf.write("</hs:sec>\n")
    _write_hwpx(out_path, buf.getvalue(), seed_files)


def gen_table_doc(out_path: str, rows: int, seed_files: dict) -> None:
    seed_section = seed_files["Contents/section0.xml"].decode("utf-8")
    buf = io.StringIO()
    buf.write(SEC_OPEN)
    buf.write(_secpr_head(seed_section))
    buf.write(_table(2, rows, TABLE_COLS))
    buf.write("</hs:sec>\n")
    _write_hwpx(out_path, buf.getvalue(), seed_files)


# ---------------------------------------------------------------------------
# 실측 러너
# ---------------------------------------------------------------------------


def _find_bin(cli_bin: str | None) -> str:
    if cli_bin:
        return cli_bin
    env = os.environ.get("RHWP_BIN")
    if env:
        return env
    exe = ".exe" if os.name == "nt" else ""
    for rel in ("target/release-test/rhwp", "target/release/rhwp"):
        p = os.path.join(REPO_ROOT, *rel.split("/")) + exe
        if os.path.isfile(p):
            return p
    raise SystemExit(
        "rhwp 바이너리를 찾지 못했다 — RHWP_BIN 환경변수 또는 --bin 으로 지정하라"
    )


def _peak_rss_mb_windows(proc: subprocess.Popen) -> float | None:
    """종료된 자식의 PeakWorkingSetSize(MB). Popen 이 핸들을 쥐고 있는 동안 유효."""
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
        ok = psapi.GetProcessMemoryInfo(
            handle, ctypes.byref(counters), counters.cb
        )
        if not ok:
            return None
        return counters.PeakWorkingSetSize / (1024.0 * 1024.0)
    except Exception:
        return None


def _run_one(
    rhwp: str, args: list, timeout: float, cwd: str
) -> tuple[float, str, float | None]:
    """(elapsed_ms, exit표기, peak_rss_mb) — 타임아웃이면 exit='timeout'."""
    if os.name != "nt":
        import resource

        before = resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss
    t0 = time.perf_counter()
    proc = subprocess.Popen(
        [rhwp] + args,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        cwd=cwd,
    )
    try:
        proc.wait(timeout=timeout)
    except subprocess.TimeoutExpired:
        proc.kill()
        proc.wait()
        elapsed = (time.perf_counter() - t0) * 1000.0
        return elapsed, "timeout", None
    elapsed = (time.perf_counter() - t0) * 1000.0
    rss: float | None
    if os.name == "nt":
        rss = _peak_rss_mb_windows(proc)
    else:
        import resource

        after = resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss
        # Linux: KB / macOS: bytes — Linux 기준 KB 로 해석 (근사치임을 보고서에 명시)
        rss = max(0, after - before) / 1024.0 if after >= before else None
    return elapsed, str(proc.returncode), rss


def _cmd_args(command: str, doc_path: str, out_dir: str, tag: str) -> list:
    if command == "info":
        return ["info", doc_path, "--json"]
    if command == "export-text":
        return ["export-text", doc_path, "--json"]
    if command == "export-structure":
        return ["export-structure", doc_path, "--json"]
    if command == "export-pdf":
        return ["export-pdf", doc_path, "-o", os.path.join(out_dir, tag + ".pdf")]
    raise SystemExit("지원하지 않는 명령: %s" % command)


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--bin", help="rhwp 바이너리 경로 (기본: RHWP_BIN → target/)")
    ap.add_argument(
        "--rungs",
        default=",".join(str(r) for r in DEFAULT_PARA_RUNGS),
        help="문단 축 사다리 (쉼표 구분, 기본: %(default)s)",
    )
    ap.add_argument(
        "--table-rungs",
        default=",".join(str(r) for r in DEFAULT_TABLE_RUNGS),
        help="표 축 사다리 — 5열 × N행 (쉼표 구분, 기본: %(default)s)",
    )
    ap.add_argument(
        "--commands",
        default=",".join(DEFAULT_COMMANDS),
        help="실측할 명령 (쉼표 구분, 기본: %(default)s)",
    )
    ap.add_argument(
        "--timeout", type=float, default=DEFAULT_TIMEOUT,
        help="명령당 타임아웃 초 (기본: %(default)s)",
    )
    ap.add_argument(
        "--out-dir", default=DEFAULT_OUT_DIR,
        help="합성 문서·산출물 폴더 (기본: output/scale_ladder)",
    )
    ap.add_argument("--tsv", help="결과 TSV 저장 경로 (생략 시 stdout 만)")
    ap.add_argument(
        "--keep-docs", action="store_true",
        help="이미 생성된 합성 문서가 있으면 재생성하지 않는다",
    )
    opts = ap.parse_args()

    rhwp = _find_bin(opts.bin)
    if not os.path.isfile(rhwp):
        raise SystemExit("rhwp 바이너리가 없다: %s" % rhwp)
    os.makedirs(opts.out_dir, exist_ok=True)

    para_rungs = [int(x) for x in opts.rungs.split(",") if x.strip()]
    table_rungs = [int(x) for x in opts.table_rungs.split(",") if x.strip()]
    commands = [c.strip() for c in opts.commands.split(",") if c.strip()]

    seed_files = _load_seed()

    # (axis, rung, path) 목록 — 작은 단부터
    docs = []
    for n in sorted(para_rungs):
        path = os.path.join(opts.out_dir, "para_%d.hwpx" % n)
        if not (opts.keep_docs and os.path.isfile(path)):
            gen_para_doc(path, n, seed_files)
        docs.append(("para", n, path))
    for n in sorted(table_rungs):
        path = os.path.join(opts.out_dir, "table_%dx%d.hwpx" % (n, TABLE_COLS))
        if not (opts.keep_docs and os.path.isfile(path)):
            gen_table_doc(path, n, seed_files)
        docs.append(("table", n, path))

    header = ["axis", "rung", "doc_bytes", "command", "elapsed_ms", "peak_rss_mb", "exit"]
    rows = ["\t".join(header)]
    print("\t".join(header))
    dead: set = set()  # (axis, command) — 작은 단에서 timeout/실패한 명령
    for axis, rung, path in docs:
        size = os.path.getsize(path)
        tag = "%s_%d" % (axis, rung)
        for command in commands:
            key = (axis, command)
            if key in dead:
                row = [axis, str(rung), str(size), command, "-", "-", "skipped"]
                rows.append("\t".join(row))
                print("\t".join(row))
                continue
            args = _cmd_args(command, path, opts.out_dir, tag)
            elapsed, exit_code, rss = _run_one(rhwp, args, opts.timeout, REPO_ROOT)
            row = [
                axis,
                str(rung),
                str(size),
                command,
                "%.0f" % elapsed,
                "%.1f" % rss if rss is not None else "-",
                exit_code,
            ]
            rows.append("\t".join(row))
            print("\t".join(row))
            if exit_code != "0":
                dead.add(key)

    if opts.tsv:
        os.makedirs(os.path.dirname(os.path.abspath(opts.tsv)), exist_ok=True)
        with open(opts.tsv, "w", encoding="utf-8", newline="\n") as f:
            f.write("\n".join(rows) + "\n")
        print("TSV 저장: %s" % opts.tsv, file=sys.stderr)


if __name__ == "__main__":
    main()
