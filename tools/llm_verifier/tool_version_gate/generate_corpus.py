#!/usr/bin/env python3
"""Emit the V-fresh toolVersion gate corpus.

Each row is a distinct
``(attest_version, verify_version, reproduced, accepted)`` case.
Comment padding is not used. Versions are real-looking rhwp / cargo /
git / channel identities. This is not V-replay (same-version re-run).
"""

from __future__ import annotations

import hashlib
import json
from collections import Counter
from pathlib import Path

HERE = Path(__file__).resolve().parent
CORPUS = HERE / "corpus"
SHARDS = CORPUS / "shards"

SCHEMA_VERSION = "v-fresh.1.0"
CLAIM = "V-fresh"
KIND = "toolVersionGate"
GENERATED_BY = "tools/llm_verifier/tool_version_gate/generate_corpus.py"
UNIQUENESS = "attestVersion|verifyVersion|reproduced|accepted"
TUPLE_FIELDS = ["attestVersion", "verifyVersion", "reproduced", "accepted"]
DEFAULT_MIN_LINES = 112000
SHARD_ROWS = 4000
HEADER = "id\tattest_version\tverify_version\treproduced\taccepted\treason\tfamily"


def decide(attest: str, verify: str, reproduced: bool | None) -> tuple[bool, str]:
    a = attest.strip()
    v = verify.strip()
    if not a:
        return False, "ATTEST_VERSION_MISSING"
    if not v:
        return False, "VERIFY_VERSION_MISSING"
    if a != v:
        if reproduced is True:
            return False, "STALE_TOOL"
        if reproduced is False:
            return False, "STALE_AND_NOT_REPRODUCED"
        return False, "STALE_AND_ABSENT"
    if reproduced is True:
        return True, "FRESH_REPRODUCED"
    if reproduced is False:
        return False, "FRESH_NOT_REPRODUCED"
    return False, "FRESH_ABSENT"


def sha_hex(label: str, n: int) -> str:
    return hashlib.sha256(label.encode("utf-8")).hexdigest()[:n]


def build_catalog() -> list[str]:
    out: list[str] = []
    seen: set[str] = set()

    def add(token: str) -> None:
        if "\t" in token or "\n" in token or "\r" in token:
            raise ValueError(f"illegal version token {token!r}")
        if token not in seen:
            seen.add(token)
            out.append(token)

    for minor in range(0, 10):
        for patch in range(0, 16):
            add(f"0.{minor}.{patch}")
    for patch in range(0, 8):
        add(f"1.0.{patch}")
    for kind in ("alpha", "beta", "rc"):
        for i in range(1, 5):
            add(f"1.0.0-{kind}.{i}")
    for v in ("0.7.15", "0.8.0", "0.8.1", "0.8.2", "0.8.3", "0.8.4", "0.9.0"):
        add(f"rhwp {v}")
    for v in ("0.8.0", "0.8.1", "0.8.2", "0.8.3", "0.8.4", "1.0.0"):
        add(f"v{v}")
    for i in range(1, 9):
        add(f"0.8.4+git.{sha_hex(f'git-short-{i}', 8)}")
    for i in range(1, 7):
        add(f"0.8.4+build.{i}")
    for i in range(1, 9):
        add(f"0.8.4-{i}-g{sha_hex(f'describe-{i}', 7)}")
    for i in range(1, 9):
        add(sha_hex(f"rhwp-tool-rev-{i}", 40))
    for day, n in (
        ("20260801", 1),
        ("20260808", 2),
        ("20260815", 3),
        ("20260818", 1),
        ("20260818", 2),
        ("20260901", 1),
    ):
        add(f"devel-{day}.{n}")
    for i, day in enumerate(("2026-07-01", "2026-07-15", "2026-08-01", "2026-08-18"), start=1):
        add(f"rhwp 0.8.4 ({sha_hex(f'rustc-{i}', 7)} {day})")
    add("llm-verifier 0.1.0")
    add("llm-verifier 0.1.1")
    add("llm-verifier 0.2.0")
    add("rhwp-cli/0.8.4")

    if len(out) < 237:
        raise SystemExit(f"catalog too small: {len(out)}")
    catalog = out[:237]
    if len(catalog) != len(set(catalog)):
        raise SystemExit("catalog not unique")
    return catalog


def family_of(attest: str, verify: str) -> str:
    a = attest.strip()
    v = verify.strip()
    if not a or not v:
        return "missing"
    if a == v:
        return "exact_match"
    if a.startswith("v") and a[1:] == v:
        return "v_prefix"
    if v.startswith("v") and v[1:] == a:
        return "v_prefix"
    if a.startswith("rhwp ") or v.startswith("rhwp "):
        return "crate_prefix"
    if a.startswith("llm-verifier") or v.startswith("llm-verifier"):
        return "verifier_crate"
    if "+" in a or "+" in v:
        return "build_meta"
    if len(a) == 40 and all(c in "0123456789abcdef" for c in a):
        return "git_sha"
    if len(v) == 40 and all(c in "0123456789abcdef" for c in v):
        return "git_sha"
    if a.startswith("devel-") or v.startswith("devel-"):
        return "channel"
    if "-g" in a or "-g" in v:
        return "git_describe"
    if "-" in a or "-" in v:
        return "prerelease"
    ap = a.split(".")
    vp = v.split(".")
    if len(ap) == 3 and len(vp) == 3 and ap[:2] == vp[:2] and ap[2] != vp[2]:
        return "patch_drift"
    if len(ap) == 3 and len(vp) == 3 and ap[0] == vp[0] and ap[1] != vp[1]:
        return "minor_drift"
    if len(ap) == 3 and len(vp) == 3 and ap[0] != vp[0]:
        return "major_drift"
    return "identity_mismatch"


def reproduced_token(reproduced: bool | None) -> str:
    if reproduced is True:
        return "true"
    if reproduced is False:
        return "false"
    return "null"


def accepted_token(accepted: bool) -> str:
    return "true" if accepted else "false"


class Emitter:
    def __init__(self) -> None:
        self.rows: list[tuple[str, str, str, bool | None, bool, str, str]] = []
        self.seen: set[tuple[str, str, str, bool]] = set()

    def add(self, attest: str, verify: str, reproduced: bool | None, family: str | None = None) -> bool:
        accepted, reason = decide(attest, verify, reproduced)
        key = (attest, verify, reproduced_token(reproduced), accepted)
        if key in self.seen:
            return False
        self.seen.add(key)
        fam = family or family_of(attest, verify)
        record_id = f"tvg-{len(self.rows):06d}"
        self.rows.append((record_id, attest, verify, reproduced, accepted, reason, fam))
        return True

    def write(self) -> dict:
        SHARDS.mkdir(parents=True, exist_ok=True)
        for old in SHARDS.glob("shard_*.tsv"):
            old.unlink()
        reason_counts: Counter[str] = Counter()
        family_counts: Counter[str] = Counter()
        accepted_count = 0
        stale_tool = 0
        shard_meta = []
        for start in range(0, len(self.rows), SHARD_ROWS):
            chunk = self.rows[start : start + SHARD_ROWS]
            shard_id = start // SHARD_ROWS
            rel = f"shards/shard_{shard_id:04d}.tsv"
            path = CORPUS / rel
            lines = [HEADER]
            for rec in chunk:
                record_id, attest, verify, reproduced, accepted, reason, fam = rec
                lines.append(
                    "\t".join(
                        [
                            record_id,
                            attest,
                            verify,
                            reproduced_token(reproduced),
                            accepted_token(accepted),
                            reason,
                            fam,
                        ]
                    )
                )
                reason_counts[reason] += 1
                family_counts[fam] += 1
                if accepted:
                    accepted_count += 1
                if reason == "STALE_TOOL":
                    stale_tool += 1
            path.write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")
            shard_meta.append({"path": rel.replace("\\", "/"), "count": len(chunk)})

        rejected = len(self.rows) - accepted_count
        manifest = {
            "schemaVersion": SCHEMA_VERSION,
            "claim": CLAIM,
            "kind": KIND,
            "generatedBy": GENERATED_BY,
            "recordCount": len(self.rows),
            "shardCount": len(shard_meta),
            "uniqueness": UNIQUENESS,
            "tupleFields": TUPLE_FIELDS,
            "acceptedCount": accepted_count,
            "rejectedCount": rejected,
            "staleToolCount": stale_tool,
            "minLineFloor": DEFAULT_MIN_LINES,
            "reasonCounts": dict(sorted(reason_counts.items())),
            "familyCounts": dict(sorted(family_counts.items())),
            "shards": shard_meta,
        }
        (CORPUS / "manifest.json").write_text(
            json.dumps(manifest, ensure_ascii=False, indent=2) + "\n",
            encoding="utf-8",
            newline="\n",
        )
        return manifest


def emit_catalog_grid(em: Emitter, catalog: list[str]) -> None:
    for attest in catalog:
        for verify in catalog:
            em.add(attest, verify, True)
            em.add(attest, verify, False)


def emit_null_claims(em: Emitter, catalog: list[str]) -> None:
    for i, v in enumerate(catalog):
        em.add(v, v, None, "exact_match")
        nxt = catalog[(i + 1) % len(catalog)]
        if nxt != v:
            em.add(v, nxt, None)


def emit_missing_and_trim(em: Emitter, catalog: list[str]) -> None:
    samples = catalog[:24]
    for v in samples:
        em.add("", v, True, "missing_attest")
        em.add("", v, False, "missing_attest")
        em.add(v, "", True, "missing_verify")
        em.add(v, "", False, "missing_verify")
        em.add(" ", v, True, "whitespace_attest")
        em.add(v, " ", True, "whitespace_verify")
        em.add(v, f" {v}", True, "trim_equal")
        em.add(f" {v}", v, False, "trim_equal")
    em.add("", "", True, "missing_both")
    em.add("", "", False, "missing_both")
    em.add("  ", "  ", True, "whitespace_both")
    em.add("   ", "0.8.4", True, "whitespace_attest")


def main() -> int:
    catalog = build_catalog()
    em = Emitter()
    emit_catalog_grid(em, catalog)
    emit_null_claims(em, catalog)
    emit_missing_and_trim(em, catalog)
    if len(em.rows) < DEFAULT_MIN_LINES:
        raise SystemExit(f"corpus too small: {len(em.rows)} < {DEFAULT_MIN_LINES}")
    if len(em.seen) != len(em.rows):
        raise SystemExit("uniqueness broken")
    man = em.write()
    print(
        json.dumps(
            {
                "recordCount": man["recordCount"],
                "shardCount": man["shardCount"],
                "acceptedCount": man["acceptedCount"],
                "staleToolCount": man["staleToolCount"],
                "catalog": len(catalog),
            },
            ensure_ascii=False,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
