#!/usr/bin/env python3
"""RHWP Agent Toolkit - 공통 라이브러리

워크플로 스크립트가 공유하는 기반:
- rhwp 바이너리 해석 (--rhwp-bin > RHWP_BIN 환경변수 > PATH)
- 서브프로세스 호출 (UTF-8 고정 — Windows 콘솔 cp949 오염 방지)
- JSON 봉투 / NDJSON 스트림 파싱
- 재독 검증 헬퍼 (누름틀 값 대조, CSV 격자 읽기)

표준 라이브러리만 사용한다.
"""

import csv
import io
import json
import os
import shutil
import subprocess
import sys
from enum import Enum
from pathlib import Path
from typing import Any, Dict, List, Optional, Sequence, Tuple

# 워크플로 공통 종료 코드 계약
EXIT_OK = 0          # 성공 — 산출물이 실제로 존재하고 재독 검증까지 통과
EXIT_RUNTIME = 1     # 실행/검증 실패 — 부분 실패 포함
EXIT_USAGE = 2       # 입력 오류 — 없는 파일, 잘못된 데이터, 잘못된 인자
EXIT_DIFFERENT = 3   # (distribution_verify 전용) 두 문서가 다름 — rhwp 검증 게이트(3)와 동일 계열


class ExitCode(Enum):
    """rhwp CLI 종료 코드 (참고용 매핑)"""
    SUCCESS = 0
    GENERAL_ERROR = 1
    USAGE_ERROR = 2
    VERIFY_GATE_FAIL = 3
    PAGE_COUNT_MISMATCH = 4


class ToolkitError(Exception):
    """워크플로를 중단시키는 오류. exit_code 로 종료 코드를 지정한다."""

    def __init__(self, message: str, exit_code: int = EXIT_RUNTIME):
        super().__init__(message)
        self.exit_code = exit_code


def ensure_utf8_stdio() -> None:
    """Windows 콘솔(cp949)에서 한글 출력이 UnicodeEncodeError 로 죽지 않게 한다."""
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")
        except (AttributeError, ValueError):
            pass


def resolve_rhwp(explicit: Optional[str] = None) -> str:
    """rhwp 바이너리 경로 해석: --rhwp-bin > RHWP_BIN > PATH 의 rhwp.

    해석 실패 시 ToolkitError(EXIT_USAGE).
    """
    candidate = explicit or os.environ.get("RHWP_BIN") or "rhwp"
    # 경로로 지정된 경우 실존 확인, 이름만 온 경우 PATH 검색
    if os.path.sep in candidate or (os.altsep and os.altsep in candidate):
        if not Path(candidate).is_file():
            raise ToolkitError(
                f"rhwp 바이너리를 찾을 수 없습니다: {candidate} "
                "(--rhwp-bin 또는 RHWP_BIN 확인)",
                EXIT_USAGE,
            )
        return candidate
    found = shutil.which(candidate)
    if not found:
        raise ToolkitError(
            f"PATH 에서 '{candidate}' 를 찾을 수 없습니다 "
            "(--rhwp-bin 또는 RHWP_BIN 으로 경로를 지정하세요)",
            EXIT_USAGE,
        )
    return found


def add_common_args(parser) -> None:
    """모든 워크플로가 공유하는 인자."""
    parser.add_argument(
        "--rhwp-bin",
        default=None,
        metavar="경로",
        help="rhwp 바이너리 경로 (기본: RHWP_BIN 환경변수 > PATH 의 rhwp)",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="요약을 JSON 한 줄로 stdout 에 출력",
    )
    parser.add_argument(
        "-v", "--verbose",
        action="store_true",
        help="실행하는 rhwp 명령을 stderr 로 보여준다",
    )


def ensure_output_absent(path: Path, label: str) -> None:
    """기존 산출물을 덮어쓰거나 실패 정리에서 지우지 않게 막는다."""
    if os.path.lexists(path):
        raise ToolkitError(
            f"{label}가 이미 존재합니다: {path} — 덮어쓰기를 허용하지 않습니다",
            EXIT_USAGE,
        )


class RhwpToolkit:
    """rhwp CLI 자동화 도구킷 — 서브프로세스 호출과 봉투 파싱."""

    def __init__(self, rhwp_binary: str = "rhwp", verbose: bool = False):
        self.rhwp_binary = rhwp_binary
        self.verbose = verbose

    def log(self, message: str, level: str = "INFO") -> None:
        if self.verbose or level in ("ERROR", "WARNING"):
            print(f"[{level}] {message}", file=sys.stderr)

    def run_command(
        self, args: Sequence[str], stdin_data: Optional[str] = None
    ) -> subprocess.CompletedProcess:
        """rhwp <args> 실행. UTF-8 입출력 고정 (rhwp 는 UTF-8 JSON 을 낸다)."""
        cmd = [self.rhwp_binary] + list(args)
        self.log("실행: " + " ".join(cmd))
        return subprocess.run(
            cmd,
            input=stdin_data,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            check=False,
        )

    def run_json(
        self,
        args: Sequence[str],
        ok_exits: Sequence[int] = (0,),
        usage_exits: Sequence[int] = (2,),
    ) -> Tuple[Dict[str, Any], int]:
        """--json 계약 명령 실행 → (봉투, 종료코드).

        ok_exits 밖의 종료 코드는 ToolkitError 로 승격한다
        (usage_exits 에 속하면 EXIT_USAGE, 그 외 EXIT_RUNTIME).
        """
        result = self.run_command(args)
        if result.returncode not in ok_exits:
            detail = (result.stderr or result.stdout or "").strip()
            code = EXIT_USAGE if result.returncode in usage_exits else EXIT_RUNTIME
            raise ToolkitError(
                f"rhwp {' '.join(args[:2])} 실패 (exit {result.returncode}): {detail}",
                code,
            )
        try:
            envelope = json.loads(result.stdout)
        except json.JSONDecodeError as e:
            raise ToolkitError(
                f"rhwp {' '.join(args[:2])} 출력이 JSON 이 아닙니다: {e}",
                EXIT_RUNTIME,
            )
        return envelope, result.returncode

    def run_ndjson(
        self, args: Sequence[str], stdin_data: str
    ) -> Tuple[List[Dict[str, Any]], int, str]:
        """batch 축 실행 → (NDJSON 레코드 목록, 종료코드, stderr 요약).

        batch 는 일부 실패 시 exit 1 이지만 성공 레코드는 유효하므로
        여기서는 오류로 승격하지 않고 그대로 돌려준다. exit 2(사용법)만 승격.
        """
        result = self.run_command(args, stdin_data=stdin_data)
        if result.returncode == 2:
            detail = (result.stderr or "").strip()
            raise ToolkitError(f"rhwp batch 사용법 오류: {detail}", EXIT_USAGE)
        records = parse_ndjson(result.stdout)
        return records, result.returncode, (result.stderr or "").strip()


def parse_ndjson(ndjson_str: str) -> List[Dict[str, Any]]:
    """NDJSON 파싱 — 깨진 줄은 오류로 승격한다 (조용히 버리면 유실이 숨는다)."""
    records: List[Dict[str, Any]] = []
    for line_no, line in enumerate(ndjson_str.splitlines(), 1):
        line = line.strip()
        if not line:
            continue
        try:
            records.append(json.loads(line))
        except json.JSONDecodeError as e:
            raise ToolkitError(
                f"NDJSON {line_no}번째 줄 파싱 실패: {e}: {line[:120]}",
                EXIT_RUNTIME,
            )
    return records


# ---------------------------------------------------------------- 재독 검증 헬퍼

def field_values_by_occurrence(
    fields_envelope: Dict[str, Any]
) -> Dict[Tuple[str, int], str]:
    """fields --json 봉투 → {(이름, 등장순번): 현재값}.

    fields[] 는 문서 순서이므로 같은 이름의 n번째 항목이 occurrence n 이다
    (fill-fields 봉투의 filled[].occurrence 와 같은 좌표계)."""
    seen: Dict[str, int] = {}
    values: Dict[Tuple[str, int], str] = {}
    for f in fields_envelope.get("fields", []):
        name = f.get("name", "")
        occ = seen.get(name, 0)
        seen[name] = occ + 1
        values[(name, occ)] = f.get("value", "")
    return values


def verify_filled(
    filled: List[Dict[str, Any]], reread_envelope: Dict[str, Any]
) -> List[Dict[str, Any]]:
    """fill-fields 봉투의 filled[] 를 재독 fields 봉투와 대조 → 불일치 목록."""
    actual = field_values_by_occurrence(reread_envelope)
    mismatches = []
    for entry in filled:
        key = (entry.get("name", ""), int(entry.get("occurrence", 0)))
        got = actual.get(key)
        want = entry.get("value", "")
        if got != want:
            mismatches.append(
                {
                    "name": key[0],
                    "occurrence": key[1],
                    "expected": want,
                    "actual": got,
                }
            )
    return mismatches


def read_csv_grid(path: Path) -> List[List[str]]:
    """RFC 4180 CSV 파일 → 행 목록 (UTF-8, 선두 BOM 허용)."""
    with io.open(path, "r", encoding="utf-8-sig", newline="") as fh:
        return [row for row in csv.reader(fh)]


# ---------------------------------------------------------------- 입력 수집

DEFAULT_EXTS = (".hwp", ".hwpx")


def collect_input_files(
    paths: Sequence[str], exts: Sequence[str] = DEFAULT_EXTS
) -> List[Path]:
    """파일/디렉터리 목록 → 대상 문서 파일 목록 (디렉터리는 재귀, 정렬 고정).

    존재하지 않는 경로는 ToolkitError(EXIT_USAGE)."""
    out: List[Path] = []
    lowered = tuple(e.lower() for e in exts)
    for raw in paths:
        p = Path(raw)
        if p.is_dir():
            out.extend(
                child
                for child in sorted(p.rglob("*"))
                if child.is_file() and child.suffix.lower() in lowered
            )
        elif p.is_file():
            out.append(p)
        else:
            raise ToolkitError(f"입력 경로가 없습니다: {raw}", EXIT_USAGE)
    return out


def emit_summary(summary: Dict[str, Any], as_json: bool, human_lines: List[str]) -> None:
    """--json 이면 요약 봉투 한 줄, 아니면 사람용 줄들을 stdout 으로."""
    if as_json:
        print(json.dumps(summary, ensure_ascii=False))
    else:
        for line in human_lines:
            print(line)
