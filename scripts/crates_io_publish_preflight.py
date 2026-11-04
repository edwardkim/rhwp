#!/usr/bin/env python3
"""Read-only exact archive/registry checks for a manual crates.io publication.

This module does not read credentials or implement an upload endpoint.
"""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import subprocess
import sys
import tarfile
import time
import tomllib
import urllib.error
import urllib.request
import zlib


class CheckError(ValueError):
    """Do not proceed with the proposed publication."""


def require(condition, message):
    if not condition:
        raise CheckError(message)


def output_path(root, value):
    output = value.absolute()
    require(output.resolve().is_relative_to((root / "output").resolve())
            and not output.is_symlink(),
            "output must be a regular file under repository output/")
    return output


def dependency_order(specs):
    by_name = {p["published_name"]: p for p in specs}
    require(len(by_name) == len(specs), "duplicate package name")
    ordered, pending = [], set(by_name)
    while pending:
        ready = sorted(n for n in pending
                       if set(by_name[n]["local_dependencies"]) <= set(ordered))
        require(ready, "cyclic or unknown local dependency")
        ordered.extend(ready)
        pending.difference_update(ready)
    return ordered


def audit_gzip_stream(raw, name):
    """Validate the whole single member, including bytes after the tar end."""
    stream = zlib.decompressobj(31)
    try:
        for offset in range(0, len(raw), 65536):
            pending = raw[offset:offset + 65536]
            while True:
                decoded = stream.decompress(pending, 65536)
                require(not stream.unused_data, f"trailing gzip bytes: {name}")
                pending = stream.unconsumed_tail
                if not pending and len(decoded) < 65536:
                    break
    except zlib.error as error:
        raise CheckError(f"invalid gzip stream: {name}") from error
    require(stream.eof, f"incomplete gzip stream: {name}")


def audit_archives(ledger, policy, archive_dir):
    sha = ledger["source_sha"]
    require(re.fullmatch(r"[0-9a-f]{40}", sha), "invalid source SHA")
    specs = {p["published_name"]: p for p in policy["packages"]}
    packages = ledger["packages"]
    require(len(packages) == len(specs), "package count differs")
    require({p["name"] for p in packages} == set(specs), "package set differs")
    audited = {}
    for row in packages:
        name, version = row["name"], row["version"]
        require(re.fullmatch(r"[a-z0-9-]+", name), "unsafe package name")
        require(re.fullmatch(r"[0-9A-Za-z.+-]+", version), "unsafe version")
        require(version == specs[name]["version"], f"version differs: {name}")
        path = archive_dir / f"{name}-{version}.crate"
        require(not path.is_symlink(), f"symlink archive: {name}")
        raw = path.read_bytes()
        require(len(raw) == row["bytes"] and len(raw) < 10_000_000,
                f"archive size differs: {name}")
        require(hashlib.sha256(raw).hexdigest() == row["sha256"],
                f"archive checksum differs: {name}")
        audit_gzip_stream(raw, name)
        prefix = f"{name}-{version}/"
        with tarfile.open(path, "r:gz") as archive:
            entries = archive.getmembers()
            require(len(entries) <= 10000 and sum(m.size for m in entries) <= 512 * 1024**2,
                    f"archive expansion exceeds guard: {name}")
            seen = set()
            for m in entries:
                require(m.name.startswith(prefix) and ".." not in PurePosixPath(m.name).parts
                        and "\\" not in m.name and (m.isfile() or m.isdir()),
                        f"unsafe archive entry: {name}")
                require(m.name not in seen, f"duplicate archive entry: {name}")
                seen.add(m.name)
            manifest = tomllib.loads(archive.extractfile(prefix + "Cargo.toml").read().decode())
            vcs = json.loads(archive.extractfile(prefix + ".cargo_vcs_info.json").read())
        pkg = manifest["package"]
        require((pkg["name"], pkg["version"]) == (name, version), f"identity differs: {name}")
        require(pkg.get("publish") == ["crates-io"], f"publication guard differs: {name}")
        require(pkg.get("rust-version") == "1.93.1", f"Rust support differs: {name}")
        require(vcs["git"]["sha1"] == sha and not vcs["git"].get("dirty", False),
                f"VCS source differs: {name}")
        require("patch" not in manifest and "workspace" not in manifest,
                f"repository override remains: {name}")
        audited[name] = row
    dependency_order(policy["packages"])
    return audited


def public_json(url):
    request = urllib.request.Request(url, headers={
        "User-Agent": "rhwp-maintainer-preflight/8174 (https://github.com/edwardkim/rhwp)",
        "Accept": "application/json",
    })
    try:
        with urllib.request.urlopen(request, timeout=40) as response:
            return json.load(response)
    except urllib.error.HTTPError as error:
        if error.code == 404:
            return None
        raise CheckError(f"registry HTTP {error.code}: {url}") from error


def registry_state(name):
    url = "https://crates.io/api/v1/crates/" + name
    data = public_json(url)
    if data is None:
        return {"status": "not-found", "ownership_verified": False}
    owners = public_json(url + "/owners")
    require(owners is not None, f"owners unavailable: {name}")
    return {
        "status": "exists", "owners": [u["login"] for u in owners["users"]],
        "versions": [{"version": v["num"], "checksum": v["checksum"], "yanked": v["yanked"]}
                     for v in data["versions"]],
    }


def disposition(row, state, owner):
    if state["status"] == "not-found":
        return "first-publication-pending-approval"
    require(state["status"] == "exists", f"unknown registry state: {row['name']}")
    require(owner in state["owners"], f"maintainer is not an owner: {row['name']}")
    exact = [v for v in state["versions"] if v["version"] == row["version"]]
    require(len(exact) <= 1, f"duplicate registry version: {row['name']}")
    if not exact:
        return "new-version-pending-approval"
    require(not exact[0]["yanked"], f"version is yanked: {row['name']}")
    require(exact[0]["checksum"] == row["sha256"], f"published checksum differs: {row['name']}")
    return "already-present"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--ledger", type=Path, required=True)
    parser.add_argument("--archive-dir", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    output = output_path(root, args.output)
    output.parent.mkdir(parents=True, exist_ok=True)
    report = {"status": "failed", "upload_performed": False, "permission_changes": False}
    try:
        policy = json.loads((root / "packaging/crates-io/source-policy.json").read_text())
        ledger = json.loads(args.ledger.read_text())
        head = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip()
        require(head == ledger["source_sha"], "HEAD differs from packaged source; rebuild ledger")
        require(not subprocess.check_output(["git", "status", "--porcelain"], cwd=root),
                "worktree is dirty; commit before preflight")
        audited = audit_archives(ledger, policy, args.archive_dir)
        order = dependency_order(policy["packages"])
        rows = []
        for name in order:
            state = registry_state(name)
            rows.append({"name": name, "version": audited[name]["version"],
                         "archive_sha256": audited[name]["sha256"], "registry": state,
                         "action": disposition(audited[name], state, "edwardkim")})
            time.sleep(0.25)
        report.update(status="local-preflight-passed", source_sha=head,
                      observed_at=datetime.now(timezone.utc).isoformat(),
                      owner="edwardkim", dependency_order=order, packages=rows,
                      public_parent_dry_run_verified=False, publication_authorized=False)
        return 0
    except (CheckError, OSError, ValueError, KeyError, tarfile.TarError) as error:
        report["error"] = str(error)
        print(f"preflight failed: {error}", file=sys.stderr)
        return 2
    finally:
        output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")


if __name__ == "__main__":
    sys.exit(main())
