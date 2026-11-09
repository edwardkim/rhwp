"""Native 시각 출력에 실제 누락된 글꼴의 Studio 웹 공급 규칙을 적용한다."""
from __future__ import annotations

import hashlib
import json
import os
import subprocess
import tempfile
import urllib.parse
import urllib.request
from pathlib import Path

try:
    from scripts import visual_sweep_handbook as handbook
except ModuleNotFoundError:
    import visual_sweep_handbook as handbook

REGISTRY = Path('assets/font-rules/font_rule_registry_v2.json')
FALLBACK = 'fonts/NotoSansKR-Regular.woff2'
MAX_BYTES = 32 * 1024 * 1024
_PREPARED = {}
PUBLIC_SOURCE_SHA256 = {
    'ROKG_R.woff2': '2c79a998f6b730afe2e311b6098055e9b6b29c1f2ba75f2871fd5dd8f4787426',
    'NotoSerif%5Bwdth,wght%5D.ttf': '4d8e6761424656867019081a1a01336f3cb086982682698714054fc33f782713',
    'MunhwaGothicStd-Regular.woff2': '0dca5496b7fa2ddf453fdb4f249b99bc9032dd3bfac4affef146622a4829ec04',
    'MunhwaGothicStd-Bold.woff2': 'b98cea218075a5f7fb23e5ab62c8d33e523c65cc2bd0540305430da15807f599',
    'MunhwaMyungjoStd-Regular.woff2': '2c4ae1f41288e30a32b0f71db5db8eeda2bd6d89e202fa9d9cabbb99565bdb97',
    'MunhwaMyungjoStd-Bold.woff2': '03adfe78705e2c7503a6d138395f02c9e15887b2340fe574322577d944b90435',
    'MunhwaHoonminStd-Regular.woff2': 'b83ade4c534b8b0da58f2dcdb3ddabdf948d6a6a34853598aa7e48107ecac414',
    'MunhwaScriptStd-Light.woff2': '1701512f8985df3e1e72114c94ff83139fe61fc63a2843edaeb6437f84c3cb2b',
    'MunhwaScriptStd-Bold.woff2': '8fd4d8c82fa524626e8004a2aa736b141dd8cd9578546bd37c40e9bee5c9ec61',
    'MunhwaScriptFreestyleStd-Light.woff2': '3f6ce804a614348ea6a854140513fb1bf619a1714b5910e01969e952be29c7b7',
    'MunhwaScriptFreestyleStd-Bold.woff2': '4ba3802099a14561d804c1b5cf898c3fd03d7236295cd6e1b59a2197c5f51aac',
}
PUBLIC_NOTO_SERIF = ('https://cdn.jsdelivr.net/gh/google/fonts@'
                     '2fe1e8c6b137fb13041a7cfca6fdcd8a910ea4dc/'
                     'ofl/notoserif/NotoSerif%5Bwdth,wght%5D.ttf')
HANCOM_BATANG_WEB = 'https://cdn.jsdelivr.net/gh/projectnoonnu/noonfonts_2104@1.0/HANBatang.woff'
MUNHWA_CDN_PATHS = {
    f'/korean-webfonts/1/orgs/govs/mcst/{family}/{name}'
    for family, names in {
        'MunhwaGothic': ('MunhwaGothicStd-Regular.woff2', 'MunhwaGothicStd-Bold.woff2'),
        'MunhwaMyungjo': ('MunhwaMyungjoStd-Regular.woff2', 'MunhwaMyungjoStd-Bold.woff2'),
        'MunhwaHoonmin': ('MunhwaHoonminStd-Regular.woff2',),
        'MunhwaScript': ('MunhwaScriptStd-Light.woff2', 'MunhwaScriptStd-Bold.woff2'),
        'MunhwaScriptFreestyle': ('MunhwaScriptFreestyleStd-Light.woff2', 'MunhwaScriptFreestyleStd-Bold.woff2'),
    }.items() for name in names
}


def _normal(name):
    return ''.join(name.casefold().split())


def _sha(data):
    return hashlib.sha256(data).hexdigest()


def _run(command, root):
    p = subprocess.run(command, cwd=root, text=True, encoding='utf-8',
                       capture_output=True, check=False)
    if p.returncode:
        raise RuntimeError(f'Native 글꼴 준비 명령 실패: {command[1]} ({p.returncode}): {p.stderr[-2000:]}')
    return json.loads(p.stdout)


def _installed_names(root, paths):
    """시스템 및 Native가 직접 읽는 명시 경로의 family/full/PostScript 이름."""
    names = set()
    try:
        p = subprocess.run(['fc-list', '--format', '%{file}\t%{family}\t%{fullname}\t%{postscriptname}\n'],
                           capture_output=True, text=True, check=True, timeout=30)
        readable = {}
        for line in p.stdout.splitlines():
            columns = line.split('\t', 3)
            if len(columns) != 4:
                continue
            path, *identities = columns
            # Fontconfig 캐시는 삭제되거나 접근이 차단된 파일도 반환할 수 있다.
            # 같은 TTC의 여러 face는 파일의 읽기 가능 여부를 한 번만 확인한다.
            if path not in readable:
                readable[path] = handbook.readable_font_file(path)
            if readable[path]:
                for identity in identities:
                    names.update(_normal(x) for x in identity.split(',') if x.strip())
    except (OSError, subprocess.SubprocessError):
        pass
    paths = [*paths, root / 'ttfs/opensource']
    names.update(name for name, _ in handbook._file_font_inventory(tuple(str(x) for x in paths)))
    return names


def _postscript_aliases(paths):
    """제공한 실제 face의 하이픈 차이만 보정하며 모호한 이름은 제외한다."""
    from fontTools.ttLib import TTCollection, TTFont, TTLibError
    candidates = {}
    for root in paths:
        try:
            files = [root] if root.is_file() else list(root.iterdir()) if root.is_dir() else []
        except OSError:
            continue
        for path in files:
            if not path.is_file() or path.suffix.lower() not in {'.ttf', '.otf', '.ttc'}:
                continue
            try:
                collection = TTCollection(path, lazy=True) if path.suffix.lower() == '.ttc' else None
                fonts = collection.fonts if collection else [TTFont(path, lazy=True)]
                try:
                    for font in fonts:
                        name = font['name'].getDebugName(6)
                        if name:
                            candidates.setdefault(_normal(name).replace('-', ''), set()).add(name)
                finally:
                    for font in fonts:
                        font.close()
            except (OSError, ValueError, KeyError, IndexError, TTLibError):
                continue
    return {name: next(iter(values)) for name, values in candidates.items() if len(values) == 1}


def _rules(root):
    data = (root / REGISTRY).read_bytes()
    registry = json.loads(data)
    selected = {}
    for item in registry['rules']:
        supply = item.get('supply') or {}
        source = item.get('sourceFace')
        if (item.get('status') == 'active' and isinstance(source, str)
                and supply.get('kind') == 'canvas2d-webfont'
                and isinstance(supply.get('sourceUrl'), str)
                and not supply.get('unicodeRange')):
            selected.setdefault(_normal(source), item)
    return selected, _sha(data)


def _resource(root, cache, url):
    """고정 공급 규칙만 읽고 입력 파일명으로 임의 URL을 만들지 않는다."""
    from fontTools.ttLib import TTFont
    known = {name: digest for name, digest in handbook.WEBFONTS.values()}
    known.update(PUBLIC_SOURCE_SHA256)
    if url.startswith('fonts/'):
        source = (root / 'assets/fonts' / url.removeprefix('fonts/')).resolve()
        if not source.is_relative_to((root / 'assets/fonts').resolve()) or not source.is_file():
            raise ValueError(f'프로젝트 웹폰트가 없습니다: {url}')
        data = source.read_bytes()
    else:
        parsed = urllib.parse.urlsplit(url)
        if (parsed.scheme != 'https' or parsed.netloc != 'cdn.jsdelivr.net'
                or ('@' not in parsed.path and parsed.path not in MUNHWA_CDN_PATHS)
                or parsed.query or parsed.fragment):
            raise ValueError(f'고정되지 않은 웹폰트 공급 URL: {url}')
        key = _sha(url.encode())
        item = cache / 'downloads' / key
        identity = item.with_suffix('.json')
        data = None
        if item.is_file() and identity.is_file():
            try:
                saved = json.loads(identity.read_text(encoding='utf-8'))
                candidate = item.read_bytes()
                if saved == {'url': url, 'sha256': _sha(candidate)}:
                    data = candidate
            except (OSError, ValueError, TypeError):
                pass
        if data is None:
            with urllib.request.urlopen(url, timeout=60) as response:
                if urllib.parse.urlsplit(response.geturl()).netloc != 'cdn.jsdelivr.net':
                    raise ValueError('웹폰트가 허용되지 않은 서버로 리디렉션됐습니다')
                data = response.read(MAX_BYTES + 1)
            if not data or len(data) > MAX_BYTES:
                raise ValueError('웹폰트 크기가 허용 범위 밖입니다')
            item.parent.mkdir(parents=True, exist_ok=True)
            with tempfile.NamedTemporaryFile(dir=item.parent, delete=False) as out:
                temporary = Path(out.name)
                out.write(data)
            try:
                os.replace(temporary, item)
            finally:
                temporary.unlink(missing_ok=True)
            encoded = json.dumps({'url': url, 'sha256': _sha(data)}, sort_keys=True).encode()
            with tempfile.NamedTemporaryFile(dir=item.parent, delete=False) as out:
                temporary = Path(out.name)
                out.write(encoded)
            try:
                os.replace(temporary, identity)
            finally:
                temporary.unlink(missing_ok=True)
    if not data or len(data) > MAX_BYTES:
        raise ValueError(f'웹폰트 크기가 허용 범위 밖입니다: {url}')
    digest = _sha(data)
    expected = known.get(Path(urllib.parse.urlsplit(url).path).name)
    if expected is not None and digest != expected:
        raise ValueError(f'고정 웹폰트 해시가 다릅니다: {url}')
    folder = cache / 'programs' / digest
    folder.mkdir(parents=True, exist_ok=True)
    original_suffix = Path(urllib.parse.urlsplit(url).path).suffix.lower()
    if original_suffix in {'.ttf', '.otf'}:
        native = folder / ('source' + original_suffix)
        if not native.is_file() or _sha(native.read_bytes()) != digest:
            with tempfile.NamedTemporaryFile(dir=folder, delete=False) as out:
                temporary = Path(out.name)
                out.write(data)
            try:
                os.replace(temporary, native)
            finally:
                temporary.unlink(missing_ok=True)
        # 정부상징 등 원본 TTF/OTF는 한 바이트도 수정하지 않는다.
        font = TTFont(native, lazy=True)
    else:
        original = folder / ('source' + original_suffix)
        if not original.is_file() or _sha(original.read_bytes()) != digest:
            with tempfile.NamedTemporaryFile(dir=folder, delete=False) as out:
                temporary = Path(out.name)
                out.write(data)
            try:
                os.replace(temporary, original)
            finally:
                temporary.unlink(missing_ok=True)
        # 동일한 원본의 Native 프로그램 신원을 변환 시각으로 바꾸지 않는다.
        # 매번 원본을 다시 읽어 캐시 변조를 검증하는 계약은 유지한다.
        font = TTFont(original, recalcTimestamp=False)
        native = folder / ('native.otf' if font.sfntVersion == 'OTTO' else 'native.ttf')
        font.flavor = None
        with tempfile.NamedTemporaryFile(dir=folder, suffix='.ttf', delete=False) as out:
            temporary = Path(out.name)
        try:
            font.save(temporary, reorderTables=False)
            os.replace(temporary, native)
        finally:
            temporary.unlink(missing_ok=True)
    try:
        postscript = font['name'].getDebugName(6) or font['name'].getDebugName(1)
        if not postscript or not font.getBestCmap():
            raise ValueError(f'웹폰트 이름 또는 Unicode cmap이 없습니다: {url}')
    finally:
        font.close()
    return {'url': url, 'source_sha256': digest, 'native_path': str(native),
            'native_sha256': _sha(native.read_bytes()), 'native_face': postscript}


def prepare(root, hwp, binary, paths, explicit_environment, compat, dpi):
    """문서 요청에서 로컬을 우선하고 없는 face에만 웹폰트를 준비한다."""
    root, hwp = Path(root), Path(hwp)
    paths = tuple(Path(p) if Path(p).is_absolute() else root / p for p in paths)
    if explicit_environment is not None:
        explicit_environment = Path(explicit_environment)
        if not explicit_environment.is_absolute():
            explicit_environment = root / explicit_environment
    registry_sha = _sha((root / REGISTRY).read_bytes())
    environment_sha = _sha(Path(explicit_environment).read_bytes()) if explicit_environment else None
    # 프로세스 내 쪽 묶음은 동일 source·binary·font 공급을 재사용한다.
    key = (str(root), _sha(hwp.read_bytes()), str(binary), tuple(str(x) for x in paths),
           environment_sha, compat, dpi, registry_sha)
    if key in _PREPARED:
        return _PREPARED[key]
    plan = _run([str(binary), 'info', str(hwp), '--font-plan'], root)
    requests = plan.get('requests')
    embedded = {_normal(x) for x in plan.get('embeddedFamilies', []) if isinstance(x, str)}
    if not isinstance(requests, list):
        raise ValueError('Native 글꼴 선택 계획에 요청 목록이 없습니다')
    installed = _installed_names(root, paths)
    postscript_aliases = _postscript_aliases(paths)
    rules, registry_sha = _rules(root)
    cache = root / 'output/pr-review/native-webfont-cache'
    mappings, resources, unresolved, local_substitutes = {}, {}, [], []
    explicitly_mapped = set()
    if explicit_environment:
        specified = json.loads(Path(explicit_environment).read_text(encoding='utf-8'))
        if isinstance(specified, dict):
            explicitly_mapped.update(specified.get('substitutions', {}))
            explicitly_mapped.update(entry['source'] for entry in specified.get('documentSubstitutions', [])
                                     if isinstance(entry, dict) and 'source' in entry)
    groups = {}
    possible_aliases = {}
    observed_environment = None
    for request in requests:
        if not isinstance(request, dict) or not isinstance(request.get('source'), str):
            raise ValueError('Native 글꼴 요청 형식이 올바르지 않습니다')
        source = request['source']
        if _normal(source) in embedded or source in explicitly_mapped:
            continue
        primary = request.get('primary')
        if not isinstance(primary, list) or any(not isinstance(x, str) for x in primary):
            raise ValueError('Native 주 글꼴 후보가 올바르지 않습니다')
        substitute = request.get('substitute', [])
        if not isinstance(substitute, list) or any(not isinstance(x, str) for x in substitute):
            raise ValueError('Native 대체 글꼴 후보가 올바르지 않습니다')
        group = groups.setdefault(source, {'primary': [], 'substitute': []})
        group['primary'].extend(primary)
        group['substitute'].extend(substitute)
    for source, group in groups.items():
        primary = group['primary']
        # 공급된 정확한 PostScript face의 하이픈 차이를 해소한다. 이때
        # 문서 대체 글꼴보다 원 프로그램을 우선하고, 동일 키가 모호하면 건너뛴다.
        aliases = {postscript_aliases[_normal(x).replace('-', '')] for x in primary
                   if _normal(x).replace('-', '') in postscript_aliases
                   and _normal(x) != _normal(postscript_aliases[_normal(x).replace('-', '')])}
        # full name은 설치 목록에 있지만 Native family 조회가 실패할 수 있다.
        # 실제 문서 대체가 선택됐는지는 아래에서 Native 환경을 확인한다.
        if len(aliases) == 1:
            possible_aliases[source] = next(iter(aliases))
        # 같은 원본의 어느 언어 슬롯에든 주 프로그램이 있으면 전역 치환하지 않는다.
        if any(_normal(x) in installed for x in primary):
            continue
        if len(aliases) == 1:
            mappings[source] = next(iter(aliases))
            continue
        rule = next((rules[_normal(x)] for x in [source, *primary] if _normal(x) in rules), None)
        if rule is None:
            unresolved.append(source)
            if any(_normal(x) in installed for x in group['substitute']):
                local_substitutes.append(source)
                continue
            url = FALLBACK
        else:
            url = rule['supply']['sourceUrl']
        if url not in resources:
            resources[url] = _resource(root, cache, url)
        mappings[source] = resources[url]['native_face']
    if possible_aliases and not explicit_environment:
        args = [str(binary), 'export-png', str(hwp), '--metadata-only', '--profile', 'print',
                '--json', '--dpi', str(dpi), '--scale', str(dpi / 96), '--compat', compat]
        for path in paths:
            args += ['--font-path', str(path)]
        observed_environment = _run(args, root).get('fontEnvironment')
        if isinstance(observed_environment, dict):
            replaced = {entry.get('source') for entry in observed_environment.get('documentSubstitutions', [])}
            mappings.update({source: alias for source, alias in possible_aliases.items() if source in replaced})
    scoped_fallbacks = []
    if not explicit_environment:
        for request in plan.get('scopedRequests', []):
            if not isinstance(request, dict):
                raise ValueError('슬롯별 글꼴 요청이 올바르지 않습니다')
            source, primary, substitute = request.get('source'), request.get('primary'), request.get('substitute')
            slot, font_id = request.get('languageSlot'), request.get('fontId')
            if (not isinstance(source, str) or not isinstance(primary, list)
                    or not isinstance(substitute, list) or not isinstance(slot, int)
                    or not isinstance(font_id, int) or not 0 <= slot < 7 or not 0 <= font_id <= 65535
                    or any(not isinstance(name, str) for name in [*primary, *substitute])):
                raise ValueError('슬롯별 글꼴 요청이 올바르지 않습니다')
            if (_normal(source) in embedded or source in mappings
                    or not any(_normal(name) in installed for name in primary)
                    or any(_normal(name) in installed for name in substitute)):
                continue
            rule = next((rules[_normal(name)] for name in substitute if _normal(name) in rules), None)
            if rule is None:
                continue
            url = rule['supply']['sourceUrl']
            if url not in resources:
                resources[url] = _resource(root, cache, url)
            resource = resources[url]
            # 같은 원본을 이미 변환한 명시 입력은 재사용해 글꼴 예산을 중복 소비하지 않는다.
            source_name = Path(urllib.parse.urlsplit(url).path).name
            for directory in paths:
                source_file = directory / source_name
                if not source_file.is_file() or _sha(source_file.read_bytes()) != resource['source_sha256']:
                    continue
                from fontTools.ttLib import TTFont
                for candidate in directory.glob('*.ttf'):
                    font = TTFont(candidate, lazy=True)
                    try:
                        same_face = font['name'].getDebugName(6) == resource['native_face']
                    finally:
                        font.close()
                    if same_face:
                        resource['native_path'] = str(candidate)
                        resource['native_sha256'] = _sha(candidate.read_bytes())
                        break
            target = resource['native_face']
            fallback_faces = [target]
            # 문서가 한컴바탕을 요청했지만 원본 프로그램이 없는 경우에만
            # 검증한 공개 face를 앞에 둔다. 원 glyph hit와 설치된 문서
            # 대체 글꼴은 위에서 우선하며, 문서 이름·쪽 번호로 분기하지 않는다.
            source_key = _normal(source)
            if url == HANCOM_BATANG_WEB and slot == 5 and source_key in {
                    _normal('KoPub바탕체 Light'), _normal('KoPubBatangLight')}:
                if _normal('Noto Serif') not in installed:
                    if PUBLIC_NOTO_SERIF not in resources:
                        resources[PUBLIC_NOTO_SERIF] = _resource(root, cache, PUBLIC_NOTO_SERIF)
                fallback_faces.insert(0, 'Noto Serif')
            elif url == HANCOM_BATANG_WEB and slot == 0 and source_key in {
                    _normal('KoPub돋움체 Light'), _normal('KoPubDotumLight')}:
                candidate = 'Noto Sans KR ExtraLight'
                if _normal(candidate) not in installed:
                    extra_rule = rules.get(_normal(candidate))
                    if extra_rule is not None:
                        extra_url = extra_rule['supply']['sourceUrl']
                        if extra_url not in resources:
                            resources[extra_url] = _resource(root, cache, extra_url)
                        candidate = resources[extra_url]['native_face']
                    else:
                        candidate = None
                if candidate:
                    fallback_faces.insert(0, candidate)
            if _normal(target) != _normal(source):
                scoped_fallbacks.append({'languageSlot': slot, 'fontId': font_id,
                                         'source': source, 'faces': list(dict.fromkeys(fallback_faces))})
    directories = list(dict.fromkeys(str(Path(x['native_path']).parent) for x in resources.values()))
    environment_path = explicit_environment
    if mappings or scoped_fallbacks:
        if explicit_environment:
            env = json.loads(Path(explicit_environment).read_text(encoding='utf-8'))
            if env is None:
                # 명시적인 환경 해제는 호출자의 선택이므로 덮어쓰지 않는다.
                mappings = {}
        elif observed_environment is not None:
            env = observed_environment
        else:
            args = [str(binary), 'export-png', str(hwp), '--metadata-only', '--profile', 'print',
                    '--json', '--dpi', str(dpi), '--scale', str(dpi / 96), '--compat', compat]
            for path in paths:
                args += ['--font-path', str(path)]
            env = _run(args, root).get('fontEnvironment')
        if mappings or scoped_fallbacks:
            env = dict(env or {'id': 'native-webfont-fallback', 'substitutions': {},
                               'preserveSourceMetrics': True})
            if scoped_fallbacks:
                declared = list(env.get('documentGlyphFallbacks') or [])
                seen_slots = {(item['languageSlot'], item['fontId']) for item in declared}
                declared.extend(item for item in scoped_fallbacks
                                if (item['languageSlot'], item['fontId']) not in seen_slots)
                env['documentGlyphFallbacks'] = declared
            if not explicit_environment:
                # 주 face를 웹폰트로 복구해도 문서가 지정한 문자별 대체는
                # 유지한다. 글리프가 없는 경우 측정과 paint가 함께 사용한다.
                glyph_fallbacks = {key: list(value) for key, value in (env.get('glyphFallbacks') or {}).items()}
                for entry in env.get('documentSubstitutions', []):
                    alias = mappings.get(entry.get('source'))
                    target = entry.get('target')
                    if alias and target and target != alias:
                        chain = glyph_fallbacks.setdefault(alias, [])
                        if target not in chain:
                            chain.append(target)
                if glyph_fallbacks:
                    env['glyphFallbacks'] = glyph_fallbacks
            existing = dict(env.get('substitutions') or {})
            if explicit_environment:
                # 호출자가 명시한 치환은 자동 웹폰트보다 우선한다.
                mappings = {k: v for k, v in mappings.items() if k not in existing and not any(
                    entry.get('source') == k for entry in env.get('documentSubstitutions', []))}
            existing.update(mappings)
            env['substitutions'] = existing
            if not explicit_environment:
                env['documentSubstitutions'] = [entry for entry in env.get('documentSubstitutions', [])
                                                if entry.get('source') not in mappings]
            for source in mappings:
                (env.get('selectionRules') or {}).pop(source, None)
            env['id'] = 'native-webfont-' + _sha(json.dumps(env, sort_keys=True, ensure_ascii=False).encode())[:24]
            encoded = (json.dumps(env, ensure_ascii=False, sort_keys=True) + '\n').encode()
            envdir = cache / 'environments'
            envdir.mkdir(parents=True, exist_ok=True)
            dest = envdir / (_sha(encoded) + '.json')
            if not dest.is_file() or dest.read_bytes() != encoded:
                with tempfile.NamedTemporaryFile(dir=envdir, delete=False) as out:
                    temporary = Path(out.name)
                    out.write(encoded)
                try:
                    os.replace(temporary, dest)
                finally:
                    temporary.unlink(missing_ok=True)
            environment_path = dest
    # 문자 대체 전용 공개 face가 일반 대체 글꼴보다 뒤에서 등록되면
    # 같은 glyph를 제공하는 일반 face가 먼저 선택될 수 있다.
    priority = ([Path(resources[PUBLIC_NOTO_SERIF]['native_path']).parent]
                if PUBLIC_NOTO_SERIF in resources else [])
    result = {'font_paths': list(dict.fromkeys([*priority, *paths, *(Path(x) for x in directories)])),
              'font_environment': environment_path,
              'evidence': {'registry_sha256': registry_sha, 'mappings': mappings,
                           'document_glyph_fallbacks': scoped_fallbacks,
                           'unmapped_sources': sorted(set(unresolved)),
                           'local_document_substitutes': sorted(set(local_substitutes)),
                           'resources': list(resources.values()),
                           'module_sha256': _sha(Path(__file__).read_bytes())}}
    _PREPARED[key] = result
    return result
