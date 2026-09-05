#!/usr/bin/env python3
"""Release package workflow의 gate와 채널별 결과를 비민감 증적으로 집계한다."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path
from typing import Any, Mapping


SHA_PATTERN = re.compile(r"^[0-9a-f]{40}$")
EXPECTED_GATES = ("validate-release-source", "build-wasm", "build-vsix")
EXPECTED_CHANNELS = (
    "npm-core",
    "npm-editor",
    "vscode-marketplace",
    "open-vsx",
)
SUCCESS_STATES = {"already-present", "published"}


def evaluate_publish_evidence(context: Mapping[str, Any]) -> dict[str, Any]:
    """Actions 결과를 명시적 성공/실패와 채널별 최종 상태로 정규화한다."""

    mode = str(context.get("mode", ""))
    extensions_requested = context.get("extensionsRequested") is True
    github_sha = str(context.get("githubSha", "")).lower()
    ref_name = str(context.get("refName", ""))
    gates_value = context.get("gates", {})
    channels_value = context.get("channels", {})
    gates = dict(gates_value) if isinstance(gates_value, Mapping) else {}
    channels = dict(channels_value) if isinstance(channels_value, Mapping) else {}

    errors: list[str] = []
    if mode not in {"verify", "publish"}:
        errors.append("invalid-mode")
    if not SHA_PATTERN.fullmatch(github_sha):
        errors.append("invalid-github-sha")
    if not ref_name:
        errors.append("ref-name-missing")

    normalized_gates: dict[str, str] = {}
    for gate in EXPECTED_GATES:
        result = str(gates.get(gate, "missing"))
        normalized_gates[gate] = result
        if result != "success":
            errors.append(f"gate-not-success:{gate}:{result}")

    normalized_channels: dict[str, dict[str, str]] = {}
    for name in EXPECTED_CHANNELS:
        value = channels.get(name)
        item = dict(value) if isinstance(value, Mapping) else {}
        job_result = str(item.get("jobResult", "missing"))
        input_state = str(item.get("state", ""))
        extension_channel = name in {"vscode-marketplace", "open-vsx"}

        if mode == "verify":
            state = "verify-only"
            if job_result != "skipped":
                errors.append(f"verify-channel-not-skipped:{name}:{job_result}")
        elif extension_channel and not extensions_requested:
            state = "not-requested"
            if job_result != "skipped":
                errors.append(f"unrequested-channel-not-skipped:{name}:{job_result}")
        elif job_result == "success" and input_state in SUCCESS_STATES:
            state = input_state
        else:
            state = "failed"
            errors.append(f"channel-not-success:{name}:{job_result}:{input_state or 'none'}")

        normalized_channels[name] = {
            "jobResult": job_result,
            "inputState": input_state,
            "state": state,
        }

    accepted = not errors
    return {
        "schemaVersion": 1,
        "accepted": accepted,
        "verdict": "completed" if accepted else "failed",
        "mode": mode,
        "extensionsRequested": extensions_requested,
        "githubSha": github_sha,
        "refName": ref_name,
        "gates": normalized_gates,
        "channels": normalized_channels,
        "errors": errors,
    }


def _pairs(values: list[list[str]], expected: int, label: str) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for value in values:
        if len(value) != expected:
            raise ValueError(f"{label}-argument-invalid")
        name = value[0]
        if name in result:
            raise ValueError(f"{label}-duplicate:{name}")
        result[name] = value[1] if expected == 2 else {
            "jobResult": value[1],
            "state": value[2],
        }
    return result


def _boolean(value: str) -> bool:
    if value == "true":
        return True
    if value == "false":
        return False
    raise ValueError("boolean-input-invalid")


def _summary(verdict: Mapping[str, Any]) -> str:
    status = "accepted" if verdict["accepted"] else "failed"
    lines = [
        "## Release publish evidence",
        "",
        f"- verdict: `{status}`",
        f"- mode: `{verdict['mode']}`",
        f"- ref: `{verdict['refName']}`",
        f"- commit: `{verdict['githubSha']}`",
        "",
        "| Channel | Job result | State |",
        "| --- | --- | --- |",
    ]
    for name, value in verdict["channels"].items():
        lines.append(f"| {name} | {value['jobResult']} | {value['state']} |")
    if verdict["errors"]:
        lines.extend(["", "Errors:"])
        lines.extend(f"- `{error}`" for error in verdict["errors"])
    return "\n".join(lines) + "\n"


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mode", required=True, choices=("verify", "publish"))
    parser.add_argument("--extensions-requested", required=True)
    parser.add_argument("--github-sha", required=True)
    parser.add_argument("--ref-name", required=True)
    parser.add_argument("--gate", nargs=2, action="append", default=[])
    parser.add_argument("--channel", nargs=3, action="append", default=[])
    parser.add_argument("--output", required=True)
    parser.add_argument("--summary")
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    try:
        context = {
            "mode": args.mode,
            "extensionsRequested": _boolean(args.extensions_requested),
            "githubSha": args.github_sha,
            "refName": args.ref_name,
            "gates": _pairs(args.gate, 2, "gate"),
            "channels": _pairs(args.channel, 3, "channel"),
        }
        verdict = evaluate_publish_evidence(context)
    except ValueError as error:
        verdict = {
            "schemaVersion": 1,
            "accepted": False,
            "verdict": "failed",
            "mode": args.mode,
            "extensionsRequested": args.extensions_requested,
            "githubSha": args.github_sha,
            "refName": args.ref_name,
            "gates": {},
            "channels": {},
            "errors": [str(error)],
        }

    rendered = json.dumps(verdict, ensure_ascii=False, indent=2, sort_keys=True) + "\n"
    Path(args.output).write_text(rendered, encoding="utf-8")
    print(rendered, end="")
    if args.summary:
        with Path(args.summary).open("a", encoding="utf-8") as summary:
            summary.write(_summary(verdict))
    return 0 if verdict["accepted"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
