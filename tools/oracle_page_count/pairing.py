"""정답지 PDF 파일명에서 원본 형식·한컴 엔진·폰트 조건을 읽는다.

자동 선택은 **형식이 샘플과 같고 한컴 연도가 확인된 PDF**만 고른다. 형식 미표기,
상대 형식, kopub/no-ttf 같은 과거 폰트 조건은 같은 허용 집합에 섞지 않는다.
`--args` 는 그 집합이 비거나 같은 연도에 두 장이 있으면 비교 인자를 내지 않고 실패한다.

#6374 — 편람 HWPX 의 2010-kopub(388)·미표기 2024(383) 를 2020/2024 정본(384)과
섞으면 rhwp 382쪽이 기존 격차로 통과한다.
"""
from __future__ import annotations

import os
import re

SUFFIX = re.compile(r'-(20\d\d|hwp|hwpx|kopub|no-ttf|current)+$', re.I)
ENGINE = re.compile(r'^20\d\d$')
FORMATS = frozenset({'hwp', 'hwpx'})
FONT_CONDITIONS = frozenset({'kopub', 'no-ttf'})
OTHER_CONDITIONS = frozenset({'current'})


def sample_format(sample: str) -> str:
    return 'hwpx' if sample.lower().endswith('.hwpx') else 'hwp'


def stem(path: str) -> str:
    name = re.sub(r'\.(pdf|hwp|hwpx)$', '', os.path.basename(path), flags=re.I)
    prev = None
    while prev != name:
        prev = name
        name = SUFFIX.sub('', name)
    return name


def subdir(path: str, root: str) -> str:
    d = os.path.dirname(path).replace(os.sep, '/')
    return d[len(root):].lstrip('/') if d.startswith(root) else d


def _peel_tags(name: str) -> list[str]:
    tags: list[str] = []
    work = name
    while True:
        match = SUFFIX.search(work)
        if not match:
            break
        tags.append(match.group(1).lower())
        work = work[:match.start()]
    tags.reverse()
    return tags


def parse_oracle(pdf_path: str) -> dict:
    """정답지 파일명의 형식·엔진 연도·폰트 조건. 확인되지 않은 칸은 비운다."""
    name = re.sub(r'\.pdf$', '', os.path.basename(pdf_path), flags=re.I)
    fmt = None
    if name.lower().endswith('.hwpx'):
        fmt = 'hwpx'
        name = name[:-5]
    elif name.lower().endswith('.hwp'):
        fmt = 'hwp'
        name = name[:-4]

    tags = _peel_tags(name)
    engines = [tag for tag in tags if ENGINE.match(tag)]
    formats = [tag for tag in tags if tag in FORMATS]
    fonts = [tag for tag in tags if tag in FONT_CONDITIONS]
    extras = [tag for tag in tags if tag in OTHER_CONDITIONS]
    if formats:
        fmt = formats[-1]
    return {
        'format': fmt,
        'engines': engines,
        'fonts': fonts,
        'extras': extras,
    }


def oracle_format(pdf_path: str) -> str | None:
    return parse_oracle(pdf_path)['format']


def newest_engine_year(pdf_path: str) -> str | None:
    engines = parse_oracle(pdf_path)['engines']
    return max(engines) if engines else None


def is_canonical_oracle(pdf_path: str, want_format: str) -> bool:
    """원본 형식과 한컴 엔진이 확인되고 과거 폰트 조건이 없는 정답지."""
    parsed = parse_oracle(pdf_path)
    return (
        parsed['format'] == want_format
        and bool(parsed['engines'])
        and not parsed['fonts']
        and not parsed['extras']
    )


def pick_canonical_oracles(sample: str, candidates: list[str]) -> list[str]:
    """같은 디렉터리를 우선하고, canonical 만 남긴다. 없으면 빈 목록(fail-closed)."""
    want = sample_format(sample)
    same_dir = [
        path for path in candidates
        if subdir(path, 'pdf') == subdir(sample, 'samples')
    ]
    pool = same_dir if same_dir else list(candidates)
    return sorted(path for path in pool if is_canonical_oracle(path, want))


def newest_engine_oracles(canonical: list[str]) -> list[str]:
    """확인된 엔진 중 가장 최근 연도의 정답지만. 연도가 섞인 허용 집합을 만들지 않는다."""
    years = [newest_engine_year(path) for path in canonical]
    years = [year for year in years if year]
    if not years:
        return []
    latest = max(years)
    return sorted(
        path for path in canonical if newest_engine_year(path) == latest
    )


def select_args_pdf(canonical: list[str]) -> tuple[str | None, str | None]:
    """비교에 쓸 canonical PDF 하나. 없거나 같은 연도에 여러 장이면 실패 사유."""
    newest = newest_engine_oracles(canonical)
    if not newest:
        return None, '형식·엔진이 확인된 canonical PDF가 없다'
    if len(newest) != 1:
        return None, (
            'canonical PDF가 %d장이라 모호하다: %s'
            % (len(newest), ', '.join(newest))
        )
    return newest[0], None
