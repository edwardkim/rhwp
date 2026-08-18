"""gym 코퍼스 퍼징 발견 엔진 — 전 코퍼스 × 다명령 × 다변형을 병렬로 두들겨 rhwp 의
DoS(패닉·무한루프)를 **근본원인별로 클러스터링**한다.

## 왜 이 도구인가 (강건성 감사와의 분업)

`robustness.py`(#4814 / #5218)는 릴리스 **게이트**다 — 바운드된 부분집합으로
"패닉·행 0"을 강제해 회귀를 막는다. 이 도구는 그 앞단의 **발견 엔진**이다 —
전 코퍼스를 여러 명령·여러 손상으로 **exhaustive** 하게 두들겨 아직 안 고쳐진
DoS 를 찾아, 패닉을 **소스 위치(file:line)별로 묶어** "고쳐야 할 고유 버그
목록"을 낸다. 아무도 손으로 수백 문서를 수천 가지로 퍼징하지 않는다 —
에이전트가 이걸 돌려 rhwp 를 계속 경화한다.

- 패닉: stderr 의 `panicked at file:line` → 그 위치로 클러스터. 스택 오버플로·
  시그널·비-0 어보트도 별도 버킷.
- 무한루프: timeout → 명령별 버킷.
- 도구 예외(없는 바이너리·빈 코퍼스·읽기 실패)는 DoS 로 위장하지 않는다.
  분류해 보고하고 엔진은 산다.

카탈로그·분류·봉투 계약은 `gym/docs/fuzz_corpus.md` 가 정본이다. 시험은
`scripts/tests/test_gym_fuzz_corpus.py` 가 바이너리 없이 고정한다. 작업 기록은
`mydocs/working/gym_fuzz_corpus.md`.

무작위는 없다. 같은 바이트는 같은 (라벨, 바이트) 를 낸다.

## 사용

    python gym/tools/fuzz_corpus.py --bin target/debug/rhwp                    # 기본 명령·전 코퍼스
    python gym/tools/fuzz_corpus.py --bin <bin> --commands info,export-text    # 명령 지정
    python gym/tools/fuzz_corpus.py --bin <bin> --limit 40 --workers 8 --json  # 부분집합·기계용
"""

from __future__ import annotations

import argparse
import errno
import json
import os
import re
import subprocess
import sys
import tempfile
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

HERE = os.path.dirname(os.path.abspath(__file__))
GYM_ROOT = os.path.dirname(HERE)
REPO_ROOT = os.path.dirname(GYM_ROOT)
sys.path.insert(0, GYM_ROOT)

from core import runner  # noqa: E402

DEFAULT_COMMANDS = ["info", "export-text", "export-structure", "export-render-tree"]
PANIC_RE = re.compile(r"panicked at ([^\n]+?:\d+)")

REPORT_KIND = "gymFuzzCorpus"
SCHEMA_VERSION = "1.0"
SAMPLE_EXTS = (".hwp", ".hwpx", ".hml")
PROBE_HEAD = 160
TINY_MAX = 64
HUGE_MIN = 1_048_576
RUST_ABORT = 101
WINDOWS_EXCEPTION_MASK = 0xC0000000
SIGNAL_FLOOR = 132

OLE_MAGIC = b"\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1"
ZIP_LOCAL = b"PK\x03\x04"
ZIP_CD = b"PK\x01\x02"
ZIP_EOCD = b"PK\x05\x06"
HEADER_SMASH_PAT = b"\xde\xad\xbe\xef" * 16
HWP3_SIG = b"HWP Document File"
XOR_STRIDE_MASK = 0xA5
LENGTH_BOMB_U32 = b"\xff\xff\xff\x7f"
LENGTH_ZERO_U32 = b"\x00\x00\x00\x00"
LENGTH_ONE_U32 = b"\x01\x00\x00\x00"
I32_MIN_LE = b"\x00\x00\x00\x80"
U16_MAX_LE = b"\xff\xff"
UTF16_BOM_LE = b"\xff\xfe"
UTF8_OVERLONG_NUL = b"\xc0\x80"
CTRL_BYTE = 0x01
STRIPE_STEP = 16
XOR_STRIDE = 7

# 종료 코드. 0=DoS 없음, 1=패닉/행 발견, 2=도구 실패(바이너리 부재 등).
EXIT_OK = 0
EXIT_DOS = 1
EXIT_TOOL_FAILED = 2

# JSON 보고 고정 키. 시험이 이 집합을 계약으로 고정한다.
REPORT_KEYS = (
    "kind",
    "schemaVersion",
    "ok",
    "samplesTested",
    "totalSamples",
    "commands",
    "mutantsPerSample",
    "runsChecked",
    "distinctPanicSites",
    "panicClusters",
    "hangClusters",
    "unreadables",
    "probeErrors",
    "toolErrors",
    "emptyCorpus",
    "missingBin",
    "toolFailed",
    "inputShapes",
    "exit",
)

REPORT_LIST_KEYS = (
    "commands",
    "panicClusters",
    "hangClusters",
    "unreadables",
    "probeErrors",
    "toolErrors",
)
REPORT_INT_KEYS = (
    "samplesTested",
    "totalSamples",
    "mutantsPerSample",
    "runsChecked",
    "distinctPanicSites",
    "exit",
)
REPORT_BOOL_KEYS = ("ok", "emptyCorpus", "missingBin", "toolFailed")
SHAPE_KEYS = ("empty", "tiny", "normal", "huge")

# 패닉 문자열 표식. 소문자 비교. 깨끗한 CLI 오류 문구와 겹치지 않게 고른다.
PANIC_MARKERS = (
    "panicked",
    "stack overflow",
    "core dumped",
    "fatal runtime error",
    "sigsegv",
    "sigabrt",
    "sigill",
    "sigbus",
    "access violation",
    "segmentation fault",
    "illegal instruction",
    "abort trap",
)

TIMEOUT_MARKERS = (
    "timeout",
    "timed out",
    "time-out",
    "time expired",
    "deadline exceeded",
)

# 프로브가 내는 kind. hang 은 classify() 가 아니라 TimeoutExpired 가 낸다.
OUTCOME_KINDS = ("panic", "hang", "error", None)

# 프로브/도구 예외 kind. 시험과 문서가 같은 표를 본다.
EXCEPTION_KINDS = (
    "missing-bin",
    "empty-corpus",
    "unreadable",
    "permission",
    "timeout",
    "os-error",
    "type-error",
    "value-error",
    "decode-error",
    "invalid-timeout",
    "invalid-workers",
    "probe-error",
    "unexpected",
)

EXCEPTION_KIND_BY_TYPE = {
    FileNotFoundError: "missing-bin",
    PermissionError: "permission",
    TimeoutError: "timeout",
    subprocess.TimeoutExpired: "timeout",
    UnicodeError: "decode-error",
    UnicodeDecodeError: "decode-error",
    UnicodeEncodeError: "decode-error",
    ValueError: "value-error",
    TypeError: "type-error",
    KeyError: "value-error",
    IndexError: "value-error",
    AttributeError: "type-error",
    OSError: "os-error",
}

FATAL_EXCEPTIONS = (KeyboardInterrupt, SystemExit, MemoryError, GeneratorExit)

CATCHABLE_EXCEPTIONS = (
    FileNotFoundError,
    PermissionError,
    TimeoutError,
    subprocess.TimeoutExpired,
    UnicodeError,
    ValueError,
    TypeError,
    KeyError,
    IndexError,
    AttributeError,
    OSError,
    RuntimeError,
)

# 정상(비어 있지 않은) 입력에서 항상 나오기를 기대하는 원 라벨.
# 원본과 바이트가 같아지면 add() 가 버리므로 극소 입력에서는 일부만 남는다.
LEGACY_ALWAYS_LABELS = (
    "trunc5",
    "trunc25",
    "trunc50",
    "trunc75",
    "trunc95",
    "flip10",
    "flip30",
    "flip50",
    "flip70",
    "flip90",
    "biglen10",
    "biglen40",
    "biglen70",
)

# 2KiB 정상 입력에서 추가로 항상 나오는 확대 라벨. ZIP 조건부 라벨은 제외.
EXPANDED_ALWAYS_LABELS = (
    "trunc1",
    "trunc10",
    "trunc99",
    "flip0",
    "flip25",
    "flip75",
    "flip99",
    "chop-last",
    "cut-first",
    "zero-header",
    "header-smash",
    "rotate-header",
    "increment-header",
    "nibble-swap-head",
    "ole-trunc-tail",
    "ole-magic-poison",
    "ole-sector-shift-poison",
    "ff-run",
    "aa-run",
    "nul-mid",
    "00-run",
    "55-run",
    "utf16-nul-sprinkle",
    "utf16-bom-inject",
    "utf8-overlong",
    "ascii-ctrl-sprinkle",
    "path-sep-sprinkle",
    "zip-magic-inject",
    "length-zero30",
    "length-one60",
    "i32-min20",
    "u16-max12",
    "reverse-prefix",
    "swap-ends",
    "high-bit-stripe",
    "low-bit-stripe",
    "xor-stride7",
    "interleave-zero-head",
    "duplicate-prefix",
    "tail-over-head",
    "invert-tail-64",
    "complement-mid-32",
    "bit-rotate-head",
    "decrement-tail",
    "slide-window-left",
    "slide-window-right",
    "repeat-mid-block",
    "odd-length-chop",
    "splice-nul-mid",
    "crlf-inject",
    "pad-eof",
    "widen-gap",
    "shrink-gap",
    "hwp3-sig-inject",
)

FAMILY_IDS = (
    "empty",
    "truncate",
    "flip",
    "length",
    "header",
    "ole",
    "run",
    "unicode",
    "zip",
    "permute",
    "stripe",
    "splice",
    "hwp3",
)

# 변형 가족 카탈로그 — 문서·시험이 같은 표를 본다. 무작위 필드 없음.
MUTANT_CATALOG = (
    {
        "id": "empty-to-nul",
        "family": "empty",
        "when": "n==0",
        "why": "빈 입력은 위치 기반 절단/플립을 할 수 없다. NUL 한 바이트로 고정한다.",
    },
    {
        "id": "trunc",
        "family": "truncate",
        "when": "n>0",
        "why": "잘린 복합문서·ZIP 중앙 디렉터리 부재를 재현한다. 원 라벨 truncN.",
        "params": (1, 5, 10, 25, 50, 75, 95, 99),
    },
    {
        "id": "chop-last",
        "family": "truncate",
        "when": "n>=2",
        "why": "마지막 1바이트만 잘라 오프바이원 레코드 끝을 재현한다.",
    },
    {
        "id": "cut-first",
        "family": "truncate",
        "when": "n>=1",
        "why": "선두 1바이트를 버려 매직이 한 칸 밀린 파일을 재현한다.",
    },
    {
        "id": "flip",
        "family": "flip",
        "when": "n>0",
        "why": "헤더·본문·꼬리 근처 한 바이트를 뒤집어 매직/길이 필드를 오염한다.",
        "params": (0, 10, 25, 30, 50, 70, 75, 90, 99),
    },
    {
        "id": "biglen",
        "family": "length",
        "when": "n>=4",
        "why": "추정 길이 필드에 0x7FFFFFFF 를 넣어 할당 폭주를 유도한다. 원 라벨.",
        "params": (10, 40, 70),
    },
    {
        "id": "zero-header",
        "family": "header",
        "when": "n>0",
        "why": "선두 512바이트를 0 으로 지워 OLE/ZIP 매직을 없앤다.",
    },
    {
        "id": "header-smash",
        "family": "header",
        "when": "n>0",
        "why": "고정 DEADBEEF 패턴으로 헤더를 덮어 매직만 다른 손상과 구분한다.",
    },
    {
        "id": "rotate-header",
        "family": "header",
        "when": "n>=2",
        "why": "선두 8바이트를 한 칸 왼쪽으로 순환시켜 매직 순서를 깨뜨린다.",
    },
    {
        "id": "increment-header",
        "family": "header",
        "when": "n>0",
        "why": "선두 8바이트에 1 을 더해(랩어라운드) 매직을 미세 오염한다.",
    },
    {
        "id": "nibble-swap-head",
        "family": "header",
        "when": "n>0",
        "why": "선두 32바이트의 니블을 뒤집어 길이 필드의 상위/하위를 맞바꾼다.",
    },
    {
        "id": "ole-trunc-tail",
        "family": "ole",
        "when": "n>0",
        "why": "CFB 디렉터리/FAT 가 잘린 꼬리, 또는 잘린 OLE 매직을 심는다.",
    },
    {
        "id": "ole-magic-poison",
        "family": "ole",
        "when": "n>0",
        "why": "선두에 깨진 OLE 매직(원본 매직 XOR 0xFF)을 덮어쓴다.",
    },
    {
        "id": "ole-sector-shift-poison",
        "family": "ole",
        "when": "n>=32",
        "why": "CFB 섹터 시프트(오프셋 30)를 0xFFFF 로 바꿔 거대 섹터 할당을 유도한다.",
    },
    {
        "id": "ole-mini-fat-poison",
        "family": "ole",
        "when": "n>=72",
        "why": "CFB MiniFAT 시작 섹터(오프셋 60)를 0xFFFFFFFF 로 채운다.",
    },
    {
        "id": "ff-run",
        "family": "run",
        "when": "n>0",
        "why": "본문 1/3 지점에 0xFF 런을 넣어 길이·유니코드 필드를 포화시킨다.",
    },
    {
        "id": "aa-run",
        "family": "run",
        "when": "n>0",
        "why": "선두 1/4 에 0xAA 런을 넣어 0xFF 와 다른 비트 패턴을 본다.",
    },
    {
        "id": "nul-mid",
        "family": "run",
        "when": "n>0",
        "why": "한가운데 최대 64바이트를 NUL 로 지워 UTF-16 종료를 위조한다.",
    },
    {
        "id": "00-run",
        "family": "run",
        "when": "n>0",
        "why": "2/3 지점에 NUL 런을 넣어 레코드 조기 종료를 위조한다.",
    },
    {
        "id": "55-run",
        "family": "run",
        "when": "n>0",
        "why": "선두 1/5 에 0x55 런을 넣어 교차 비트 패턴을 본다.",
    },
    {
        "id": "utf16-nul-sprinkle",
        "family": "unicode",
        "when": "n>=2",
        "why": "짝수 오프셋에 U+0000 을 뿌려 UTF-16LE 본문 절단을 위조한다.",
    },
    {
        "id": "utf16-bom-inject",
        "family": "unicode",
        "when": "n>=2",
        "why": "선두에 UTF-16LE BOM 을 심어 인코딩 오인을 유도한다.",
    },
    {
        "id": "utf8-overlong",
        "family": "unicode",
        "when": "n>=2",
        "why": "1/5 지점에 overlong NUL(C0 80)을 심어 UTF-8 검증을 흔든다.",
    },
    {
        "id": "ascii-ctrl-sprinkle",
        "family": "unicode",
        "when": "n>0",
        "why": "고정 퍼센트 위치에 SOH(0x01)를 넣어 제어문자 경로를 본다.",
    },
    {
        "id": "path-sep-sprinkle",
        "family": "unicode",
        "when": "n>0",
        "why": "고정 위치에 경로 구분자(\\/)를 심어 경로 해석 오류를 유도한다.",
    },
    {
        "id": "zip-local-header-flip",
        "family": "zip",
        "when": "ZIP 로컬 헤더가 있을 때",
        "why": "HWPX 의 PK\\x03\\x04 만 뒤집어 아카이브 인식을 깨뜨린다.",
    },
    {
        "id": "zip-magic-inject",
        "family": "zip",
        "when": "선두가 ZIP 로컬 헤더가 아닐 때",
        "why": "비-ZIP 문서 선두에 PK 매직을 심어 형식 오인을 유도한다.",
    },
    {
        "id": "zip-cd-magic-flip",
        "family": "zip",
        "when": "ZIP 중앙 디렉터리 매직이 있을 때",
        "why": "PK\\x01\\x02 만 뒤집어 중앙 디렉터리 탐색을 깨뜨린다.",
    },
    {
        "id": "zip-eocd-flip",
        "family": "zip",
        "when": "ZIP EOCD 매직이 있을 때",
        "why": "PK\\x05\\x06 만 뒤집어 아카이브 끝 탐색을 깨뜨린다.",
    },
    {
        "id": "length-zero",
        "family": "length",
        "when": "n>=4",
        "why": "추정 길이 필드에 0 을 넣어 빈 레코드 조기 종료를 유도한다.",
        "params": (30,),
    },
    {
        "id": "length-one",
        "family": "length",
        "when": "n>=4",
        "why": "추정 길이 필드에 1 을 넣어 오프바이원 슬라이스를 유도한다.",
        "params": (60,),
    },
    {
        "id": "i32-min",
        "family": "length",
        "when": "n>=4",
        "why": "부호 있는 최소값(0x80000000)을 넣어 음수 길이 경로를 본다.",
        "params": (20,),
    },
    {
        "id": "u16-max",
        "family": "length",
        "when": "n>=14",
        "why": "오프셋 12 의 u16 을 0xFFFF 로 채워 카운트 필드를 포화시킨다.",
    },
    {
        "id": "reverse-prefix",
        "family": "permute",
        "when": "n>=2",
        "why": "선두 16바이트를 뒤집어 매직 순서를 깨뜨린다.",
    },
    {
        "id": "swap-ends",
        "family": "permute",
        "when": "n>=16",
        "why": "선두 8바이트와 꼬리 8바이트를 맞바꾼다.",
    },
    {
        "id": "slide-window-left",
        "family": "permute",
        "when": "n>=8",
        "why": "선두 32바이트를 한 바이트 왼쪽으로 밀어 필드 정렬을 흔든다.",
    },
    {
        "id": "slide-window-right",
        "family": "permute",
        "when": "n>=8",
        "why": "선두 32바이트를 한 바이트 오른쪽으로 밀어 필드 정렬을 흔든다.",
    },
    {
        "id": "repeat-mid-block",
        "family": "permute",
        "when": "n>=32",
        "why": "한가운데 32바이트를 바로 앞에 복사해 레코드 중복을 위조한다.",
    },
    {
        "id": "high-bit-stripe",
        "family": "stripe",
        "when": "n>0",
        "why": "16바이트마다 상위 비트를 켜 부호 있는 길이 해석을 흔든다.",
    },
    {
        "id": "low-bit-stripe",
        "family": "stripe",
        "when": "n>0",
        "why": "16바이트마다 최하위 비트를 뒤집어 홀짝 플래그를 깨뜨린다.",
    },
    {
        "id": "xor-stride7",
        "family": "stripe",
        "when": "n>0",
        "why": "7바이트마다 0xA5 를 XOR 해 주기적 체크섬·정렬을 흔든다.",
    },
    {
        "id": "interleave-zero-head",
        "family": "stripe",
        "when": "n>0",
        "why": "선두 32바이트의 홀수 인덱스를 0 으로 지워 교차 필드를 끊는다.",
    },
    {
        "id": "duplicate-prefix",
        "family": "stripe",
        "when": "n>=16",
        "why": "선두 8바이트를 바로 다음에 복제해 이중 매직을 만든다.",
    },
    {
        "id": "tail-over-head",
        "family": "stripe",
        "when": "n>=32",
        "why": "꼬리 16바이트를 선두에 덮어 매직을 본문 잔해로 바꾼다.",
    },
    {
        "id": "invert-tail-64",
        "family": "stripe",
        "when": "n>0",
        "why": "꼬리 최대 64바이트를 비트 반전한다.",
    },
    {
        "id": "complement-mid-32",
        "family": "stripe",
        "when": "n>0",
        "why": "한가운데 최대 32바이트를 비트 반전한다.",
    },
    {
        "id": "bit-rotate-head",
        "family": "stripe",
        "when": "n>0",
        "why": "선두 16바이트를 1비트 왼쪽으로 회전한다.",
    },
    {
        "id": "decrement-tail",
        "family": "stripe",
        "when": "n>0",
        "why": "꼬리 8바이트에서 1 을 빼 체크섬·길이를 미세 오염한다.",
    },
    {
        "id": "odd-length-chop",
        "family": "truncate",
        "when": "n>=2 이고 짝수",
        "why": "짝수 길이를 홀수로 만들어 UTF-16 워드 정렬을 깨뜨린다.",
    },
    {
        "id": "even-length-pad",
        "family": "splice",
        "when": "n 이 홀수이고 거대 아님",
        "why": "홀수 길이에 NUL 하나를 붙여 UTF-16 워드로 맞춘다.",
    },
    {
        "id": "splice-nul-mid",
        "family": "splice",
        "when": "n>0 이고 거대 아님",
        "why": "한가운데에 NUL 16바이트를 끼워 레코드 경계를 밀어낸다.",
    },
    {
        "id": "crlf-inject",
        "family": "splice",
        "when": "n>0 이고 거대 아님",
        "why": "한가운데에 CRLF 를 끼워 텍스트/바이너리 경계 오인을 유도한다.",
    },
    {
        "id": "pad-eof",
        "family": "splice",
        "when": "n>0 이고 거대 아님",
        "why": "파일 끝에 SUB(0x1A)를 붙여 구식 EOF 표식을 심는다.",
    },
    {
        "id": "widen-gap",
        "family": "splice",
        "when": "n>0 이고 거대 아님",
        "why": "1/4 지점에 NUL 4바이트를 끼워 오프셋을 밀어낸다.",
    },
    {
        "id": "shrink-gap",
        "family": "truncate",
        "when": "n>=8",
        "why": "1/4 지점의 4바이트를 삭제해 뒤 레코드를 앞으로 당긴다.",
    },
    {
        "id": "hwp3-sig-flip",
        "family": "hwp3",
        "when": "HWP3 서명이 있을 때",
        "why": "HWP Document File 서명의 첫 4바이트만 뒤집어 HWP3 인식을 깨뜨린다.",
    },
    {
        "id": "hwp3-sig-inject",
        "family": "hwp3",
        "when": "HWP3 서명이 없고 n 이 서명보다 길 때",
        "why": "비-HWP3 선두에 HWP3 서명을 심어 형식 오인을 유도한다.",
    },
)


def is_fatal_exception(exc) -> bool:
    """도구를 접으면 안 되는 치명 예외인가. 순수."""
    return isinstance(exc, FATAL_EXCEPTIONS)


def exception_kind(exc, context="probe") -> str:
    """예외를 kind 로 접는다. 순수.

    context:
      - probe: 한 명령 실행. FileNotFound → missing-bin.
      - read: 표본 읽기. FileNotFound → unreadable.
      - write: 변형 쓰기. FileNotFound → os-error.
      - select: 코퍼스 나열. FileNotFound → empty-corpus.
      - find-bin: 바이너리 탐색. FileNotFound → missing-bin.
    """
    if exc is None:
        return "unexpected"
    if isinstance(exc, subprocess.TimeoutExpired) or isinstance(exc, TimeoutError):
        return "timeout"
    if isinstance(exc, FileNotFoundError):
        if context in ("read", "select"):
            return "unreadable" if context == "read" else "empty-corpus"
        return "missing-bin"
    if isinstance(exc, PermissionError):
        return "permission" if context != "read" else "unreadable"
    if isinstance(exc, json.JSONDecodeError):
        return "value-error"
    for typ, kind in EXCEPTION_KIND_BY_TYPE.items():
        if isinstance(exc, typ):
            if context == "read" and kind in ("os-error", "permission", "missing-bin"):
                return "unreadable"
            if context == "select" and kind in ("os-error", "missing-bin"):
                return "empty-corpus"
            return kind
    return "unexpected"


def coerce_bytes(data) -> bytes:
    """바이트열만 받는다. bytearray/memoryview 는 복사하고, 그 외는 TypeError.

    발견 엔진이 str/int/None 을 조용히 인코딩하면 결정성이 깨진다. 호출자가
    파일에서 읽은 바이트만 넘기도록 강제한다.
    """
    if isinstance(data, bytes):
        return data
    if isinstance(data, bytearray):
        return bytes(data)
    if isinstance(data, memoryview):
        return data.tobytes()
    raise TypeError(
        "deterministic_mutants 는 bytes-like 만 받습니다"
        f" (got {type(data).__name__})"
    )


def classify_input_shape(data) -> str:
    """입력 형태 분류: empty / tiny / normal / huge. 비-바이트는 TypeError."""
    payload = coerce_bytes(data)
    n = len(payload)
    if n == 0:
        return "empty"
    if n <= TINY_MAX:
        return "tiny"
    if n >= HUGE_MIN:
        return "huge"
    return "normal"


def normalize_limit(limit) -> int:
    """표본 수 한도. 변환 불능·음수는 0 (전수 또는 표본 없음은 호출 측)."""
    try:
        n = int(limit)
    except (TypeError, ValueError):
        return 0
    return max(0, n)


def normalize_timeout(timeout) -> int:
    """프로브 초. 변환 불능·비양수는 0 — 호출 측에서 거른다."""
    try:
        n = int(timeout)
    except (TypeError, ValueError):
        return 0
    return n if n > 0 else 0


def normalize_workers(workers) -> int:
    """워커 수. 변환 불능·비양수는 1."""
    try:
        n = int(workers)
    except (TypeError, ValueError):
        return 1
    return n if n > 0 else 1


def parse_commands(raw) -> list:
    """쉼표구분 명령 문자열/시퀀스를 정규화. 빈 토큰은 버린다. 순서는 유지."""
    if raw is None:
        return list(DEFAULT_COMMANDS)
    if isinstance(raw, (list, tuple)):
        tokens = [str(item).strip() for item in raw]
    else:
        tokens = [part.strip() for part in str(raw).split(",")]
    out, seen = [], set()
    for token in tokens:
        if not token or token in seen:
            continue
        seen.add(token)
        out.append(token)
    return out or list(DEFAULT_COMMANDS)


def is_sample_name(name: str) -> bool:
    """코퍼스 파일명인가. 확장자만 본다. 대소문자 무시."""
    if not isinstance(name, str) or not name:
        return False
    lower = name.lower()
    return lower.endswith(SAMPLE_EXTS)


def mutant_family(label: str) -> str:
    """라벨을 가족 id 로 접는다. 카탈로그 확장 시 여기만 고치면 된다."""
    if not isinstance(label, str) or not label:
        return "other"
    if label == "empty-to-nul":
        return "empty"
    if (
        label.startswith("trunc")
        or label in ("chop-last", "cut-first", "odd-length-chop", "shrink-gap")
    ):
        return "truncate"
    if label.startswith("flip"):
        return "flip"
    if (
        label.startswith("biglen")
        or label.startswith("length-zero")
        or label.startswith("length-one")
        or label.startswith("i32-min")
        or label.startswith("u16-max")
    ):
        return "length"
    if label in (
        "zero-header",
        "header-smash",
        "rotate-header",
        "increment-header",
        "nibble-swap-head",
    ):
        return "header"
    if label in (
        "ole-trunc-tail",
        "ole-magic-poison",
        "ole-sector-shift-poison",
        "ole-mini-fat-poison",
    ):
        return "ole"
    if label in ("ff-run", "aa-run", "nul-mid", "00-run", "55-run"):
        return "run"
    if label in (
        "utf16-nul-sprinkle",
        "utf16-bom-inject",
        "utf8-overlong",
        "ascii-ctrl-sprinkle",
        "path-sep-sprinkle",
    ):
        return "unicode"
    if label in (
        "zip-local-header-flip",
        "zip-magic-inject",
        "zip-cd-magic-flip",
        "zip-eocd-flip",
    ):
        return "zip"
    if label in (
        "reverse-prefix",
        "swap-ends",
        "slide-window-left",
        "slide-window-right",
        "repeat-mid-block",
    ):
        return "permute"
    if label in (
        "high-bit-stripe",
        "low-bit-stripe",
        "xor-stride7",
        "interleave-zero-head",
        "duplicate-prefix",
        "tail-over-head",
        "invert-tail-64",
        "complement-mid-32",
        "bit-rotate-head",
        "decrement-tail",
    ):
        return "stripe"
    if label in ("splice-nul-mid", "crlf-inject", "pad-eof", "widen-gap", "even-length-pad"):
        return "splice"
    if label in ("hwp3-sig-flip", "hwp3-sig-inject"):
        return "hwp3"
    return "other"


def mutant_catalog() -> tuple:
    """변형 가족 카탈로그 사본. 시험·문서가 같은 표를 참조한다."""
    return tuple(dict(row) for row in MUTANT_CATALOG)


def catalog_ids() -> tuple:
    """카탈로그 id 튜플. 시험이 누락 가족을 잡는다."""
    return tuple(row["id"] for row in MUTANT_CATALOG)


def catalog_families() -> tuple:
    """카탈로그 가족 id 의 등장 순서 유일 목록."""
    seen, out = set(), []
    for row in MUTANT_CATALOG:
        fam = row["family"]
        if fam not in seen:
            seen.add(fam)
            out.append(fam)
    return tuple(out)


def probe_head(text, limit: int = PROBE_HEAD) -> str:
    """프로브 출력 머리. None/비문자는 빈 문자열."""
    if text is None:
        return ""
    if not isinstance(text, str):
        text = str(text)
    if limit <= 0:
        return ""
    return text[:limit]


def _rotl8(value: int, bits: int = 1) -> int:
    bits &= 7
    value &= 0xFF
    return ((value << bits) | (value >> (8 - bits))) & 0xFF


def _swap_nibbles(value: int) -> int:
    value &= 0xFF
    return ((value & 0x0F) << 4) | ((value & 0xF0) >> 4)


def _add_wrap(value: int, delta: int) -> int:
    return (value + delta) & 0xFF


def _find_magic(data: bytes, magic: bytes) -> int:
    if not magic:
        return -1
    return data.find(magic)


def _xor_slice(data: bytearray, start: int, width: int) -> None:
    end = min(len(data), start + width)
    for i in range(max(0, start), end):
        data[i] ^= 0xFF


def deterministic_mutants(data):
    """결정적 손상 변형 — (라벨, 바이트). 무작위 없음(재현 가능).

    같은 입력 → 같은 라벨·바이트. 원본과 동일한 무의미 변형은 넣지 않는다.
    비-바이트는 TypeError. 빈 입력은 empty-to-nul 한 건만.
    원 라벨(truncN/flipN/biglenN)은 유지하고, 확대 가족을 뒤에 덧붙인다.
    """
    data = coerce_bytes(data)
    n = len(data)
    if n == 0:
        return [("empty-to-nul", b"\0")]
    out = []
    huge = n >= HUGE_MIN

    def add(label: str, mut: bytes) -> None:
        if mut != data:
            out.append((label, mut))

    for pct in (1, 5, 10, 25, 50, 75, 95, 99):
        add(f"trunc{pct}", data[: max(1, n * pct // 100)])
    if n >= 2:
        add("chop-last", data[:-1])
    add("cut-first", data[1:])

    for pct in (0, 10, 25, 30, 50, 70, 75, 90, 99):
        pos = min(n - 1, n * pct // 100)
        b = bytearray(data)
        b[pos] ^= 0xFF
        add(f"flip{pct}", bytes(b))

    if n >= 4:
        for pct in (10, 40, 70):
            pos = min(n - 4, n * pct // 100)
            b = bytearray(data)
            b[pos : pos + 4] = LENGTH_BOMB_U32
            add(f"biglen{pct}", bytes(b))
        pos = min(n - 4, n * 30 // 100)
        b = bytearray(data)
        b[pos : pos + 4] = LENGTH_ZERO_U32
        add("length-zero30", bytes(b))
        pos = min(n - 4, n * 60 // 100)
        b = bytearray(data)
        b[pos : pos + 4] = LENGTH_ONE_U32
        add("length-one60", bytes(b))
        pos = min(n - 4, n * 20 // 100)
        b = bytearray(data)
        b[pos : pos + 4] = I32_MIN_LE
        add("i32-min20", bytes(b))

    if n >= 14:
        b = bytearray(data)
        b[12:14] = U16_MAX_LE
        add("u16-max12", bytes(b))

    b = bytearray(data)
    for i in range(min(n, 512)):
        b[i] = 0
    add("zero-header", bytes(b))

    smash_n = min(n, len(HEADER_SMASH_PAT))
    b = bytearray(data)
    b[:smash_n] = HEADER_SMASH_PAT[:smash_n]
    add("header-smash", bytes(b))

    if n >= 2:
        pref = min(8, n)
        b = bytearray(data)
        head = bytes(data[:pref])
        b[:pref] = head[1:] + head[:1]
        add("rotate-header", bytes(b))

    inc_n = min(8, n)
    b = bytearray(data)
    for i in range(inc_n):
        b[i] = _add_wrap(data[i], 1)
    add("increment-header", bytes(b))

    nib_n = min(32, n)
    b = bytearray(data)
    for i in range(nib_n):
        b[i] = _swap_nibbles(data[i])
    add("nibble-swap-head", bytes(b))

    if n > 64:
        add("ole-trunc-tail", data[:-64])
    else:
        k = min(4, n)
        add("ole-trunc-tail", data[:-k] + OLE_MAGIC[:k])

    poison_n = min(n, len(OLE_MAGIC))
    b = bytearray(data)
    poisoned = bytes(x ^ 0xFF for x in OLE_MAGIC[:poison_n])
    b[:poison_n] = poisoned
    add("ole-magic-poison", bytes(b))

    if n >= 32:
        b = bytearray(data)
        b[30:32] = U16_MAX_LE
        add("ole-sector-shift-poison", bytes(b))
    if n >= 72:
        b = bytearray(data)
        b[60:64] = b"\xff\xff\xff\xff"
        add("ole-mini-fat-poison", bytes(b))

    start = n // 3
    run = min(128, max(1, n - start))
    b = bytearray(data)
    b[start : start + run] = b"\xff" * run
    add("ff-run", bytes(b))

    aa_n = min(n, max(1, n // 4))
    b = bytearray(data)
    b[:aa_n] = b"\xaa" * aa_n
    add("aa-run", bytes(b))

    mid = n // 2
    nul_n = min(64, max(1, n - mid))
    b = bytearray(data)
    b[mid : mid + nul_n] = b"\x00" * nul_n
    add("nul-mid", bytes(b))

    two_third = (n * 2) // 3
    zero_n = min(64, max(1, n - two_third))
    b = bytearray(data)
    b[two_third : two_third + zero_n] = b"\x00" * zero_n
    add("00-run", bytes(b))

    five_n = min(n, max(1, n // 5))
    b = bytearray(data)
    b[:five_n] = b"\x55" * five_n
    add("55-run", bytes(b))

    if n >= 2:
        b = bytearray(data)
        for pct in (20, 40, 60, 80):
            pos = min(n - 2, n * pct // 100) & ~1
            b[pos : pos + 2] = b"\x00\x00"
        add("utf16-nul-sprinkle", bytes(b))
        b = bytearray(data)
        b[:2] = UTF16_BOM_LE
        add("utf16-bom-inject", bytes(b))
        over_at = min(n - 2, max(0, n // 5))
        b = bytearray(data)
        b[over_at : over_at + 2] = UTF8_OVERLONG_NUL
        add("utf8-overlong", bytes(b))

    b = bytearray(data)
    for pct in (15, 35, 55, 75):
        pos = min(n - 1, n * pct // 100)
        b[pos] = CTRL_BYTE
    add("ascii-ctrl-sprinkle", bytes(b))

    b = bytearray(data)
    seps = (0x2F, 0x5C)
    for i, pct in enumerate((18, 42, 66, 88)):
        pos = min(n - 1, n * pct // 100)
        b[pos] = seps[i % 2]
    add("path-sep-sprinkle", bytes(b))

    idx = _find_magic(data, ZIP_LOCAL)
    if idx >= 0:
        b = bytearray(data)
        _xor_slice(b, idx, len(ZIP_LOCAL))
        add("zip-local-header-flip", bytes(b))
    elif n >= len(ZIP_LOCAL):
        b = bytearray(data)
        b[: len(ZIP_LOCAL)] = ZIP_LOCAL
        add("zip-magic-inject", bytes(b))

    cd_idx = _find_magic(data, ZIP_CD)
    if cd_idx >= 0:
        b = bytearray(data)
        _xor_slice(b, cd_idx, len(ZIP_CD))
        add("zip-cd-magic-flip", bytes(b))

    eocd_idx = _find_magic(data, ZIP_EOCD)
    if eocd_idx >= 0:
        b = bytearray(data)
        _xor_slice(b, eocd_idx, len(ZIP_EOCD))
        add("zip-eocd-flip", bytes(b))

    if n >= 2:
        pref = min(16, n)
        b = bytearray(data)
        b[:pref] = bytes(reversed(data[:pref]))
        add("reverse-prefix", bytes(b))

    if n >= 16:
        b = bytearray(data)
        b[:8], b[-8:] = data[-8:], data[:8]
        add("swap-ends", bytes(b))

    if n >= 8:
        win = min(32, n)
        chunk = bytearray(data[:win])
        left = chunk[1:] + chunk[:1]
        b = bytearray(data)
        b[:win] = left
        add("slide-window-left", bytes(b))
        right = chunk[-1:] + chunk[:-1]
        b = bytearray(data)
        b[:win] = right
        add("slide-window-right", bytes(b))

    if n >= 32:
        mid = n // 2
        block = data[mid : mid + 32]
        dest = max(0, mid - 32)
        b = bytearray(data)
        b[dest : dest + 32] = block
        add("repeat-mid-block", bytes(b))

    b = bytearray(data)
    for i in range(0, n, STRIPE_STEP):
        b[i] = data[i] | 0x80
    add("high-bit-stripe", bytes(b))

    b = bytearray(data)
    for i in range(0, n, STRIPE_STEP):
        b[i] = data[i] ^ 0x01
    add("low-bit-stripe", bytes(b))

    b = bytearray(data)
    for i in range(0, n, XOR_STRIDE):
        b[i] = data[i] ^ XOR_STRIDE_MASK
    add("xor-stride7", bytes(b))

    head_n = min(32, n)
    b = bytearray(data)
    for i in range(1, head_n, 2):
        b[i] = 0
    add("interleave-zero-head", bytes(b))

    if n >= 16:
        b = bytearray(data)
        b[8:16] = data[:8]
        add("duplicate-prefix", bytes(b))

    if n >= 32:
        b = bytearray(data)
        b[:16] = data[-16:]
        add("tail-over-head", bytes(b))

    tail_n = min(64, n)
    b = bytearray(data)
    for i in range(n - tail_n, n):
        b[i] = data[i] ^ 0xFF
    add("invert-tail-64", bytes(b))

    mid = n // 2
    mid_n = min(32, max(1, n - mid))
    b = bytearray(data)
    for i in range(mid, mid + mid_n):
        b[i] = data[i] ^ 0xFF
    add("complement-mid-32", bytes(b))

    rot_n = min(16, n)
    b = bytearray(data)
    for i in range(rot_n):
        b[i] = _rotl8(data[i], 1)
    add("bit-rotate-head", bytes(b))

    dec_n = min(8, n)
    b = bytearray(data)
    for i in range(n - dec_n, n):
        b[i] = _add_wrap(data[i], -1)
    add("decrement-tail", bytes(b))

    if n >= 2 and (n % 2 == 0):
        add("odd-length-chop", data[:-1])

    hwp3_idx = _find_magic(data, HWP3_SIG)
    if hwp3_idx >= 0:
        b = bytearray(data)
        _xor_slice(b, hwp3_idx, min(4, len(HWP3_SIG)))
        add("hwp3-sig-flip", bytes(b))
    elif n >= len(HWP3_SIG):
        b = bytearray(data)
        b[: len(HWP3_SIG)] = HWP3_SIG
        add("hwp3-sig-inject", bytes(b))

    if not huge:
        mid = n // 2
        add("splice-nul-mid", data[:mid] + (b"\x00" * 16) + data[mid:])
        add("crlf-inject", data[:mid] + b"\r\n" + data[mid:])
        add("pad-eof", data + b"\x1a")
        gap = n // 4
        add("widen-gap", data[:gap] + (b"\x00" * 4) + data[gap:])
        if n % 2 == 1:
            add("even-length-pad", data + b"\x00")
    if n >= 8:
        gap = n // 4
        end = min(n, gap + 4)
        if end > gap:
            add("shrink-gap", data[:gap] + data[end:])

    return out


def list_sample_names(samples_dir: str):
    """코퍼스 파일명을 정렬해 돌려준다. 디렉터리를 못 읽으면 (None, 이유)."""
    try:
        names = os.listdir(samples_dir)
    except OSError as exc:
        return None, f"{exception_kind(exc, 'select')}: {type(exc).__name__}: {exc}"
    except Exception as exc:  # noqa: BLE001
        if is_fatal_exception(exc):
            raise
        return None, f"{exception_kind(exc, 'select')}: {type(exc).__name__}: {exc}"
    everything = sorted(f for f in names if is_sample_name(f))
    return everything, None


def select_samples(samples_dir: str, limit: int):
    """정렬된 .hwp/.hwpx/.hml 을 결정적 stride 로 limit 개 뽑는다.

    디렉터리를 읽을 수 없으면 빈 목록을 돌려 발견 엔진이 예외로 죽지 않게 한다.
    limit<=0 이면 전수. 변환 불능 limit 은 0 으로 접어 전수다.
    """
    limit = normalize_limit(limit)
    everything, err = list_sample_names(samples_dir)
    if everything is None:
        return [], 0
    if not everything or (limit <= 0):
        return everything, len(everything)
    if len(everything) <= limit:
        return everything, len(everything)
    stride = len(everything) / limit
    picked, seen = [], set()
    for i in range(limit):
        f = everything[min(len(everything) - 1, int(i * stride))]
        if f not in seen:
            seen.add(f)
            picked.append(f)
    return picked, len(everything)


def classify(code, err: str):
    """(kind, bucket) — kind in {panic, hang, None}. bucket 은 클러스터 키.

    기존 계약: `panicked at file:line` → 그 위치. 스택 오버플로·어보트 코드
    (101)·음수·>=132 는 패닉. 깨끗한 비-0 실패는 (None, None).
    hang 은 이 함수가 내지 않는다 — TimeoutExpired 가 낸다.
    """
    if err is None:
        err = ""
    elif not isinstance(err, str):
        err = str(err)
    low = err.lower()
    m = PANIC_RE.search(err)
    if m:
        return "panic", m.group(1)
    if "stack overflow" in low:
        return "panic", "stack-overflow"
    if "panicked" in low or code == RUST_ABORT or (
        code is not None and isinstance(code, int) and (code < 0 or code >= SIGNAL_FLOOR)
    ):
        return "panic", f"code{code}"
    return None, None


def is_panic_code(code, err: str) -> bool:
    """우아한 실패와 패닉을 가른다. classify 의 패닉 판정과 같다."""
    kind, _ = classify(code, err)
    return kind == "panic"


def classify_timeout(timed_out) -> bool:
    """행(timeout) 분류. True / TimeoutExpired / TimeoutError / 표식 문자열."""
    if isinstance(timed_out, BaseException):
        if isinstance(timed_out, (subprocess.TimeoutExpired, TimeoutError)):
            return True
        if isinstance(timed_out, OSError):
            timed = {getattr(errno, name, None) for name in ("ETIMEDOUT", "ETIME")}
            timed.discard(None)
            return getattr(timed_out, "errno", None) in timed
        return False
    if isinstance(timed_out, str):
        low = timed_out.lower()
        return any(marker in low for marker in TIMEOUT_MARKERS)
    return timed_out is True


def classify_probe_outcome(kind, bucket) -> str:
    """probe 가 낸 (kind, bucket) 을 hang / panic / error / clean 으로 접는다."""
    if kind == "hang" or classify_timeout(kind):
        return "hang"
    if kind == "panic":
        return "panic"
    if kind == "error" or (isinstance(bucket, str) and bucket in EXCEPTION_KINDS):
        return "error"
    if kind in (None, "clean", "ok"):
        return "clean"
    return "error"


def read_sample(path: str):
    """샘플을 읽는다. 성공 시 (bytes, None), 실패 시 (None, 이유)."""
    try:
        with open(path, "rb") as fh:
            return fh.read(), None
    except OSError as exc:
        return None, f"{exception_kind(exc, 'read')}: {type(exc).__name__}: {exc}"
    except Exception as exc:  # noqa: BLE001
        if is_fatal_exception(exc):
            raise
        return None, f"{exception_kind(exc, 'read')}: {type(exc).__name__}: {exc}"


def write_mutant(path: str, mut: bytes):
    """변형 바이트를 쓴다. 성공 시 None, 실패 시 이유 문자열."""
    try:
        payload = coerce_bytes(mut)
    except TypeError as exc:
        return f"type-error: {exc}"
    try:
        with open(path, "wb") as fh:
            fh.write(payload)
        return None
    except OSError as exc:
        return f"{exception_kind(exc, 'write')}: {type(exc).__name__}: {exc}"
    except Exception as exc:  # noqa: BLE001
        if is_fatal_exception(exc):
            raise
        return f"{exception_kind(exc, 'write')}: {type(exc).__name__}: {exc}"


def find_bin_safe(cli_arg):
    """바이너리 경로를 찾는다. 성공 시 (path, None), 실패 시 (None, 이유)."""
    try:
        path = runner.find_bin(cli_arg)
    except CATCHABLE_EXCEPTIONS as exc:
        return None, f"{exception_kind(exc, 'find-bin')}: {type(exc).__name__}: {exc}"
    except Exception as exc:  # noqa: BLE001
        if is_fatal_exception(exc):
            raise
        return None, f"{exception_kind(exc, 'find-bin')}: {type(exc).__name__}: {exc}"
    if not path:
        return None, "missing-bin: empty-path"
    if not os.path.exists(path):
        return None, f"missing-bin: not-found: {path}"
    return path, None


def probe(bin_path, cmd, mut_path, timeout):
    """한 명령 × 한 변형을 실행 — (kind, bucket).

    TimeoutExpired → ("hang", cmd). 없는 바이너리·권한·그 외 예외는
    ("error", kind) 로 접고 엔진을 죽이지 않는다. 치명 예외는 삼키지 않는다.
    """
    seconds = normalize_timeout(timeout)
    if seconds <= 0:
        return "error", "invalid-timeout"
    if not bin_path:
        return "error", "missing-bin"
    if not cmd:
        return "error", "value-error"
    args = [bin_path, cmd, mut_path]
    if cmd == "convert":
        args.append(mut_path + ".out.hwpx")
    try:
        p = subprocess.run(args, cwd=REPO_ROOT, capture_output=True, timeout=seconds)
        err = p.stderr.decode("utf-8", "replace") + p.stdout.decode("utf-8", "replace")
        return classify(p.returncode, err)
    except subprocess.TimeoutExpired:
        return "hang", cmd
    except CATCHABLE_EXCEPTIONS as exc:
        return "error", exception_kind(exc, "probe")
    except Exception as exc:  # noqa: BLE001
        if is_fatal_exception(exc):
            raise
        return "error", exception_kind(exc, "probe")


def empty_shape_counts() -> dict:
    return {key: 0 for key in SHAPE_KEYS}


def empty_report(commands=None) -> dict:
    """스키마를 만족하는 빈 보고. 예외 경로에서 엔진이 죽지 않게 쓴다."""
    cmds = list(commands) if commands else []
    return {
        "kind": REPORT_KIND,
        "schemaVersion": SCHEMA_VERSION,
        "ok": True,
        "samplesTested": 0,
        "totalSamples": 0,
        "commands": cmds,
        "mutantsPerSample": 0,
        "runsChecked": 0,
        "distinctPanicSites": 0,
        "panicClusters": [],
        "hangClusters": [],
        "unreadables": [],
        "probeErrors": [],
        "toolErrors": [],
        "emptyCorpus": False,
        "missingBin": False,
        "toolFailed": False,
        "inputShapes": empty_shape_counts(),
        "exit": EXIT_OK,
    }


def _cluster_ok(report) -> bool:
    return not report.get("panicClusters") and not report.get("hangClusters")


def resolve_exit(report) -> int:
    """보고에서 종료 코드를 계산한다. 순수."""
    if report.get("toolFailed") or report.get("missingBin"):
        return EXIT_TOOL_FAILED
    if report.get("panicClusters") or report.get("hangClusters"):
        return EXIT_DOS
    return EXIT_OK


def validate_report(report) -> list:
    """보고 스키마 위반 목록. 비어 있으면 계약 충족."""
    issues = []
    if not isinstance(report, dict):
        return ["report-not-dict"]
    missing = [key for key in REPORT_KEYS if key not in report]
    extra = [key for key in report if key not in REPORT_KEYS]
    if missing:
        issues.append("missing:" + ",".join(missing))
    if extra:
        issues.append("extra:" + ",".join(sorted(extra)))
    if report.get("kind") != REPORT_KIND:
        issues.append("kind")
    if report.get("schemaVersion") != SCHEMA_VERSION:
        issues.append("schemaVersion")
    for key in REPORT_BOOL_KEYS:
        if not isinstance(report.get(key), bool):
            issues.append(f"{key}-type")
    for key in REPORT_INT_KEYS:
        if not isinstance(report.get(key), int):
            issues.append(f"{key}-type")
        elif report.get(key) < 0:
            issues.append(f"{key}-negative")
    for key in REPORT_LIST_KEYS:
        value = report.get(key)
        if not isinstance(value, list):
            issues.append(f"{key}-type")
    shapes = report.get("inputShapes")
    if not isinstance(shapes, dict):
        issues.append("inputShapes-type")
    else:
        for key in SHAPE_KEYS:
            if key not in shapes:
                issues.append(f"inputShapes-missing-{key}")
            elif not isinstance(shapes[key], int) or shapes[key] < 0:
                issues.append(f"inputShapes-{key}")
    panics = report.get("panicClusters")
    if isinstance(panics, list):
        for item in panics:
            if not isinstance(item, dict):
                issues.append("panicClusters-item-type")
                break
            if "location" not in item or "count" not in item or "example" not in item:
                issues.append("panicClusters-item-keys")
                break
    hangs = report.get("hangClusters")
    if isinstance(hangs, list):
        for item in hangs:
            if not isinstance(item, dict):
                issues.append("hangClusters-item-type")
                break
            if "command" not in item or "count" not in item:
                issues.append("hangClusters-item-keys")
                break
    if isinstance(report.get("ok"), bool):
        expected_ok = _cluster_ok(report) and not report.get("toolFailed")
        if report["ok"] != expected_ok:
            issues.append("ok-mismatch")
    if isinstance(report.get("emptyCorpus"), bool) and isinstance(report.get("totalSamples"), int):
        if report["emptyCorpus"] and report["totalSamples"] != 0:
            issues.append("emptyCorpus-mismatch")
    if isinstance(report.get("exit"), int):
        if report["exit"] != resolve_exit(report):
            issues.append("exit-mismatch")
    if isinstance(report.get("distinctPanicSites"), int) and isinstance(panics, list):
        if report["distinctPanicSites"] != len(panics):
            issues.append("distinctPanicSites-mismatch")
    return issues


def format_human_report(report: dict) -> str:
    """사람용 요약. JSON 이 아닐 때 stdout 에 쓴다."""
    if report.get("missingBin") or report.get("toolFailed"):
        lines = ["코퍼스 퍼징: 도구 실패 — 바이너리 또는 코퍼스를 쓰지 못했다."]
        for item in report.get("toolErrors", []):
            lines.append(f"  TOOL {item}")
        return "\n".join(lines)
    if report.get("emptyCorpus"):
        return "코퍼스 퍼징: 빈 코퍼스 — 표본 0 (.hwp/.hwpx/.hml 없음)"
    if report.get("ok"):
        return (
            f"코퍼스 퍼징: 샘플 {report['samplesTested']}/{report['totalSamples']} × "
            f"명령 {len(report.get('commands') or [])} × {report['runsChecked']} 실행 — DoS 0"
            f" (읽기실패 {len(report.get('unreadables') or [])}"
            f" · 프로브오류 {len(report.get('probeErrors') or [])})"
        )
    lines = [
        f"코퍼스 퍼징: 고유 패닉 {report.get('distinctPanicSites', 0)}곳 · "
        f"행 클러스터 {len(report.get('hangClusters') or [])}개 — 고쳐야 할 DoS:"
    ]
    for p in report.get("panicClusters") or []:
        lines.append(f"  PANIC {p.get('location')}  ({p.get('count')}건)  예: {p.get('example')}")
    for h in report.get("hangClusters") or []:
        samples = h.get("samples") or []
        lines.append(
            f"  HANG  {h.get('command')}  ({h.get('count')}건, {len(samples)}샘플)  예: {h.get('example')}"
        )
    return "\n".join(lines)


def _sort_panic_clusters(panic_clusters: dict) -> list:
    return sorted(
        ({"location": loc, "count": len(c), "example": c[0]} for loc, c in panic_clusters.items()),
        key=lambda x: (-x["count"], x["location"]),
    )


def _sort_hang_clusters(hang_clusters: dict) -> list:
    return sorted(
        (
            {
                "command": cmd,
                "count": len(c),
                "samples": sorted({t.split(":")[0] for t in c}),
                "example": c[0],
            }
            for cmd, c in hang_clusters.items()
        ),
        key=lambda x: (-x["count"], x["command"]),
    )


def _mutants_per_sample_hint() -> int:
    """보고용 변형 수 힌트. 4KiB 정상 입력 기준. 결정적.

    시험이 `deterministic_mutants` 를 바꿔 끼워도 보고 조립이 죽지 않게
    예외는 0 으로 접는다. 실제 표본의 변형 생성 실패는 unreadables 다.
    """
    try:
        return len(deterministic_mutants(b"x" * 4096))
    except Exception:  # noqa: BLE001 — 힌트일 뿐 엔진을 죽이지 않는다
        return 0


def _finish_report(report, picked, total, commands, panic_clusters, hang_clusters,
                   unreadables, probe_errors, tool_errors, shapes, checked,
                   missing_bin=False, tool_failed=False, empty_corpus=False):
    panics = _sort_panic_clusters(panic_clusters)
    hangs = _sort_hang_clusters(hang_clusters)
    report.update(
        {
            "samplesTested": len(picked),
            "totalSamples": total,
            "commands": list(commands),
            "mutantsPerSample": _mutants_per_sample_hint(),
            "runsChecked": checked,
            "distinctPanicSites": len(panics),
            "panicClusters": panics,
            "hangClusters": hangs,
            "unreadables": list(unreadables),
            "probeErrors": list(probe_errors),
            "toolErrors": list(tool_errors),
            "emptyCorpus": bool(empty_corpus),
            "missingBin": bool(missing_bin),
            "toolFailed": bool(tool_failed or missing_bin),
            "inputShapes": shapes,
        }
    )
    report["ok"] = _cluster_ok(report) and not report["toolFailed"]
    report["exit"] = resolve_exit(report)
    return report


def fuzz(bin_path, samples_dir, commands, limit, workers, timeout, work_dir):
    """코퍼스 × 명령 × 변형을 두들겨 gymFuzzCorpus 봉투를 낸다.

    예외 경로(없는 디렉터리·빈 코퍼스·읽기 실패·쓰기 실패·프로브 예외)에서도
    엔진은 죽지 않는다. 패닉/행만 ok 를 뒤집고, 도구 실패(missingBin)는
    toolFailed 로 따로 표시한다.
    """
    commands = parse_commands(commands)
    workers = normalize_workers(workers)
    report = empty_report(commands)
    unreadables, probe_errors, tool_errors = [], [], []
    shapes = empty_shape_counts()
    panic_clusters, hang_clusters = {}, {}
    picked, total = [], 0

    if not bin_path:
        tool_errors.append("missing-bin: empty-path")
        return _finish_report(
            report, picked, total, commands, panic_clusters, hang_clusters,
            unreadables, probe_errors, tool_errors, shapes, 0,
            missing_bin=True, tool_failed=True,
        )

    try:
        picked, total = select_samples(samples_dir, limit)
    except Exception as exc:  # noqa: BLE001
        if is_fatal_exception(exc):
            raise
        tool_errors.append(f"select_samples: {type(exc).__name__}: {exc}")
        return _finish_report(
            report, [], 0, commands, panic_clusters, hang_clusters,
            unreadables, probe_errors, tool_errors, shapes, 0,
            tool_failed=True,
        )

    names, list_err = list_sample_names(samples_dir)
    if list_err is not None and names is None:
        # 디렉터리 자체 부재는 빈 코퍼스와 구분한다.
        tool_errors.append(list_err)
        empty = "empty-corpus" in list_err or "unreadable" in list_err
        return _finish_report(
            report, [], 0, commands, panic_clusters, hang_clusters,
            unreadables, probe_errors, tool_errors, shapes, 0,
            tool_failed=not empty, empty_corpus=empty,
        )

    if total == 0:
        return _finish_report(
            report, picked, total, commands, panic_clusters, hang_clusters,
            unreadables, probe_errors, tool_errors, shapes, 0,
            empty_corpus=True,
        )

    jobs = []
    for i, name in enumerate(picked):
        data, read_err = read_sample(os.path.join(samples_dir, name))
        if read_err is not None or data is None:
            unreadables.append(f"{name}: {read_err or 'unreadable'}")
            continue
        try:
            shape = classify_input_shape(data)
        except TypeError as exc:
            unreadables.append(f"{name}: type-error: {exc}")
            continue
        shapes[shape] = shapes.get(shape, 0) + 1
        try:
            mutants = deterministic_mutants(data)
        except TypeError as exc:
            unreadables.append(f"{name}: type-error: {exc}")
            continue
        except Exception as exc:  # noqa: BLE001
            if is_fatal_exception(exc):
                raise
            unreadables.append(f"{name}: {type(exc).__name__}: {exc}")
            continue
        for label, mut in mutants:
            jobs.append((i, name, label, mut))

    checked = 0
    missing_bin_hits = 0

    def run_one(job):
        idx, name, label, mut = job
        p = os.path.join(work_dir, f"m{idx}_{label}.hwp")
        write_err = write_mutant(p, mut)
        if write_err is not None:
            return "write-error", write_err, f"{name}:{label}"
        try:
            results = []
            for cmd in commands:
                try:
                    kind, bucket = probe(bin_path, cmd, p, timeout)
                except Exception as exc:  # noqa: BLE001
                    if is_fatal_exception(exc):
                        raise
                    results.append(("error", exception_kind(exc, "probe"), f"{name}:{label}:{cmd}"))
                    continue
                if kind:
                    results.append((kind, bucket, f"{name}:{label}:{cmd}"))
            return "ok", results, None
        finally:
            try:
                os.remove(p)
            except OSError:
                pass

    if not jobs:
        return _finish_report(
            report, picked, total, commands, panic_clusters, hang_clusters,
            unreadables, probe_errors, tool_errors, shapes, 0,
            empty_corpus=False,
        )

    def absorb(status, payload, tag):
        nonlocal checked, missing_bin_hits
        if status == "write-error":
            probe_errors.append(f"{tag}: {payload}")
            return
        checked += 1
        for kind, bucket, item_tag in payload:
            outcome = classify_probe_outcome(kind, bucket)
            if outcome == "panic":
                panic_clusters.setdefault(bucket, []).append(item_tag)
            elif outcome == "hang":
                hang_clusters.setdefault(bucket, []).append(item_tag)
            elif outcome == "error":
                probe_errors.append(f"{item_tag}: {bucket}")
                if bucket == "missing-bin":
                    missing_bin_hits += 1

    if workers <= 1:
        for job in jobs:
            absorb(*run_one(job))
    else:
        with ThreadPoolExecutor(max_workers=workers) as ex:
            for fut in as_completed([ex.submit(run_one, j) for j in jobs]):
                try:
                    absorb(*fut.result())
                except Exception as exc:  # noqa: BLE001
                    if is_fatal_exception(exc):
                        raise
                    probe_errors.append(f"worker: {type(exc).__name__}: {exc}")

    missing_bin = missing_bin_hits > 0 and not panic_clusters and not hang_clusters
    if missing_bin:
        tool_errors.append("missing-bin: probe FileNotFound")
    return _finish_report(
        report, picked, total, commands, panic_clusters, hang_clusters,
        unreadables, probe_errors, tool_errors, shapes,
        checked * len(commands),
        missing_bin=missing_bin, tool_failed=missing_bin,
    )


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description="gym 코퍼스 퍼징 발견 엔진 — DoS 를 근본원인별로 색출")
    ap.add_argument("--bin", required=True)
    ap.add_argument("--commands", default=",".join(DEFAULT_COMMANDS),
                    help="쉼표구분 rhwp 명령 (기본: %(default)s)")
    ap.add_argument("--limit", type=int, default=0, help="샘플 수(0=전수)")
    ap.add_argument("--workers", type=int, default=8)
    ap.add_argument("--timeout", type=int, default=10)
    ap.add_argument("--json", action="store_true")
    a = ap.parse_args(argv)
    commands = parse_commands(a.commands)
    bin_path, bin_err = find_bin_safe(a.bin)
    if bin_err is not None or not bin_path:
        report = empty_report(commands)
        report["missingBin"] = True
        report["toolFailed"] = True
        report["ok"] = False
        report["toolErrors"] = [bin_err or "missing-bin: empty-path"]
        report["exit"] = EXIT_TOOL_FAILED
        if a.json:
            sys.stdout.write(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
        else:
            print(format_human_report(report), file=sys.stderr)
        return EXIT_TOOL_FAILED
    samples_dir = os.path.join(REPO_ROOT, "samples")
    try:
        work_cm = tempfile.TemporaryDirectory()
    except Exception as exc:  # noqa: BLE001
        if is_fatal_exception(exc):
            raise
        report = empty_report(commands)
        report["toolFailed"] = True
        report["ok"] = False
        report["toolErrors"] = [f"tempdir: {type(exc).__name__}: {exc}"]
        report["exit"] = EXIT_TOOL_FAILED
        if a.json:
            sys.stdout.write(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
        else:
            print(format_human_report(report), file=sys.stderr)
        return EXIT_TOOL_FAILED
    with work_cm as work:
        try:
            report = fuzz(bin_path, samples_dir, commands, a.limit, a.workers, a.timeout, work)
        except Exception as exc:  # noqa: BLE001
            if is_fatal_exception(exc):
                raise
            report = empty_report(commands)
            report["toolFailed"] = True
            report["ok"] = False
            report["toolErrors"] = [f"fuzz: {type(exc).__name__}: {exc}"]
            report["exit"] = EXIT_TOOL_FAILED
    issues = validate_report(report)
    if issues:
        report.setdefault("toolErrors", []).append("schema: " + ";".join(issues))
    if a.json:
        sys.stdout.write(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    else:
        print(format_human_report(report))
    return int(report.get("exit", EXIT_OK if report.get("ok") else EXIT_DOS))


if __name__ == "__main__":
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")  # type: ignore[attr-defined]
        except Exception:
            pass
    sys.exit(main())
