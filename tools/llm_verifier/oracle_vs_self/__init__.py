"""V-oracle: Hangul-official PDF vs self-consistency selection tree.

This package decides *when* an independent Hangul-official PDF may be
trusted as an oracle, and when the only honest claim is self-consistency
(render-diff A==A / page-count self).

It consumes the published contracts of ``tools/fidelity_compare``,
``tools/oracle_public``, and ``scripts/visual_sweep.py`` as *data*.
Those tools are not imported and not rewritten.
"""

from .decide import (
    CLAIM_ID,
    SCHEMA_VERSION,
    Decision,
    DecisionInputs,
    decide,
    decide_row,
)
from .versions import (
    ALLOWED_HANCOM_YEARS,
    RESOLVER_HANCOM_YEARS,
    ParsedVersions,
    parse_versions,
)

__all__ = [
    "ALLOWED_HANCOM_YEARS",
    "CLAIM_ID",
    "Decision",
    "DecisionInputs",
    "ParsedVersions",
    "RESOLVER_HANCOM_YEARS",
    "SCHEMA_VERSION",
    "decide",
    "decide_row",
    "parse_versions",
]
