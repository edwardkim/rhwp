"""gym 손상-강건성 감사 — rhwp 가 적대적/손상 입력에 절대 패닉·행 하지 않는가.

## 왜 이 도구인가 (도구 강건성이 능력의 천장)

2026 프론티어(AgentHijack 등)는 에이전트가 **환경 손상**에 견디는지 잰다. gym 은
에이전트가 rhwp 를 몰아 능력을 낸다 — 그런데 rhwp 가 손상 문서에 **패닉**하면
에이전트가 아무리 유능해도 과제를 못 끝낸다. 도구의 적대적 강건성이 능력의 천장이다.

이 감사기는 코퍼스를 **결정적으로 손상**시켜(무작위 없음 — 재현 가능) rhwp 가 언제나
우아하게 실패/부분복구하는지 인증한다:

- **패닉**(exit 101 · 시그널/음수 코드 · 'panicked' · 스택 오버플로) → 실패.
- **행**(timeout) → 실패.
- 그 외(깨끗한 비-0 실패 · 경고 후 부분복구 · 정상 파싱) → 우아함(정상).

종점 무결성(#4808 판별력)·경로 무결성(#4810 트라젝토리)에 이은 세 번째 기둥 —
도구 자체의 손상 강건성. 이것이 다른 문서 벤치마크가 안 재는 축이다: 벤치마크가
자기 도구가 적대적 입력에 죽지 않음을 CI 로 인증한다.

감사기 자신은 예외 경로에서도 죽지 않는다. 읽을 수 없는 샘플, 빈/극소/거대 입력,
바이트가 아닌 입력, 프로브 시간초과·OS 오류는 분류해 보고하고 중단하지 않는다.

카탈로그·분류·봉투 계약은 `gym/docs/robustness.md` 와
`gym/tools/README_robustness.md` 가 같은 표를 본다. 시험은
`scripts/tests/test_gym_robustness.py` 가 고정한다.

## 사용

    python gym/tools/robustness.py --bin target/debug/rhwp            # 결정적 부분집합
    python gym/tools/robustness.py --bin target/debug/rhwp --limit 40 # 더 넓게
    python gym/tools/robustness.py --bin target/debug/rhwp --json
"""

from __future__ import annotations

import argparse
import errno
import json
import os
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
GYM_ROOT = os.path.dirname(HERE)
REPO_ROOT = os.path.dirname(GYM_ROOT)
sys.path.insert(0, GYM_ROOT)

from core import runner  # noqa: E402

OLE_MAGIC = b"\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1"
ZIP_LOCAL = b"PK\x03\x04"
ZIP_CD = b"PK\x01\x02"
ZIP_EOCD = b"PK\x05\x06"
HEADER_SMASH_PAT = b"\xde\xad\xbe\xef" * 16  # 64바이트 고정 패턴
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

# 입력 크기 분류 — 빈/극소/거대는 위치 기반 변형이 다르게 동작한다.
TINY_MAX = 64
HUGE_MIN = 1_048_576  # 1 MiB. 크기 증가 변형은 거대 입력에서 건너뛴다.
PROBE_HEAD = 160
SCHEMA_VERSION = "1.0"
REPORT_KIND = "gymRobustness"

# 감사 보고 고정 키. 시험이 이 집합을 계약으로 고정한다.
REPORT_KEYS = (
    "kind",
    "schemaVersion",
    "ok",
    "samplesTested",
    "totalSamples",
    "mutantsChecked",
    "gracefullyDegraded",
    "panics",
    "hangs",
    "unreadables",
    "probeErrors",
    "inputShapes",
)

REPORT_LIST_KEYS = ("panics", "hangs", "unreadables", "probeErrors")
REPORT_INT_KEYS = (
    "samplesTested",
    "totalSamples",
    "mutantsChecked",
    "gracefullyDegraded",
)
SHAPE_KEYS = ("empty", "tiny", "normal", "huge")

# 정상(비어 있지 않은) 입력에서 항상 나오기를 기대하는 라벨.
# 원본과 바이트가 같아지면 `add()` 가 버리므로 극소 입력에서는 일부만 남는다.
ALWAYS_LABELS = (
    "truncate@25%",
    "truncate@50%",
    "truncate@75%",
    "truncate@95%",
    "flip@10%",
    "flip@50%",
    "flip@90%",
    "zero-header",
    "header-smash",
    "ole-trunc-tail",
    "ff-run",
    "utf16-nul-sprinkle",
)

# 2KiB 정상 입력에서 추가로 항상 나오는 확대 라벨. ZIP 조건부 라벨은 제외.
EXPANDED_ALWAYS_LABELS = (
    "truncate@10%",
    "truncate@99%",
    "flip@0%",
    "flip@25%",
    "flip@75%",
    "flip@99%",
    "chop-last",
    "cut-first",
    "aa-run",
    "nul-mid",
    "00-run",
    "55-run",
    "ole-magic-poison",
    "ole-sector-shift-poison",
    "zip-magic-inject",
    "length-bomb@10%",
    "length-bomb@40%",
    "length-bomb@70%",
    "length-zero@30%",
    "length-one@60%",
    "i32-min@20%",
    "u16-max@12",
    "reverse-prefix",
    "swap-ends",
    "high-bit-stripe",
    "low-bit-stripe",
    "xor-stride7",
    "rotate-header",
    "nibble-swap-head",
    "increment-header",
    "decrement-tail",
    "interleave-zero-head",
    "duplicate-prefix",
    "tail-over-head",
    "invert-tail-64",
    "complement-mid-32",
    "bit-rotate-head",
    "utf16-bom-inject",
    "ascii-ctrl-sprinkle",
    "utf8-overlong",
    "path-sep-sprinkle",
    "slide-window-left",
    "slide-window-right",
    "repeat-mid-block",
    "odd-length-chop",
    "splice-nul-mid",
    "crlf-inject",
    "pad-eof",
    "widen-gap",
    "shrink-gap",
)

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

# 행(timeout) 문자열 표식. 소문자 비교.
TIMEOUT_MARKERS = (
    "timeout",
    "timed out",
    "time-out",
    "time expired",
    "deadline exceeded",
)

# POSIX 음수 종료 코드(시그널)와 Windows NTSTATUS 상위 비트.
RUST_ABORT = 101
WINDOWS_EXCEPTION_MASK = 0xC0000000

# 변형 가족 카탈로그 — 문서·시험이 같은 표를 본다. 무작위 필드 없음.
MUTANT_CATALOG = (
    {
        "id": "empty-to-nul",
        "family": "empty",
        "when": "n==0",
        "why": "빈 입력은 위치 기반 절단/플립을 할 수 없다. NUL 한 바이트로 고정한다.",
    },
    {
        "id": "truncate",
        "family": "truncate",
        "when": "n>0",
        "why": "잘린 복합문서·ZIP 중앙 디렉터리 부재를 재현한다.",
        "params": (10, 25, 50, 75, 95, 99),
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
        "params": (10, 25, 50, 75, 90),
    },
    {
        "id": "flip-edge",
        "family": "flip",
        "when": "n>0",
        "why": "첫 바이트와 마지막 바이트 플립은 퍼센트 위치와 겹치지 않을 수 있다.",
        "params": (0, 99),
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
        "id": "length-bomb",
        "family": "length",
        "when": "n>=4",
        "why": "추정 길이 필드에 0x7FFFFFFF 를 넣어 할당 폭주를 유도한다.",
        "params": (10, 40, 70),
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
        "why": "한가운데에 NUL 16바이트를 끼워 레코드 경계를 밀어낸다. 거대 입력은 크기가 늘지 않게 생략.",
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


def coerce_bytes(data) -> bytes:
    """바이트열만 받는다. bytearray/memoryview 는 복사하고, 그 외는 TypeError.

    감사기가 str/int/None 을 조용히 인코딩하면 결정성이 깨진다. 호출자가
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
    """표본 수 한도. 변환 불능·음수는 0 (표본 없음)."""
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


def mutant_family(label: str) -> str:
    """라벨을 가족 id 로 접는다. 카탈로그 확장 시 여기만 고치면 된다."""
    if not isinstance(label, str) or not label:
        return "other"
    if label == "empty-to-nul":
        return "empty"
    if (
        label.startswith("truncate@")
        or label in ("chop-last", "cut-first", "odd-length-chop", "shrink-gap")
    ):
        return "truncate"
    if label.startswith("flip@"):
        return "flip"
    if label in (
        "zero-header",
        "header-smash",
        "rotate-header",
        "increment-header",
        "nibble-swap-head",
    ):
        return "header"
    if label in ("ole-trunc-tail", "ole-magic-poison", "ole-sector-shift-poison", "ole-mini-fat-poison"):
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
    if (
        label.startswith("length-bomb")
        or label.startswith("length-zero")
        or label.startswith("length-one")
        or label.startswith("i32-min")
        or label.startswith("u16-max")
    ):
        return "length"
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


def probe_head(text: str, limit: int = PROBE_HEAD) -> str:
    """프로브 출력 머리. None/비문자는 빈 문자열."""
    if text is None:
        return ""
    if not isinstance(text, str):
        text = str(text)
    if limit <= 0:
        return ""
    return text[:limit]


def _posix_signal_timeout(code) -> bool:
    """일부 환경은 시간초과를 SIGKILL(-9) / SIGXCPU 로 돌려준다. 행으로 보지 않는다.

    감사는 subprocess.TimeoutExpired 를 행의 유일한 권위로 둔다. 시그널은 패닉 쪽
    (음수 코드)으로 분류한다. 이 헬퍼는 문서화용이며 classify_timeout 은 쓰지 않는다.
    """
    return code in (-9, -24, -30)


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
    """결정적 손상 변형들 — (라벨, 바이트). 무작위 없음(재현 가능).

    같은 입력 → 같은 라벨·바이트. 원본과 동일한 무의미 변형은 넣지 않는다.
    비-바이트는 TypeError. 빈 입력은 empty-to-nul 한 건만.
    """
    data = coerce_bytes(data)
    n = len(data)
    if n == 0:
        # 빈 입력은 위치 기반 flip/절단을 할 수 없지만, 감사 자체가 예외를 내면 안 된다.
        return [("empty-to-nul", b"\0")]
    out = []
    huge = n >= HUGE_MIN

    def add(label: str, mut: bytes) -> None:
        if mut != data:
            out.append((label, mut))

    for pct in (10, 25, 50, 75, 95, 99):  # 절단
        add(f"truncate@{pct}%", data[: max(1, n * pct // 100)])
    if n >= 2:
        add("chop-last", data[:-1])
    add("cut-first", data[1:])

    for pct in (0, 10, 25, 50, 75, 90, 99):  # 바이트 플립
        pos = min(n - 1, n * pct // 100)
        b = bytearray(data)
        b[pos] ^= 0xFF
        add(f"flip@{pct}%", bytes(b))

    b = bytearray(data)  # 헤더 매직 파손
    for i in range(min(n, 512)):
        b[i] = 0
    add("zero-header", bytes(b))

    smash_n = min(n, len(HEADER_SMASH_PAT))  # 헤더를 고정 패턴으로 덮어씀
    b = bytearray(data)
    b[:smash_n] = HEADER_SMASH_PAT[:smash_n]
    add("header-smash", bytes(b))

    if n >= 2:  # 선두 순환
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

    # OLE CFB 섹터(512)보다 짧은 꼬리 절단 — 디렉터리/FAT 가 잘린 복합문서.
    if n > 64:
        add("ole-trunc-tail", data[:-64])
    else:
        k = min(4, n)  # 8바이트 매직의 앞 4바이트만 = 잘린 OLE 꼬리
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

    start = n // 3  # 본문 한가운데 0xFF 런
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

    if n >= 2:  # UTF-16LE NUL(U+0000) 뿌림
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

    idx = _find_magic(data, ZIP_LOCAL)  # HWPX 등 ZIP 로컬 헤더만
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

    if n >= 4:
        for pct in (10, 40, 70):
            pos = min(n - 4, n * pct // 100)
            b = bytearray(data)
            b[pos : pos + 4] = LENGTH_BOMB_U32
            add(f"length-bomb@{pct}%", bytes(b))
        pos = min(n - 4, n * 30 // 100)
        b = bytearray(data)
        b[pos : pos + 4] = LENGTH_ZERO_U32
        add("length-zero@30%", bytes(b))
        pos = min(n - 4, n * 60 // 100)
        b = bytearray(data)
        b[pos : pos + 4] = LENGTH_ONE_U32
        add("length-one@60%", bytes(b))
        pos = min(n - 4, n * 20 // 100)
        b = bytearray(data)
        b[pos : pos + 4] = I32_MIN_LE
        add("i32-min@20%", bytes(b))

    if n >= 14:
        b = bytearray(data)
        b[12:14] = U16_MAX_LE
        add("u16-max@12", bytes(b))

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

    # 거대 입력에 바이트를 끼우면 사본이 커진다. 게이트는 헤더/절단만으로 충분.
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


def select_samples(samples_dir: str, limit: int):
    """정렬된 .hwp 를 결정적 stride 로 limit 개 뽑는다(형식·크기 다양성 확보).

    디렉터리를 읽을 수 없으면 빈 목록을 돌려 감사기가 예외로 죽지 않게 한다.
    """
    limit = normalize_limit(limit)
    try:
        names = os.listdir(samples_dir)
    except OSError:
        return [], 0
    everything = sorted(f for f in names if f.endswith(".hwp"))
    if not everything or limit <= 0:
        return everything[: max(0, limit)], len(everything)
    if len(everything) <= limit:
        return everything, len(everything)
    stride = len(everything) / limit
    picked = [everything[min(len(everything) - 1, int(i * stride))] for i in range(limit)]
    # stride 반올림 중복 제거(순서 유지)
    seen, uniq = set(), []
    for f in picked:
        if f not in seen:
            seen.add(f)
            uniq.append(f)
    return uniq, len(everything)


def is_panic(code, err: str) -> bool:
    """우아한 실패(비-0)와 패닉(크래시)을 가른다."""
    if err is None:
        err = ""
    elif not isinstance(err, str):
        err = str(err)
    low = err.lower()
    if any(marker in low for marker in PANIC_MARKERS):
        return True
    if code is None:
        return False
    try:
        code = int(code)
    except (TypeError, ValueError):
        return False
    # POSIX subprocess는 signal 종료를 음수로 돌려준다. Windows NTSTATUS 기반
    # 크래시는 큰 양수로 오므로 상위 두 비트로 구분한다. 임의의 CLI 오류 코드
    # (예: 255)를 패닉으로 오판하지 않는다.
    windows_exception = code >= 0 and (code & WINDOWS_EXCEPTION_MASK) == WINDOWS_EXCEPTION_MASK
    return code == RUST_ABORT or code < 0 or windows_exception


def classify_panic(code, err: str) -> bool:
    """패닉 분류 — `is_panic` 과 동일 판정(감사·시험 공통 진입점)."""
    return is_panic(code, err)


def classify_timeout(timed_out) -> bool:
    """행(timeout) 분류 — probe 의 timed_out, TimeoutExpired, 표식 문자열.

    True 만 행이다. False/None/그 외 예외는 행이 아니다.
    TimeoutExpired 와 TimeoutError 는 행. errno 시간초과 OSError 도 행.
    """
    if isinstance(timed_out, BaseException):
        if isinstance(timed_out, subprocess.TimeoutExpired):
            return True
        if isinstance(timed_out, TimeoutError):
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


def classify_probe_outcome(code, panicked, timed_out, head: str) -> str:
    """한 프로브 결과를 hang / panic / graceful / ok / error 로 접는다."""
    if classify_timeout(timed_out):
        return "hang"
    if classify_panic(code, head or "") or panicked:
        return "panic"
    if isinstance(head, str) and head.startswith(("oserror ", "probe-error ", "unreadable ")):
        return "error"
    if code not in (0, None):
        return "graceful"
    if code == 0:
        return "ok"
    return "error"


def read_sample(path: str):
    """샘플을 읽는다. 성공 시 (bytes, None), 실패 시 (None, 이유)."""
    try:
        with open(path, "rb") as fh:
            return fh.read(), None
    except OSError as exc:
        return None, f"{type(exc).__name__}: {exc}"
    except Exception as exc:  # noqa: BLE001 — 감사기 생존이 우선
        return None, f"{type(exc).__name__}: {exc}"


def write_mutant(path: str, mut: bytes):
    """변형 바이트를 쓴다. 성공 시 None, 실패 시 이유 문자열."""
    try:
        payload = coerce_bytes(mut)
    except TypeError as exc:
        return f"TypeError: {exc}"
    try:
        with open(path, "wb") as fh:
            fh.write(payload)
        return None
    except OSError as exc:
        return f"{type(exc).__name__}: {exc}"
    except Exception as exc:  # noqa: BLE001 — 감사기 생존이 우선
        return f"{type(exc).__name__}: {exc}"


def probe(bin_path: str, path: str, timeout: int):
    """한 손상 파일을 파싱 시도 — (code, panicked, timed_out, head).

    TimeoutExpired → 행. OSError(없는 바이너리·권한) → 패닉/행이 아닌 오류 머리.
    그 외 예외도 감사기를 죽이지 않는다.
    """
    seconds = normalize_timeout(timeout)
    if seconds <= 0:
        return None, False, False, "probe-error invalid-timeout"
    if not bin_path:
        return None, False, False, "probe-error missing-bin"
    try:
        p = subprocess.run(
            [bin_path, "info", path, "--json"],
            cwd=REPO_ROOT,
            capture_output=True,
            timeout=seconds,
        )
        err = p.stderr.decode("utf-8", "replace") + p.stdout.decode("utf-8", "replace")
        return p.returncode, classify_panic(p.returncode, err), False, probe_head(err)
    except subprocess.TimeoutExpired as exc:
        return None, False, classify_timeout(exc), f"timeout {seconds}s"
    except OSError as exc:
        return None, False, False, probe_head(f"oserror {type(exc).__name__}: {exc}")
    except Exception as exc:  # noqa: BLE001 — 감사기 생존이 우선
        return None, False, False, probe_head(f"probe-error {type(exc).__name__}: {exc}")


def empty_shape_counts() -> dict:
    return {key: 0 for key in SHAPE_KEYS}


def empty_report() -> dict:
    """스키마를 만족하는 빈 보고. 예외 경로에서 감사기가 죽지 않게 쓴다."""
    return {
        "kind": REPORT_KIND,
        "schemaVersion": SCHEMA_VERSION,
        "ok": True,
        "samplesTested": 0,
        "totalSamples": 0,
        "mutantsChecked": 0,
        "gracefullyDegraded": 0,
        "panics": [],
        "hangs": [],
        "unreadables": [],
        "probeErrors": [],
        "inputShapes": empty_shape_counts(),
    }


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
    if not isinstance(report.get("ok"), bool):
        issues.append("ok-type")
    for key in REPORT_INT_KEYS:
        if not isinstance(report.get(key), int):
            issues.append(f"{key}-type")
        elif report.get(key) < 0:
            issues.append(f"{key}-negative")
    for key in REPORT_LIST_KEYS:
        value = report.get(key)
        if not isinstance(value, list):
            issues.append(f"{key}-type")
        elif any(not isinstance(item, str) for item in value):
            issues.append(f"{key}-item-type")
    shapes = report.get("inputShapes")
    if not isinstance(shapes, dict):
        issues.append("inputShapes-type")
    else:
        for key in SHAPE_KEYS:
            if key not in shapes:
                issues.append(f"inputShapes-missing-{key}")
            elif not isinstance(shapes[key], int) or shapes[key] < 0:
                issues.append(f"inputShapes-{key}")
    if isinstance(report.get("ok"), bool):
        expected_ok = not report.get("panics") and not report.get("hangs")
        if report["ok"] != expected_ok:
            issues.append("ok-mismatch")
    return issues


def format_human_report(report: dict) -> str:
    """사람용 한 줄/여러 줄 요약. JSON 이 아닐 때 stdout 에 쓴다."""
    if report.get("ok"):
        return (
            f"gym 손상-강건성 감사: 샘플 {report['samplesTested']}/{report['totalSamples']} × "
            f"손상 {report['mutantsChecked']}건 — 패닉 0 · 행 0 "
            f"(우아한 실패/부분복구 {report['gracefullyDegraded']}"
            f" · 읽기실패 {len(report.get('unreadables', []))}"
            f" · 프로브오류 {len(report.get('probeErrors', []))})"
        )
    lines = [
        f"gym 손상-강건성 감사: 패닉 {len(report.get('panics', []))} · "
        f"행 {len(report.get('hangs', []))} — rhwp 가 손상 입력에 죽는다:"
    ]
    for item in list(report.get("panics", [])) + list(report.get("hangs", [])):
        lines.append(f"  - {item}")
    return "\n".join(lines)


def audit(bin_path: str, samples_dir: str, limit: int, timeout: int) -> dict:
    report = empty_report()
    try:
        picked, total = select_samples(samples_dir, limit)
    except Exception as exc:  # noqa: BLE001
        report["unreadables"].append(f"select_samples: {type(exc).__name__}: {exc}")
        report["ok"] = True
        return report
    report["samplesTested"] = len(picked)
    report["totalSamples"] = total
    panics, hangs = [], []
    unreadables, probe_errors = [], []
    shapes = empty_shape_counts()
    checked = 0
    degraded = 0
    try:
        work_cm = tempfile.TemporaryDirectory()
    except Exception as exc:  # noqa: BLE001
        report["probeErrors"].append(f"tempdir: {type(exc).__name__}: {exc}")
        return report
    with work_cm as work:
        mut_path = os.path.join(work, "mutant.hwp")
        for name in picked:
            data, read_err = read_sample(os.path.join(samples_dir, name))
            if read_err is not None or data is None:
                unreadables.append(f"{name}: {read_err or 'unreadable'}")
                continue
            try:
                shape = classify_input_shape(data)
            except TypeError as exc:
                unreadables.append(f"{name}: TypeError: {exc}")
                continue
            shapes[shape] = shapes.get(shape, 0) + 1
            try:
                mutants = deterministic_mutants(data)
            except TypeError as exc:
                unreadables.append(f"{name}: TypeError: {exc}")
                continue
            except Exception as exc:  # noqa: BLE001
                unreadables.append(f"{name}: {type(exc).__name__}: {exc}")
                continue
            for label, mut in mutants:
                write_err = write_mutant(mut_path, mut)
                if write_err is not None:
                    probe_errors.append(f"{name}:{label}: {write_err}")
                    continue
                try:
                    code, panicked, timed_out, head = probe(bin_path, mut_path, timeout)
                except Exception as exc:  # noqa: BLE001
                    probe_errors.append(f"{name}:{label}: probe-error {type(exc).__name__}: {exc}")
                    continue
                checked += 1
                tag = f"{name}:{label}"
                outcome = classify_probe_outcome(code, panicked, timed_out, head)
                if outcome == "hang":
                    hangs.append(tag)
                elif outcome == "panic":
                    panics.append(f"{tag} (code {code}): {head}")
                elif outcome == "error":
                    probe_errors.append(f"{tag}: {head}")
                elif outcome == "graceful":
                    degraded += 1
    report.update(
        {
            "ok": len(panics) == 0 and len(hangs) == 0,
            "samplesTested": len(picked),
            "totalSamples": total,
            "mutantsChecked": checked,
            "gracefullyDegraded": degraded,
            "panics": panics,
            "hangs": hangs,
            "unreadables": unreadables,
            "probeErrors": probe_errors,
            "inputShapes": shapes,
        }
    )
    return report


def main() -> int:
    ap = argparse.ArgumentParser(description="gym 손상-강건성 감사 — rhwp 패닉·행 색출")
    ap.add_argument("--bin", required=True)
    ap.add_argument("--limit", type=int, default=16, help="감사할 샘플 수(결정적 부분집합)")
    ap.add_argument("--timeout", type=int, default=20)
    ap.add_argument("--json", action="store_true")
    a = ap.parse_args()
    if a.timeout <= 0:
        ap.error("--timeout은 양수여야 합니다")
    try:
        bin_path = runner.find_bin(a.bin)
    except Exception as exc:  # noqa: BLE001
        print(f"gym 손상-강건성 감사: 바이너리를 찾지 못함: {type(exc).__name__}: {exc}", file=sys.stderr)
        return 2
    samples_dir = os.path.join(REPO_ROOT, "samples")
    try:
        report = audit(bin_path, samples_dir, a.limit, a.timeout)
    except Exception as exc:  # noqa: BLE001
        report = empty_report()
        report["probeErrors"].append(f"audit: {type(exc).__name__}: {exc}")
    issues = validate_report(report)
    if issues:
        report.setdefault("probeErrors", []).append("schema: " + ";".join(issues))
    if a.json:
        sys.stdout.write(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    else:
        print(format_human_report(report))
    return 0 if report.get("ok") else 1


if __name__ == "__main__":
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")  # type: ignore[attr-defined]
        except Exception:
            pass
    sys.exit(main())
