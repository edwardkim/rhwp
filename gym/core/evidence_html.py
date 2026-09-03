"""Deterministic, self-contained HTML view for sealed Gym evidence (#6669).

This module is deliberately presentation-only.  The caller must validate the
input bundle and its seal before invoking :func:`render_html`.
"""

from __future__ import annotations

import hashlib
import html
import re


STATUS_SYMBOL = {
    "PASS": "✓",
    "FAIL": "✕",
    "INCOMPLETE": "!",
    "NOT_APPLICABLE": "—",
}

ROLE_LABELS = {
    "unit": "Unit tests",
    "audit": "Structure audit",
    "oracleStructural": "Oracle structure",
    "oracleSelftest": "Oracle self-test",
    "authorityLedger": "Authority ledger",
    "positive": "Positive baseline",
    "discrimination": "Discrimination",
    "trajectory": "Trajectory necessity",
}

DOCUMENT_TOKEN_RE = re.compile(
    r"(?i)(?<![\w.-])[^\s/\\<>\"']+\.(?:hwp|hwpx|pdf|docx?|xlsx?|pptx?)(?![\w.-])"
)
WINDOWS_PATH_RE = re.compile(r"(?i)(?<![\w])(?:[a-z]:[\\/]|\\\\)[^\s<>\"']+")
POSIX_PATH_RE = re.compile(r"(?<![\w.-])/(?:[^\s<>\"']+)")


def _hash_text(value: str) -> str:
    return hashlib.sha256(value.encode("utf-8")).hexdigest()


def _redact(value, *, limit: int = 400) -> str:
    """Escape free text and disclose deterministic redaction metadata."""
    raw = "" if value is None else str(value)
    masked = WINDOWS_PATH_RE.sub("[absolute-path]", raw)
    masked = POSIX_PATH_RE.sub("[absolute-path]", masked)
    masked = DOCUMENT_TOKEN_RE.sub("[document-file]", masked)
    changed = masked != raw
    if len(masked) > limit:
        masked = masked[:limit] + "…"
        changed = True
    rendered = html.escape(masked, quote=True)
    if changed:
        rendered += (
            '<span class="redaction">'
            f"redacted · source chars {len(raw)} · sha256 {_hash_text(raw)}"
            "</span>"
        )
    return rendered


def _safe_platform(value) -> str:
    """Hide uname's hostname while retaining reproducibility-relevant fields."""
    raw = "" if value is None else str(value)
    parts = raw.split()
    if len(parts) >= 2 and parts[0] in {"Linux", "Darwin", "FreeBSD", "OpenBSD", "NetBSD"}:
        parts[1] = "[host]"
        raw = " ".join(parts)
    return _redact(raw)


def _status_badge(status: str) -> str:
    normalized = status if status in STATUS_SYMBOL else "INCOMPLETE"
    return (
        f'<span class="status status-{normalized.lower()}" '
        f'aria-label="{html.escape(normalized)}">'
        f'<span aria-hidden="true">{STATUS_SYMBOL[normalized]}</span> {normalized}</span>'
    )


def _number(value) -> str:
    return f"{value:,}" if type(value) is int else "—"


def _metric(label: str, value) -> str:
    return (
        '<div class="metric"><span class="metric-value">'
        f"{html.escape(str(value))}</span><span class=\"metric-label\">"
        f"{html.escape(label)}</span></div>"
    )


def _row(label: str, value: str) -> str:
    return f"<dt>{html.escape(label)}</dt><dd>{value}</dd>"


def _short_hash(value) -> str:
    text = "" if value is None else str(value)
    return html.escape(text, quote=True)


def _role_metrics(role: str, bundle: dict) -> list[tuple[str, object]]:
    docs = bundle["documents"]
    process = bundle["processes"][role]
    seconds = process.get("seconds")
    metrics: list[tuple[str, object]] = [("seconds", _number(seconds))]
    if role == "unit":
        return [("exit", _number(process.get("exit"))), *metrics]
    report = docs[role]
    if role == "audit":
        return [
            ("packs", _number(report.get("packCount"))),
            ("tasks", _number(report.get("taskCount"))),
            ("issues", _number(report.get("issueCount"))),
            *metrics,
        ]
    if role == "oracleStructural":
        return [("issues", _number(report.get("issueCount"))), *metrics]
    if role == "oracleSelftest":
        return [
            ("checks", _number(report.get("checkCount"))),
            ("failed", _number(report.get("issueCount"))),
            *metrics,
        ]
    if role == "authorityLedger":
        return [
            ("entries", _number(report.get("entryCount"))),
            ("issues", _number(report.get("issueCount"))),
            *metrics,
        ]
    if role == "positive":
        return [
            ("tasks", _number(report.get("taskCount"))),
            ("built", _number(report.get("built"))),
            ("failed", _number(report.get("failed"))),
            *metrics,
        ]
    if role == "discrimination":
        return [
            ("controls", _number(report.get("controlCount"))),
            ("rejected", _number(report.get("discriminating"))),
            ("false-pass", _number(len(report.get("falsePassControls") or []))),
            *metrics,
        ]
    return [
        ("tasks", _number(report.get("taskCount"))),
        ("load-bearing", _number(report.get("loadBearing"))),
        ("N/A", _number(bundle["status"]["roles"][role].get("notApplicable"))),
        *metrics,
    ]


def _role_cards(bundle: dict) -> str:
    cards = []
    for role in ROLE_LABELS:
        state = bundle["status"]["roles"][role]
        metrics = "".join(_metric(label, value) for label, value in _role_metrics(role, bundle))
        reasons = state.get("reasons") or []
        reason_html = ""
        if reasons:
            reason_html = '<ul class="reasons">' + "".join(
                f"<li>{_redact(reason)}</li>" for reason in reasons
            ) + "</ul>"
        trajectory_flags = ""
        if role == "trajectory":
            report = bundle["documents"][role]
            trajectory_flags = (
                '<dl class="flags">'
                + _row("trajectory.ok", _redact(str(report.get("ok")).lower()))
                + _row("trajectory.trusted", _redact(str(report.get("trusted")).lower()))
                + "</dl>"
            )
        cards.append(
            '<article class="card">'
            f"<h3>{html.escape(ROLE_LABELS[role])}</h3>"
            f"{_status_badge(state['status'])}"
            f'<div class="metrics">{metrics}</div>{trajectory_flags}{reason_html}'
            "</article>"
        )
    return "".join(cards)


def _pack_statistics(bundle: dict) -> list[dict]:
    docs = bundle["documents"]
    rows = {}
    for pack in docs["audit"].get("packs") or []:
        if not isinstance(pack, dict):
            continue
        pack_id = pack.get("id")
        if isinstance(pack_id, str) and pack_id:
            rows[pack_id] = {
                "pack": pack_id,
                "tasks": pack.get("taskCount", 0),
                "references": pack.get("referenceCount", 0),
                "positivePass": 0,
                "positiveTotal": 0,
                "controlsRejected": 0,
                "controlsTotal": 0,
                "trajectoryBearing": 0,
                "trajectoryTotal": 0,
                "trajectoryNa": 0,
            }

    def ensure(pack_id):
        if isinstance(pack_id, str) and pack_id:
            return rows.setdefault(pack_id, {
                "pack": pack_id,
                "tasks": 0,
                "references": 0,
                "positivePass": 0,
                "positiveTotal": 0,
                "controlsRejected": 0,
                "controlsTotal": 0,
                "trajectoryBearing": 0,
                "trajectoryTotal": 0,
                "trajectoryNa": 0,
            })
        return None

    for result in docs["positive"].get("results") or []:
        row = ensure(result.get("pack") if isinstance(result, dict) else None)
        if row is not None:
            row["positiveTotal"] += 1
            row["positivePass"] += int(result.get("ok") is True and result.get("kind") == "ok")
    for result in docs["discrimination"].get("results") or []:
        row = ensure(result.get("pack") if isinstance(result, dict) else None)
        if row is not None:
            row["controlsTotal"] += 1
            row["controlsRejected"] += int(result.get("discriminates") is True)
    for result in docs["trajectory"].get("results") or []:
        row = ensure(result.get("pack") if isinstance(result, dict) else None)
        if row is not None:
            row["trajectoryTotal"] += 1
            row["trajectoryBearing"] += int(result.get("loadBearing") is True)
    for result in docs["trajectory"].get("skipped") or []:
        row = ensure(result.get("pack") if isinstance(result, dict) else None)
        if row is not None and result.get("reason") == "single-step":
            row["trajectoryNa"] += 1
    return [rows[key] for key in sorted(rows)]


def _ratio_bar(numerator, denominator) -> str:
    n = numerator if type(numerator) is int and numerator >= 0 else 0
    d = denominator if type(denominator) is int and denominator > 0 else 0
    percent = min(100.0, max(0.0, n * 100.0 / d)) if d else 0.0
    return (
        '<div class="bar" aria-hidden="true"><span style="width:'
        f'{percent:.1f}%"></span></div>'
        f'<span class="ratio">{_number(numerator)} / {_number(denominator)}</span>'
    )


def _pack_table(bundle: dict) -> str:
    body = []
    for row in _pack_statistics(bundle):
        body.append(
            "<tr>"
            f"<th scope=\"row\">{_redact(row['pack'])}</th>"
            f"<td>{_number(row['tasks'])}</td>"
            f"<td>{_number(row['references'])}</td>"
            f"<td>{_ratio_bar(row['positivePass'], row['positiveTotal'])}</td>"
            f"<td>{_ratio_bar(row['controlsRejected'], row['controlsTotal'])}</td>"
            f"<td>{_ratio_bar(row['trajectoryBearing'], row['trajectoryTotal'])}"
            f"<small>N/A {_number(row['trajectoryNa'])}</small></td>"
            "</tr>"
        )
    if not body:
        body.append('<tr><td colspan="6">No pack rows</td></tr>')
    return (
        '<div class="table-wrap"><table><caption>Pack-level measured distribution</caption>'
        "<thead><tr><th>Pack</th><th>Tasks</th><th>References</th>"
        "<th>Positive pass</th><th>Controls rejected</th><th>Load-bearing</th></tr></thead>"
        f"<tbody>{''.join(body)}</tbody></table></div>"
    )


def _item_summary(item) -> str:
    if not isinstance(item, dict):
        return _redact(item)
    safe_keys = (
        "pack", "task", "control", "kind", "reason", "removedStep",
        "message", "error", "text", "count",
    )
    values = []
    for key in safe_keys:
        if key in item and item[key] not in (None, ""):
            values.append(f"<strong>{html.escape(key)}</strong>: {_redact(item[key])}")
    return " · ".join(values) if values else "redacted structured item"


def _detail_block(title: str, items, *, open_by_default: bool = False) -> str:
    values = list(items or [])
    if not values:
        return ""
    rendered = "".join(f"<li>{_item_summary(item)}</li>" for item in values)
    opened = " open" if open_by_default else ""
    return (
        f"<details{opened}><summary>{html.escape(title)} ({len(values):,})</summary>"
        f"<ol>{rendered}</ol></details>"
    )


def _diagnostics(bundle: dict) -> str:
    docs = bundle["documents"]
    sections = []
    positive_bad = [
        row for row in docs["positive"].get("results") or []
        if not isinstance(row, dict) or row.get("kind") != "ok" or row.get("ok") is not True
    ]
    sections.append(_detail_block("Positive failures, missing items, and skips", positive_bad, open_by_default=True))
    sections.append(_detail_block("Audit issues", docs["audit"].get("issues"), open_by_default=True))
    sections.append(_detail_block("Oracle structural issues", docs["oracleStructural"].get("issues"), open_by_default=True))
    sections.append(_detail_block("Oracle self-test failures", docs["oracleSelftest"].get("failed"), open_by_default=True))
    sections.append(_detail_block("Authority issues", docs["authorityLedger"].get("issues"), open_by_default=True))

    disc = docs["discrimination"]
    sections.append(_detail_block("Discrimination false-pass tasks", disc.get("falsePass"), open_by_default=True))
    sections.append(_detail_block("Discrimination false-pass controls", disc.get("falsePassControls"), open_by_default=True))
    for key, label in (
        ("loadErrors", "Discrimination load errors"),
        ("buildErrors", "Discrimination build errors"),
        ("toolErrors", "Discrimination tool errors"),
        ("skipped", "Discrimination skips"),
    ):
        sections.append(_detail_block(label, disc.get(key), open_by_default=True))
    sections.append(_detail_block(
        "Unexplained score errors",
        bundle["scoreErrorAccounting"].get("unexplained"),
        open_by_default=True,
    ))

    trajectory = docs["trajectory"]
    sections.append(_detail_block("Trajectory theater", trajectory.get("theater"), open_by_default=True))
    sections.append(_detail_block("Trajectory exceptions", trajectory.get("exceptions"), open_by_default=True))
    sections.append(_detail_block("Trajectory tool errors", trajectory.get("toolErrors"), open_by_default=True))
    sections.append(_detail_block("Trajectory N/A and skips", trajectory.get("skipped")))
    visible = "".join(section for section in sections if section)
    return visible or '<p class="quiet">No failure or incomplete details.</p>'


def _authority_notice(bundle: dict) -> str:
    summary = bundle["documents"]["authorityLedger"].get("summary") or {}
    by_authority = summary.get("byAuthority") or {}
    counts = " · ".join(
        f"{_redact(key)} {_number(value)}" for key, value in sorted(by_authority.items())
    )
    return (
        '<aside class="notice"><strong>Authority boundary</strong>'
        "This report measures whether an agent used the rhwp API and Gym task path as specified. "
        "It is not, by itself, an independent oracle for Hancom layout or product correctness."
        f"<span>{counts or 'No authority classes reported'}</span></aside>"
    )


def _identity_section(bundle: dict, manifest: dict) -> str:
    identity = manifest["identity"]
    metadata = bundle["metadata"]
    return (
        '<div class="identity-grid"><dl>'
        + _row("Run ID", _redact(identity.get("runId")))
        + _row("Run started", _redact(identity.get("runStarted")))
        + _row("Runner head", f"<code>{_short_hash(identity.get('runnerHead'))}</code>")
        + _row("Runner tree", f"<code>{_short_hash(identity.get('runnerTree'))}</code>")
        + _row("Product source", f"<code>{_short_hash(identity.get('productSourceHead'))}</code>")
        + "</dl><dl>"
        + _row("Product version", _redact(metadata.get("rhwpVersion")))
        + _row("Binary", _redact(identity.get("binaryName")))
        + _row("Binary SHA-256", f"<code>{_short_hash(identity.get('binarySha256'))}</code>")
        + _row("Binary path fingerprint", f"<code>{_short_hash(identity.get('binaryPathSha256'))}</code>")
        + _row("Identity fingerprint", f"<code>{_short_hash(manifest.get('identityFingerprint'))}</code>")
        + "</dl><dl>"
        + _row("Platform", _safe_platform(metadata.get("platform")))
        + _row("Python", _redact(metadata.get("pythonVersion")))
        + _row("Rust", _redact(metadata.get("rustVersion")))
        + "</dl></div>"
    )


def _provenance_table(manifest: dict) -> str:
    rows = []
    for item in manifest.get("inputs") or []:
        rows.append(
            "<tr>"
            f"<th scope=\"row\">{_redact(item.get('path'))}</th>"
            f"<td>{_redact(item.get('kind') or '—')}</td>"
            f"<td>{_redact(item.get('mode') or '—')}</td>"
            f"<td>{_redact(item.get('schemaVersion') or '—')}</td>"
            f"<td>{_number(item.get('bytes'))}</td>"
            f"<td><code>{_short_hash(item.get('sha256'))}</code></td>"
            "</tr>"
        )
    return (
        '<div class="table-wrap"><table><caption>Sealed input provenance</caption>'
        "<thead><tr><th>Input</th><th>Kind</th><th>Mode</th><th>Schema</th><th>Bytes</th><th>SHA-256</th></tr></thead>"
        f"<tbody>{''.join(rows)}</tbody></table></div>"
    )


def render_html(bundle: dict, manifest: dict) -> bytes:
    """Render a verified bundle. The function performs no I/O or re-scoring."""
    overall = bundle["status"]["overall"]
    score = bundle["scoreErrorAccounting"]
    generator = manifest.get("generator") or {}
    document = f"""<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline'; base-uri 'none'; form-action 'none'">
<title>Gym evidence report — {html.escape(overall)}</title>
<style>
:root{{--ink:#17202a;--muted:#5d6772;--line:#ccd3da;--panel:#f5f7f9;--pass:#146c43;--fail:#b42318;--incomplete:#8a4b08;--na:#53606d;--accent:#2764c5}}
*{{box-sizing:border-box}}body{{margin:0;background:#fff;color:var(--ink);font:15px/1.5 system-ui,-apple-system,"Segoe UI",sans-serif}}
main{{max-width:1180px;margin:auto;padding:2rem 1rem 4rem}}h1{{font-size:clamp(1.8rem,4vw,3rem);margin:.25rem 0}}h2{{margin-top:2.3rem}}h3{{margin:.1rem 0 .7rem}}p{{max-width:78ch}}code{{font:12px/1.35 ui-monospace,SFMono-Regular,Consolas,monospace;overflow-wrap:anywhere}}
.eyebrow,.quiet{{color:var(--muted)}}.hero{{border-bottom:3px solid var(--ink);padding-bottom:1.4rem}}.overall{{display:flex;gap:1rem;align-items:center;flex-wrap:wrap}}
.status{{display:inline-flex;gap:.35rem;align-items:center;border:2px solid currentColor;border-radius:999px;padding:.2rem .65rem;font-weight:750;letter-spacing:.035em}}.status-pass{{color:var(--pass)}}.status-fail{{color:var(--fail)}}.status-incomplete{{color:var(--incomplete)}}.status-not_applicable{{color:var(--na)}}
.notice{{margin:1.25rem 0;padding:1rem;border-left:5px solid var(--incomplete);background:#fff8ec}}.notice strong,.notice span{{display:block}}.notice span{{margin-top:.4rem;color:var(--muted)}}
.cards{{display:grid;grid-template-columns:repeat(auto-fit,minmax(235px,1fr));gap:.8rem}}.card{{border:1px solid var(--line);border-radius:8px;padding:1rem;background:var(--panel)}}.metrics{{display:flex;gap:.8rem;flex-wrap:wrap;margin-top:.9rem}}.metric{{display:flex;flex-direction:column}}.metric-value{{font-size:1.25rem;font-weight:700}}.metric-label{{color:var(--muted);font-size:.78rem;text-transform:uppercase}}.reasons{{padding-left:1.2rem;color:var(--incomplete)}}
.flags,.identity-grid dl{{display:grid;grid-template-columns:max-content 1fr;gap:.25rem .7rem}}dt{{font-weight:650}}dd{{margin:0;min-width:0;overflow-wrap:anywhere}}.identity-grid{{display:grid;grid-template-columns:repeat(auto-fit,minmax(280px,1fr));gap:1rem}}.identity-grid dl{{border:1px solid var(--line);padding:1rem;margin:0}}
.table-wrap{{overflow-x:auto;border:1px solid var(--line);border-radius:6px}}table{{width:100%;border-collapse:collapse;min-width:760px}}caption{{text-align:left;font-weight:700;padding:.75rem;background:var(--panel)}}th,td{{padding:.55rem .65rem;text-align:left;vertical-align:top;border-top:1px solid var(--line)}}thead th{{border-top:0;background:var(--panel)}}tbody th{{white-space:nowrap}}small{{display:block;color:var(--muted)}}
.bar{{display:inline-block;width:95px;height:.55rem;margin-right:.45rem;vertical-align:middle;background:#e1e5e9;border-radius:999px;overflow:hidden}}.bar span{{display:block;height:100%;background:var(--accent)}}.ratio{{white-space:nowrap}}details{{border:1px solid var(--line);border-radius:5px;margin:.55rem 0;padding:.65rem}}summary{{cursor:pointer;font-weight:700}}details ol{{margin-bottom:0}}.redaction{{display:block;color:var(--muted);font-size:.72rem;overflow-wrap:anywhere}}
footer{{margin-top:2.5rem;border-top:1px solid var(--line);padding-top:1rem;color:var(--muted)}}
@media (max-width:600px){{main{{padding-top:1rem}}.identity-grid{{grid-template-columns:1fr}}.card{{padding:.8rem}}}}
</style>
</head>
<body><main>
<header class="hero"><p class="eyebrow">Sealed, non-authoritative human view</p><div class="overall"><h1>Gym evidence report</h1>{_status_badge(overall)}</div>
<p>JSON envelopes remain the machine-authoritative evidence. This deterministic HTML only visualizes a verified seal and does not rerun or rescore the benchmark.</p></header>
{_authority_notice(bundle)}
<section aria-labelledby="roles"><h2 id="roles">Decision axes</h2><div class="cards">{_role_cards(bundle)}</div></section>
<section aria-labelledby="score-errors"><h2 id="score-errors">Score-error accounting</h2><div class="metrics">{_metric('reported', _number(score.get('reportedCount')))}{_metric('intended rejection', _number(score.get('intendedCount')))}{_metric('unexplained', _number(score.get('unexplainedCount')))}</div></section>
<section aria-labelledby="packs"><h2 id="packs">Pack distribution</h2>{_pack_table(bundle)}</section>
<section aria-labelledby="diagnostics"><h2 id="diagnostics">Failure, incomplete, and N/A details</h2>{_diagnostics(bundle)}</section>
<section aria-labelledby="identity"><h2 id="identity">Execution identity</h2>{_identity_section(bundle, manifest)}</section>
<section aria-labelledby="provenance"><h2 id="provenance">Input provenance</h2><p>Generator: <code>{_redact(generator.get('name'))} {_redact(generator.get('version'))}</code> · manifest schema <code>{_redact(manifest.get('schemaVersion'))}</code></p>{_provenance_table(manifest)}</section>
<footer>Generated solely from a verified <code>evidence-manifest.json</code>. No external resources or executable JavaScript are used.</footer>
</main></body></html>
"""
    return document.encode("utf-8")
