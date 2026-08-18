"""SHA-256 hex contract used by `rhwp lineage` parent/child hashes.

`rhwp lineage` compares parent `outputSha256` to child `inputSha256`.
Both tokens are 64 ASCII hex digits (lowercase after normalize). This
module only classifies tokens. It does not hash files and it does not
re-run `rhwp replay` (that is V-replay, a different axis).
"""

from __future__ import annotations

import re

SHA256_HEX_LEN = 64
_HEX_RE = re.compile(r"^[0-9a-f]{64}$")
_HEX_CHARS = re.compile(r"^[0-9a-fA-F]+$")


def normalize_sha256_hex(value: str | None) -> str:
    if value is None:
        return ""
    return str(value).strip().lower()


def is_sha256_hex(value: str | None) -> bool:
    return bool(_HEX_RE.fullmatch(normalize_sha256_hex(value)))


def sha_defect(value: str | None) -> str | None:
    """Return a closed defect token, or None when the token is a valid 64-hex."""
    if value is None:
        return "missing"
    raw = str(value)
    stripped = raw.strip()
    if stripped == "":
        return "missing"
    if stripped.startswith("0x") or stripped.startswith("0X"):
        return "prefixed"
    if any(ch.isspace() for ch in raw):
        return "whitespace"
    if len(stripped) != SHA256_HEX_LEN:
        return "length"
    if not _HEX_CHARS.fullmatch(stripped):
        return "nonhex"
    return None
