"""V-nonce: nonce/boundary sandbox around untrustedContent/untrustedFields.

Document-derived text is data. It must not become a verification criterion
or an instruction. This package is the verifier-side sandbox only.
`.agents/skills/rhwp-provenance` is not rewritten. No rhwp CLI is invented.
"""

from .decide import CLAIM_ID, SCHEMA_VERSION, Decision, decide, decide_row
from .envelope import extract_slices, untrusted_content, untrusted_fields
from .host import SandboxReport, isolate_envelope
from .leak import detect_leak
from .nonce import WrapError, wrap_untrusted
from .schema import SandboxCase
from .slot import SLOT_VALUES, Slot

__all__ = [
    "CLAIM_ID",
    "SCHEMA_VERSION",
    "Decision",
    "SLOT_VALUES",
    "SandboxCase",
    "SandboxReport",
    "Slot",
    "WrapError",
    "decide",
    "decide_row",
    "detect_leak",
    "extract_slices",
    "isolate_envelope",
    "untrusted_content",
    "untrusted_fields",
    "wrap_untrusted",
]
