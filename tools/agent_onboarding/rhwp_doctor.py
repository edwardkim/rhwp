#!/usr/bin/env python3
"""rhwp_doctor.py — 실사용 에이전트 온보딩 닥터 + 부트스트랩.

한 명령으로 "rhwp 를 처음 보는 에이전트"가 다음을 한 번에 끝낸다:

  1. 바이너리 위치·버전 확인
     PATH → RHWP_BIN → target/release → target/debug → cargo bin.
     없으면 빌드 명령만 찍고 종료 코드 3 (긴 빌드를 대신 돌리지 않는다).
  2. 번들 샘플로 읽기 전용 자가검증
     매직 바이트로 불량 샘플을 먼저 거른 뒤 info / export-text 구조 출력을 확인.
     선택적으로 explain / digest / inspect injection 도 돌려 보되 임계는 아니다.
  3. 붙여넣기용 .mcp.json 스니펫 방출 (호스트별 모양 포함)
  4. 첫 5분 레시피 지도 (실존 스킬·레시피·references/ 만 인용)
  5. 예외 경로를 데이터로 보고
     missing_binary / bad_sample / no_network. 통과를 위조하지 않는다.

설계 규약(저장소 철학과 일치):
  - 판정은 데이터다: 못 돌린 검사는 SKIP/FAIL 로 이유와 함께 정직하게 보고한다.
  - 매달리지 않는다: 모든 하위 프로세스는 타임아웃으로 감싼다.
  - 네트워크는 선택이다: 오프라인이어도 로컬 샘플 자가검증은 돌아가야 한다.
  - 새 rhwp CLI 를 만들지 않는다. 편집 로직을 발명하지 않는다.
  - 순수 Python 3 표준 라이브러리만 사용한다(외부 의존성 0).

종료 코드(에이전트 계약):
  0  모든 임계 검사 통과 — 바로 붙여도 됨
  1  임계 검사 실패 — 바이너리는 있으나 버전/자가검증/불량 샘플
  2  사용법 오류 — 잘못된 인자, --write 덮어쓰기 거부(--force 없이)
  3  바이너리 미발견 — 아직 빌드 안 됨(조치: BUILD_COMMAND)

--json 을 주면 stdout 에는 기계 판독용 리포트 JSON 하나만 나가고, 사람용 텍스트는
전부 stderr 로 간다(에이전트가 stdout 을 그대로 파싱한다).
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import socket
import struct
import subprocess
import sys
from pathlib import Path

SCHEMA_VERSION = "1.1"
TOOL_NAME = "rhwp_doctor"
BUILD_COMMAND = "cargo build --release --bin rhwp"
# 하위 프로세스 상한(초) — 어떤 검사도 매달리지 않게 한다.
VERSION_TIMEOUT = 20
SELFTEST_TIMEOUT = 45
NETWORK_TIMEOUT = 2.0

# 자가검증에 쓸 "정상 문서" 후보(첫 존재 파일 선택). 병리적 픽스처가 아니라
# 평범한 문서만 고른다 — 자가검증이 실패하면 그건 진짜 신호여야 한다.
SAMPLE_CANDIDATES = [
    "samples/basic/english.hwp",
    "samples/basic/KTX.hwp",
    "samples/basic/BookReview.hwp",
    "samples/2022년 국립국어원 업무계획.hwp",
    "samples/2022년 국립국어원 업무계획.hwpx",
]

# 자가검증에서 피해야 할 상대 경로(병리·생성물·중간 산출).
SAMPLE_AVOID_PREFIXES = (
    "samples/broken/",
    "samples/fuzz/",
    "samples/malformed/",
    "output/",
    "saved/",
    "gym/",
)

# 첫 5분 레시피 지도 — 브리프가 지정한 5대 고가치 과제.
# 각 항목의 skill/recipe 경로는 런타임에 실존을 검증해 인용한다(없으면 정직하게 표시).
RECIPES = [
    {
        "task": "문서 트리아지 — 처음 보는 문서를 컨텍스트 아끼며 파악",
        "command": 'rhwp digest "<파일>" --json',
        "skill": "rhwp-doc-triage",
        "recipe": None,
    },
    {
        "task": "표 추출 — 병합 보존 격자 / CSV 왕복",
        "command": 'rhwp export-tables "<파일>" --json',
        "skill": "rhwp-table-exchange",
        "recipe": "mydocs/manual/recipes/02_table_csv_roundtrip.md",
    },
    {
        "task": "서식 채우기 — 누름틀 조사 후 값 채워 제출본 생성",
        "command": 'rhwp fields "<파일>" --json  →  rhwp edit fill-fields "<파일>" --data @row.json -o out.hwp --json',
        "skill": "rhwp-form-fill",
        "recipe": "mydocs/manual/recipes/01_fill_form_and_submit.md",
    },
    {
        "task": "보안 스윕 — 배포 전/수신 후 주입·은닉·유니코드 점검",
        "command": 'rhwp inspect injection "<파일>" --json',
        "skill": "rhwp-security-sweep",
        "recipe": "mydocs/manual/recipes/10_security_sweep_before_share.md",
    },
    {
        "task": "작업 영수증 — 산출물을 3-해시로 증명·재현 검증",
        "command": "rhwp replay --plan-json '{\"planVersion\":\"1.0\",...}' --json",
        "skill": "rhwp-work-receipt",
        "recipe": None,
    },
]

# 온보딩 스킬 references/ 에 대응하는 첫 5분 단계.
# 명령은 이미 존재하는 CLI 만 인용한다. 편집 로직을 여기서 발명하지 않는다.
FIRST_5_MIN = [
    {
        "id": "triage",
        "title": "처음 보는 문서 트리아지 (읽기 전용)",
        "minutes": 1,
        "commands": [
            'rhwp info "<파일>" --json',
            'rhwp explain "<파일>" --json',
            'rhwp digest "<파일>" --json --max-chars 1000',
        ],
        "skill": "rhwp-doc-triage",
        "reference": ".agents/skills/rhwp-onboarding/references/first-5-min-triage.md",
        "readOnly": True,
        "gate": "format 와 pageCount 가 있고, digest.truncated 를 읽어 절단을 숨기지 않는다",
    },
    {
        "id": "tables",
        "title": "표 좌표 확인 후 CSV 추출 (읽기 전용 확인 → 기존 왕복 레시피)",
        "minutes": 1,
        "commands": [
            'rhwp export-tables "<파일>" --json',
            'rhwp table-to-csv "<파일>" --table 0 --json',
        ],
        "skill": "rhwp-table-exchange",
        "reference": ".agents/skills/rhwp-onboarding/references/first-5-min-tables.md",
        "readOnly": True,
        "gate": "tables[].index / rows / cols / colSpan / rowSpan 을 읽고 병합이면 CSV 왕복을 하지 않는다",
    },
    {
        "id": "form-read",
        "title": "서식 누름틀 조사 (읽기 전용 — 채움은 기존 스킬에 위임)",
        "minutes": 1,
        "commands": [
            'rhwp fields "<파일>" --json',
        ],
        "skill": "rhwp-form-fill",
        "reference": ".agents/skills/rhwp-onboarding/references/first-5-min-form-read.md",
        "readOnly": True,
        "gate": "fieldCount 와 fields[].name 을 읽고, fieldCount==0 이면 이 축을 포기한다",
    },
    {
        "id": "security",
        "title": "수신/배포 전 보안 스윕 3축 (읽기 전용)",
        "minutes": 1,
        "commands": [
            'rhwp inspect hidden-text "<파일>" --json',
            'rhwp inspect injection "<파일>" --json',
            'rhwp inspect unicode "<파일>" --json',
        ],
        "skill": "rhwp-security-sweep",
        "reference": ".agents/skills/rhwp-onboarding/references/first-5-min-security.md",
        "readOnly": True,
        "gate": "clean 필드로 분기한다. 신호 발견은 exit 0 이며 오류가 아니다",
    },
    {
        "id": "attach",
        "title": "MCP 배선과 작업 영수증 입구",
        "minutes": 1,
        "commands": [
            "rhwp mcp-serve",
            "rhwp capabilities --mcp",
            'rhwp replay --plan-json \'{"planVersion":"1.0","input":"<파일>","output":"<산출>","steps":[]}\' --json',
        ],
        "skill": "rhwp-mcp-session",
        "reference": ".agents/skills/rhwp-onboarding/references/mcp-json-paste.md",
        "readOnly": True,
        "gate": "stdio MCP 만 쓴다. 포트·인증을 만들지 않는다. replay 는 기존 계획 스키마만 인용한다",
    },
]

# 온보딩 스킬이 반드시 실존해야 하는 참고 문서.
ONBOARDING_REFERENCES = [
    {
        "id": "skill",
        "path": ".agents/skills/rhwp-onboarding/SKILL.md",
        "role": "스킬 진입점",
    },
    {
        "id": "first-5-min",
        "path": ".agents/skills/rhwp-onboarding/references/first-5-min.md",
        "role": "첫 5분 레시피 지도",
    },
    {
        "id": "first-5-min-triage",
        "path": ".agents/skills/rhwp-onboarding/references/first-5-min-triage.md",
        "role": "트리아지 레시피",
    },
    {
        "id": "first-5-min-tables",
        "path": ".agents/skills/rhwp-onboarding/references/first-5-min-tables.md",
        "role": "표 추출 레시피",
    },
    {
        "id": "first-5-min-form-read",
        "path": ".agents/skills/rhwp-onboarding/references/first-5-min-form-read.md",
        "role": "서식 조사 레시피(읽기 전용)",
    },
    {
        "id": "first-5-min-security",
        "path": ".agents/skills/rhwp-onboarding/references/first-5-min-security.md",
        "role": "보안 스윕 레시피",
    },
    {
        "id": "mcp-json-paste",
        "path": ".agents/skills/rhwp-onboarding/references/mcp-json-paste.md",
        "role": "호스트별 .mcp.json 붙여넣기",
    },
    {
        "id": "binary-discovery",
        "path": ".agents/skills/rhwp-onboarding/references/binary-discovery.md",
        "role": "바이너리 발견 순서",
    },
    {
        "id": "sample-selftest",
        "path": ".agents/skills/rhwp-onboarding/references/sample-selftest.md",
        "role": "샘플 자가검증 계약",
    },
    {
        "id": "exception-missing-binary",
        "path": ".agents/skills/rhwp-onboarding/references/exception-missing-binary.md",
        "role": "예외: 바이너리 없음",
    },
    {
        "id": "exception-bad-sample",
        "path": ".agents/skills/rhwp-onboarding/references/exception-bad-sample.md",
        "role": "예외: 불량 샘플",
    },
    {
        "id": "exception-no-network",
        "path": ".agents/skills/rhwp-onboarding/references/exception-no-network.md",
        "role": "예외: 네트워크 없음",
    },
    {
        "id": "exception-transcripts",
        "path": ".agents/skills/rhwp-onboarding/references/exception-transcripts.md",
        "role": "예외 리포트 트랜스크립트",
    },
    {
        "id": "first-5-min-envelopes",
        "path": ".agents/skills/rhwp-onboarding/references/first-5-min-envelopes.md",
        "role": "첫 5분 봉투 필드",
    },
    {
        "id": "binary-discovery-matrix",
        "path": ".agents/skills/rhwp-onboarding/references/binary-discovery-matrix.md",
        "role": "OS별 바이너리 자리",
    },
    {
        "id": "host-paste-examples",
        "path": ".agents/skills/rhwp-onboarding/references/host-paste-examples.md",
        "role": "호스트별 붙여넣기 예",
    },
    {
        "id": "first-5-min-receipt",
        "path": ".agents/skills/rhwp-onboarding/references/first-5-min-receipt.md",
        "role": "작업 영수증 입구",
    },
    {
        "id": "doctor-report-schema",
        "path": ".agents/skills/rhwp-onboarding/references/doctor-report-schema.md",
        "role": "닥터 JSON 리포트 스키마",
    },
    {
        "id": "onboarding-catalog",
        "path": ".agents/skills/rhwp-onboarding/references/onboarding-catalog.md",
        "role": "명령·예외·호스트 교차표",
    },
    {
        "id": "working-doc",
        "path": "mydocs/working/archives/agent_onboarding.md",
        "role": "작업 기록(스테이지)",
    },
    {
        "id": "manual-5min",
        "path": "mydocs/manual/agent_onboarding.md",
        "role": "5분 경로 정본",
    },
]

# 호스트별 MCP 붙여넣기 모양. 전송은 전부 stdio. 포트·인증 없음.
# shape A = {mcpServers:{name:{command,args}}}
# shape B = {servers:{name:{type:stdio,command,args}}}  (VS Code)
# shape zed / goose / continue 는 별도 스키마.
MCP_HOSTS = [
    {
        "id": "claude-code",
        "title": "Claude Code",
        "file": ".mcp.json",
        "shape": "A",
        "confidence": "repo-proven",
    },
    {
        "id": "claude-desktop",
        "title": "Claude Desktop",
        "file": "%APPDATA%/Claude/claude_desktop_config.json",
        "shape": "A",
        "confidence": "high",
    },
    {
        "id": "cursor",
        "title": "Cursor",
        "file": ".cursor/mcp.json",
        "shape": "A",
        "confidence": "high",
    },
    {
        "id": "cline",
        "title": "Cline",
        "file": "cline_mcp_settings.json",
        "shape": "A",
        "confidence": "high",
    },
    {
        "id": "windsurf",
        "title": "Windsurf",
        "file": "~/.codeium/windsurf/mcp_config.json",
        "shape": "A",
        "confidence": "high",
    },
    {
        "id": "vscode",
        "title": "VS Code / Copilot",
        "file": ".vscode/mcp.json",
        "shape": "B",
        "confidence": "high",
    },
    {
        "id": "gemini-cli",
        "title": "Gemini CLI",
        "file": "~/.gemini/settings.json",
        "shape": "A",
        "confidence": "high",
    },
    {
        "id": "qwen-code",
        "title": "Qwen Code",
        "file": "~/.qwen/settings.json",
        "shape": "A",
        "confidence": "medium",
    },
    {
        "id": "roo",
        "title": "Roo Code",
        "file": ".roo/mcp.json",
        "shape": "A",
        "confidence": "medium",
    },
    {
        "id": "kilo",
        "title": "Kilo Code",
        "file": ".kilocode/mcp.json",
        "shape": "A",
        "confidence": "medium",
    },
    {
        "id": "kiro",
        "title": "Kiro",
        "file": ".kiro/settings/mcp.json",
        "shape": "A",
        "confidence": "medium",
    },
    {
        "id": "amazon-q",
        "title": "Amazon Q Dev CLI",
        "file": ".amazonq/mcp.json",
        "shape": "A",
        "confidence": "medium",
    },
    {
        "id": "zed",
        "title": "Zed",
        "file": "settings.json",
        "shape": "zed",
        "confidence": "medium",
    },
    {
        "id": "goose",
        "title": "Goose",
        "file": "~/.config/goose/config.yaml",
        "shape": "goose",
        "confidence": "medium",
    },
    {
        "id": "continue",
        "title": "Continue",
        "file": "~/.continue/config.yaml",
        "shape": "continue",
        "confidence": "medium",
    },
]

# 파일 매직. HWP5 는 OLE Compound, HWPX 는 ZIP, HWP3 는 고정 시그니처.
OLE_MAGIC = b"\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1"
ZIP_MAGIC = b"PK"
HWP3_MAGIC = b"HWP Document File"
MIN_SAMPLE_BYTES = 64

# 읽기 전용 자가검증이 기대하는 봉투 키. 필드를 지어내지 않는다.
INFO_REQUIRED_KEYS = ("format", "pageCount")
EXPORT_TEXT_REQUIRED_KEYS = ("pages",)
EXPLAIN_REQUIRED_KEYS = ("format", "pageCount", "summary")
DIGEST_REQUIRED_KEYS = ("schemaVersion", "source")
INJECTION_REQUIRED_KEYS = ("clean", "signalCount")
FIELDS_REQUIRED_KEYS = ("fieldCount", "fields")
EXPORT_TABLES_REQUIRED_KEYS = ("tableCount", "tables")

PASS, FAIL, SKIP = "PASS", "FAIL", "SKIP"

EXC_MISSING_BINARY = "missing_binary"
EXC_BAD_SAMPLE = "bad_sample"
EXC_NO_NETWORK = "no_network"
EXC_WRITE_EXISTS = "write_exists"
EXC_SELFTEST_TIMEOUT = "selftest_timeout"
EXC_SELFTEST_PARSE = "selftest_parse"

KIND_MISSING = "missing"
KIND_EMPTY = "empty"
KIND_TOO_SMALL = "too_small"
KIND_NOT_DOCUMENT = "not_document"
KIND_HWP5 = "hwp5"
KIND_HWPX = "hwpx"
KIND_HWP3 = "hwp3"
KIND_AVOID = "avoid"

NETWORK_PROBES = (
    ("1.1.1.1", 443),
    ("8.8.8.8", 443),
)

ENV_BIN_KEYS = ("RHWP_BIN", "RHWP")


# --------------------------------------------------------------------------- #
# 순수 로직(바이너리 불요) — 가드 테스트가 여기를 겨눈다.
# --------------------------------------------------------------------------- #
def default_repo_root() -> Path:
    """이 스크립트 위치(tools/agent_onboarding/x.py)에서 저장소 루트를 유도한다."""
    return Path(__file__).resolve().parents[2]


def fixture_dir() -> Path:
    """닥터와 같은 폴더의 fixtures/ (테스트·레시피가 인용하는 정적 계약)."""
    return Path(__file__).resolve().parent / "fixtures"


def build_mcp_snippet(command: str, args=None):
    """붙여넣기용 .mcp.json 딕셔너리를 만든다.

    command 은 PATH 에 rhwp 가 있으면 "rhwp", 아니면 바이너리 절대 경로다
    (mcp_integration_guide.md: "PATH 에 없으면 command 에 절대 경로를 쓴다").
    """
    if args is None:
        args = ["mcp-serve"]
    return {"mcpServers": {"rhwp": {"command": command, "args": list(args)}}}


def build_mcp_snippet_for_host(host_id: str, command: str, args=None):
    """호스트 모양에 맞는 붙여넣기 조각을 만든다. 모르는 host_id 는 None.

    전송은 항상 stdio 이고 args 기본은 ["mcp-serve"] 다. 새 CLI 를 만들지 않는다.
    """
    if args is None:
        args = ["mcp-serve"]
    host = next((h for h in MCP_HOSTS if h["id"] == host_id), None)
    if host is None:
        return None
    shape = host["shape"]
    argv = list(args)
    if shape == "A":
        snippet = {"mcpServers": {"rhwp": {"command": command, "args": argv}}}
    elif shape == "B":
        snippet = {"servers": {"rhwp": {"type": "stdio", "command": command, "args": argv}}}
    elif shape == "zed":
        snippet = {
            "context_servers": {
                "rhwp": {"source": "custom", "command": {"path": command, "args": argv}}
            }
        }
    elif shape == "goose":
        snippet = {"rhwp": {"type": "stdio", "cmd": command, "args": argv}}
    elif shape == "continue":
        snippet = {"mcpServers": [{"name": "rhwp", "command": command, "args": argv}]}
    else:
        return None
    return {
        "host": host["id"],
        "title": host["title"],
        "file": host["file"],
        "shape": shape,
        "confidence": host["confidence"],
        "snippet": snippet,
    }


def list_mcp_hosts():
    """호스트 카탈로그를 복사해 돌려준다(테스트가 별칭을 못 바꾸게)."""
    return [dict(h) for h in MCP_HOSTS]


def aggregate(checks, binary_found: bool):
    """검사 목록 → (ok, exit_code). 순수 함수(가드 테스트 대상).

    ok 는 임계 검사가 하나도 실패/스킵되지 않았을 때만 True.
    exit_code: 0 정상 / 3 바이너리 미발견 / 1 임계 실패.
    """
    critical = [c for c in checks if c.get("critical")]
    all_pass = all(c["status"] == PASS for c in critical)
    if not binary_found:
        return False, 3
    if all_pass:
        return True, 0
    return False, 1


def resolve_recipe_map(repo_root: Path):
    """RECIPES 를 실존 검증과 함께 해석한다. 없는 스킬/레시피는 정직하게 표시."""
    out = []
    for r in RECIPES:
        skill_rel = f".agents/skills/{r['skill']}"
        skill_exists = (repo_root / skill_rel / "SKILL.md").is_file()
        recipe_rel = r["recipe"]
        recipe_exists = bool(recipe_rel) and (repo_root / recipe_rel).is_file()
        out.append(
            {
                "task": r["task"],
                "command": r["command"],
                "skill": r["skill"],
                "skillPath": skill_rel,
                "skillExists": skill_exists,
                "recipe": recipe_rel,
                "recipeExists": recipe_exists,
            }
        )
    return out


def resolve_first_5_min(repo_root: Path):
    """첫 5분 단계의 참고 문서 실존을 검증한다."""
    out = []
    for step in FIRST_5_MIN:
        ref = step["reference"]
        skill_rel = f".agents/skills/{step['skill']}"
        out.append(
            {
                "id": step["id"],
                "title": step["title"],
                "minutes": step["minutes"],
                "commands": list(step["commands"]),
                "skill": step["skill"],
                "skillExists": (repo_root / skill_rel / "SKILL.md").is_file(),
                "reference": ref,
                "referenceExists": (repo_root / ref).is_file(),
                "readOnly": step["readOnly"],
                "gate": step["gate"],
            }
        )
    return out


def resolve_onboarding_references(repo_root: Path):
    """온보딩 스킬 references/ 와 작업 문서의 실존을 검증한다."""
    out = []
    for item in ONBOARDING_REFERENCES:
        exists = (repo_root / item["path"]).is_file()
        out.append(
            {
                "id": item["id"],
                "path": item["path"],
                "role": item["role"],
                "exists": exists,
            }
        )
    return out


def pick_sample(repo_root: Path, override: str | None):
    """자가검증용 샘플 경로를 고른다(override 우선, 아니면 후보 중 첫 존재)."""
    if override:
        p = Path(override)
        return p if p.is_file() else None
    for rel in SAMPLE_CANDIDATES:
        p = repo_root / rel
        if p.is_file():
            return p
    return None


def should_avoid_sample(path: Path, repo_root: Path | None = None) -> bool:
    """병리·산출 경로면 True. 자가검증 후보에서 제외한다."""
    text = str(path).replace("\\", "/")
    for prefix in SAMPLE_AVOID_PREFIXES:
        if prefix in text:
            return True
    if repo_root is not None:
        try:
            rel = path.resolve().relative_to(repo_root.resolve()).as_posix()
        except ValueError:
            rel = text
        for prefix in SAMPLE_AVOID_PREFIXES:
            if rel.startswith(prefix):
                return True
    return False


def read_magic(path: Path, nbytes: int = 32) -> bytes:
    """파일 선두 바이트. 읽기 실패면 빈 바이트."""
    try:
        with path.open("rb") as fh:
            return fh.read(nbytes)
    except OSError:
        return b""


def classify_magic(blob: bytes) -> str:
    """선두 바이트만으로 문서 종류를 분류한다. 내용을 해석하지 않는다."""
    if not blob:
        return KIND_EMPTY
    if blob.startswith(OLE_MAGIC):
        return KIND_HWP5
    if blob.startswith(ZIP_MAGIC):
        return KIND_HWPX
    if blob.startswith(HWP3_MAGIC):
        return KIND_HWP3
    return KIND_NOT_DOCUMENT


def classify_sample(path: Path | None, repo_root: Path | None = None):
    """자가검증 입력을 매직·크기·회피 경로로 분류한다. rhwp 를 실행하지 않는다.

    반환 키: ok, kind, reason, sizeBytes, magicHex, path.
    ok 가 True 인 경우만 info/export-text 를 돌릴 가치가 있다.
    """
    if path is None:
        return {
            "ok": False,
            "kind": KIND_MISSING,
            "reason": "샘플 경로가 없다. --sample 로 지정하거나 samples/ 를 체크아웃한다.",
            "sizeBytes": 0,
            "magicHex": "",
            "path": None,
        }
    p = Path(path)
    if not p.is_file():
        return {
            "ok": False,
            "kind": KIND_MISSING,
            "reason": f"파일이 없다: {p}",
            "sizeBytes": 0,
            "magicHex": "",
            "path": str(p),
        }
    if should_avoid_sample(p, repo_root):
        return {
            "ok": False,
            "kind": KIND_AVOID,
            "reason": "병리·산출 경로는 자가검증 후보가 아니다.",
            "sizeBytes": _safe_size(p),
            "magicHex": read_magic(p, 8).hex(),
            "path": str(p),
        }
    size = _safe_size(p)
    if size == 0:
        return {
            "ok": False,
            "kind": KIND_EMPTY,
            "reason": "빈 파일이다. HWP/HWPX 가 아니다.",
            "sizeBytes": 0,
            "magicHex": "",
            "path": str(p),
        }
    if size < MIN_SAMPLE_BYTES:
        return {
            "ok": False,
            "kind": KIND_TOO_SMALL,
            "reason": f"{size}바이트는 문서 하한({MIN_SAMPLE_BYTES})보다 작다.",
            "sizeBytes": size,
            "magicHex": read_magic(p, 8).hex(),
            "path": str(p),
        }
    magic = read_magic(p, 32)
    kind = classify_magic(magic)
    if kind == KIND_NOT_DOCUMENT:
        return {
            "ok": False,
            "kind": KIND_NOT_DOCUMENT,
            "reason": "OLE/ZIP/HWP3 시그니처가 없다. 텍스트·잘린 파일일 가능성이 크다.",
            "sizeBytes": size,
            "magicHex": magic[:8].hex(),
            "path": str(p),
        }
    return {
        "ok": True,
        "kind": kind,
        "reason": f"{kind} 시그니처, {size}바이트",
        "sizeBytes": size,
        "magicHex": magic[:8].hex(),
        "path": str(p),
    }


def _safe_size(path: Path) -> int:
    try:
        return path.stat().st_size
    except OSError:
        return 0


def classify_selftest_failure(exit_code: int, stderr_text: str, timed_out: bool = False):
    """자가검증 실패를 예외 종류로 분류한다. 편집 실패 축을 만들지 않는다."""
    if timed_out:
        return EXC_SELFTEST_TIMEOUT
    text = (stderr_text or "").lower()
    if "json" in text and ("parse" in text or "decode" in text):
        return EXC_SELFTEST_PARSE
    if exit_code == 1:
        # 런타임 실패 — 파일 형식·파싱. 불량 샘플로 본다.
        return EXC_BAD_SAMPLE
    if exit_code == 2:
        return EXC_SELFTEST_PARSE
    return EXC_BAD_SAMPLE


def exception_playbook(kind: str):
    """예외 종류별 다음 행동. 새 CLI 를 제안하지 않는다."""
    books = {
        EXC_MISSING_BINARY: {
            "title": "바이너리 미발견",
            "nextSteps": [
                f"저장소 루트에서 `{BUILD_COMMAND}` 를 한 번 실행한다.",
                "산출물 target/release/rhwp (Windows: rhwp.exe) 를 확인한다.",
                "PATH 에 넣거나 --rhwp / RHWP_BIN 으로 절대 경로를 준다.",
                "닥터를 다시 실행한다. 긴 빌드를 닥터에게 맡기지 않는다.",
                "상세: .agents/skills/rhwp-onboarding/references/exception-missing-binary.md",
            ],
        },
        EXC_BAD_SAMPLE: {
            "title": "불량·부재 샘플",
            "nextSteps": [
                "samples/basic/english.hwp 같은 평범한 번들 문서를 쓴다.",
                "직접 준 파일이면 OLE(D0 CF 11 E0) 또는 ZIP(PK) 시그니처를 확인한다.",
                "빈 파일·txt 를 .hwp 로 이름만 바꾼 입력은 자가검증 대상이 아니다.",
                "--sample <실제.hwp|실제.hwpx> 로 정상 문서를 지정한다.",
                "상세: .agents/skills/rhwp-onboarding/references/exception-bad-sample.md",
            ],
        },
        EXC_NO_NETWORK: {
            "title": "네트워크 없음",
            "nextSteps": [
                "오프라인은 실패가 아니다. 로컬 바이너리와 samples/ 로 자가검증한다.",
                "cargo build 는 의존성이 캐시돼 있으면 오프라인에서도 된다.",
                "crates.io / GitHub 에서 새로 받기는 미룬다.",
                "--offline 으로 프로브를 건너뛸 수 있다.",
                "상세: .agents/skills/rhwp-onboarding/references/exception-no-network.md",
            ],
        },
        EXC_WRITE_EXISTS: {
            "title": ".mcp.json 이 이미 있다",
            "nextSteps": [
                "기존 파일을 읽고 mcpServers.rhwp 만 병합한다.",
                "덮어쓰려면 --write <경로> --force 를 명시한다.",
            ],
        },
        EXC_SELFTEST_TIMEOUT: {
            "title": "자가검증 타임아웃",
            "nextSteps": [
                f"{SELFTEST_TIMEOUT}s 안에 응답이 없다. 매직이 맞는 작은 샘플로 바꾼다.",
                "samples/basic/english.hwp 를 --sample 로 지정한다.",
            ],
        },
        EXC_SELFTEST_PARSE: {
            "title": "자가검증 JSON 파싱 실패",
            "nextSteps": [
                "같은 명령을 손으로 실행해 stderr 를 본다.",
                "stdout 이 JSON 하나가 아니면 바이너리/리다이렉트를 의심한다.",
            ],
        },
    }
    return books.get(
        kind,
        {"title": kind, "nextSteps": ["리포트의 detail 을 읽고 같은 명령을 손으로 재현한다."]},
    )


def make_exception(kind: str, detail: str, path: str | None = None):
    """exceptions[] 한 줄. 판정은 데이터다."""
    book = exception_playbook(kind)
    return {
        "kind": kind,
        "title": book["title"],
        "detail": detail,
        "path": path,
        "nextSteps": list(book["nextSteps"]),
    }


# --------------------------------------------------------------------------- #
# 바이너리 조달·실행
# --------------------------------------------------------------------------- #
def _exe_name() -> str:
    return "rhwp.exe" if os.name == "nt" else "rhwp"


def cargo_home() -> Path | None:
    raw = os.environ.get("CARGO_HOME")
    if raw:
        return Path(raw)
    home = Path.home()
    cand = home / ".cargo"
    return cand if cand.is_dir() else None


def binary_search_plan(repo_root: Path, override: str | None, env=None):
    """탐색 순서를 데이터로 돌려준다. 실행하지 않는다.

    순서: --rhwp → RHWP_BIN/RHWP → PATH → target/release → target/debug → cargo bin.
    """
    env = os.environ if env is None else env
    exe = _exe_name()
    rows = []
    if override:
        rows.append({"source": "--rhwp", "path": str(Path(override)), "kind": "override"})
    for key in ENV_BIN_KEYS:
        val = env.get(key)
        if val:
            rows.append({"source": key, "path": str(Path(val)), "kind": "env"})
    rows.append({"source": "PATH", "path": "rhwp", "kind": "which"})
    rows.append(
        {
            "source": "target/release",
            "path": str(repo_root / "target" / "release" / exe),
            "kind": "file",
        }
    )
    rows.append(
        {
            "source": "target/debug",
            "path": str(repo_root / "target" / "debug" / exe),
            "kind": "file",
        }
    )
    ch = cargo_home()
    if ch is not None:
        rows.append({"source": "cargo-bin", "path": str(ch / "bin" / exe), "kind": "file"})
    return rows


def discover_binary_candidates(repo_root: Path, override: str | None = None, env=None):
    """탐색 계획의 각 자리가 실제 파일인지 표시한다. 고르지는 않는다."""
    env = os.environ if env is None else env
    out = []
    for row in binary_search_plan(repo_root, override, env):
        item = dict(row)
        if row["kind"] == "which":
            found = shutil.which("rhwp") or shutil.which("rhwp.exe")
            item["resolved"] = found
            item["exists"] = bool(found)
        else:
            p = Path(row["path"])
            item["resolved"] = str(p) if p.is_file() else None
            item["exists"] = p.is_file()
        out.append(item)
    return out


def find_binary(repo_root: Path, override: str | None):
    """rhwp 바이너리를 찾는다. 반환: (path|None, source, on_path).

    호환 계약: --rhwp 가 있으면 그것만 보고, 없으면 PATH 다음 target/release.
    추가 후보는 PATH/release 가 없을 때만 쓴다(기존 동작을 깨지 않기 위해).
    """
    if override:
        p = Path(override)
        if p.is_file():
            return p, "--rhwp", False
        return None, "--rhwp(미발견)", False
    on_path = shutil.which("rhwp")
    if on_path:
        return Path(on_path), "PATH", True
    exe = _exe_name()
    cand = repo_root / "target" / "release" / exe
    if cand.is_file():
        return cand, "target/release", False
    # 확장 탐색 — 기존 두 자리가 비었을 때만.
    env_path = os.environ.get("RHWP_BIN") or os.environ.get("RHWP")
    if env_path:
        p = Path(env_path)
        if p.is_file():
            return p, "RHWP_BIN", False
    debug = repo_root / "target" / "debug" / exe
    if debug.is_file():
        return debug, "target/debug", False
    ch = cargo_home()
    if ch is not None:
        cargo_bin = ch / "bin" / exe
        if cargo_bin.is_file():
            return cargo_bin, "cargo-bin", False
    return None, "(미발견)", False


def choose_mcp_command(binary: Path | None, on_path: bool) -> str:
    """스니펫 command 칸. PATH 에 있으면 짧은 이름, 없으면 절대 경로."""
    if binary is not None and not on_path:
        return str(binary)
    return "rhwp"


def _run(binary: Path, args, timeout: int):
    """rhwp 를 실행하고 (exit, stdout_str, stderr_str) 반환. 타임아웃/오류는 예외로 던진다.

    Windows cp949 로케일에서도 UTF-8 JSON 이 깨지지 않도록 bytes 로 받아 직접 디코드한다.
    """
    proc = subprocess.run(
        [str(binary), *args],
        capture_output=True,
        timeout=timeout,
        check=False,
    )
    out = proc.stdout.decode("utf-8", errors="replace")
    err = proc.stderr.decode("utf-8", errors="replace")
    return proc.returncode, out, err


def parse_json_object(text: str):
    """stdout 에서 JSON 객체 하나를 읽는다. 실패하면 (None, 이유)."""
    raw = (text or "").strip()
    if not raw:
        return None, "stdout 이 비었다"
    try:
        obj = json.loads(raw)
    except json.JSONDecodeError as e:
        return None, f"JSON 파싱 실패: {e}"
    if not isinstance(obj, dict):
        return None, f"최상위가 object 가 아님: {type(obj).__name__}"
    return obj, None


def missing_keys(obj: dict, required) -> list:
    return [k for k in required if k not in obj]


def check_version(binary: Path):
    cmd = "rhwp --version"
    try:
        code, out, err = _run(binary, ["--version"], VERSION_TIMEOUT)
    except subprocess.TimeoutExpired:
        return _mk(
            "version",
            "바이너리 버전",
            FAIL,
            cmd,
            f"{VERSION_TIMEOUT}s 내 무응답(타임아웃)",
            True,
            exception=EXC_SELFTEST_TIMEOUT,
        )
    except OSError as e:
        return _mk("version", "바이너리 버전", FAIL, cmd, f"실행 불가: {e}", True)
    text = (out or err).strip()
    if code == 0 and text:
        return _mk(
            "version",
            "바이너리 버전",
            PASS,
            cmd,
            text.splitlines()[0],
            True,
            version=text.splitlines()[0],
        )
    return _mk("version", "바이너리 버전", FAIL, cmd, f"exit={code}, 출력='{text[:80]}'", True)


def check_info(binary: Path, sample: Path):
    cmd = f'rhwp info "{sample}" --json'
    try:
        code, out, err = _run(binary, ["info", str(sample), "--json"], SELFTEST_TIMEOUT)
    except subprocess.TimeoutExpired:
        return _mk(
            "selftest-info",
            "자가검증: info",
            FAIL,
            cmd,
            f"{SELFTEST_TIMEOUT}s 내 무응답(타임아웃)",
            True,
            exception=EXC_SELFTEST_TIMEOUT,
        )
    except OSError as e:
        return _mk("selftest-info", "자가검증: info", FAIL, cmd, f"실행 불가: {e}", True)
    if code != 0:
        kind = classify_selftest_failure(code, err or out, False)
        return _mk(
            "selftest-info",
            "자가검증: info",
            FAIL,
            cmd,
            f"exit={code}: {(err or out).strip()[:120]}",
            True,
            exception=kind,
        )
    obj, err_msg = parse_json_object(out)
    if obj is None:
        return _mk(
            "selftest-info",
            "자가검증: info",
            FAIL,
            cmd,
            err_msg,
            True,
            exception=EXC_SELFTEST_PARSE,
        )
    missing = missing_keys(obj, INFO_REQUIRED_KEYS)
    if missing:
        return _mk(
            "selftest-info",
            "자가검증: info",
            FAIL,
            cmd,
            f"구조 출력에 {','.join(missing)} 없음",
            True,
        )
    detail = f"format={obj.get('format')}, pageCount={obj.get('pageCount')}, version={obj.get('version')}"
    return _mk("selftest-info", "자가검증: info", PASS, cmd, detail, True)


def check_export_text(binary: Path, sample: Path):
    cmd = f'rhwp export-text "{sample}" --json --max-chars 2000'
    try:
        code, out, err = _run(
            binary, ["export-text", str(sample), "--json", "--max-chars", "2000"], SELFTEST_TIMEOUT
        )
    except subprocess.TimeoutExpired:
        return _mk(
            "selftest-export-text",
            "자가검증: export-text",
            FAIL,
            cmd,
            f"{SELFTEST_TIMEOUT}s 내 무응답(타임아웃)",
            True,
            exception=EXC_SELFTEST_TIMEOUT,
        )
    except OSError as e:
        return _mk(
            "selftest-export-text",
            "자가검증: export-text",
            FAIL,
            cmd,
            f"실행 불가: {e}",
            True,
        )
    if code != 0:
        kind = classify_selftest_failure(code, err or out, False)
        return _mk(
            "selftest-export-text",
            "자가검증: export-text",
            FAIL,
            cmd,
            f"exit={code}: {(err or out).strip()[:120]}",
            True,
            exception=kind,
        )
    obj, err_msg = parse_json_object(out)
    if obj is None:
        return _mk(
            "selftest-export-text",
            "자가검증: export-text",
            FAIL,
            cmd,
            err_msg,
            True,
            exception=EXC_SELFTEST_PARSE,
        )
    pages = obj.get("pages") if isinstance(obj, dict) else None
    if not isinstance(pages, list) or len(pages) < 1:
        return _mk(
            "selftest-export-text",
            "자가검증: export-text",
            FAIL,
            cmd,
            "pages 배열이 비었거나 없음",
            True,
        )
    chars = sum(len(p.get("text", "")) for p in pages if isinstance(p, dict))
    detail = f"pageCount={obj.get('pageCount')}, pages={len(pages)}, 본문문자={chars}"
    return _mk("selftest-export-text", "자가검증: export-text", PASS, cmd, detail, True)


def check_explain(binary: Path, sample: Path):
    """비임계. 구버전 바이너리에 explain 이 없으면 SKIP."""
    cmd = f'rhwp explain "{sample}" --json'
    try:
        code, out, err = _run(binary, ["explain", str(sample), "--json"], SELFTEST_TIMEOUT)
    except subprocess.TimeoutExpired:
        return _mk(
            "selftest-explain",
            "자가검증: explain",
            FAIL,
            cmd,
            f"{SELFTEST_TIMEOUT}s 내 무응답(타임아웃)",
            False,
            exception=EXC_SELFTEST_TIMEOUT,
        )
    except OSError as e:
        return _mk("selftest-explain", "자가검증: explain", SKIP, cmd, f"실행 불가: {e}", False)
    text = (err or out).strip()
    if code == 2 and "알 수 없는 명령" in text:
        return _mk("selftest-explain", "자가검증: explain", SKIP, cmd, "이 바이너리에 explain 없음", False)
    if code != 0:
        return _mk(
            "selftest-explain",
            "자가검증: explain",
            FAIL,
            cmd,
            f"exit={code}: {text[:120]}",
            False,
        )
    obj, err_msg = parse_json_object(out)
    if obj is None:
        return _mk("selftest-explain", "자가검증: explain", FAIL, cmd, err_msg, False)
    missing = missing_keys(obj, EXPLAIN_REQUIRED_KEYS)
    if missing:
        return _mk(
            "selftest-explain",
            "자가검증: explain",
            FAIL,
            cmd,
            f"키 없음: {','.join(missing)}",
            False,
        )
    summary = str(obj.get("summary") or "")
    return _mk(
        "selftest-explain",
        "자가검증: explain",
        PASS,
        cmd,
        f"format={obj.get('format')}, pageCount={obj.get('pageCount')}, summary={summary[:80]}",
        False,
    )


def check_digest(binary: Path, sample: Path):
    """비임계. digest 가 없으면 SKIP."""
    cmd = f'rhwp digest "{sample}" --json --max-chars 500'
    try:
        code, out, err = _run(
            binary, ["digest", str(sample), "--json", "--max-chars", "500"], SELFTEST_TIMEOUT
        )
    except subprocess.TimeoutExpired:
        return _mk(
            "selftest-digest",
            "자가검증: digest",
            FAIL,
            cmd,
            f"{SELFTEST_TIMEOUT}s 내 무응답(타임아웃)",
            False,
            exception=EXC_SELFTEST_TIMEOUT,
        )
    except OSError as e:
        return _mk("selftest-digest", "자가검증: digest", SKIP, cmd, f"실행 불가: {e}", False)
    text = (err or out).strip()
    if code == 2 and "알 수 없는 명령" in text:
        return _mk("selftest-digest", "자가검증: digest", SKIP, cmd, "이 바이너리에 digest 없음", False)
    if code != 0:
        return _mk("selftest-digest", "자가검증: digest", FAIL, cmd, f"exit={code}: {text[:120]}", False)
    obj, err_msg = parse_json_object(out)
    if obj is None:
        return _mk("selftest-digest", "자가검증: digest", FAIL, cmd, err_msg, False)
    missing = missing_keys(obj, DIGEST_REQUIRED_KEYS)
    if missing:
        return _mk(
            "selftest-digest",
            "자가검증: digest",
            FAIL,
            cmd,
            f"키 없음: {','.join(missing)}",
            False,
        )
    return _mk(
        "selftest-digest",
        "자가검증: digest",
        PASS,
        cmd,
        f"source={obj.get('source')}, truncated={obj.get('truncated')}",
        False,
    )


def check_inspect_injection(binary: Path, sample: Path):
    """비임계 읽기 전용. 신호 발견은 FAIL 이 아니다 — clean 필드를 보고할 뿐."""
    cmd = f'rhwp inspect injection "{sample}" --json'
    try:
        code, out, err = _run(
            binary, ["inspect", "injection", str(sample), "--json"], SELFTEST_TIMEOUT
        )
    except subprocess.TimeoutExpired:
        return _mk(
            "selftest-inspect-injection",
            "자가검증: inspect injection",
            FAIL,
            cmd,
            f"{SELFTEST_TIMEOUT}s 내 무응답(타임아웃)",
            False,
            exception=EXC_SELFTEST_TIMEOUT,
        )
    except OSError as e:
        return _mk(
            "selftest-inspect-injection",
            "자가검증: inspect injection",
            SKIP,
            cmd,
            f"실행 불가: {e}",
            False,
        )
    text = (err or out).strip()
    if code == 2 and ("알 수 없는 명령" in text or "알 수 없는 옵션" in text):
        return _mk(
            "selftest-inspect-injection",
            "자가검증: inspect injection",
            SKIP,
            cmd,
            "이 바이너리에 inspect injection 없음",
            False,
        )
    if code != 0:
        return _mk(
            "selftest-inspect-injection",
            "자가검증: inspect injection",
            FAIL,
            cmd,
            f"exit={code}: {text[:120]}",
            False,
        )
    obj, err_msg = parse_json_object(out)
    if obj is None:
        return _mk(
            "selftest-inspect-injection",
            "자가검증: inspect injection",
            FAIL,
            cmd,
            err_msg,
            False,
        )
    missing = missing_keys(obj, INJECTION_REQUIRED_KEYS)
    if missing:
        return _mk(
            "selftest-inspect-injection",
            "자가검증: inspect injection",
            FAIL,
            cmd,
            f"키 없음: {','.join(missing)}",
            False,
        )
    detail = f"clean={obj.get('clean')}, signalCount={obj.get('signalCount')}"
    return _mk("selftest-inspect-injection", "자가검증: inspect injection", PASS, cmd, detail, False)


def skipped_selftests(reason: str, exception: str | None = None):
    """샘플을 돌릴 수 없을 때 info/export-text 를 정직하게 SKIP/FAIL 로 채운다."""
    status = FAIL if exception == EXC_BAD_SAMPLE else SKIP
    return [
        _mk(
            "selftest-info",
            "자가검증: info",
            status,
            "rhwp info <샘플> --json",
            reason,
            True,
            exception=exception,
        ),
        _mk(
            "selftest-export-text",
            "자가검증: export-text",
            status,
            "rhwp export-text <샘플> --json",
            reason,
            True,
            exception=exception,
        ),
    ]


def probe_network(timeout: float = NETWORK_TIMEOUT, probes=None):
    """선택적 TCP 프로브. HTTP 를 하지 않고, 실패해도 온보딩을 막지 않는다.

    반환: {probed, reachable, offline, targets:[{host,port,ok,error}]}.
    """
    probes = NETWORK_PROBES if probes is None else probes
    targets = []
    reachable = False
    for host, port in probes:
        item = {"host": host, "port": port, "ok": False, "error": None}
        try:
            with socket.create_connection((host, port), timeout=timeout):
                item["ok"] = True
                reachable = True
        except OSError as e:
            item["error"] = str(e)
        targets.append(item)
    return {
        "probed": True,
        "reachable": reachable,
        "offline": not reachable,
        "targets": targets,
    }


def skipped_network(reason: str):
    return {
        "probed": False,
        "reachable": None,
        "offline": True,
        "targets": [],
        "reason": reason,
    }


def check_network(net: dict):
    """비임계. 오프라인은 SKIP (실패가 아님)."""
    if not net.get("probed"):
        return _mk(
            "network",
            "네트워크 프로브",
            SKIP,
            "tcp://1.1.1.1:443",
            net.get("reason") or "프로브 생략(--offline)",
            False,
            exception=EXC_NO_NETWORK,
        )
    if net.get("reachable"):
        ok_t = next((t for t in net.get("targets", []) if t.get("ok")), None)
        detail = f"{ok_t['host']}:{ok_t['port']} 도달" if ok_t else "도달"
        return _mk("network", "네트워크 프로브", PASS, "tcp://1.1.1.1:443", detail, False)
    return _mk(
        "network",
        "네트워크 프로브",
        SKIP,
        "tcp://1.1.1.1:443",
        "오프라인 — 로컬 자가검증은 계속한다",
        False,
        exception=EXC_NO_NETWORK,
    )


def check_python():
    """비임계. 닥터 자신이 돌고 있으므로 사실상 항상 PASS."""
    ver = sys.version.split()[0]
    return _mk(
        "python",
        "Python 런타임",
        PASS,
        "python --version",
        f"{ver} ({sys.executable})",
        False,
    )


def _mk(cid, title, status, command, detail, critical, version=None, exception=None, next_steps=None):
    d = {
        "id": cid,
        "title": title,
        "status": status,
        "command": command,
        "detail": detail,
        "critical": critical,
    }
    if version is not None:
        d["version"] = version
    if exception is not None:
        d["exception"] = exception
        if next_steps is None:
            next_steps = exception_playbook(exception)["nextSteps"]
    if next_steps is not None:
        d["nextSteps"] = list(next_steps)
    return d


def collect_exceptions(checks, extra):
    """검사와 명시 예외를 합쳐 중복 kind 를 한 번만 남긴다."""
    seen = set()
    out = []
    for item in extra:
        kind = item.get("kind")
        if kind and kind not in seen:
            seen.add(kind)
            out.append(item)
    for c in checks:
        kind = c.get("exception")
        if not kind or kind in seen:
            continue
        if c.get("status") == PASS:
            continue
        seen.add(kind)
        out.append(make_exception(kind, c.get("detail") or "", None))
    return out


# --------------------------------------------------------------------------- #
# 출력
# --------------------------------------------------------------------------- #
def render_human(report, out):
    p = lambda *a: print(*a, file=out)
    b = report["binary"]
    p("rhwp doctor — 에이전트 제로프릭션 온보딩 점검")
    p(f"repo: {report['repoRoot']}")
    p("")
    p("[1] 바이너리 위치·버전")
    if b["found"]:
        p(f"  [PASS] rhwp 발견: {b['path']}  (source: {b['source']})")
    else:
        p("  [FAIL] rhwp 미발견 — 아직 빌드 안 됨. 저장소 루트에서 실행:")
        p(f"           {report['buildCommand']}")
    inv = report.get("binaryInventory") or []
    if inv:
        p("  탐색 자리:")
        for item in inv:
            flag = "hit" if item.get("exists") else "miss"
            resolved = item.get("resolved") or item.get("path")
            p(f"    - {item['source']}: {flag} ({resolved})")
    for c in report["checks"]:
        p(f"  [{c['status']}] {c['title']}: {c['command']}")
        if c["detail"]:
            p(f"           → {c['detail']}")
        if c.get("exception") and c["status"] != PASS:
            p(f"           예외: {c['exception']}")
    p("")
    sample_cls = report.get("sampleClassification")
    if sample_cls:
        p("[1b] 샘플 분류")
        p(f"  kind={sample_cls.get('kind')} ok={sample_cls.get('ok')} size={sample_cls.get('sizeBytes')}")
        p(f"  → {sample_cls.get('reason')}")
        p("")
    p("[2] 붙여넣기용 .mcp.json  (호스트 프로젝트 루트에 두거나 mcpServers 키를 병합)")
    for line in json.dumps(report["mcpJson"], ensure_ascii=False, indent=2).splitlines():
        p(f"  {line}")
    if report.get("mcpJsonWritten"):
        p(f"  → 기록함: {report['mcpJsonWritten']}")
    host = report.get("mcpHost")
    if host:
        p(f"  호스트 모양: {host.get('title')} ({host.get('file')}, shape={host.get('shape')})")
    p("")
    p("[3] 첫 5분 레시피 지도 (실존 스킬·레시피만 인용)")
    for r in report["recipes"]:
        sflag = "OK" if r["skillExists"] else "missing"
        p(f"  · {r['task']}")
        p(f"      명령: {r['command']}")
        p(f"      스킬: {r['skill']} [{sflag}]  ({r['skillPath']})")
        if r["recipe"]:
            rflag = "OK" if r["recipeExists"] else "missing"
            p(f"      레시피: {r['recipe']} [{rflag}]")
    first5 = report.get("first5Min") or []
    if first5:
        p("")
        p("[3b] 첫 5분 단계 (references/)")
        for step in first5:
            rflag = "OK" if step.get("referenceExists") else "missing"
            p(f"  · {step['id']}: {step['title']}  [{rflag}]")
            p(f"      {step['reference']}")
            for cmd in step.get("commands") or []:
                p(f"      $ {cmd}")
    p("")
    net = report.get("network") or {}
    p("[4] 네트워크")
    if not net.get("probed"):
        p(f"  [SKIP] 프로브 생략 — {net.get('reason') or 'offline'}")
    elif net.get("reachable"):
        p("  [PASS] 외부 TCP 도달. 온보딩 자체는 오프라인에서도 된다.")
    else:
        p("  [SKIP] 오프라인. 로컬 바이너리·samples/ 로 계속한다.")
    p("")
    exceptions = report.get("exceptions") or []
    if exceptions:
        p("[5] 예외 경로")
        for exc in exceptions:
            p(f"  · {exc['kind']}: {exc['title']}")
            p(f"      {exc['detail']}")
            for step in exc.get("nextSteps") or []:
                p(f"      - {step}")
        p("")
    verdict = "정상 — 바로 붙여도 됩니다" if report["ok"] else "미완 — 위 FAIL/빌드 안내를 먼저 처리하세요"
    p(f"판정: {verdict}  (exit={report['exitCode']})")


def _force_utf8_streams():
    """stdout/stderr 를 UTF-8 로 맞춘다. Windows 콘솔(cp949)에서도 한글·em-dash·
    UTF-8 JSON 이 깨지지 않게 한다. 에이전트는 어차피 stdout 을 UTF-8 로 파싱한다."""
    for stream in (sys.stdout, sys.stderr):
        reconfigure = getattr(stream, "reconfigure", None)
        if reconfigure is not None:
            try:
                reconfigure(encoding="utf-8", errors="replace")
            except (ValueError, OSError):
                pass


def build_parser():
    ap = argparse.ArgumentParser(
        prog="rhwp_doctor.py",
        description="rhwp 에이전트 온보딩 닥터 — 바이너리 검증 + 자가검증 + .mcp.json + 레시피 지도",
    )
    ap.add_argument("--json", action="store_true", help="기계 판독용 리포트 JSON 을 stdout 으로")
    ap.add_argument("--write", metavar="PATH", help=".mcp.json 스니펫을 이 경로에 기록(기존 파일은 --force 필요)")
    ap.add_argument("--force", action="store_true", help="--write 시 기존 파일 덮어쓰기 허용")
    ap.add_argument("--rhwp", metavar="PATH", help="rhwp 바이너리 경로를 직접 지정")
    ap.add_argument("--sample", metavar="PATH", help="자가검증에 쓸 샘플 문서 경로")
    ap.add_argument("--repo-root", metavar="PATH", help="저장소 루트(기본: 스크립트 위치에서 유도)")
    ap.add_argument("--offline", action="store_true", help="네트워크 프로브를 건너뛴다(오프라인 온보딩)")
    ap.add_argument("--skip-selftest", action="store_true", help="info/export-text 자가검증을 건너뛴다")
    ap.add_argument(
        "--skip-extra",
        action="store_true",
        help="explain/digest/inspect 비임계 자가검증을 건너뛴다",
    )
    ap.add_argument(
        "--host",
        metavar="NAME",
        help="MCP 스니펫 호스트 모양(claude-code, cursor, vscode, zed, goose, continue, …)",
    )
    ap.add_argument("--list-hosts", action="store_true", help="지원 호스트 id 를 나열하고 종료")
    ap.add_argument("--list-recipes", action="store_true", help="첫 5분 레시피 지도를 JSON 으로 나열하고 종료")
    return ap


def main(argv=None) -> int:
    _force_utf8_streams()
    ap = build_parser()
    args = ap.parse_args(argv)

    if args.list_hosts:
        payload = {"hosts": list_mcp_hosts()}
        print(json.dumps(payload, ensure_ascii=False, indent=2))
        return 0
    if args.list_recipes:
        root = Path(args.repo_root).resolve() if args.repo_root else default_repo_root()
        payload = {
            "recipes": resolve_recipe_map(root),
            "first5Min": resolve_first_5_min(root),
            "references": resolve_onboarding_references(root),
        }
        print(json.dumps(payload, ensure_ascii=False, indent=2))
        return 0

    if args.host and not any(h["id"] == args.host for h in MCP_HOSTS):
        known = ", ".join(h["id"] for h in MCP_HOSTS)
        print(f"오류: 알 수 없는 --host {args.host}. 알려진 값: {known}", file=sys.stderr)
        return 2

    # --json 모드: 사람용 텍스트는 stderr, stdout 은 순수 JSON 만.
    human_out = sys.stderr if args.json else sys.stdout

    repo_root = Path(args.repo_root).resolve() if args.repo_root else default_repo_root()
    binary, source, on_path = find_binary(repo_root, args.rhwp)
    inventory = discover_binary_candidates(repo_root, args.rhwp)

    checks = []
    extra_exceptions = []
    sample = None
    sample_cls = None

    checks.append(check_python())

    if binary is not None:
        checks.append(check_version(binary))
        if args.skip_selftest:
            sample = pick_sample(repo_root, args.sample)
            sample_cls = classify_sample(sample, repo_root) if sample else classify_sample(None)
        else:
            sample = pick_sample(repo_root, args.sample)
            sample_cls = classify_sample(sample, repo_root)
            if sample is None:
                note = "샘플 문서를 찾지 못함(samples/ 없음). --sample 로 지정하세요."
                checks.extend(skipped_selftests(note))
                extra_exceptions.append(make_exception(EXC_BAD_SAMPLE, note, None))
            elif not sample_cls["ok"]:
                note = sample_cls["reason"]
                checks.extend(skipped_selftests(note, EXC_BAD_SAMPLE))
                extra_exceptions.append(make_exception(EXC_BAD_SAMPLE, note, sample_cls.get("path")))
            else:
                checks.append(check_info(binary, sample))
                checks.append(check_export_text(binary, sample))
                if not args.skip_extra:
                    checks.append(check_explain(binary, sample))
                    checks.append(check_digest(binary, sample))
                    checks.append(check_inspect_injection(binary, sample))
    else:
        extra_exceptions.append(
            make_exception(
                EXC_MISSING_BINARY,
                f"rhwp 를 찾지 못했다 (source={source}). {BUILD_COMMAND}",
                None,
            )
        )

    if args.offline:
        net = skipped_network("--offline")
    else:
        net = probe_network()
    checks.append(check_network(net))
    if net.get("offline"):
        extra_exceptions.append(
            make_exception(EXC_NO_NETWORK, "외부 TCP 프로브가 실패했거나 생략됐다.", None)
        )

    # .mcp.json 스니펫(바이너리 유무와 무관하게 방출 — 문서 산출물).
    mcp_command = choose_mcp_command(binary, on_path)
    snippet = build_mcp_snippet(mcp_command)
    host_pack = None
    if args.host:
        host_pack = build_mcp_snippet_for_host(args.host, mcp_command)
        if host_pack is not None:
            # --write 는 항상 A형(.mcp.json)만 쓴다. 호스트 모양은 리포트에만 싣는다.
            pass

    ok, exit_code = aggregate(checks, binary is not None)
    exceptions = collect_exceptions(checks, extra_exceptions)

    report = {
        "schemaVersion": SCHEMA_VERSION,
        "tool": TOOL_NAME,
        "ok": ok,
        "exitCode": exit_code,
        "repoRoot": str(repo_root),
        "binary": {
            "found": binary is not None,
            "path": str(binary) if binary else None,
            "source": source,
            "onPath": on_path,
            "version": next((c.get("version") for c in checks if c.get("id") == "version" and c.get("version")), None),
        },
        "binaryInventory": inventory,
        "sample": str(sample) if sample else None,
        "sampleClassification": sample_cls,
        "checks": checks,
        "exceptions": exceptions,
        "network": net,
        "mcpJson": snippet,
        "mcpJsonWritten": None,
        "mcpHost": host_pack,
        "recipes": resolve_recipe_map(repo_root),
        "first5Min": resolve_first_5_min(repo_root),
        "references": resolve_onboarding_references(repo_root),
        "buildCommand": BUILD_COMMAND,
        "python": {"version": sys.version.split()[0], "executable": sys.executable},
    }

    # --write 처리(덮어쓰기 보호).
    if args.write:
        target = Path(args.write)
        if target.exists() and not args.force:
            print(f"경고: {target} 가 이미 있어 기록하지 않았습니다. 덮어쓰려면 --force 를 주세요.", file=sys.stderr)
            extra = make_exception(EXC_WRITE_EXISTS, f"{target} 가 이미 있다", str(target))
            report["exceptions"] = collect_exceptions(checks, extra_exceptions + [extra])
            _emit(report, args.json, human_out)
            return 2
        try:
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(json.dumps(snippet, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
            report["mcpJsonWritten"] = str(target)
        except OSError as e:
            print(f"경고: {target} 기록 실패: {e}", file=sys.stderr)
            _emit(report, args.json, human_out)
            return 2

    _emit(report, args.json, human_out)
    return exit_code


def _emit(report, as_json, human_out):
    if as_json:
        print(json.dumps(report, ensure_ascii=False, indent=2))
        # 계약: stdout 은 JSON 하나, 사람용 리포트는 stderr.
        render_human(report, human_out)
    else:
        render_human(report, human_out)


# Windows PE 헤더를 흉내 내지 않는다. 테스트용 가짜 바이너리는 실행하지 않는다.
def looks_like_pe(path: Path) -> bool:
    blob = read_magic(path, 2)
    return blob == b"MZ"


def looks_like_elf(path: Path) -> bool:
    blob = read_magic(path, 4)
    return blob == b"\x7fELF"


def looks_like_macho(path: Path) -> bool:
    blob = read_magic(path, 4)
    if len(blob) < 4:
        return False
    (magic,) = struct.unpack("<I", blob[:4])
    return magic in {0xFEEDFACE, 0xFEEDFACF, 0xCEFAEDFE, 0xCFFAEDFE, 0xCAFEBABE}


if __name__ == "__main__":
    sys.exit(main())
