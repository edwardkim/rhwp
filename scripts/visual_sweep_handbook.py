"""#8389 편람 Native 전쪽 품질 하한과 검증용 웹폰트 공급."""
from __future__ import annotations

import csv
import hashlib
import os
import subprocess
import tempfile
import urllib.request
from decimal import Decimal, InvalidOperation
from functools import lru_cache
from pathlib import Path

DOC = Path('samples/2025 행정업무운영 편람(최종).hwpx')
PDF = Path('pdf/2025 행정업무운영 편람(최종)-hwpx-kopub-2024.pdf')
BASELINE = Path('mydocs/pr/assets/issue_8153/final-50cdf-validation-20261030/handbook-hwpx-native.tsv')
DOC_SHA = 'c6dd7e847a99f219681afc5a29c80a9665c04df9cda4d820a3350d739664fdf6'
PDF_SHA = '299cb2d994bba8b63f78b681bd74580286e298f25689b4ff4f40c66c54bfa85e'
BASELINE_SHA = 'd5006cd9110c1247282aa5627f1728d9c751c3212925204582b9b40f39900308'
PAGES = 383
METRIC = 'silhouette_raw_match_percent'
POLICY = 'pdfium_smooth_rgb_exact_dpi_v2'
METHOD = 'threshold_boundary_color_support_v1'

# Studio의 기존 webfont projection에 있는 고정 버전이다. 글꼴은 저장소가 아니라
# output에만 다운로드하며, 원본 WOFF 해시 확인 후 Native용 SFNT로 읽는다.
WEBFONTS = {
    ('KoPubBatang', 'Light'): ('KoPubBatang-Light.woff', '2e8500e2fca3501397620a8d3459bf927502989a2e1f5967d38c3f4e97c965b1'),
    ('KoPubBatang', 'Medium'): ('KoPubBatang-Medium.woff', 'd88dc4aa42af7ff0f468ff4a33b64d8d21ad69a6cc7d505775edb928a3644466'),
    ('KoPubBatang', 'Bold'): ('KoPubBatang-Bold.woff', 'aca3dbb68fa1f09e9ea34753596a8dc7aec29cd15b9559d4ed1ece442fbc8542'),
    ('KoPubDotum', 'Light'): ('KoPubDotum-Light.woff', '697dd52dcdf4d294924fc0dd58ed82ec568bfa64f1a785ff5e3a35b67f5313fd'),
    ('KoPubDotum', 'Medium'): ('KoPubDotum-Medium.woff', '16b8b274949c3382a0f68f85ed6b4143be19c5c3ddd003568d26f22cfc3ef9d8'),
    ('KoPubDotum', 'Bold'): ('KoPubDotum-Bold.woff', '8935b31c9b49aac2cb81eaa1de931ae84645d92b72e905bb722f84857f0a9fc9'),
    ('HCR Batang', 'Regular'): ('HANBatang.woff', 'b3192018e3bd4da76b29b0530f71c79fb6f465a2b7cd41308bc38f9a4ae09a6c'),
    ('Government_16040911', 'Regular'): ('Government_16040911.ttf', '9ff914274d89c97abe3c22934c1f5f049d5c82de3cf0a3bc6053ac139b8a111a'),
}


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


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
        files = [root] if root.is_file() else root.iterdir() if root.is_dir() else []
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
                and Path(path).is_file()):
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
        elif name == 'Government_16040911.ttf':
            url = 'https://cdn.jsdelivr.net/gh/jangster77/korea-government-symbol-font@v1.0.0/fonts/' + name
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
            # 정부상징 원본은 공공누리 제4유형으로 변환하지 않고 그대로 사용한다.
            records.append({'family': family, 'style': style, 'url': url,
                            'source_sha256': expected, 'native_path': str(source),
                            'native_sha256': expected,
                            'attribution': '문화체육관광부 대한민국정부상징서체',
                            'license': '공공누리 제4유형: 출처 표시, 비상업, 변경 금지',
                            'license_url': 'https://github.com/jangster77/korea-government-symbol-font/blob/v1.0.0/LICENSE.md'})
            continue
        from fontTools.ttLib import TTFont
        target = face_cache / (name + '.ttf')
        # 매번 원본에서 변환해 기존 TTF 캐시 변조를 통과시키지 않는다.
        font = TTFont(source)
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


def compare(root, manifest, *, dpi=96):
    """383쪽 raw 표시값이 #8389의 2자리 기준보다 내려가면 보류한다."""
    root = Path(root)
    try:
        for path, expected in ((DOC, DOC_SHA), (PDF, PDF_SHA), (BASELINE, BASELINE_SHA)):
            if sha(root / path) != expected:
                raise ValueError(f'기준 파일 해시가 다릅니다: {path}')
        p = manifest['provenance']
        if (dpi != 96 or manifest['run_state'] != 'complete'
                or p['hwp']['sha256'] != DOC_SHA or p['pdf']['sha256'] != PDF_SHA
                or p['raster_backend'] != 'native-skia' or p['layout_generation'] != '2024'
                or p['comparison_profile'] != 'print' or p['pdf_rasterizer']['policy'] != POLICY
                or manifest['radius_px'] != 2 or manifest['silhouette_method'] != METHOD
                or manifest['pr_review_gate']['document_page_counts'] != {'rhwp': PAGES, 'pdf': PAGES}
                or manifest.get('excluded_regions')):
            raise ValueError('기준과 출력 조건이 다르거나 쪽수·제외 영역이 올바르지 않습니다')
        with (root / BASELINE).open(encoding='utf-8', newline='') as stream:
            rows = list(csv.DictReader(stream, delimiter='\t'))
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
                'metric': METRIC, 'precision': '0.01 percentage points', 'pages': PAGES,
                'regressions': losses}
    except (OSError, KeyError, TypeError, ValueError, OverflowError, InvalidOperation) as error:
        return {'status': 're_review_required', 'reason': 'invalid_handbook_evidence', 'error': str(error)}
