#!/usr/bin/env python3
"""pdf/ · pdf-2020/ · pdf-large/ 한컴 오라클 편입 + 다중버전 쪽수 불일치 목록.

M01-5 (#5345). 전부 신규 파일. scripts/visual_sweep.py 는 건드리지 않는다.

무엇을 하는가
-------------
세 오라클 트리를 순회해 stem(원본 문서 이름) × 한글 버전(2010/2018/2020/2022/2024)으로
묶고, 같은 stem 에 버전이 둘 이상이면 pypdf 로 쪽수를 잰다. 쪽수가 갈리면 불일치
목록에 넣고, 갈리지 않으면 일치로 남긴다. 픽셀 차이는 이 클레임 범위 밖이라
측정하지 않고 보고서에도 적지 않는다.

쪽수를 못 잰 파일(LFS 포인터, %PDF 아님, pypdf 실패)은 page_count=null 로 두고
이유를 적는다. 추측 쪽수를 넣지 않는다.

버전 토큰
---------
파일명 끝에서 `-2010`/`-2018`/`-2020`/`-2022`/`-2024` 와 변이
(`kopub`/`no-ttf`/`hwp`/`hwpx`/`hancom`)를 벗긴 나머지가 stem 이다.
`hwp3-sample16-hwp5-2018-2020.pdf` 처럼 연도가 두 개면 **마지막**만 오라클
버전이고 앞 연도는 stem 에 남긴다(변환에 쓴 한글 버전).
접미가 없으면 디렉터리 기본값만 쓴다: pdf/=2022, pdf-2020/=2020.
pdf-large/ 는 기본값이 없다.

종료 코드: 0 성공 / 2 사용법·경로 오류.
"""

from __future__ import annotations

import argparse
import json
import sys
from collections import defaultdict
from pathlib import Path
from typing import Any

SCHEMA_VERSION = "1.0"
CLAIM_ID = "M01-5"
METRIC = "pypdf_page_count"
PIXEL_DIFF_SCOPE = "out_of_scope"

HANGUL_YEARS = frozenset({"2010", "2018", "2020", "2022", "2024"})
VARIANT_TOKENS = frozenset({"kopub", "no-ttf", "hwp", "hwpx", "hancom"})
DEFAULT_TREES = ("pdf", "pdf-2020", "pdf-large")
DIR_DEFAULT_VERSION = {
    "pdf": "2022",
    "pdf-2020": "2020",
    "pdf-2010": "2010",
    "pdf-large": None,
}
LFS_PREFIX = b"version https://git-lfs.github.com"
REPORT_DIRNAME = "reports"


def repo_root_from_here() -> Path:
    return Path(__file__).resolve().parents[2]


def parse_oracle_name(stem: str) -> tuple[str, str | None, list[str]]:
    """파일 stem 에서 (문서stem, 한글버전|None, 변이목록)을 벗긴다."""
    tokens = stem.split("-")
    version: str | None = None
    variants: list[str] = []
    changed = True
    while tokens and changed:
        changed = False
        last = tokens[-1]
        if version is None and last in HANGUL_YEARS:
            version = last
            tokens.pop()
            changed = True
            continue
        if last.lower() in VARIANT_TOKENS:
            variants.append(last.lower())
            tokens.pop()
            changed = True
            continue
    variants.reverse()
    return "-".join(tokens), version, variants


def tree_of(rel_posix: str) -> str:
    return rel_posix.split("/", 1)[0]


def classify_bytes(head: bytes) -> str | None:
    """측정 불가면 이유 코드를, 측정 가능하면 None."""
    if head.startswith(LFS_PREFIX):
        return "lfs_pointer"
    if not head.startswith(b"%PDF"):
        return "not_pdf"
    return None


def measure_page_count(path: Path) -> tuple[int | None, str]:
    """쪽수 또는 (None, 이유). 추측하지 않는다."""
    try:
        head = path.read_bytes()[:80]
    except OSError as exc:
        return None, f"io_error:{type(exc).__name__}"
    blocked = classify_bytes(head)
    if blocked is not None:
        return None, blocked
    try:
        from pypdf import PdfReader
    except ImportError:
        return None, "pypdf_missing"
    try:
        import logging

        logging.getLogger("pypdf").setLevel(logging.ERROR)
        reader = PdfReader(str(path), strict=False)
        return len(reader.pages), "pypdf"
    except Exception as exc:  # noqa: BLE001 — 측정 실패는 이유만 남긴다
        return None, f"pypdf_error:{type(exc).__name__}"


def discover_pdfs(root: Path, trees: tuple[str, ...] | list[str]) -> list[Path]:
    found: list[Path] = []
    for tree in trees:
        base = root / tree
        if not base.is_dir():
            continue
        found.extend(p for p in base.rglob("*") if p.is_file() and p.suffix.lower() == ".pdf")
    found.sort(key=lambda p: p.as_posix().lower())
    return found


def describe_file(root: Path, path: Path, *, measure: bool) -> dict[str, Any]:
    rel = path.relative_to(root).as_posix()
    tree = tree_of(rel)
    stem, explicit, variants = parse_oracle_name(path.stem)
    inferred = False
    version = explicit
    version_source = "explicit" if explicit is not None else "unknown"
    if explicit is None:
        default = DIR_DEFAULT_VERSION.get(tree)
        if default is not None:
            version = default
            inferred = True
            version_source = "inferred_dir"
    rec: dict[str, Any] = {
        "path": rel,
        "tree": tree,
        "filename": path.name,
        "stem": stem,
        "hangul_version": version,
        "version_source": version_source,
        "version_inferred": inferred,
        "variants": variants,
        "size_bytes": path.stat().st_size,
        "page_count": None,
        "page_count_status": "skipped",
    }
    if measure:
        pages, status = measure_page_count(path)
        rec["page_count"] = pages
        rec["page_count_status"] = "measured" if pages is not None else status
    return rec


def _version_pages(files: list[dict[str, Any]]) -> dict[str, dict[str, Any]]:
    by_ver: dict[str, dict[str, Any]] = {}
    for rec in files:
        ver = rec.get("hangul_version")
        if not ver:
            continue
        slot = by_ver.setdefault(
            ver,
            {"hangul_version": ver, "files": [], "page_counts": [], "unmeasured": []},
        )
        slot["files"].append(rec["path"])
        if rec["page_count"] is None:
            slot["unmeasured"].append(
                {"path": rec["path"], "status": rec["page_count_status"]}
            )
        else:
            slot["page_counts"].append(rec["page_count"])
    return dict(sorted(by_ver.items()))


def classify_stem(files: list[dict[str, Any]]) -> dict[str, Any]:
    versions = sorted({r["hangul_version"] for r in files if r.get("hangul_version")})
    by_ver = _version_pages(files)
    measured_sets = {
        ver: sorted(set(slot["page_counts"]))
        for ver, slot in by_ver.items()
        if slot["page_counts"]
    }
    all_pages: set[int] = set()
    for pages in measured_sets.values():
        all_pages.update(pages)
    if len(versions) < 2:
        kind = "single_version"
    elif len(measured_sets) < 2:
        kind = "incomplete"
    elif len(all_pages) > 1:
        kind = "page_count_disagree"
    else:
        kind = "page_count_agree"
    return {
        "stem": files[0]["stem"] if files else "",
        "kind": kind,
        "hangul_versions": versions,
        "file_count": len(files),
        "trees": sorted({r["tree"] for r in files}),
        "by_version": by_ver,
        "measured_page_counts": measured_sets,
        "min_pages": min(all_pages) if all_pages else None,
        "max_pages": max(all_pages) if all_pages else None,
        "files": [r["path"] for r in files],
    }


def build_index(
    root: Path,
    trees: tuple[str, ...] | list[str] = DEFAULT_TREES,
    *,
    measure: bool = True,
) -> dict[str, Any]:
    root = root.resolve()
    files = [describe_file(root, p, measure=measure) for p in discover_pdfs(root, trees)]
    grouped: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for rec in files:
        grouped[rec["stem"]].append(rec)
    stems = [classify_stem(grouped[name]) for name in sorted(grouped)]
    multiver = [s for s in stems if len(s["hangul_versions"]) >= 2]
    disagreements = [s for s in multiver if s["kind"] == "page_count_disagree"]
    agrees = [s for s in multiver if s["kind"] == "page_count_agree"]
    incomplete = [s for s in multiver if s["kind"] == "incomplete"]

    tree_stats: dict[str, Any] = {}
    for tree in trees:
        subset = [r for r in files if r["tree"] == tree]
        tree_stats[tree] = {
            "present": (root / tree).is_dir(),
            "default_hangul_version": DIR_DEFAULT_VERSION.get(tree),
            "file_count": len(subset),
            "measured": sum(1 for r in subset if r["page_count"] is not None),
            "unmeasured": sum(1 for r in subset if r["page_count"] is None),
        }

    incorporation = {
        tree: [r for r in files if r["tree"] == tree]
        for tree in trees
        if tree != "pdf"
    }

    return {
        "schema_version": SCHEMA_VERSION,
        "claim": CLAIM_ID,
        "metric": METRIC,
        "pixel_diff": PIXEL_DIFF_SCOPE,
        "root": str(root),
        "trees": tree_stats,
        "counts": {
            "files": len(files),
            "stems": len(stems),
            "multiver_stems": len(multiver),
            "page_count_disagree": len(disagreements),
            "page_count_agree": len(agrees),
            "incomplete": len(incomplete),
            "measured_files": sum(1 for r in files if r["page_count"] is not None),
            "unmeasured_files": sum(1 for r in files if r["page_count"] is None),
        },
        "files": files,
        "stems": stems,
        "multiver_stems": multiver,
        "disagreements": disagreements,
        "incorporation": incorporation,
    }


def render_markdown(index: dict[str, Any]) -> str:
    counts = index["counts"]
    trees = index["trees"]
    lines: list[str] = [
        "# M01-5 한컴 오라클 편입 · 다중버전 쪽수 불일치",
        "",
        f"- 클레임: `{index['claim']}`",
        f"- 측정: `{index['metric']}` (픽셀 차이는 `{index['pixel_diff']}`)",
        f"- 파일 {counts['files']} / stem {counts['stems']} / "
        f"다중버전 stem {counts['multiver_stems']}",
        f"- 쪽수 불일치 {counts['page_count_disagree']} · "
        f"쪽수 일치 {counts['page_count_agree']} · "
        f"미완(쪽수 미측정) {counts['incomplete']}",
        f"- 측정 {counts['measured_files']} · 미측정 {counts['unmeasured_files']}",
        "",
        "## 트리 편입",
        "",
        "| 트리 | 존재 | 기본 한글 버전 | 파일 | 측정 | 미측정 |",
        "| --- | --- | --- | ---: | ---: | ---: |",
    ]
    for name, stat in trees.items():
        default = stat["default_hangul_version"] or "없음"
        present = "예" if stat["present"] else "아니오"
        lines.append(
            f"| `{name}/` | {present} | {default} | {stat['file_count']} | "
            f"{stat['measured']} | {stat['unmeasured']} |"
        )
    lines.extend(["", "## pdf-2020/ · pdf-large/ 편입 목록", ""])
    for tree, recs in index["incorporation"].items():
        lines.append(f"### `{tree}/` ({len(recs)}건)")
        lines.append("")
        if not recs:
            lines.append("파일 없음.")
            lines.append("")
            continue
        lines.append("| 경로 | stem | 한글 버전 | 출처 | 쪽수 | 상태 |")
        lines.append("| --- | --- | --- | --- | ---: | --- |")
        for rec in recs:
            ver = rec["hangul_version"] or "—"
            pages = rec["page_count"] if rec["page_count"] is not None else "—"
            lines.append(
                f"| `{rec['path']}` | `{rec['stem']}` | {ver} | "
                f"{rec['version_source']} | {pages} | {rec['page_count_status']} |"
            )
        lines.append("")

    lines.extend(["", "## 쪽수 불일치 (다중버전)", ""])
    if not index["disagreements"]:
        lines.append(
            "측정된 쪽수가 갈리는 다중버전 stem 은 없다. "
            "픽셀 차이는 측정하지 않았다."
        )
        lines.append("")
    else:
        lines.append("| stem | 버전별 쪽수(실측) | 최소 | 최대 | 파일 수 |")
        lines.append("| --- | --- | ---: | ---: | ---: |")
        for stem in index["disagreements"]:
            parts = []
            for ver, pages in stem["measured_page_counts"].items():
                shown = ",".join(str(p) for p in pages)
                parts.append(f"{ver}={shown}")
            lines.append(
                f"| `{stem['stem']}` | {'; '.join(parts)} | "
                f"{stem['min_pages']} | {stem['max_pages']} | {stem['file_count']} |"
            )
        lines.append("")
        lines.append("버전별 파일:")
        lines.append("")
        for stem in index["disagreements"]:
            lines.append(f"### `{stem['stem']}`")
            lines.append("")
            for ver, slot in stem["by_version"].items():
                pages = slot["page_counts"]
                page_txt = ",".join(str(p) for p in pages) if pages else "미측정"
                lines.append(f"- {ver} (쪽 {page_txt})")
                for path in slot["files"]:
                    lines.append(f"  - `{path}`")
            lines.append("")

    lines.extend(["", "## 다중버전 · 쪽수 일치", ""])
    agrees = [s for s in index["multiver_stems"] if s["kind"] == "page_count_agree"]
    if not agrees:
        lines.append("없음.")
        lines.append("")
    else:
        lines.append(
            "같은 stem 에 한글 버전이 둘 이상이고, **잰 쪽수는 같다**. "
            "시각(픽셀) 일치로 읽지 말 것."
        )
        lines.append("")
        lines.append("| stem | 버전 | 쪽수 | 파일 수 |")
        lines.append("| --- | --- | ---: | ---: |")
        for stem in agrees:
            vers = ",".join(stem["hangul_versions"])
            lines.append(
                f"| `{stem['stem']}` | {vers} | {stem['min_pages']} | {stem['file_count']} |"
            )
        lines.append("")

    incomplete = [s for s in index["multiver_stems"] if s["kind"] == "incomplete"]
    lines.extend(["", "## 다중버전 · 쪽수 미완", ""])
    if not incomplete:
        lines.append("없음.")
        lines.append("")
    else:
        lines.append("버전이 둘 이상이나 쪽수를 두 버전 이상에서 못 잰 stem.")
        lines.append("")
        for stem in incomplete:
            lines.append(f"- `{stem['stem']}` 버전 {', '.join(stem['hangul_versions'])}")
        lines.append("")

    lines.extend(
        [
            "## 정직 한계",
            "",
            "- 쪽수는 pypdf `len(reader.pages)` 만 사용한다.",
            "- 픽셀/렌더 차이는 측정하지 않았고, 불일치로 세지 않는다.",
            "- LFS 포인터·비PDF·pypdf 실패는 `page_count=null` + 상태 코드다.",
            "- `scripts/visual_sweep.py` 는 이 도구가 수정하지 않는다.",
            "",
        ]
    )
    return "\n".join(lines)


def _json_ready(index: dict[str, Any]) -> dict[str, Any]:
    """보고서용으로 큰 목록을 나눈다."""
    return index


def write_reports(index: dict[str, Any], out_dir: Path) -> list[Path]:
    out_dir.mkdir(parents=True, exist_ok=True)
    written: list[Path] = []

    manifest = {
        "schema_version": index["schema_version"],
        "claim": index["claim"],
        "metric": index["metric"],
        "pixel_diff": index["pixel_diff"],
        "trees": index["trees"],
        "counts": index["counts"],
        "incorporation": index["incorporation"],
    }
    p_manifest = out_dir / "incorporation_manifest.json"
    p_manifest.write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
        newline="\n",
    )
    written.append(p_manifest)

    disagreements = {
        "schema_version": index["schema_version"],
        "claim": index["claim"],
        "metric": index["metric"],
        "pixel_diff": index["pixel_diff"],
        "counts": {
            "multiver_stems": index["counts"]["multiver_stems"],
            "page_count_disagree": index["counts"]["page_count_disagree"],
            "page_count_agree": index["counts"]["page_count_agree"],
            "incomplete": index["counts"]["incomplete"],
        },
        "disagreements": index["disagreements"],
        "multiver_stems": index["multiver_stems"],
    }
    p_dis = out_dir / "multiver_disagreements.json"
    p_dis.write_text(
        json.dumps(disagreements, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
        newline="\n",
    )
    written.append(p_dis)

    p_md = out_dir / "multiver_index.md"
    p_md.write_text(render_markdown(index), encoding="utf-8", newline="\n")
    written.append(p_md)
    return written


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="pdf/pdf-2020/pdf-large 다중버전 한글 오라클 색인"
    )
    parser.add_argument(
        "--root",
        type=Path,
        default=None,
        help="저장소 루트 (기본: 이 파일 기준 두 단계 위)",
    )
    parser.add_argument(
        "--trees",
        default=",".join(DEFAULT_TREES),
        help="쉼표 구분 트리 (기본: pdf,pdf-2020,pdf-large)",
    )
    parser.add_argument(
        "--write-reports",
        action="store_true",
        help="tools/oracle_public/reports/ 에 JSON·MD 를 쓴다",
    )
    parser.add_argument(
        "--out-dir",
        type=Path,
        default=None,
        help="보고서 디렉터리 (--write-reports 기본: tools/oracle_public/reports)",
    )
    parser.add_argument(
        "--no-measure",
        action="store_true",
        help="쪽수를 재지 않고 파일·stem 만 묶는다",
    )
    parser.add_argument("--json", action="store_true", help="전체 색인을 stdout JSON")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    root = args.root.resolve() if args.root else repo_root_from_here()
    trees = tuple(t.strip() for t in args.trees.split(",") if t.strip())
    if not trees:
        print("trees 가 비었다", file=sys.stderr)
        return 2
    if not root.is_dir():
        print(f"root 가 없다: {root}", file=sys.stderr)
        return 2

    index = build_index(root, trees, measure=not args.no_measure)
    if args.write_reports:
        out_dir = args.out_dir or (Path(__file__).resolve().parent / REPORT_DIRNAME)
        written = write_reports(index, out_dir)
        for path in written:
            print(path.as_posix(), file=sys.stderr)
    if args.json:
        json.dump(_json_ready(index), sys.stdout, ensure_ascii=False, indent=2)
        sys.stdout.write("\n")
    else:
        sys.stdout.write(render_markdown(index))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
