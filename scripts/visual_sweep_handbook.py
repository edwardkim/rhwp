"""정본 보호 TSV의 편람 전쪽 품질 하한과 검증용 웹폰트 공급."""
from __future__ import annotations

import csv
import hashlib
import json
import os
import subprocess
import tempfile
import urllib.request
from decimal import Decimal, InvalidOperation
from functools import lru_cache
from pathlib import Path

DOC = Path('samples/2025 행정업무운영 편람(최종).hwpx')
PDF = Path('pdf/2025 행정업무운영 편람(최종)-hwpx-kopub-2024.pdf')
BASELINE = Path('mydocs/pr/assets/visual-protected-baseline/final.tsv')
BASELINE_MANIFEST = BASELINE.with_name('manifest.json')
DOC_SHA = 'c6dd7e847a99f219681afc5a29c80a9665c04df9cda4d820a3350d739664fdf6'
PDF_SHA = '299cb2d994bba8b63f78b681bd74580286e298f25689b4ff4f40c66c54bfa85e'
BASELINE_SHA = 'ee41455cf7f05585487d92961ccb9fd8160d5d1e15c3567019ea89e2b7bf2aeb'
PAGES = 383
METRIC = 'silhouette_raw_match_percent'
POLICY = 'pdfium_smooth_rgb_exact_dpi_v2'
METHOD = 'threshold_boundary_color_support_v1'

# KoPub TTF는 설치본과 SHA가 같은 공개 배포본을 고정해 사용한다. 글꼴은
# 저장소가 아닌 output에만 다운로드하며 원본 바이트와 굵기를 바꾸지 않는다.
WEBFONTS = {
    ('KoPubBatang', 'Light'): ('KoPubBatang-Light.ttf', 'b7438d61595fe9584706ab2cb3c9aa2acdc486d61c8299374a30b3e77bc2045f'),
    ('KoPubBatang', 'Medium'): ('KoPubBatang-Medium.ttf', '7a798592beb0d35bb557b010c2794cbcc988e89fcbcee0eb5edbb35161141ab4'),
    ('KoPubBatang', 'Bold'): ('KoPubBatang-Bold.ttf', '94de26a11aa25619526ac8183fde4f46e74e22c527acb42aea2e93f5313bbc5b'),
    ('KoPubDotum', 'Light'): ('KoPubDotum-Light.ttf', '018a0f18613a98d446696ea3d36194e288d5d8004aa58327789c830f6179e23d'),
    ('KoPubDotum', 'Medium'): ('KoPubDotum-Medium.ttf', 'd0adfdcb40b7e73389128cc0222b53a91e5aa0e653f89967485a1f8b46124eb7'),
    ('KoPubDotum', 'Bold'): ('KoPubDotum-Bold.ttf', '967efe18063dd971c63dde3e8fcb247e0fcf16486a24ba2e284a716e44b90275'),
    ('HCR Batang', 'Regular'): ('HANBatang.woff', 'b3192018e3bd4da76b29b0530f71c79fb6f465a2b7cd41308bc38f9a4ae09a6c'),
}


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def readable_font_file(path):
    """설치 캐시의 존재 정보와 실제 프로그램 읽기 가능 여부를 구분한다."""
    try:
        with Path(path).open('rb') as stream:
            return bool(stream.read(1))
    except OSError:
        return False


def _normal_name(value):
    return ''.join(value.lower().split())


@lru_cache(maxsize=8)
def _file_font_inventory(paths):
    """Fontconfig이 없거나 명시 경로에만 있는 시스템 글꼴도 확인한다."""
    from fontTools.ttLib import TTCollection, TTFont, TTLibError
    inventory = set()
    for value in paths:
        root = Path(value).expanduser()
        # Native의 명시 font-path는 디렉터리의 직계 파일만 읽는다.
        try:
            files = [root] if root.is_file() else list(root.iterdir()) if root.is_dir() else []
        except OSError:
            # 격리된 환경의 접근 불가 디렉터리는 설치 근거가 아니다.
            continue
        for path in files:
            if not path.is_file() or path.suffix.lower() not in {'.ttf', '.otf', '.ttc'}:
                continue
            try:
                collection = TTCollection(path, lazy=True) if path.suffix.lower() == '.ttc' else None
                fonts = collection.fonts if collection else [TTFont(path, lazy=True)]
                try:
                    for font in fonts:
                        weight = font['OS/2'].usWeightClass if 'OS/2' in font else 400
                        for record in font['name'].names:
                            if record.nameID in (1, 4, 6, 16):
                                try:
                                    inventory.add((_normal_name(record.toUnicode()), weight))
                                except (UnicodeError, ValueError):
                                    continue
                finally:
                    for font in fonts:
                        font.close()
            except (OSError, ValueError, KeyError, IndexError, TTLibError):
                continue
    return inventory


def installed_face(family, style, font_paths=()):
    """Fontconfig이 다른 패밀리로 대체한 결과는 설치된 것으로 간주하지 않는다."""
    try:
        p = subprocess.run(['fc-match', '-f', '%{family}|%{style}|%{file}',
                            f'{family}:style={style}'], capture_output=True, text=True,
                           check=True, timeout=15)
        names, styles, path = p.stdout.strip().split('|', 2)
        if (_normal_name(family) in {_normal_name(x) for x in names.split(',')}
                and _normal_name(style) in {_normal_name(x) for x in styles.split(',')}
                and readable_font_file(path)):
            return True
    except (OSError, ValueError, subprocess.SubprocessError):
        pass
    home = Path.home()
    roots = [*font_paths, home / 'Library/Fonts', Path('/Library/Fonts'),
             home / '.local/share/fonts', home / '.fonts', Path('/usr/share/fonts'),
             Path('/usr/local/share/fonts')]
    if os.environ.get('WINDIR'):
        roots.append(Path(os.environ['WINDIR']) / 'Fonts')
    if os.environ.get('LOCALAPPDATA'):
        roots.append(Path(os.environ['LOCALAPPDATA']) / 'Microsoft/Windows/Fonts')
    ranges = {'Light': (1, 350), 'Medium': (351, 599), 'Bold': (600, 1000),
              'Regular': (351, 500)}
    low, high = ranges[style]
    wanted = _normal_name(family)
    return any((name == wanted or name == wanted + _normal_name(style)) and low <= weight <= high
               for name, weight in _file_font_inventory(tuple(str(p) for p in roots)))


def supply_webfonts(root, cache, has_face=None, *, font_paths=()):
    """미설치 face만 고정 웹폰트로 보완하고 실제 공급본의 해시를 반환한다."""
    cache = Path(cache)
    cache.mkdir(parents=True, exist_ok=True)
    has_face = has_face or (lambda family, style: installed_face(family, style, font_paths))
    records = []
    for (family, style), (name, expected) in WEBFONTS.items():
        if has_face(family, style):
            continue
        if name == 'HANBatang.woff':
            url = 'https://cdn.jsdelivr.net/gh/projectnoonnu/noonfonts_2104@1.0/HANBatang.woff'
        elif family in {'KoPubBatang', 'KoPubDotum'}:
            revision = ('7ef37ed4015cf0f638952e3d9937e4192f7cea97' if family == 'KoPubBatang'
                        else '295e0fc83931fda749f129b6690c0fa93f798007')
            url = f'https://cdn.jsdelivr.net/gh/fonts-archive/{family}@{revision}/{name}'
        else:
            url = 'https://cdn.jsdelivr.net/npm/font-kopub@1.0.2/fonts/' + name
        # Native fontdb는 파일 경로가 아닌 디렉터리를 스캔한다. face별 폴더를
        # 사용해 이미 설치된 다른 웹폰트가 우연히 우선 선택되는 것을 막는다.
        face_cache = cache / expected
        face_cache.mkdir(parents=True, exist_ok=True)
        source = face_cache / name
        if not source.is_file() or sha(source) != expected:
            with urllib.request.urlopen(url, timeout=60) as response:
                data = response.read(32 * 1024 * 1024 + 1)
            if len(data) > 32 * 1024 * 1024 or hashlib.sha256(data).hexdigest() != expected:
                raise ValueError(f'웹폰트 원본 해시가 다릅니다: {name}')
            with tempfile.NamedTemporaryFile(dir=face_cache, delete=False) as out:
                temp = Path(out.name)
                out.write(data)
            try:
                os.replace(temp, source)
            finally:
                temp.unlink(missing_ok=True)
        if name.endswith('.ttf'):
            # TTF 원본을 변환하지 않고 해시가 일치하는 파일 그대로 사용한다.
            records.append({'family': family, 'style': style, 'url': url,
                            'source_sha256': expected, 'native_path': str(source),
                            'native_sha256': expected})
            continue
        from fontTools.ttLib import TTFont
        target = face_cache / (name + '.ttf')
        # 매번 원본에서 변환해 기존 TTF 캐시 변조를 통과시키지 않는다.
        # 원본을 재검증해 변환하되 실행 시각으로 프로그램 해시를 바꾸지 않는다.
        font = TTFont(source, recalcTimestamp=False)
        try:
            font.flavor = None
            with tempfile.NamedTemporaryFile(dir=face_cache, suffix='.ttf', delete=False) as out:
                temp = Path(out.name)
            try:
                font.save(temp, reorderTables=False)
                os.replace(temp, target)
            finally:
                temp.unlink(missing_ok=True)
        finally:
            font.close()
        records.append({'family': family, 'style': style, 'url': url,
                        'source_sha256': expected, 'native_path': str(target),
                        'native_sha256': sha(target)})
    return records


def baseline_rows(root, backend='native-skia'):
    """정본에서 동일 입력·PDF·backend의 편람 HWPX 전쪽만 선택한다."""
    root = Path(root)
    backend_key = {'native-skia': 'native', 'wasm-canvas2d': 'wasm'}.get(backend)
    if backend_key is None:
        raise ValueError('지원하지 않는 편람 backend')
    baseline_manifest = json.loads((root / BASELINE_MANIFEST).read_text(encoding='utf-8'))
    if (sha(root / BASELINE) != BASELINE_SHA
            or baseline_manifest['table'] != BASELINE.name
            or baseline_manifest['table_sha256'] != BASELINE_SHA):
        raise ValueError('정본 보호 TSV 해시가 다릅니다')
    datasets = [item for item in baseline_manifest['datasets']
                if item['input']['sha256'] == DOC_SHA
                and item['reference_pdf']['sha256'] == PDF_SHA
                and item['backend'] == backend_key]
    if len(datasets) != 1:
        raise ValueError('정본에 편람 입력·PDF·backend가 하나씩 있어야 합니다')
    dataset = datasets[0]
    identity = dataset['identity']
    if (dataset['page_count'] != PAGES or str(identity['compat']) != '2024'
            or identity['dpi'] != 96 or identity['profile'] != 'print'
            or identity['metric'] != METRIC or identity['radius_px'] != 2):
        raise ValueError('정본 편람의 측정 조건이 다릅니다')
    with (root / BASELINE).open(encoding='utf-8', newline='') as stream:
        rows = [row for row in csv.DictReader(stream, delimiter='\t')
                if row['document_id'] == dataset['document_id'] and row['backend'] == backend_key]
    if any(row['input_sha256'] != DOC_SHA or row['reference_pdf_sha256'] != PDF_SHA
           or row['compat'] != '2024' or row['dpi'] != '96' for row in rows):
        raise ValueError('정본 TSV와 편람 manifest가 다릅니다')
    return rows


def compare(root, manifest, *, dpi=96, backend='native-skia'):
    """383쪽 raw 표시값이 정본의 2자리 기준보다 내려가면 보류한다."""
    root = Path(root)
    try:
        for path, expected in ((DOC, DOC_SHA), (PDF, PDF_SHA), (BASELINE, BASELINE_SHA)):
            if sha(root / path) != expected:
                raise ValueError(f'기준 파일 해시가 다릅니다: {path}')
        p = manifest['provenance']
        if (backend not in {'native-skia', 'wasm-canvas2d'}
                or dpi != 96 or manifest['run_state'] != 'complete'
                or p['hwp']['sha256'] != DOC_SHA or p['pdf']['sha256'] != PDF_SHA
                or p['raster_backend'] != backend or p['layout_generation'] != '2024'
                or p['comparison_profile'] != 'print' or p['pdf_rasterizer']['policy'] != POLICY
                or manifest['radius_px'] != 2 or manifest['silhouette_method'] != METHOD
                or manifest['pr_review_gate']['document_page_counts'] != {'rhwp': PAGES, 'pdf': PAGES}
                or manifest.get('excluded_regions')):
            raise ValueError('기준과 출력 조건이 다르거나 쪽수·제외 영역이 올바르지 않습니다')
        rows = baseline_rows(root, backend)
        def scores(items, rounded=False):
            result = {}
            for row in items:
                page = int(row['page'])
                value = Decimal(f"{float(row[METRIC]):.2f}") if rounded else Decimal(str(row[METRIC]))
                if page in result or page < 1 or not value.is_finite() or not 0 <= value <= 100:
                    raise ValueError('중복·범위 밖 쪽 또는 유효하지 않은 점수')
                result[page] = value
            if set(result) != set(range(1, PAGES + 1)):
                raise ValueError('383쪽 전체의 점수가 필요합니다')
            return result
        baseline, candidate = scores(rows), scores(manifest['metrics'], rounded=True)
        losses = [{'page': p, 'baseline': str(baseline[p]), 'candidate': str(candidate[p])}
                  for p in baseline if candidate[p] < baseline[p]]
        return {'status': 're_review_required' if losses else 'passed',
                'baseline': str(BASELINE), 'baseline_sha256': BASELINE_SHA,
                'metric': METRIC, 'backend': backend, 'precision': '0.01 percentage points', 'pages': PAGES,
                'regressions': losses}
    except (OSError, KeyError, TypeError, ValueError, OverflowError, InvalidOperation) as error:
        return {'status': 're_review_required', 'reason': 'invalid_handbook_evidence', 'error': str(error)}
