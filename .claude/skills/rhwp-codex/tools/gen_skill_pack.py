# -*- coding: utf-8 -*-
"""[#5318] 생성 장에서 스킬 로컬 봉투 전사만 갱신한다.

정본은 mydocs/manual/agent_codex/ 와 tools/gen_agent_codex.py.
이 스크립트는 생성 장을 읽기만 하고 수기 수정하지 않는다.
references/ · examples/ · intent/journey 표는 커밋된 손글·표를 유지한다.
"""

from __future__ import annotations

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
CODEX = ROOT / "mydocs" / "manual" / "agent_codex"
FIX = ROOT / ".claude" / "skills" / "rhwp-codex" / "fixtures"
ENV_DIR = FIX / "envelopes"
TRACE_DIR = FIX / "traces"
ISSUE = 5318


def write_unix(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    if not text.endswith("\n"):
        text += "\n"
    path.write_text(text.replace("\r\n", "\n"), encoding="utf-8", newline="\n")


def slim_envelope(command: str, env: dict) -> dict:
    raw = json.dumps(env, ensure_ascii=False)
    if len(raw) <= 2500:
        return env
    keep = {
        k: env[k]
        for k in (
            "schemaVersion",
            "tool",
            "version",
            "source",
            "untrustedContent",
            "untrustedFields",
            "mode",
            "identical",
            "diffCount",
            "dryRun",
        )
        if k in env
    }
    keep["_truncatedForSkillFixture"] = True
    keep["_originalTopLevelKeys"] = sorted(env.keys())
    keep["_originalChars"] = len(raw)
    keep["_command"] = command
    return keep


def slug(name: str) -> str:
    return re.sub(r"[^a-z0-9]+", "_", name.lower()).strip("_")


def parse_blocks(chapter: str, text: str) -> list[dict]:
    parts = re.split(r"(?m)^### `([^`]+)` — ", text)
    out = []
    it = iter(parts[1:])
    for name, rest in zip(it, it):
        bash = re.search(r"실측 표본[^\n]*\n\n```bash\n(.*?)\n```", rest, re.S)
        js = re.search(r"```json\n(\{.*?\n\})\n```", rest, re.S)
        exit_m = re.search(r"실측 표본[^\n]*\(exit (\d+)\)", rest)
        live = None
        if bash and js:
            try:
                env = json.loads(js.group(1))
            except json.JSONDecodeError:
                env = None
            if env is not None:
                live = {
                    "cmd": bash.group(1).strip(),
                    "exit": int(exit_m.group(1)) if exit_m else None,
                    "envelope": env,
                }
        out.append(
            {
                "name": name,
                "chapter": chapter,
                "contractOnly": "**계약만**" in rest,
                "live": live,
                "developerOnly": chapter == "85_진단_프로브.md",
            }
        )
    return out


def main() -> int:
    commands = []
    for path in sorted(CODEX.glob("*.md")):
        if path.name in {"README.md", "00_서문.md", "01_판단트리.md"}:
            continue
        commands.extend(parse_blocks(path.name, path.read_text(encoding="utf-8")))

    ENV_DIR.mkdir(parents=True, exist_ok=True)
    TRACE_DIR.mkdir(parents=True, exist_ok=True)
    envelopes = 0
    traces = 0
    for c in commands:
        live = c.get("live")
        if not live:
            continue
        rec = {
            "schemaVersion": "1.0",
            "issue": ISSUE,
            "kind": "live",
            "command": c["name"],
            "sourceChapter": c["chapter"],
            "cmd": live["cmd"],
            "exit": live["exit"],
            "envelope": slim_envelope(c["name"], live["envelope"]),
            "extractedFromGenerated": True,
            "handEdited": False,
        }
        write_unix(ENV_DIR / (slug(c["name"]) + ".json"), json.dumps(rec, ensure_ascii=False, indent=2))
        envelopes += 1
        env = rec["envelope"]
        traces += 1
        write_unix(
            TRACE_DIR / f"T{traces:03d}.json",
            json.dumps(
                {
                    "id": f"T{traces:03d}",
                    "command": c["name"],
                    "chapter": c["chapter"],
                    "cmd": live["cmd"],
                    "exit": live["exit"],
                    "keys": sorted(env.keys()) if isinstance(env, dict) else [],
                    "untrustedContent": env.get("untrustedContent") if isinstance(env, dict) else None,
                    "untrustedFields": env.get("untrustedFields") if isinstance(env, dict) else None,
                    "schemaVersion": env.get("schemaVersion") if isinstance(env, dict) else None,
                    "source": "mydocs/manual/agent_codex/" + c["chapter"],
                    "notInvented": True,
                },
                ensure_ascii=False,
                indent=2,
            ),
        )
    print(f"commands {len(commands)} · envelopes {envelopes} · traces {traces}")
    print("references/examples 는 이 스크립트가 덮지 않는다.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
