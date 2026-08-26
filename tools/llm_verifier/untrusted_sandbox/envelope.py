"""Read existing rhwp untrustedContent / untrustedFields. No new CLI."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Iterator, Mapping


@dataclass(frozen=True)
class UntrustedSlice:
    path: str
    excerpt: str
    command: str


def provenance_present(envelope: Mapping[str, Any]) -> bool:
    return "untrustedContent" in envelope or "untrustedFields" in envelope


def untrusted_content(envelope: Mapping[str, Any]) -> bool | None:
    if "untrustedContent" not in envelope:
        return None
    return bool(envelope["untrustedContent"])


def untrusted_fields(envelope: Mapping[str, Any]) -> tuple[str, ...]:
    raw = envelope.get("untrustedFields")
    if raw is None:
        return ()
    if not isinstance(raw, list):
        raise TypeError("untrustedFields must be a list of paths")
    return tuple(str(item) for item in raw)


def walk_path(root: Any, path: str) -> Iterator[Any]:
    """Expand a provenance path (dot + [])."""
    tokens = _tokenize(path)
    stack: list[Any] = [root]
    for token in tokens:
        nxt: list[Any] = []
        for cur in stack:
            if token == "[]":
                if isinstance(cur, list):
                    nxt.extend(cur)
            else:
                if isinstance(cur, Mapping) and token in cur:
                    nxt.append(cur[token])
        stack = nxt
        if not stack:
            return
    for item in stack:
        yield item


def _tokenize(path: str) -> list[str]:
    tokens: list[str] = []
    buf: list[str] = []
    i = 0
    while i < len(path):
        if path.startswith("[]", i):
            if buf:
                tokens.append("".join(buf))
                buf = []
            tokens.append("[]")
            i += 2
            if i < len(path) and path[i] == ".":
                i += 1
            continue
        ch = path[i]
        if ch == ".":
            if buf:
                tokens.append("".join(buf))
                buf = []
            i += 1
            continue
        buf.append(ch)
        i += 1
    if buf:
        tokens.append("".join(buf))
    return tokens


def extract_slices(envelope: Mapping[str, Any], command: str = "") -> list[UntrustedSlice]:
    flag = untrusted_content(envelope)
    fields = untrusted_fields(envelope)
    if flag is None:
        # Missing keys: treat the whole envelope as unmarked untrusted.
        return [UntrustedSlice(path="<missing-provenance>", excerpt="", command=command)]
    if flag is False:
        return []
    slices: list[UntrustedSlice] = []
    for path in fields:
        for value in walk_path(envelope, path):
            if value is None:
                continue
            if isinstance(value, (dict, list)):
                text = repr(value)
            else:
                text = str(value)
            if text == "":
                continue
            slices.append(UntrustedSlice(path=path, excerpt=text, command=command))
    return slices


# Provenance-map field paths this sandbox knows how to isolate.
# Authority remains `rhwp export-provenance-map --json`.
KNOWN_UNTRUSTED_PATHS: dict[str, tuple[str, ...]] = {
    "info": ("title", "fonts[]"),
    "export-text": ("pages[].text", "text"),
    "export-structure": ("structure.roots[].heading", "structure.roots[].text"),
    "digest": ("outline[]", "excerpt", "sections[].heading", "sections[].excerpt"),
    "search": ("matches[].text", "matches[].context"),
    "fields": (
        "fields[].name",
        "fields[].guide",
        "fields[].memo",
        "fields[].command",
        "fields[].value",
        "textSecurity.findings[].names[]",
    ),
    "export-tables": (
        "tables[].caption",
        "tables[].cells[].text",
        "tables[].cells[].nested[]",
    ),
    "dump-pages": ("pages[].columns[].items[].textPreview",),
    "edit": ("oldText", "confusable[].lookalikes"),
    "run": ("steps[].oldText", "steps[].confusable[].lookalikes"),
    "thumbnail": ("base64", "dataUri"),
    "ir-diff": ("categories",),
}

COMMANDS: tuple[str, ...] = tuple(KNOWN_UNTRUSTED_PATHS)
