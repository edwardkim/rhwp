"""독립 PDF의 논리 쪽 대응을 전쪽 검증한 등록만 모아 찍기 제외에서 구분한다."""

import csv
import hashlib
import io
import math
from pathlib import Path


def verified_bytes(root, relative, expected, prefix):
    root = Path(root).resolve()
    name = Path(relative)
    path = (root / name).resolve()
    if (name.is_absolute() or '..' in name.parts or name.as_posix() != relative
            or not path.is_relative_to(root / prefix)):
        raise ValueError(f'논리 쪽 증거 경로 오류: {relative}')
    data = path.read_bytes()
    if hashlib.sha256(data).hexdigest() != expected:
        raise ValueError(f'논리 쪽 증거 바이트 불일치: {relative}')
    return data


def validate_full_page_tsv(data, pages):
    rows = list(csv.DictReader(io.StringIO(data.decode('utf-8')), delimiter='\t'))
    try:
        complete = [int(row['page']) for row in rows] == list(range(1, pages + 1))
        raw = [float(row['silhouette_raw_match_percent']) for row in rows]
    except (KeyError, TypeError, ValueError) as error:
        raise ValueError('논리 쪽 전쪽 TSV 형식 오류') from error
    if not complete or not all(math.isfinite(value) and 98 <= value <= 100 for value in raw):
        raise ValueError('논리 쪽 전쪽 TSV의 페이지 또는 98% 선행 조건 미달')


def validate_logical_page_comparison(root, entry):
    if 'logical_page_comparison' not in entry:
        return
    proof = entry['logical_page_comparison']
    if not isinstance(proof, dict) or set(proof) != {'native', 'wasm'}:
        raise ValueError('논리 쪽 비교에는 Native와 fresh WASM 전쪽 증거가 필요하다')
    # 원본/PDF의 경로·SHA·출처·쪽수는 호출자의 등록 검증에서 먼저 확인한다.
    for backend in ('native', 'wasm'):
        record = proof[backend]
        data = verified_bytes(root, record['tsv'], record['sha256'], 'mydocs/pr/assets')
        validate_full_page_tsv(data, entry['pages'])


def permits_logical_pages(entry):
    """load_verified_references가 바이트와 전쪽 증거를 검증한 등록만 전달한다."""
    return entry is not None and entry.get('logical_page_comparison') is not None
