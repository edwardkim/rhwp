"""별도 PR에서 재등록할 문서의 회귀 fixture 수집 경계."""
from pathlib import Path

_NAMES = frozenset(
    line for line in
    (Path(__file__).resolve().parents[1] / "tests/fixtures/deferred_regression_documents.txt")
    .read_text(encoding="utf-8").splitlines()
    if line and not line.startswith("#")
)


def is_deferred_regression_input(path: str | Path, axis: str | None = None) -> bool:
    name = str(path).replace("\\", "/").rsplit("/", 1)[-1]
    return (any(name.startswith(value[:-1]) if value.endswith("*") else name == value
                for value in _NAMES)
            or (axis is not None and is_deferred_visual_input(path, axis)))


def references_deferred_regression_input(text: str) -> bool:
    return any(name.removesuffix("*") in text for name in _NAMES)


_VISUAL_SCOPES = tuple(
    (columns[0], frozenset(columns[1].split(",")))
    for line in (Path(__file__).resolve().parents[1]
                 / "tests/fixtures/deferred_visual_regression_inputs.tsv")
    .read_text(encoding="utf-8").splitlines()
    if line and not line.startswith("#")
    for columns in [line.split("\t")]
)


def is_deferred_visual_input(path: str | Path, axis: str) -> bool:
    """원본은 유지하고 승인된 렌더링 축의 실제 수집만 제외한다."""
    normalized = str(path).replace("\\", "/")
    return any(axis in axes and (normalized == entry or normalized.endswith("/" + entry))
               for entry, axes in _VISUAL_SCOPES)
