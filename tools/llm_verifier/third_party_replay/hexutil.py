"""SHA-256 hex contract used by `rhwp replay --expect-output-sha256`.

The CLI normalizes ASCII hex to lowercase and rejects anything that is not
exactly 64 hex digits (exit 2, no envelope). This module wraps that rule.
It does not hash files; it only classifies the expect-sha token.
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


def expect_sha_defect(value: str | None) -> str | None:
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
