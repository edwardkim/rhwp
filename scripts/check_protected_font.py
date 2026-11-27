"""정본 공급 항목에 사용할 수 있는 로컬 글꼴의 파일명과 SHA-256을 확인한다."""

import argparse
import hashlib
import json
from pathlib import Path


def accepted_hashes(required, pairs):
    for pair in pairs.values():
        if required in pair:
            return [required, *(digest for digest in pair if digest != required)]
    return [required]


def supply_hashes(manifest, required, document=None):
    excluded = set(manifest.get("excluded_sha256", []))
    if required in excluded:
        return []
    pairs = manifest.get("accepted_sha256_pairs", {})
    groups = manifest["font_sets"].values()
    if document is not None:
        pairs = {**pairs, **document.get("accepted_sha256_pairs", {})}
        groups = (manifest["font_sets"][backend["font_set"]] for backend in document["backends"].values())

    allowed = [digest for digest in accepted_hashes(required, pairs) if digest not in excluded]
    if document is None and len(allowed) > 1:
        return allowed
    if any(entry["sha256"] in allowed for group in groups for entry in group["files"]):
        return allowed
    return []


def excluded_font_path(path, suffixes):
    excluded = tuple(suffix.casefold() for suffix in suffixes)
    return path.name.casefold().endswith(excluded) or path.resolve().name.casefold().endswith(excluded)


def main():
    root = Path(__file__).resolve().parents[1]
    manifest_path = root / "mydocs/verification/protected-font-supply/manifest.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    documents = {document["key"]: document for document in manifest["documents"]}

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--document", choices=documents, metavar="KEY", help="문서별 공급 조건을 적용할 문서 key")
    parser.add_argument("required_sha256", help="문서별 font_set에 기록된 SHA-256")
    parser.add_argument("file", type=Path, help="공급할 로컬 글꼴 파일")
    args = parser.parse_args()

    required = args.required_sha256.lower()
    allowed = supply_hashes(manifest, required, documents.get(args.document))
    if not allowed:
        parser.error(f"공급 목록에 없는 SHA-256: {required}")
    checksum = hashlib.sha256()
    try:
        if excluded_font_path(args.file, manifest.get("excluded_filename_suffixes", [])):
            parser.exit(1, f"정본 보호 공급에서 제외한 글꼴: {args.file}\n")
        with args.file.open("rb") as stream:
            for block in iter(lambda: stream.read(1024 * 1024), b""):
                checksum.update(block)
    except OSError as error:
        parser.exit(1, f"{error}\n")
    actual = checksum.hexdigest()
    if actual in manifest.get("excluded_sha256", []):
        parser.exit(1, f"정본 보호 공급에서 제외한 SHA-256: {args.file}: {actual}\n")
    if actual not in allowed:
        parser.exit(1, f"{args.file}: {actual}\n허용 SHA-256: {', '.join(allowed)}\n")
    print(actual)


if __name__ == "__main__":
    main()
