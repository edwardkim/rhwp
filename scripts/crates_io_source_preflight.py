#!/usr/bin/env python3
"""Check immutable Git source candidates; never normalize Cargo or publish."""

from __future__ import annotations

import argparse
import fnmatch
import gzip
import hashlib
import io
import json
import posixpath
import re
import subprocess
import sys
import tarfile
import tomllib
from pathlib import Path, PurePosixPath


class PreflightError(ValueError):
    """The source candidate cannot be validated."""


def require(condition: object, message: str) -> None:
    if not condition:
        raise PreflightError(message)


def git(repo: Path, *args: str, data: bytes | None = None) -> bytes:
    result = subprocess.run(
        ["git", *args], cwd=repo, input=data, capture_output=True, check=True
    )
    return result.stdout


def relative(value: str, *, root_allowed: bool = False) -> str:
    require(isinstance(value, str) and bool(value), "empty/non-string source path")
    p = PurePosixPath(value)
    require(
        not p.is_absolute() and ".." not in p.parts and "\\" not in value
        and p.as_posix() == value and (value != "." or root_allowed),
        f"unsafe or non-canonical source path: {value}",
    )
    return value


def select_files(spec: dict, tree: dict) -> dict[str, str]:
    source = relative(spec["source"], root_allowed=True)
    prefix = "" if source == "." else source + "/"
    patterns = spec["include"]
    require(isinstance(patterns, list) and patterns, "include patterns are required")
    files: dict[str, str] = {}
    for pattern in patterns:
        relative(pattern)
        matches = {
            p[len(prefix):]: p for p in tree
            if p.startswith(prefix) and fnmatch.fnmatchcase(p[len(prefix):], pattern)
        }
        require(matches, f"unmatched include pattern: {source}/{pattern}")
        files.update(matches)
    for destination, origin in spec.get("extra_repository_files", {}).items():
        relative(destination)
        relative(origin)
        require(origin in tree, f"extra source is not committed: {origin}")
        require(destination not in files, f"duplicate archive destination: {destination}")
        files[destination] = origin
    require("Cargo.toml" in files, f"manifest omitted: {source}")
    for destination, origin in files.items():
        relative(destination)
        parts = PurePosixPath(origin).parts
        require(
            not origin.startswith(("output/", "mydocs/privacy/", "samples/", "pdf/"))
            and not any(p == ".git" or p == ".env" or p.startswith(".env.") for p in parts),
            f"private/generated/corpus source selected: {origin}",
        )
        require(tree[origin][0] in {"100644", "100755"}, f"non-regular Git entry: {origin}")
    return files


def source_blobs(repo: Path, tree: dict, paths: set[str]) -> dict[str, bytes]:
    oids = sorted({tree[path][1] for path in paths})
    payload = ("\n".join(oids) + "\n").encode()
    sizes = git(repo, "cat-file", "--batch-check", data=payload).decode().splitlines()
    total = 0
    for line in sizes:
        _, kind, size = line.split()
        require(kind == "blob", "selected Git object is not a blob")
        require(int(size) <= 64 * 1024 * 1024, "source blob exceeds local 64 MiB guard")
        total += int(size)
    require(total <= 512 * 1024 * 1024, "source set exceeds local 512 MiB guard")
    stream = io.BytesIO(git(repo, "cat-file", "--batch", data=payload))
    objects = {}
    for expected in oids:
        oid, kind, size = stream.readline().decode().split()
        require(oid == expected and kind == "blob", "unexpected Git batch response")
        objects[oid] = stream.read(int(size))
        require(stream.read(1) == b"\n", "truncated Git batch response")
    return {path: objects[tree[path][1]] for path in paths}


def metadata(manifest: dict, root_manifest: dict, field: str):
    value = manifest["package"].get(field)
    if isinstance(value, dict) and value.get("workspace") is True:
        workspace = manifest.get("workspace", root_manifest.get("workspace", {}))
        value = workspace.get("package", {}).get(field)
    return value


def output_file(path: Path) -> Path:
    require(not path.is_symlink() and not path.is_dir(), f"unsafe output file: {path}")
    return path


def path_dependencies(manifest: dict, inherited: dict, workspace_source: str, source: str):
    for section in ("dependencies", "build-dependencies", "dev-dependencies"):
        for alias, dep in manifest.get(section, {}).items():
            if isinstance(dep, dict) and dep.get("workspace") is True:
                require(alias in inherited, f"inherited dependency missing: {alias}")
                base = inherited[alias]
                base = {"version": base} if isinstance(base, str) else dict(base)
                if "path" in base:
                    base["path"] = posixpath.relpath(
                        posixpath.join(workspace_source, base["path"]), source
                    )
                dep = {**base, **dep}
            if isinstance(dep, dict) and "path" in dep:
                yield section, alias, dep
    for target, config in manifest.get("target", {}).items():
        for section, alias, dep in path_dependencies(config, inherited, workspace_source, source):
            yield f"target.{target}.{section}", alias, dep
    for registry, patches in manifest.get("patch", {}).items():
        for alias, dep in patches.items():
            if isinstance(dep, dict) and "path" in dep:
                yield f"patch.{registry}", alias, dep


def check_fonts(selections: dict, data: dict, registry_path: str) -> list[dict]:
    fonts = {p for files in selections.values() for p in files.values()
             if PurePosixPath(p).suffix.lower() in {".otf", ".ttf", ".ttc", ".woff", ".woff2"}}
    if not fonts:
        return []
    registry = json.loads(data[registry_path])
    rows = {r["repositoryPath"]: r for r in registry.get("repositoryOnlyFonts", [])}
    rows.update({"assets/fonts/" + r["file"]: {
        **r, "licenseFiles": ["assets/fonts/licenses/" + p for p in r["licenseFiles"]]
    } for r in registry.get("fonts", [])})
    checked = []
    for path in sorted(fonts):
        row = rows.get(path)
        require(row, f"font missing from notice registry: {path}")
        require(hashlib.sha256(data[path]).hexdigest() == row["sha256"], f"font digest mismatch: {path}")
        companions = row.get("licenseFiles", []) + ([row["provenance"]] if row.get("provenance") else [])
        require(row.get("licenseFiles"), f"font license files missing: {path}")
        for files in selections.values():
            if path in files.values():
                require(set(companions) <= set(files.values()), f"font license/provenance omitted: {path}")
        checked.append({"path": path, "sha256": row["sha256"], "companions": companions})
    return checked


def output_directory(repo: Path, output: Path) -> Path:
    output = output.resolve()
    require(output.is_relative_to(repo / "output"), "output must stay below repository output/")
    require(
        subprocess.run(["git", "check-ignore", "-q", str(output)], cwd=repo).returncode == 0,
        "output directory must be Git-ignored",
    )
    return output


def build_report(repo: Path, policy: dict, runtime: dict, output: Path, ref: str = "HEAD") -> dict:
    repo = repo.resolve()
    output = output_directory(repo, output)
    require(policy.get("schema_version") == 1, "unsupported source policy schema")
    require(runtime.get("schema_version") == 1, "unsupported runtime contract schema")
    sha = git(repo, "rev-parse", "--verify", f"{ref}^{{commit}}").decode().strip()
    tree = {}
    for item in git(repo, "ls-tree", "-r", "-z", "--full-tree", sha).split(b"\0"):
        if item:
            meta, path = item.split(b"\t", 1)
            mode, _, oid = meta.decode().split()
            tree[path.decode()] = (mode, oid)
    specs = policy["packages"]
    require(isinstance(specs, list) and bool(specs), "package candidates are required")
    names, selections, by_source = [], {}, {}
    for spec in specs:
        name = spec["published_name"]
        require(re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_-]*", name), f"unsafe package name: {name}")
        require(name not in names and spec["source"] not in by_source, "duplicate package name/source")
        names.append(name)
        by_source[spec["source"]] = name
        selections[name] = select_files(spec, tree)
    paths = {p for files in selections.values() for p in files.values()}
    paths.add("Cargo.toml")
    font_registry = relative(runtime.get("font_registry", "assets/fonts/licenses/font-notices.json"))
    if any(PurePosixPath(p).suffix.lower() in {".otf", ".ttf", ".ttc", ".woff", ".woff2"} for p in paths):
        paths.add(font_registry)
    require(paths <= tree.keys(), "required source is not committed")
    data = source_blobs(repo, tree, paths)
    root_manifest = tomllib.loads(data["Cargo.toml"].decode())
    graph, blockers, package_info = {}, [], {}
    for spec in specs:
        name, source = spec["published_name"], spec["source"]
        prefix = "" if source == "." else source + "/"
        files = selections[name]
        manifest = tomllib.loads(data[files["Cargo.toml"]].decode())
        license_name = metadata(manifest, root_manifest, "license")
        license_file = metadata(manifest, root_manifest, "license-file")
        notices = [p for p in tree if p.startswith(prefix) and (
            (source != "." and PurePosixPath(p).name.startswith(("LICENSE", "NOTICE", "COPYING")))
            or ("/" not in p[len(prefix):] and PurePosixPath(p).name in {
                "LICENSE", "THIRD_PARTY_LICENSES.md", "RHWP_PATCHES.md", "UPSTREAM.json",
                "UPSTREAM_FILES.json", "NOTICE", "TRADEMARKS.md",
            })
        )]
        require(set(notices) <= set(files.values()), f"license/provenance source omitted: {name}")
        require(license_name or license_file, f"license metadata missing: {name}")
        require(any(PurePosixPath(p).name.startswith("LICENSE") for p in files), f"license text missing: {name}")
        if license_file:
            require(license_file in files, f"declared license-file omitted: {name}")
        if manifest["package"].get("publish") is False:
            blockers.append({"package": name, "kind": "publish_false"})
        dependencies, edges = [], set()
        workspace = manifest.get("workspace", root_manifest.get("workspace", {}))
        workspace_source = source if "workspace" in manifest else "."
        for section, alias, dep in path_dependencies(manifest, workspace.get("dependencies", {}), workspace_source, source):
            target = posixpath.normpath(posixpath.join(source, dep["path"]))
            require(target in by_source, f"local dependency outside candidate graph: {source}: {target}")
            edges.add(by_source[target])
            dependencies.append({"section": section, "alias": alias, "source": target,
                                 "version": dep.get("version")})
            kind = "root_patch_not_propagated" if section.startswith("patch.") else "path_version_missing"
            if section.startswith("patch.") or not dep.get("version"):
                blockers.append({"package": name, "kind": kind, "dependency": alias})
        require(edges == set(spec["local_dependencies"]), f"declared/source graph mismatch: {name}")
        graph[name] = edges
        package_info[name] = {
            "current_name": manifest["package"]["name"],
            "current_version": metadata(manifest, root_manifest, "version"),
            "proposed_name": name, "proposed_version": spec["version"],
            "license": license_name, "notices_checked": len(notices),
            "path_dependencies": dependencies,
        }
    order = []
    while len(order) < len(names):
        ready = sorted(n for n in names if n not in order and graph[n] <= set(order))
        require(ready, "candidate dependency cycle")
        order.extend(ready)
    includes = re.compile(r'include_(?:str|bytes)!\s*\(\s*"([^"\n]+)"')
    checked = []
    for entry in runtime["files"]:
        consumer, target = relative(entry["consumer"]), relative(entry["path"])
        require(consumer in data and target in data, f"runtime source omitted: {target}")
        literals = includes.findall(data[consumer].decode())
        matches = [raw for raw in literals if posixpath.normpath(posixpath.join(posixpath.dirname(consumer), raw)) == target]
        require(matches, f"runtime include no longer found: {consumer}: {target}")
        require(any(
            files.get(posixpath.normpath(posixpath.join(posixpath.dirname(dest), raw))) == target
            for files in selections.values() for dest, origin in files.items() if origin == consumer
            for raw in matches
        ), f"runtime include not preserved within one package: {target}")
        checked.append(entry)
    fonts_checked = check_fonts(selections, data, font_registry)
    output.mkdir(parents=True, exist_ok=True)
    rows = []
    for name in order:
        archive = output_file(output / f"{name}.source.tar.gz")
        ledger = []
        with archive.open("wb") as f, gzip.GzipFile(filename="", fileobj=f, mode="wb", mtime=0) as gz, tarfile.open(fileobj=gz, mode="w|", format=tarfile.PAX_FORMAT) as tar:
            for destination, origin in sorted(selections[name].items()):
                payload = data[origin]
                info = tarfile.TarInfo(f"{name}-source/{destination}")
                info.size, info.mtime = len(payload), 0
                info.mode = 0o755 if tree[origin][0] == "100755" else 0o644
                tar.addfile(info, io.BytesIO(payload))
                ledger.append({"package_path": destination, "source_path": origin,
                               "git_blob": tree[origin][1], "bytes": len(payload),
                               "git_mode": tree[origin][0], "archive_mode": info.mode,
                               "sha256": hashlib.sha256(payload).hexdigest()})
        require(archive.stat().st_size < 10_000_000, f"source archive exceeds local 10 MB guard: {name}")
        with tarfile.open(archive) as tar:
            members = tar.getmembers()
            require(len(members) == len(ledger), f"archive file count mismatch: {name}")
            for member, expected in zip(members, ledger, strict=True):
                require(member.isfile() and member.name == f"{name}-source/{expected['package_path']}", f"archive member mismatch: {name}")
                require(member.mode == expected["archive_mode"], f"archive mode mismatch: {member.name}")
                require(hashlib.sha256(tar.extractfile(member).read()).hexdigest() == expected["sha256"], f"archive bytes mismatch: {member.name}")
        output_file(output / f"{name}.files.json").write_text(json.dumps(ledger, ensure_ascii=False, indent=2) + "\n")
        rows.append({**package_info[name], "files": len(ledger),
                     "raw_bytes": sum(p["bytes"] for p in ledger), "archive_readback_verified": True,
                     "archive_bytes": archive.stat().st_size,
                     "archive_sha256": hashlib.sha256(archive.read_bytes()).hexdigest()})
    return {
        "schema_version": 1, "source_sha": sha,
        "tool_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        "input_digest_format": "json.dumps(sort_keys=True, ensure_ascii=True)",
        "policy_sha256": hashlib.sha256(json.dumps(policy, sort_keys=True).encode()).hexdigest(),
        "runtime_contract_sha256": hashlib.sha256(json.dumps(runtime, sort_keys=True).encode()).hexdigest(),
        "source_check": "pass", "kind": "source-candidates-not-Cargo-crates",
        "policy_status": policy.get("status"), "publish_ready": False,
        "cargo_normalized": False, "cargo_verified": False,
        "registry_consumption_verified": False, "decision_selection": None,
        "dependency_order": order, "runtime_includes_checked": checked,
        "font_assets_checked": fonts_checked,
        "manifest_blockers": blockers, "packages": rows,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--policy", type=Path, default=Path("packaging/crates-io/source-policy.json"))
    parser.add_argument("--runtime-files", type=Path, default=Path("packaging/crates-io/runtime-files.json"))
    parser.add_argument("--output-dir", type=Path, default=Path("output/pr-review/issue8174/source-preflight"))
    parser.add_argument("--ref", default="HEAD")
    args = parser.parse_args()
    report_path = None
    try:
        repo = args.repo.resolve()
        output = output_directory(repo, repo / args.output_dir)
        output.mkdir(parents=True, exist_ok=True)
        report_path = output_file(output / "source-preflight.json")
        report_path.write_text(json.dumps({"source_check": "in_progress", "publish_ready": False}) + "\n")
        policy = json.loads((repo / args.policy).read_text())
        runtime = json.loads((repo / args.runtime_files).read_text())
        report = build_report(repo, policy, runtime, output, args.ref)
        report_path.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
        print(json.dumps({k: report[k] for k in ("source_sha", "source_check", "publish_ready", "dependency_order", "manifest_blockers")}, ensure_ascii=False, indent=2))
        return 0
    except (ValueError, OSError, KeyError, TypeError, subprocess.CalledProcessError) as error:
        if report_path is not None:
            try:
                report_path.write_text(json.dumps({"source_check": "failed", "publish_ready": False, "error": str(error)}, ensure_ascii=False) + "\n")
            except OSError:
                pass
        print(f"source preflight failed: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
