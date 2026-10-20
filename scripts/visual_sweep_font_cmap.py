"""같은 실행에서 반복 임베딩된 전체 글꼴의 cmap 검증 결과만 재사용한다."""

import base64
import hashlib
import io
import re
from collections.abc import Iterator
from pathlib import Path

# 원본 바이트는 저장하지 않는다. 새 바이트는 매번 해시를 계산해 별도로 검사한다.
_UNICODE_COUNTS: dict[str, int] = {}
_CACHE_LIMIT = 128
_RULE_RECORDS: dict[str, tuple[dict[str, object], ...]] = {}
_RULE_START = re.compile(r"@font-face\s*\{", re.IGNORECASE)


def inspect_svg_embedded_fonts(svg_path: Path) -> list[dict[str, object]]:
    """쪽 경로만 작업자에게 전달하고 기존 글꼴 보고서의 바이트 검사를 유지한다."""
    records = []
    for rule in embedded_font_rules(svg_path.read_text(encoding="utf-8")):
        records.extend(inspect_embedded_font_rule(rule))
    return records


def embedded_font_rules(svg: str) -> Iterator[str]:
    """기존 @font-face 정규식과 같은 평면 규칙을 큰 payload 순회 없이 찾는다.

    여는 규칙만 정규식으로 찾고 괄호는 문자열 검색으로 건너뛴다. 중첩 괄호가
    있으면 기존 [^{}]* 조건처럼 바깥 규칙을 버리고 내부 후보를 계속 찾는다.
    """
    cursor = 0
    while match := _RULE_START.search(svg, cursor):
        end = svg.find("}", match.end())
        if end < 0:
            return
        if svg.find("{", match.end(), end) < 0:
            yield svg[match.start():end + 1]
            cursor = end + 1
        else:
            cursor = match.end()


def embedded_font_unicode_count(data: bytes) -> tuple[str, int]:
    """실제 바이트의 SHA와 검증된 Unicode 매핑 수를 반환한다."""
    digest = hashlib.sha256(data).hexdigest()
    return digest, _unicode_count(data, digest)


def _unicode_count(data: bytes, digest: str) -> int:
    """내부에서 계산한 바이트 해시로 cmap 결과만 재사용한다."""
    from fontTools.ttLib import TTFont

    count = _UNICODE_COUNTS.get(digest)
    if count is None:
        with TTFont(io.BytesIO(data), fontNumber=0) as font:
            cmap = font.getBestCmap() if "cmap" in font else None
            if not cmap:
                raise ValueError("Unicode cmap이 없거나 비어 있음")
            count = len(cmap)
        if len(_UNICODE_COUNTS) >= _CACHE_LIMIT:
            _UNICODE_COUNTS.pop(next(iter(_UNICODE_COUNTS)))
        _UNICODE_COUNTS[digest] = count
    return count


def inspect_embedded_font_rule(rule: str) -> list[dict[str, object]]:
    """같은 CSS 바이트의 성공 결과만 재사용하며 원문/글꼴 바이트는 보관하지 않는다.

    family·URI 헤더·payload 전체를 매번 해시하므로 같은 이름의 변경된 글꼴도
    새로 검사한다. 보고서는 복사해 호출자가 캐시를 변경할 수 없게 한다.
    실패 결과는 재사용하지 않아 설치 환경이나 손상 데이터의 검사를 생략하지 않는다.
    """
    key = hashlib.sha256(rule.encode("utf-8")).hexdigest()
    cached = _RULE_RECORDS.get(key)
    if cached is not None:
        return [dict(record) for record in cached]
    family = re.search(r"font-family\s*:\s*([^;}]+)", rule, re.IGNORECASE)
    name = family.group(1).strip().strip("\"'") if family else "(unknown)"
    records: list[dict[str, object]] = []
    for data_url in re.findall(r"url\(\s*[\"']?(data:[^\"')]+)", rule, re.IGNORECASE):
        record: dict[str, object] = {"family": name}
        try:
            header, payload = data_url.split(",", 1)
            if not header.lower().endswith(";base64"):
                raise ValueError("base64 font data URI가 아님")
            data = base64.b64decode("".join(payload.split()), validate=True)
            digest = hashlib.sha256(data).hexdigest()
            # cmap 실패도 실제 디코딩된 글꼴의 해시를 기존 보고서대로 남긴다.
            record["sha256"] = digest
            record["unicode_mapping_count"] = _unicode_count(data, digest)
            record["status"] = "passed"
        except ImportError:
            record.update(status="failed", reason="임베딩 폰트 검사를 위해 fonttools 설치 필요")
        except Exception as error:
            record.update(status="failed", reason=str(error))
        records.append(record)
    if all(record["status"] == "passed" for record in records):
        if len(_RULE_RECORDS) >= _CACHE_LIMIT:
            _RULE_RECORDS.pop(next(iter(_RULE_RECORDS)))
        _RULE_RECORDS[key] = tuple(dict(record) for record in records)
    return records
