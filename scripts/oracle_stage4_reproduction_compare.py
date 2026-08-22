#!/usr/bin/env python3
"""Validate a local Hyper-V three-state Oracle reproduction without publishing it."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path
from typing import Any

from oracle_stage2_common import (
    OracleStage2Error,
    canonical_json_bytes,
    output_path,
    pretty_json_bytes,
    regular_input,
    sha256_bytes,
    sha256_file,
    write_bytes,
)


STATES = ("exact-only", "subst-only", "none-related")
MAX_JSON = 32 * 1024 * 1024
MAX_PDF = 64 * 1024 * 1024
SHA256 = re.compile(r"^[0-9a-f]{64}$")
ABSOLUTE_PATH = re.compile(r"^(?:/home/|/mnt/|[A-Za-z]:[\\/])")


def _json(root: Path, relative: str, maximum: int = MAX_JSON) -> tuple[Path, Any]:
    path = regular_input(root, relative, maximum)
    try:
        return path, json.loads(path.read_text(encoding="utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise OracleStage2Error(f"invalid reproduction JSON: {relative}") from error


def _require(condition: bool, message: str) -> None:
    if not condition:
        raise OracleStage2Error(message)


def _path_free(value: Any, label: str = "evidence") -> None:
    if isinstance(value, dict):
        for key, child in value.items():
            _path_free(child, f"{label}.{key}")
    elif isinstance(value, list):
        for index, child in enumerate(value):
            _path_free(child, f"{label}[{index}]")
    elif isinstance(value, str) and ABSOLUTE_PATH.match(value):
        raise OracleStage2Error(f"{label} exposes an absolute path")


def _projection(observation: dict[str, Any]) -> str:
    projected = dict(observation)
    for key in ("canonicalSha256", "inputSha256", "toolVersions"):
        projected.pop(key, None)
    return sha256_bytes(canonical_json_bytes(projected))


def compare(evidence_root: Path, config: dict[str, Any]) -> dict[str, Any]:
    _require(config.get("schemaVersion") == 1, "config schemaVersion is invalid")
    _require(
        config.get("kind") == "font-oracle-hyperv-reproduction-config",
        "config kind is invalid",
    )
    _require(config.get("issue") == 4963, "config issue is invalid")
    _require(isinstance(config.get("queueRank"), int), "config queueRank is invalid")
    _require(bool(config.get("documentFace")), "config documentFace is required")
    fixture_hash = config.get("fixtureSha256")
    _require(
        isinstance(fixture_hash, str) and SHA256.fullmatch(fixture_hash) is not None,
        "config fixtureSha256 is invalid",
    )
    baseline = config.get("baseline", {})
    baseline_manifest = baseline.get("manifestSha256")
    unrelated_projection = baseline.get("unrelatedProjectionSha256")
    _require(
        all(
            isinstance(value, str) and SHA256.fullmatch(value) is not None
            for value in (baseline_manifest, unrelated_projection)
        ),
        "baseline hashes are invalid",
    )

    state_config = config.get("states")
    _require(isinstance(state_config, dict), "config states are required")
    records: dict[str, dict[str, Any]] = {}
    for state in STATES:
        specification = state_config.get(state, {})
        directory = specification.get("directory")
        stem = specification.get("stem")
        _require(
            isinstance(directory, str) and directory and isinstance(stem, str) and stem,
            f"{state} path configuration is invalid",
        )
        expected_managed = specification.get("managedFontSha256", [])
        expected_count = 0 if state == "none-related" else 1
        _require(
            isinstance(expected_managed, list)
            and len(expected_managed) == expected_count
            and all(
                isinstance(value, str) and SHA256.fullmatch(value) is not None
                for value in expected_managed
            ),
            f"{state} managed font configuration is invalid",
        )
        prefix = f"{directory}/{stem}"
        run_path, run = _json(evidence_root, f"{prefix}.interactive.json", 1024 * 1024)
        manifest_path, manifest = _json(
            evidence_root, f"{prefix}.ambient-manifest.json", 4 * 1024 * 1024
        )
        observation_path, observation = _json(
            evidence_root, f"{prefix}.pdf-observation.json"
        )
        recovered_path, recovered = _json(
            evidence_root,
            f"{directory}/recovered.ambient-manifest.json",
            4 * 1024 * 1024,
        )
        pdf_path = regular_input(evidence_root, f"{prefix}.pdf", MAX_PDF)

        for label, value in (
            (f"{state}.run", run),
            (f"{state}.manifest", manifest),
            (f"{state}.observation", observation),
            (f"{state}.recovered", recovered),
        ):
            _path_free(value, label)

        _require(run.get("status") == "observed", f"{state} run did not complete")
        _require(run.get("queueRank") == config["queueRank"], f"{state} rank drift")
        _require(
            run.get("documentFace") == config["documentFace"],
            f"{state} face drift",
        )
        _require(run.get("inputSha256") == fixture_hash, f"{state} input drift")
        feature = run.get("featureDetection", {})
        _require(feature.get("opened") is True, f"{state} HWPX open failed")
        _require(feature.get("pageCount", 0) >= 1, f"{state} page guard failed")
        _require(feature.get("textLength", 0) >= 1, f"{state} text guard failed")
        environment = run.get("environment", {})
        _require(
            environment.get("securityModuleRegistered") is True,
            f"{state} security module was not registered",
        )
        _require(
            environment.get("processReset") is True,
            f"{state} process reset drift",
        )
        pdf_hash = sha256_file(pdf_path)
        _require(run.get("export", {}).get("pdfSha256") == pdf_hash, f"{state} PDF drift")
        _require(
            observation.get("inputSha256") == pdf_hash,
            f"{state} observation input drift",
        )
        claimed = observation.get("canonicalSha256")
        canonical_input = dict(observation)
        canonical_input.pop("canonicalSha256", None)
        _require(
            claimed == sha256_bytes(canonical_json_bytes(canonical_input)),
            f"{state} observation self hash drift",
        )
        _require(
            manifest.get("unrelatedProjectionSha256") == unrelated_projection,
            f"{state} unrelated font projection drift",
        )
        _require(
            manifest.get("managedInstalledByExactBytes") == expected_managed,
            f"{state} managed font state drift",
        )
        _require(manifest.get("hwpProcessCount") == 0, f"{state} manifest saw Hwp.exe")
        if state == "none-related":
            _require(
                manifest.get("manifestSha256") == baseline_manifest,
                "none-related state does not match the baseline manifest",
            )
        _require(
            recovered.get("manifestSha256") == baseline_manifest,
            f"{state} baseline manifest was not recovered",
        )
        _require(
            recovered.get("unrelatedProjectionSha256") == unrelated_projection,
            f"{state} recovered unrelated projection drift",
        )
        _require(recovered.get("hwpProcessCount") == 0, f"{state} left Hwp.exe running")
        _require(
            recovered.get("managedInstalledByExactBytes") == [],
            f"{state} recovered managed font set is not empty",
        )

        _require(observation.get("pageCount", 0) >= 1, f"{state} PDF page guard failed")
        _require(
            observation.get("visualLineCount", 0) >= 1,
            f"{state} PDF line guard failed",
        )
        _require(
            observation.get("glyphObservationCount", 0) >= 1,
            f"{state} PDF glyph guard failed",
        )

        privacy = run.get("privacy", {})
        _require(privacy.get("privateCorpusAccessed") is False, f"{state} privacy drift")
        records[state] = {
            "runFileSha256": sha256_file(run_path),
            "manifestFileSha256": sha256_file(manifest_path),
            "observationFileSha256": sha256_file(observation_path),
            "recoveredManifestFileSha256": sha256_file(recovered_path),
            "pdfSha256": pdf_hash,
            "typesettingProjectionSha256": _projection(observation),
            "fonts": [entry.get("name") for entry in observation.get("fonts", [])],
            "pageCount": observation.get("pageCount"),
            "visualLineCount": observation.get("visualLineCount"),
            "glyphObservationCount": observation.get("glyphObservationCount"),
        }

    return {
        "schemaVersion": 1,
        "kind": "font-oracle-hyperv-reproduction-summary",
        "issue": 4963,
        "queueRank": config["queueRank"],
        "documentFace": config["documentFace"],
        "fixtureSha256": fixture_hash,
        "baseline": {
            "manifestSha256": baseline_manifest,
            "unrelatedProjectionSha256": unrelated_projection,
        },
        "states": records,
        "comparisons": {
            "exactEqualsNone": (
                records["exact-only"]["typesettingProjectionSha256"]
                == records["none-related"]["typesettingProjectionSha256"]
            ),
            "substitutionEqualsNone": (
                records["subst-only"]["typesettingProjectionSha256"]
                == records["none-related"]["typesettingProjectionSha256"]
            ),
        },
        "privacy": {
            "absolutePathIncluded": False,
            "fontBytesIncluded": False,
            "privateCorpusAccessed": False,
        },
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--evidence-root", required=True)
    parser.add_argument("--config", required=True)
    parser.add_argument("--output-root", required=True)
    parser.add_argument("--output", required=True)
    arguments = parser.parse_args()
    evidence_root = Path(arguments.evidence_root).resolve()
    config_path = Path(arguments.config).resolve()
    config = json.loads(config_path.read_text(encoding="utf-8"))
    summary = compare(evidence_root, config)
    destination = output_path(Path(arguments.output_root).resolve(), arguments.output)
    write_bytes(destination, pretty_json_bytes(summary))
    print(json.dumps({"ok": True, "output": arguments.output}, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
