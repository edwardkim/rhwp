"""명시적으로 호출하는 과거 페이지 품질 비교 도구다.

기록된 측정 조건에서 페이지별 raw 점수를 비교한다.
Visual Sweep의 기본 검토 판정에는 이 과거 PR 비교를 자동으로 연결하지 않는다.
글꼴 예외·보정값·평균값·일부 페이지 실행으로 품질 하락을 통과시키지 않는다.
"""
from __future__ import annotations

import csv
import hashlib
import json
from decimal import Decimal, InvalidOperation
from pathlib import Path

METRIC = "silhouette_raw_match_percent"
MAX_DROP_PP = Decimal("2")
MIN_RAW = Decimal("90")


def _sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _scores(rows):
    result = {}
    for row in rows:
        page = int(row["page"])
        value = Decimal(str(row[METRIC]))
        if page < 1 or page in result or not value.is_finite() or not 0 <= value <= 100:
            raise ValueError("invalid, duplicate, or non-finite raw page score")
        result[page] = value
    return result


def compare_pages(reference, candidate, pages):
    """Decimal로 2%p 이상 하락하는 경계값을 정확하게 판정한다."""
    if pages < 1:
        raise ValueError("reference page count must be positive")
    expected = set(range(1, pages + 1))
    if set(reference) != expected:
        raise ValueError("reference must contain every page exactly once")
    missing = sorted(expected - candidate.keys())
    extra = sorted(candidate.keys() - expected)
    regressions = [
        {"page": page, "reference": str(reference[page]), "candidate": str(candidate[page]),
         "drop_pp": str(reference[page] - candidate[page])}
        for page in sorted(expected & candidate.keys())
        if reference[page] - candidate[page] >= MAX_DROP_PP
    ]
    below = [page for page, score in candidate.items() if score < MIN_RAW]
    return {"status": "re_review_required" if missing or extra or regressions or below else "passed",
            "missing_pages": missing, "unexpected_pages": extra,
            "below_90_pages": sorted(below), "regressions": regressions}


def check_quality(root, provenance, metrics, dpi, radius_px, document_counts, method):
    """별칭이나 최저 점수 행 대신 입력·PDF 바이트로 비교 계약을 찾는다."""
    registry = root / "scripts/visual_sweep_quality_contracts.json"
    if not registry.exists():
        return {"status": "not_registered"}
    checks = []
    try:
        for contract in json.loads(registry.read_text(encoding="utf-8"))["contracts"]:
            # 각 계약은 명시적으로 등록된 출력 경로만 비교한다.
            # 특히 #8064 래칫의 비교 범위는 사용자 요청에 따라 Native로 한정한다.
            if provenance.get("raster_backend") not in contract["backends"]:
                continue
            summary = root / contract["summary"]
            if _sha(summary) != contract["summary_sha256"]:
                raise ValueError("accepted summary hash changed")
            with summary.open(encoding="utf-8", newline="") as stream:
                entries = list(csv.DictReader(stream, delimiter="\t"))
            if len(entries) != contract["conditions"] or sum(int(x["pages"]) for x in entries) != contract["pages"]:
                raise ValueError("accepted scope changed")
            protected_input = False
            matched = False
            for entry in entries:
                same_input = (entry["input_sha256"] == provenance.get("hwp", {}).get("sha256") or
                              entry["executed_input"] == provenance.get("hwp", {}).get("path"))
                protected_input |= same_input
                if (entry["input_sha256"] != provenance.get("hwp", {}).get("sha256") or
                        entry["pdf_sha256"] != provenance.get("pdf", {}).get("sha256")):
                    continue
                matched = True
                backend = provenance.get("raster_backend")
                side = contract["backends"].get(backend)
                actual = {"dpi": dpi, "radius_px": radius_px, "compat": provenance.get("layout_generation"),
                          "profile": provenance.get("comparison_profile"), "method": method,
                          "pdf_policy": provenance.get("pdf_rasterizer", {}).get("policy"),
                          "pdf_version": provenance.get("pdf_rasterizer", {}).get("version")}
                wanted = {**contract["measurement"], "compat": entry["compat"]}
                item = {"contract": contract["id"], "reference_source": contract["source_head"],
                        "condition": entry["key"], "backend": backend, "expected": wanted, "actual": actual}
                checks.append(item)
                if side is None or actual != wanted:
                    item.update(status="re_review_required", reason="measurement_conditions_differ")
                    continue
                ref = summary.parent / entry[f"{side}_tsv"]
                if _sha(ref) != entry[f"{side}_tsv_sha256"]:
                    raise ValueError("accepted page TSV hash changed")
                with ref.open(encoding="utf-8", newline="") as stream:
                    reference = _scores(csv.DictReader(stream, delimiter="\t"))
                pages = int(entry["pages"])
                item.update(compare_pages(reference, _scores(metrics), pages))
                item["reference_tsv"] = str(ref.relative_to(root))
                if document_counts != {"rhwp": pages, "pdf": pages}:
                    item.update(status="re_review_required", reason="document_page_counts_differ")
            if protected_input and not matched:
                checks.append({"contract": contract["id"], "status": "re_review_required",
                               "reason": "protected_input_or_reference_changed"})
        if not checks:
            return {"status": "not_registered"}
        return {"status": "re_review_required" if any(x["status"] != "passed" for x in checks) else "passed",
                "kind": "accepted_quality_floor", "metric": METRIC,
                "maximum_drop_pp_exclusive": str(MAX_DROP_PP), "minimum_raw": str(MIN_RAW),
                "checks": checks,
                "remaining_review": "same-font devel/head comparison, font provenance, content ownership and direct reading"}
    except (OSError, KeyError, TypeError, ValueError, InvalidOperation) as error:
        return {"status": "re_review_required", "reason": "invalid_quality_evidence", "error": str(error), "checks": checks}


def enforce_quality(root, manifest, metrics, dpi, counts, method):
    result = check_quality(root, manifest["provenance"], metrics, dpi, 2, counts, method)
    manifest["accepted_quality_regression"] = result
    if result["status"] == "re_review_required":
        manifest["pr_review_gate"] = {**manifest["pr_review_gate"],
                                      "status": "re_review_required", "quality_regression": result}
    return result
