#!/usr/bin/env python3
"""형식과 엔진이 파일명으로 확인되는 한컴 PDF 정답지 선택 규칙.

저장소의 과거 PDF는 같은 stem이라도 HWP/HWPX, 한컴 버전, 폰트 조건이 다를 수 있다.
자동 비교는 `pdf/<원본의 samples/ 하위 경로>/<stem>-<hwp|hwpx>-<2020|2024>.pdf`만
canonical으로 취급한다. 이 규칙으로 형식 미표기 PDF, 서로 다른 원본 형식의 결과, 또는 같은
이름의 다른 경로 원본을 조용히 재사용하지 않는다.
"""

import hashlib
import os
import re
import unicodedata


SUFFIX = re.compile(r'-(20\d\d|hwp|hwpx|kopub|no-ttf|current)+$', re.I)
CANONICAL = re.compile(r'-(hwp|hwpx)-(2020|2024)$', re.I)


def stem(path):
    """PDF/원본 파일명에서 형식·출력 조건 접미사를 제거한 NFC stem."""
    name = re.sub(r'\.(pdf|hwp|hwpx)$', '', os.path.basename(path), flags=re.I)
    prev = None
    while prev != name:
        prev = name
        name = SUFFIX.sub('', name)
    return unicodedata.normalize('NFC', name)


def source_stem(path):
    """원본의 식별자 접미사를 보존한 파일명 stem을 반환한다.

    `-2022` 같은 원본명 일부는 출력 조건이 아니라 문서를 식별한다. canonical PDF를
    만들 때 `stem()`을 쓰면 이런 서로 다른 원본이 같은 출력 경로를 공유하므로, 여기서는
    확장자만 제거한다.
    """
    name = re.sub(r'\.(hwp|hwpx)$', '', os.path.basename(path), flags=re.I)
    return unicodedata.normalize('NFC', name)


def source_format(path):
    fmt = os.path.splitext(path)[1].lstrip('.').lower()
    if fmt not in ('hwp', 'hwpx'):
        raise ValueError(f'지원하지 않는 원본 형식: {path}')
    return fmt


def engine_for_product(product):
    """`rhwp info --json`의 저장 제품에서 MCP 엔진을 고른다.

    확장자는 엔진 선택 근거가 아니다. 2024 저장본만 HOffice130(2024)을 쓰고, 메타데이터가
    없거나 이전 한컴 저장본은 HOffice120 호환 profile(2020)을 사용한다.
    """
    return '2024' if product == 'hancom-office-2024' else '2020'


def source_relative_path(path):
    """운영체제·호출 위치와 무관한 `samples/` 아래 상대 원본 경로를 만든다."""
    normalized = unicodedata.normalize('NFC', str(path).replace('\\', '/'))
    while normalized.startswith('./'):
        normalized = normalized[2:]
    if normalized.startswith('samples/'):
        return normalized[len('samples/'):]
    marker = '/samples/'
    if marker in normalized:
        return normalized.split(marker, 1)[1]
    raise ValueError(f'samples/ 아래의 원본 경로가 아니다: {path}')


def canonical_filename(sample, engine):
    """원본 형식과 선택 엔진이 드러나는 PDF 파일 이름을 만든다."""
    return '%s-%s-%s.pdf' % (source_stem(sample), source_format(sample), engine)


def canonical_pdf_path(sample, engine):
    """원본 하위 경로까지 보존한 충돌 없는 canonical PDF 상대 경로를 만든다."""
    relative = source_relative_path(sample)
    directory = os.path.dirname(relative)
    parts = ['pdf']
    if directory:
        parts.append(directory)
    parts.append(canonical_filename(sample, engine))
    return '/'.join(parts)


def path_token_canonical_filename(sample, engine):
    """경로 보존 방식으로 이관할 때만 인식하는 이전 path-token PDF 이름."""
    # 이전 이관 커밋은 repository-relative `samples/...` 전체를 hash 입력으로 사용했다.
    # 이 값은 최종 canonical 규칙이 아니라, 이미 만든 산출물을 재변환 없이 옮기기 위한 호환용이다.
    token = hashlib.sha256(('samples/' + source_relative_path(sample)).encode('utf-8')).hexdigest()[:16]
    return '%s-%s-%s-%s.pdf' % (stem(sample), source_format(sample), engine, token)


def canonical_engine(path):
    """canonical PDF의 `(원본 형식, 한컴 출력 엔진)` 또는 `None`을 돌려준다."""
    name = os.path.splitext(os.path.basename(path))[0]
    match = CANONICAL.search(name)
    if not match:
        return None
    return match.group(1).lower(), match.group(2)


def canonical_candidates(sample, candidates):
    """이 원본의 경로·형식에 정확히 대응하는 canonical PDF만 반환한다."""
    expected = {
        canonical_pdf_path(sample, '2020'),
        canonical_pdf_path(sample, '2024'),
    }
    return sorted(
        path for path in candidates
        if path.replace(os.sep, '/') in expected and canonical_engine(path)
    )


def choose_canonical(sample, candidates, engine=None):
    """하나의 검증 가능한 reference PDF를 고른다.

    2020과 2024가 함께 존재하면 사용자가 엔진을 지정해야 한다. 여러 형식·폰트 조건 중
    사전순 첫 파일을 고르는 fall-through는 허용하지 않는다.
    """
    choices = canonical_candidates(sample, candidates)
    if not choices:
        raise ValueError(f'형식이 확인된 canonical PDF가 없다: {sample}')
    if engine is not None:
        choices = [p for p in choices if canonical_engine(p)[1] == str(engine)]
        if len(choices) != 1:
            raise ValueError(f'{sample}: 엔진 {engine}의 canonical PDF가 하나가 아니다')
        return choices[0]
    if len(choices) != 1:
        engines = ', '.join(sorted({canonical_engine(p)[1] for p in choices}))
        raise ValueError(f'{sample}: canonical PDF가 여러 개다 ({engines}); --engine을 지정해야 한다')
    return choices[0]
