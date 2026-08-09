"""API 1층 — 무상태 명령 래퍼.

각 함수는 CLI `--json` 봉투를 :class:`~rhwp.models.Envelope` 로 돌려준다.
호출 한 번 = 프로세스 한 번 = 문서 재파싱 한 번이다. 같은 문서를 반복해서
만질 거라면 :mod:`rhwp.session` 의 2층을 쓰는 편이 빠르다.

판정 규약: exit 3/4 는 예외가 아니라 봉투의 판정 필드다. 예외를 원하면
``raise_on_verdict=True`` 를 넘긴다.
"""

from __future__ import annotations

from pathlib import Path
from typing import Any, Dict, List, Mapping, Optional, Sequence, Union

from ._process import DEFAULT_TIMEOUT, run_json, run_ndjson
from .errors import EXIT_USAGE, UsageError
from .models import Envelope

__all__ = [
    "info",
    "export_text",
    "export_structure",
    "export_tables",
    "table_to_csv",
    "export_svg",
    "export_pdf",
    "export_markdown",
    "export_hml",
    "export_doclang",
    "export_hwpx",
    "convert",
    "search",
    "fields",
    "digest",
    "extract_data",
    "inspect",
    "explain",
    "export_provenance_map",
    "export_plan_schema",
    "export_agent_manifest",
    "ir_diff",
    "render_diff",
    "thumbnail",
    "extract_pages",
    "build_from_ingest",
    "fill_fields",
    "replace_text",
    "set_cell",
    "csv_to_table",
    "scan",
    "batch",
    "capabilities",
]

PathLike = Union[str, Path]


def _flag(args: List[Any], name: str, value: Optional[Any]) -> None:
    """값이 ``None`` 이 아니면 ``--name value`` 를 붙인다."""
    if value is not None:
        args.extend([name, value])


def _switch(args: List[Any], name: str, enabled: bool) -> None:
    """참이면 플래그만 붙인다."""
    if enabled:
        args.append(name)


# ── 조회 ────────────────────────────────────────────────────────────────


def info(path: PathLike, *, timeout: Optional[float] = DEFAULT_TIMEOUT) -> Envelope:
    """문서 요약 — 포맷·쪽수·구역수·문단수·글꼴."""
    return Envelope(run_json(["info", path, "--json"], timeout=timeout))


def export_text(
    path: PathLike,
    *,
    page: Optional[int] = None,
    max_chars: Optional[int] = None,
    timeout: Optional[float] = DEFAULT_TIMEOUT,
) -> Envelope:
    """쪽별 평문 추출.

    [트랙 G R61 D-12] ``page``/``max_chars`` 는 CLI 에 있었지만 이 래퍼에는
    없었다(Node 바인딩엔 있었음). ``page`` 로 특정 쪽만, ``max_chars`` 로
    쪽당 상한을 준다(기본은 무제한 — #3787 S7).
    """
    args: List[Any] = ["export-text", path]
    _flag(args, "-p", page)
    _flag(args, "--max-chars", max_chars)
    args.append("--json")
    return Envelope(run_json(args, timeout=timeout))


def export_structure(
    path: PathLike,
    *,
    mode: Optional[str] = None,
    timeout: Optional[float] = DEFAULT_TIMEOUT,
) -> Envelope:
    """문서 구조(제목 계층·절).

    [트랙 G R61 D-12] ``mode``(``auto``/``outline``/``clause``)가 CLI 에 있었지만
    이 래퍼에는 없었다.
    """
    args: List[Any] = ["export-structure", path]
    _flag(args, "--mode", mode)
    args.append("--json")
    return Envelope(run_json(args, timeout=timeout))


def export_tables(path: PathLike, *, timeout: Optional[float] = DEFAULT_TIMEOUT) -> Envelope:
    """표 전량을 셀 좌표와 함께."""
    return Envelope(run_json(["export-tables", path, "--json"], timeout=timeout))


def table_to_csv(
    path: PathLike,
    *,
    table: Optional[int] = None,
    out: Optional[PathLike] = None,
    bom: bool = False,
    timeout: Optional[float] = DEFAULT_TIMEOUT,
) -> Envelope:
    """본문 최상위 표를 RFC 4180 CSV로 내보낸다."""
    args: List[Any] = ["table-to-csv", path]
    _flag(args, "--table", table)
    _flag(args, "-o", out)
    _switch(args, "--bom", bom)
    args.append("--json")
    return Envelope(run_json(args, timeout=timeout))


def fields(path: PathLike, *, timeout: Optional[float] = DEFAULT_TIMEOUT) -> Envelope:
    """누름틀(필드) 목록 — 이름·순번·현재값."""
    return Envelope(run_json(["fields", path, "--json"], timeout=timeout))


def search(
    path: PathLike,
    query: str,
    *,
    case_sensitive: bool = True,
    limit: Optional[int] = None,
    timeout: Optional[float] = DEFAULT_TIMEOUT,
) -> Envelope:
    """주소가 붙은 검색 — 매치마다 (구역·문단·**쪽**·문자 오프셋).

    Args:
        query: 검색어. ``-`` 로 시작하는 어휘도 그대로 넘길 수 있다
            (내부에서 ``--`` 구분자를 쓴다).
    """
    args: List[Any] = ["search", path]
    _flag(args, "--limit", limit)
    if not case_sensitive:
        args.append("--ignore-case")
    args.append("--json")
    # 검색어가 옵션처럼 보여도 값으로 읽히도록 마지막에 구분자와 함께.
    args.extend(["--", query])
    return Envelope(run_json(args, timeout=timeout))


def digest(
    path: PathLike,
    *,
    sections: bool = False,
    pages: Optional[str] = None,
    max_chars: Optional[int] = None,
    timeout: Optional[float] = DEFAULT_TIMEOUT,
) -> Envelope:
    """요약용 청킹 — 주소를 보존한 절 단위 또는 쪽 범위 창.

    [트랙 G R61 D-12] ``max_chars``가 CLI 에 있었지만 이 래퍼에는 없었다.
    """
    args: List[Any] = ["digest", path]
    _switch(args, "--sections", sections)
    _flag(args, "--pages", pages)
    _flag(args, "--max-chars", max_chars)
    args.append("--json")
    return Envelope(run_json(args, timeout=timeout))


def extract_data(
    path: PathLike,
    *,
    kind: str = "all",
    limit: Optional[int] = None,
    timeout: Optional[float] = DEFAULT_TIMEOUT,
) -> Envelope:
    """날짜·금액·수량을 문서 주소와 함께 추출한다."""
    args: List[Any] = ["extract-data", path]
    _flag(args, "--kind", kind)
    _flag(args, "--limit", limit)
    args.append("--json")
    return Envelope(run_json(args, timeout=timeout))


def capabilities(*, mcp: bool = False, timeout: Optional[float] = DEFAULT_TIMEOUT) -> Envelope:
    """도구 자기서술 — 명령 목록·플래그·봉투 필드·종료 코드 사전.

    이 봉투가 바인딩의 단일 출처다. 명령이 늘었는지, 어떤 필드가 나오는지를
    여기서 읽으면 수기 목록을 둘 필요가 없다.
    """
    args: List[Any] = ["capabilities"]
    _switch(args, "--mcp", mcp)
    return Envelope(run_json(args, timeout=timeout))


def inspect(
    path: PathLike,
    subcommand: str,
    *,
    threshold_pt: Optional[float] = None,
    include_offpage: bool = False,
    min_confidence: Optional[str] = None,
    include_fields: bool = False,
    kind: Optional[str] = None,
    timeout: Optional[float] = DEFAULT_TIMEOUT,
) -> Envelope:
    """읽기 전용 문서 보안 검사.

    subcommand는 "hidden-text", "injection", "unicode" 중 하나다.
    각 검사에만 유효한 옵션을 다른 검사에 넘기면 조용히 무시하지 않고 거부한다.
    """
    args: List[Any] = ["inspect", subcommand, path]
    if subcommand == "hidden-text":
        if min_confidence is not None or include_fields or kind is not None:
            raise ValueError("hidden-text 검사에는 threshold_pt와 include_offpage만 사용할 수 있습니다")
        _flag(args, "--threshold-pt", threshold_pt)
        _switch(args, "--include-offpage", include_offpage)
    elif subcommand == "injection":
        if threshold_pt is not None or include_offpage or kind is not None:
            raise ValueError("injection 검사에는 min_confidence와 include_fields만 사용할 수 있습니다")
        _flag(args, "--min-confidence", min_confidence)
        _switch(args, "--include-fields", include_fields)
    elif subcommand == "unicode":
        if (
            threshold_pt is not None
            or include_offpage
            or min_confidence is not None
            or include_fields
        ):
            raise ValueError("unicode 검사에는 kind만 사용할 수 있습니다")
        _flag(args, "--kind", kind)
    else:
        raise ValueError("subcommand는 hidden-text, injection, unicode 중 하나여야 합니다")
    args.append("--json")
    return Envelope(run_json(args, timeout=timeout))


def export_provenance_map(
    *, timeout: Optional[float] = DEFAULT_TIMEOUT
) -> Envelope:
    """봉투 필드의 문서 출처·신뢰 표지를 내보낸다."""
    return Envelope(run_json(["export-provenance-map", "--json"], timeout=timeout))


def explain(path: PathLike, *, timeout: Optional[float] = DEFAULT_TIMEOUT) -> Envelope:
    """문서의 형식·쪽수·표·누름틀·각주를 한 번에 요약한다."""
    return Envelope(run_json(["explain", path, "--json"], timeout=timeout))


def export_plan_schema(
    *,
    bare: bool = False,
    out: Optional[PathLike] = None,
    timeout: Optional[float] = DEFAULT_TIMEOUT,
) -> Envelope:
    """``run`` 계획서 문법의 JSON Schema."""
    args: List[Any] = ["export-plan-schema"]
    _switch(args, "--bare", bare)
    _flag(args, "-o", out)
    args.append("--json")
    return Envelope(run_json(args, timeout=timeout))


def export_agent_manifest(
    *, bare: bool = False, timeout: Optional[float] = DEFAULT_TIMEOUT
) -> Envelope:
    """capabilities·IR·provenance·plan schema 를 한 봉투로 조립한 에이전트 매니페스트."""
    args: List[Any] = ["export-agent-manifest"]
    _switch(args, "--bare", bare)
    args.append("--json")
    return Envelope(run_json(args, timeout=timeout))


def export_ontology(
    *,
    bare: bool = False,
    out: Optional[PathLike] = None,
    timeout: Optional[float] = DEFAULT_TIMEOUT,
) -> Envelope:
    """자기서술에서 기계 유도한 JSON-LD 온톨로지를 내보낸다."""
    args: List[Any] = ["export-ontology"]
    _switch(args, "--bare", bare)
    _flag(args, "-o", out)
    args.append("--json")
    return Envelope(run_json(args, timeout=timeout))


# ── 산출 ────────────────────────────────────────────────────────────────


def export_svg(
    path: PathLike,
    *,
    out: Optional[PathLike] = None,
    page: Optional[int] = None,
    timeout: Optional[float] = DEFAULT_TIMEOUT,
) -> Envelope:
    """SVG 렌더."""
    args: List[Any] = ["export-svg", path]
    _flag(args, "-o", out)
    _flag(args, "-p", page)
    args.append("--json")
    return Envelope(run_json(args, timeout=timeout))


def export_pdf(
    path: PathLike,
    *,
    out: Optional[PathLike] = None,
    timeout: Optional[float] = DEFAULT_TIMEOUT,
) -> Envelope:
    """PDF 산출."""
    args: List[Any] = ["export-pdf", path]
    _flag(args, "-o", out)
    args.append("--json")
    return Envelope(run_json(args, timeout=timeout))


def export_markdown(
    path: PathLike,
    *,
    out: Optional[PathLike] = None,
    timeout: Optional[float] = DEFAULT_TIMEOUT,
) -> Envelope:
    """Markdown 산출."""
    args: List[Any] = ["export-markdown", path]
    _flag(args, "-o", out)
    args.append("--json")
    return Envelope(run_json(args, timeout=timeout))


def export_hml(
    path: PathLike,
    *,
    out: Optional[PathLike] = None,
    timeout: Optional[float] = DEFAULT_TIMEOUT,
) -> Envelope:
    """HML 재직렬화."""
    args: List[Any] = ["export-hml", path]
    _flag(args, "-o", out)
    args.append("--json")
    return Envelope(run_json(args, timeout=timeout))


def export_doclang(
    path: PathLike,
    *,
    out: Optional[PathLike] = None,
    timeout: Optional[float] = DEFAULT_TIMEOUT,
) -> Envelope:
    """DocLang XML 산출 — 소비 파이프라인용 매니페스트."""
    args: List[Any] = ["export-doclang", path]
    _flag(args, "-o", out)
    args.append("--json")
    return Envelope(run_json(args, timeout=timeout))


def thumbnail(
    path: PathLike,
    *,
    out: Optional[PathLike] = None,
    timeout: Optional[float] = DEFAULT_TIMEOUT,
) -> Envelope:
    """첫 쪽 미리보기 이미지."""
    args: List[Any] = ["thumbnail", path]
    _flag(args, "-o", out)
    args.append("--json")
    return Envelope(run_json(args, timeout=timeout))


def extract_pages(
    path: PathLike,
    pages: str,
    *,
    out: Optional[PathLike] = None,
    timeout: Optional[float] = DEFAULT_TIMEOUT,
) -> Envelope:
    """쪽 범위를 잘라 새 문서로."""
    args: List[Any] = ["extract-pages", path, "--pages", pages]
    _flag(args, "-o", out)
    args.append("--json")
    return Envelope(run_json(args, timeout=timeout))


def build_from_ingest(
    spec: PathLike,
    *,
    out: Optional[PathLike] = None,
    timeout: Optional[float] = DEFAULT_TIMEOUT,
) -> Envelope:
    """구조 명세(JSON)에서 새 문서를 생성."""
    args: List[Any] = ["build-from-ingest", spec]
    _flag(args, "-o", out)
    args.append("--json")
    return Envelope(run_json(args, timeout=timeout))


# ── 변환·검증 ───────────────────────────────────────────────────────────


def export_hwpx(
    path: PathLike,
    *,
    out: Optional[PathLike] = None,
    verify: bool = False,
    verify_pages: bool = False,
    raise_on_verdict: bool = False,
    timeout: Optional[float] = DEFAULT_TIMEOUT,
) -> Envelope:
    """HWP → HWPX 변환.

    ``out`` 은 CLI 의 **위치 인자**다(``export-hwpx <입력> [출력] ...`` — 플래그가
    아니다). 생략하면 CLI 가 자체 기본 경로를 쓴다.

    ``verify=True`` 면 봉투에 ``verify.identical`` 이 담긴다. 판정 실패(exit 3)는
    기본적으로 예외가 아니다 — 봉투를 읽어 판단하라.
    """
    args: List[Any] = ["export-hwpx", path]
    if out is not None:
        args.append(out)
    _switch(args, "--verify", verify)
    _switch(args, "--verify-pages", verify_pages)
    args.append("--json")
    return Envelope(run_json(args, timeout=timeout, raise_on_verdict=raise_on_verdict))


def convert(
    path: PathLike,
    *,
    out: Optional[PathLike] = None,
    verify: bool = False,
    raise_on_verdict: bool = False,
    timeout: Optional[float] = DEFAULT_TIMEOUT,
) -> Envelope:
    """HWPX → HWP 변환.

    ``out`` 은 CLI 의 **위치 인자**이고(``convert <입력> <출력> ...``), 여기서는
    **필수**다 — 기본 산출 경로가 없어서, 빠뜨리면 CLI 가 사용법 오류로 끝난다.
    프로세스를 띄우기 전에 여기서 같은 판정을 내려 무엇이 빠졌는지 이름으로
    알린다(Node 바인딩의 ``convert`` 와 같은 계약).

    Raises:
        UsageError: ``out`` 을 주지 않았을 때.
    """
    if out is None:
        raise UsageError(
            "convert 는 산출 경로가 필요합니다 — out 인자를 지정하세요",
            argv=["convert", str(path), "--json"],
            exit_code=EXIT_USAGE,
        )
    args: List[Any] = ["convert", path, out]
    _switch(args, "--verify", verify)
    args.append("--json")
    return Envelope(run_json(args, timeout=timeout, raise_on_verdict=raise_on_verdict))


def ir_diff(
    a: PathLike,
    b: PathLike,
    *,
    section: Optional[int] = None,
    paragraph: Optional[int] = None,
    timeout: Optional[float] = DEFAULT_TIMEOUT,
) -> Envelope:
    """두 문서의 IR 차이 — 무엇이 달라졌는지 범주별로.

    [트랙 G R61 D-12] ``section``/``paragraph``(``-s``/``-p`` — 특정 구역·
    문단으로 좁혀서 비교)가 CLI 에 있었지만 이 래퍼에는 없었다.
    """
    args: List[Any] = ["ir-diff", a, b]
    _flag(args, "-s", section)
    _flag(args, "-p", paragraph)
    args.append("--json")
    return Envelope(run_json(args, timeout=timeout))


def render_diff(
    path: PathLike,
    path_b: Optional[PathLike] = None,
    *,
    via: Optional[str] = None,
    page: Optional[int] = None,
    max_disp: Optional[float] = None,
    raise_on_verdict: bool = False,
    timeout: Optional[float] = DEFAULT_TIMEOUT,
) -> Envelope:
    """시각 회귀 판정 — 페이지별 렌더 결과를 자기 왕복(1건) 또는 전/후(2건) 비교한다.

    ``path_b`` 생략 시 ``path`` 를 자기 자신과 렌더 왕복 비교(회귀 도구 자체
    검증). 판정 실패(exit 3)는 기본적으로 예외가 아니다 — 봉투를 읽어 판단하라.
    """
    args: List[Any] = ["render-diff", path]
    if path_b is not None:
        args.append(path_b)
    _flag(args, "--via", via)
    _flag(args, "-p", page)
    _flag(args, "--max-disp", max_disp)
    args.append("--json")
    return Envelope(run_json(args, timeout=timeout, raise_on_verdict=raise_on_verdict))


# ── 편집 ────────────────────────────────────────────────────────────────


def fill_fields(
    path: PathLike,
    data: Mapping[str, Any],
    *,
    out: Optional[PathLike] = None,
    dry_run: bool = False,
    verify: bool = False,
    raise_on_verdict: bool = False,
    timeout: Optional[float] = DEFAULT_TIMEOUT,
) -> Envelope:
    """누름틀 채우기 (메일머지).

    Args:
        data: ``{"필드이름": "값"}``. 동명 필드는 ``"이름#1"`` 로 순번 지정.
    """
    import json as _json

    args: List[Any] = ["edit", "fill-fields", path, "--data", _json.dumps(data, ensure_ascii=False)]
    _flag(args, "-o", out)
    _switch(args, "--dry-run", dry_run)
    _switch(args, "--verify", verify)
    args.append("--json")
    return Envelope(run_json(args, timeout=timeout, raise_on_verdict=raise_on_verdict))


def replace_text(
    path: PathLike,
    find: str,
    replace: str,
    *,
    out: Optional[PathLike] = None,
    occurrence: Optional[int] = None,
    ignore_case: bool = False,
    dry_run: bool = False,
    verify: bool = False,
    raise_on_verdict: bool = False,
    timeout: Optional[float] = DEFAULT_TIMEOUT,
) -> Envelope:
    """문자열 치환. ``occurrence`` 를 주면 그 순번 하나만."""
    args: List[Any] = ["edit", "replace-text", path, "--find", find, "--replace", replace]
    _flag(args, "--occurrence", occurrence)
    _flag(args, "-o", out)
    _switch(args, "--ignore-case", ignore_case)
    _switch(args, "--dry-run", dry_run)
    _switch(args, "--verify", verify)
    args.append("--json")
    return Envelope(run_json(args, timeout=timeout, raise_on_verdict=raise_on_verdict))


def set_cell(
    path: PathLike,
    table: int,
    row: int,
    col: int,
    text: str,
    *,
    out: Optional[PathLike] = None,
    keep_style: bool = False,
    dry_run: bool = False,
    verify: bool = False,
    raise_on_verdict: bool = False,
    timeout: Optional[float] = DEFAULT_TIMEOUT,
) -> Envelope:
    """표 셀에 값 기록. 좌표는 :func:`export_tables` 로 확인한다."""
    args: List[Any] = [
        "edit", "set-cell", path,
        "--table", table, "--row", row, "--col", col, "--text", text,
    ]
    _flag(args, "-o", out)
    _switch(args, "--keep-style", keep_style)
    _switch(args, "--dry-run", dry_run)
    _switch(args, "--verify", verify)
    args.append("--json")
    return Envelope(run_json(args, timeout=timeout, raise_on_verdict=raise_on_verdict))


def csv_to_table(
    path: PathLike,
    csv: PathLike,
    table: int,
    *,
    out: Optional[PathLike] = None,
    dry_run: bool = False,
    verify: bool = False,
    raise_on_verdict: bool = False,
    timeout: Optional[float] = DEFAULT_TIMEOUT,
) -> Envelope:
    """CSV 내용으로 기존 표의 셀을 덮어쓴다."""
    args: List[Any] = ["csv-to-table", path, "--csv", csv, "--table", table]
    _flag(args, "-o", out)
    _switch(args, "--dry-run", dry_run)
    _switch(args, "--verify", verify)
    args.append("--json")
    return Envelope(run_json(args, timeout=timeout, raise_on_verdict=raise_on_verdict))


# ── 대량 ────────────────────────────────────────────────────────────────


def scan(
    *paths: PathLike,
    probe: bool = False,
    max_depth: Optional[int] = None,
    limit: Optional[int] = None,
    timeout: Optional[float] = DEFAULT_TIMEOUT,
) -> Envelope:
    """디렉터리 재귀 발견·분류 — ``batch`` 의 앞 단계.

    ``batch`` 는 경로 목록을 이미 갖고 있다는 전제에서 시작한다. 이 명령이 그
    목록을 만든다: HWP 계열 파일을 찾아 확장자 주장과 매직 감지를 대조하고
    (``extMismatch``), ``probe=True`` 면 실제로 열어 파싱 가능/암호 필요를
    기록한다. 발견은 판정이 아니므로 게이트 종료 코드(3)가 없다.

    Args:
        paths: 검색할 폴더(재귀) 또는 파일 경로 — 최소 1개.
        probe: 각 파일을 실제로 열어 파싱 가능·암호 필요·쪽수를 기록.
        max_depth: 재귀 최대 깊이 (1 = 지정 폴더만).
        limit: 최대 파일 수 — 넘으면 봉투에 ``truncated: true``.
    """
    if not paths:
        raise ValueError("검색할 경로가 없습니다 — scan 은 최소 1개가 필요합니다")
    args: List[Any] = ["scan", *paths]
    _switch(args, "--probe", probe)
    _flag(args, "--max-depth", max_depth)
    _flag(args, "--limit", limit)
    args.append("--json")
    return Envelope(run_json(args, timeout=timeout))


def batch(
    subcommand: str,
    paths: Sequence[PathLike],
    *,
    extra_args: Sequence[Any] = (),
    timeout: Optional[float] = None,
) -> List[Dict[str, Any]]:
    """폴더/목록 일괄 처리 — NDJSON 레코드 목록을 돌려준다.

    부분 실패도 실패다. 실패한 항목은 ``error`` 필드를 단 레코드로 남으므로,
    스트림을 통째로 버리지 말고 레코드별로 판단하라.

    Args:
        subcommand: ``"export-text"`` 등 batch 축 이름.
        paths: 처리할 파일 경로 목록 (stdin 으로 흘려 넣는다).
        timeout: 기본값 ``None``(무제한) — 대량 작업은 오래 걸린다.
    """
    if not paths:
        raise ValueError("처리할 파일이 없습니다 — batch 는 최소 1개가 필요합니다")
    stdin = "\n".join(str(p) for p in paths) + "\n"
    args: List[Any] = ["batch", subcommand, *extra_args, "--json"]
    return run_ndjson(args, stdin=stdin, timeout=timeout)
