"""Visual Sweep 직접 PNG 생산과 공급 글꼴의 파일 기반 catalogue."""
from __future__ import annotations

import hashlib
import json
import os
import shutil
import struct
import uuid
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path


def png_backend(wasm_package: Path | None) -> str:
    return 'wasm-canvas2d' if wasm_package else 'native-skia'


def png_export_has_pages(base: Path, wasm: bool, selected) -> bool:
    metadata = base / ('wasm-export-complete.json' if wasm else 'native-export.json')
    if not metadata.is_file():
        return False
    try:
        result = json.loads(metadata.read_text(encoding='utf-8'))
    except (OSError, ValueError):
        return False
    if not isinstance(result, dict):
        return False
    count = result.get('pageCount')
    backend = result.get('rasterBackend')
    if type(count) is not int or count < 1 or backend != ('wasm-canvas2d' if wasm else 'native-skia'):
        return False
    if result.get('comparisonProfile') != 'print':
        return False
    wanted = set(selected or range(1, count + 1))
    if not wanted <= set(range(1, count + 1)):
        return False
    items = result.get('pages')
    if not isinstance(items, list) or any(not isinstance(item, dict) or type(item.get('page')) is not int for item in items):
        return False
    recorded = {item['page'] if wasm else item['page'] + 1 for item in items}
    if not wanted <= recorded:
        return False
    return all((base / 'rhwp_png' / f'rhwp_{p:03d}.png').is_file()
               and (base / 'render_tree' / f'render_tree_{p:03d}.json').is_file() for p in wanted)


def assert_native_png_capability(vs, root: Path, binary: str) -> None:
    result = vs.run([binary, 'capabilities', '--search', 'export-png', '--json'], cwd=root)
    commands = json.loads(result.stdout).get('commands', [])
    capability = next((c for c in commands if c.get('name') == 'export-png'), {})
    if not capability.get('available'):
        raise SystemExit('직접 PNG에는 native-skia 빌드가 필요합니다. 공용 target/pr-review에서 '
                         'cargo build --locked --features native-skia --bin rhwp를 실행하세요. '
                         'SVG 진단 경로는 --rasterizer svg로 선택합니다.')
    if not capability.get('json') or not {'--metadata-only', '--render-tree-output'} <= set(capability.get('flags', [])):
        raise SystemExit('PNG manifest API가 없는 이전 CLI입니다. 현재 소스로 native-skia를 다시 빌드하세요.')


def export_native_png(vs, root, hwp, binary, base, selected, font_paths, environment_args, dpi, workers):
    assert_native_png_capability(vs, root, binary)
    common = [binary, 'export-png', str(hwp), '--profile', 'print', '--json',
              '--dpi', str(dpi), '--scale', str(dpi / 96.0), *environment_args]
    compat = environment_args[environment_args.index('--compat') + 1]
    for directory in font_paths:
        common += ['--font-path', str(vs.resolve_input_path(root, directory))]
    work = base / 'native-png'
    work.mkdir(exist_ok=True)
    if selected is None and workers > 1:
        response = vs.run([*common, '--metadata-only', '-o', str(work)], cwd=root,
                          log_path=base / 'native-png-metadata.log')
        info = json.loads(response.stdout)
        count = info.get('pageCount')
        if info.get('metadataOnly') is not True or type(count) is not int or count < 1:
            raise SystemExit('Native PNG 전체 쪽수 metadata가 올바르지 않습니다.')
        requested = list(range(1, count + 1))
    else:
        count = None
        requested = selected
    chunks = [requested[i::workers] for i in range(min(workers, len(requested)))] if requested else [None]

    def export_chunk(index_pages):
        index, pages = index_pages
        directory = work / str(index)
        directory.mkdir(exist_ok=True)
        argv = [*common, '-o', str(directory), '--render-tree-output', str(directory / 'render_tree')]
        for page in pages or []:
            argv += ['-p', str(page - 1)]
        response = vs.run(argv, cwd=root, log_path=work / f'{index}.log')
        result = json.loads(response.stdout)
        total = result.get('pageCount')
        if type(total) is not int or total < 1:
            raise SystemExit('Native PNG 전체 쪽수를 읽을 수 없습니다.')
        expected = {p - 1 for p in pages} if pages else set(range(total))
        items = result.get('pages', [])
        if (result.get('backend') != 'native-skia' or result.get('comparisonProfile') != 'print'
                or result.get('layoutGeneration') != compat
                or result.get('renderedCount') != len(expected) or len(items) != len(expected)
                or any(type(item.get('page')) is not int for item in items)
                or {item.get('page') for item in items} != expected):
            raise SystemExit('Native PNG 요청 쪽의 산출물이 누락됐거나 출력 profile이 다릅니다.')
        if count is not None and total != count:
            raise SystemExit('Native PNG 작업자 간 전체 쪽수가 다릅니다.')
        for item in items:
            source = Path(item['path']).resolve()
            if not source.is_relative_to(directory.resolve()) or not source.is_file():
                raise SystemExit('Native PNG manifest의 경로가 내보내기 폴더를 벗어났거나 없습니다.')
            if source.stat().st_size != item.get('bytes'):
                raise SystemExit('Native PNG manifest와 실제 파일 크기가 다릅니다.')
            validate_png(source)
            tree = Path(item.get('renderTreePath') or '').resolve()
            if not tree.is_relative_to((directory / 'render_tree').resolve()) or not tree.is_file():
                raise SystemExit('Native PNG와 같은 문서의 render tree가 누락됐습니다.')
            vs.load_json_object(tree, 'Native PNG render tree')
            target = base / 'rhwp_png' / f"rhwp_{item['page'] + 1:03d}.png"
            shutil.move(source, target)
            item['path'] = str(target)
            tree_target = base / 'render_tree' / f"render_tree_{item['page'] + 1:03d}.json"
            shutil.move(tree, tree_target)
            item['renderTreePath'] = str(tree_target)
        return result

    with ThreadPoolExecutor(max_workers=workers) as pool:
        results = list(pool.map(export_chunk, enumerate(chunks)))
    counts = {result['pageCount'] for result in results}
    if len(counts) != 1:
        raise SystemExit('Native PNG 작업자별 전체 쪽수가 다릅니다.')
    count = counts.pop()
    items = sorted((item for result in results for item in result['pages']), key=lambda x: x['page'])
    expected = {p - 1 for p in selected} if selected else set(range(count))
    if len(items) != len(expected) or {item['page'] for item in items} != expected:
        raise SystemExit('Native PNG 전체 산출물이 누락·중복되었습니다.')
    vs.write_json_atomic(base / 'native-export.json', {
        'pageCount': count, 'all_pages_exported': selected is None,
        'rasterBackend': 'native-skia', 'comparisonProfile': 'print', 'layoutGeneration': compat, 'pages': items,
    })


def font_cache_directory(root: Path) -> Path:
    """생산 코드와 FontTools 버전이 같은 경우에만 공용 font program을 재사용한다."""
    import fontTools
    producer = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
    key = hashlib.sha256(f'{producer}:{fontTools.__version__}'.encode()).hexdigest()
    return root / 'output' / 'pr-review' / 'font-catalog-cache' / key


def cached_font_faces(vs, directory: Path, source: Path, digest: str):
    """원본과 파생 program의 SHA를 확인한다. 손상·누락 캐시는 다시 생산한다."""
    metadata = directory / f'{digest}.json'
    try:
        value = json.loads(metadata.read_text(encoding='utf-8'))
        if value.get('sourceSha256') != digest or value.get('schemaVersion') != 1:
            return None
        faces = value.get('faces')
        count = value.get('faceCount')
        if (not isinstance(faces, list) or not faces or type(count) is not int
                or count != len(faces) or count < 1):
            return None
        result = []
        indices = set()
        for entry in faces:
            index = entry.get('faceIndex')
            if type(index) is not int or index < 0 or index in indices:
                return None
            indices.add(index)
            names = entry.get('families')
            if (not isinstance(names, list) or not names
                    or any(not isinstance(name, str) or not name for name in names)
                    or type(entry.get('weight')) is not int or not 1 <= entry['weight'] <= 1000
                    or entry.get('style') not in {'normal', 'italic'}
                    or not valid_font_repairs(entry.get('repairs'))):
                return None
            kind = entry.get('cacheKind')
            if kind == 'source':
                path = source
                actual = digest
            elif kind == 'derived':
                path = directory / f'{digest}-{index}.ttf'
                actual = vs.sha256_file(path)
            else:
                return None
            if actual != entry.get('sha256'):
                return None
            face = {key: item for key, item in entry.items() if key != 'cacheKind'}
            face.update(path=str(path.resolve()), source=str(source.resolve()), sourceSha256=digest)
            result.append(face)
        if indices != set(range(count)):
            return None
        return result
    except (OSError, ValueError, TypeError, AttributeError):
        return None


def font_source_files(directory: Path):
    """등록된 디렉터리 링크를 따라가되 같은 물리 디렉터리를 반복하지 않는다."""
    pending = [directory]
    visited = set()
    while pending:
        current = pending.pop()
        try:
            stat = current.stat()
            identity = (stat.st_dev, stat.st_ino)
            if identity in visited:
                continue
            visited.add(identity)
            entries = sorted(current.iterdir())
        except OSError:
            continue
        children = []
        for source in entries:
            if source.is_dir():
                children.append(source)
            elif source.is_file() and source.suffix.lower() in {'.ttf', '.otf', '.ttc', '.woff', '.woff2'}:
                yield source
        pending.extend(reversed(children))


def valid_font_repairs(repairs) -> bool:
    """공급본에 허용한 수정만 캐시에서 수용한다. 다른 손상은 숨기지 않는다."""
    allowed = {'cmap-format4-terminal-notdef', 'drop-embedded-bitmap-strikes'} | {f'empty-vtt-source-table:TSI{i}' for i in range(6)}
    return (isinstance(repairs, list) and all(isinstance(value, str) for value in repairs)
            and len(repairs) == len(set(repairs)) and set(repairs) <= allowed)


def empty_vtt_tables(font) -> list[str]:
    """Chrome OTS가 거부하는 빈 VTT 편집 소스만 제외한다. 렌더 테이블은 보존한다."""
    return sorted(str(tag) for tag in font.reader.keys()
                  if str(tag) in {f'TSI{i}' for i in range(6)} and len(font.reader[tag]) == 0)


def font_catalog(vs, root: Path, font_paths, base: Path) -> Path:
    """실제 name/weight/face-index를 기록한다. 글꼴을 SVG마다 복제하지 않는다."""
    from fontTools.ttLib import TTFont, TTCollection
    records = []
    seen = set()
    resource_dir = font_cache_directory(root)
    for directory in font_paths:
        directory = vs.resolve_input_path(root, directory)
        for source in font_source_files(directory):
            digest = vs.sha256_file(source)
            if digest in seen:
                continue
            seen.add(digest)
            cached = cached_font_faces(vs, resource_dir, source, digest)
            if cached is not None:
                records.extend(cached)
                continue
            collection = TTCollection(source, lazy=True) if source.suffix.lower() == '.ttc' else None
            fonts = collection.fonts if collection else [TTFont(source, lazy=True)]
            cached_records = []
            try:
                for index, font in enumerate(fonts):
                    try:
                        names, name_errors = font_family_names(font)
                    except ValueError as error:
                        raise SystemExit(f'공급 글꼴의 유효한 family 이름이 없습니다: {source} face={index}: {error}') from error
                    name_warnings = [{'nameID': item['nameID'],
                                      'platformID': item['platformID'],
                                      'platEncID': item['encodingID'],
                                      'langID': item['languageID'],
                                      'error': item['reason']} for item in name_errors]
                    weight = font['OS/2'].usWeightClass if 'OS/2' in font else 400
                    italic = bool(font['head'].macStyle & 2) if 'head' in font else False
                    path = source
                    repaired_cmap = repair_terminal_cmap(font.reader['cmap'], font['maxp'].numGlyphs) if 'cmap' in font else None
                    vtt_tables = empty_vtt_tables(font)
                    dropped = embedded_bitmap_strike_tags(font.reader.keys())
                    excluded_tables = sorted(set(vtt_tables) | dropped)
                    repairs = (['cmap-format4-terminal-notdef'] if repaired_cmap is not None else [])
                    repairs += [f'empty-vtt-source-table:{tag}' for tag in vtt_tables]
                    if dropped:
                        repairs.append('drop-embedded-bitmap-strikes')
                    if collection or repaired_cmap is not None or excluded_tables:
                        resource_dir.mkdir(parents=True, exist_ok=True)
                        path = resource_dir / f'{digest}-{index}.ttf'
                        temporary = resource_dir / f'.{digest}-{index}-{os.getpid()}-{uuid.uuid4().hex}.ttf'
                        try:
                            # 원본 SFNT 테이블을 보존하고 완성한 face만 원자적으로 공개한다.
                            write_font_face(font, temporary, repaired_cmap, excluded_tables)
                            os.replace(temporary, path)
                        finally:
                            temporary.unlink(missing_ok=True)
                    face = {'path': str(path.resolve()), 'source': str(source.resolve()),
                                    'sourceSha256': digest, 'sha256': vs.sha256_file(path),
                                    'faceIndex': index, 'families': names, 'weight': weight,
                                    'nameDecodeWarnings': name_warnings,
                                    'style': 'italic' if italic else 'normal',
                                    'nameRecordErrors': name_errors,
                                    'repairs': repairs}
                    records.append(face)
                    cached_records.append({**face, 'cacheKind': 'derived' if collection or repaired_cmap is not None or excluded_tables else 'source'})
                resource_dir.mkdir(parents=True, exist_ok=True)
                vs.write_json_atomic(resource_dir / f'{digest}.json',
                                     {'schemaVersion': 1, 'sourceSha256': digest,
                                      'faceCount': len(cached_records), 'faces': cached_records})
            finally:
                if collection:
                    collection.close()
                else:
                    fonts[0].close()
    path = base / 'font-catalog.json'
    vs.write_json_atomic(path, {'faces': records})
    return path


def font_family_names(font) -> tuple[list[str], list[dict]]:
    """유효한 이름을 사용하고 해독 불가 항목의 원본 식별자를 보존한다."""
    names, errors = set(), []
    for entry in font['name'].names:
        if entry.nameID not in {1, 4, 16}:
            continue
        try:
            value = entry.toUnicode()
        except (UnicodeError, LookupError) as error:
            errors.append({'nameID': entry.nameID, 'platformID': entry.platformID,
                           'encodingID': entry.platEncID, 'languageID': entry.langID,
                           'rawSha256': hashlib.sha256(entry.string).hexdigest(),
                           'reason': str(error)})
        else:
            if value:
                names.add(value)
    if errors and not names:
        raise ValueError('공급 face의 family 이름을 해독할 수 없습니다: ' + str(errors))
    return sorted(names), errors


def write_font_face(font, path: Path, repaired_cmap: bytes | None, excluded_tables=()) -> None:
    """원본 SFNT 테이블을 복사한다. 글리프·메트릭·경계·날짜를 재컴파일하지 않는다."""
    from fontTools.ttLib.sfnt import SFNTWriter
    # TTFont.save는 읽어 둔 테이블을 재컴파일하고 CFF 전체의 bbox를 계산한다.
    # face 공급에는 directory/checksum 갱신과 명시한 cmap 종료 항목 수정만 필요하다.
    tags = sorted(tag for tag in font.reader.keys() if tag not in excluded_tables)
    with path.open('wb') as output:
        writer = SFNTWriter(output, len(tags), font.sfntVersion)
        for tag in tags:
            writer[tag] = repaired_cmap if tag == 'cmap' and repaired_cmap is not None else font.reader[tag]
        writer.close()


EMBEDDED_BITMAP_STRIKE_TAGS = frozenset({'EBDT', 'EBLC', 'EBSC'})


def embedded_bitmap_strike_tags(tags) -> frozenset:
    """외곽선 글꼴에 함께 든 흑백 비트맵 strike 테이블. 외곽선이 없으면 버리지 않는다.

    한/글 Print PDF와 Native PNG는 외곽선으로 그린다. 브라우저 canvas는 strike가 있는
    글꼴에서 그 비트맵을 고르고, HBATANG.TTF(한컴바탕, 12·14·16ppem strike)는 한글
    글자를 빈칸으로 그렸다 — 같은 글꼴의 Latin은 그려져 WASM 쪽만 글자가 사라졌다.
    """
    tags = set(tags)
    if not tags & {'glyf', 'CFF ', 'CFF2'}:
        return frozenset()
    return frozenset(tags & EMBEDDED_BITMAP_STRIKE_TAGS)


def repair_terminal_cmap(data: bytes, glyph_count: int) -> bytes | None:
    """SVG 공급과 같은 U+FFFF 종료 항목만 수정한다. 실제 문자 매핑은 보존한다."""
    if len(data) < 4:
        return None
    repaired = bytearray(data)
    changed = False
    table_count = struct.unpack_from('>H', data, 2)[0]
    for index in range(table_count):
        record = 4 + index * 8
        if record + 8 > len(data):
            return None
        offset = struct.unpack_from('>I', data, record + 4)[0]
        if offset + 14 > len(data) or struct.unpack_from('>H', data, offset)[0] != 4:
            continue
        length, _, doubled = struct.unpack_from('>HHH', data, offset + 2)
        segments = doubled // 2
        if doubled % 2 or segments < 1 or offset + length > len(data):
            continue
        end = offset + 14 + 2 * (segments - 1)
        start = offset + 16 + 2 * segments + 2 * (segments - 1)
        delta = start + 2 * segments
        range_offset = delta + 2 * segments
        if range_offset + 2 > offset + length:
            continue
        read = lambda position: struct.unpack_from('>H', data, position)[0]
        if read(end) == read(start) == 0xFFFF and read(range_offset) == 0 and ((0xFFFF + read(delta)) & 0xFFFF) >= glyph_count:
            struct.pack_into('>H', repaired, delta, 1)
            changed = True
    return bytes(repaired) if changed else None


def export_wasm_png(vs, root, hwp, package, base, selected, font_paths, environment, compat, dpi, workers):
    import sys
    catalogue = font_catalog(vs, root, font_paths, base)
    options = base / 'canvas-options.json'
    vs.write_json_atomic(options, {'pages': selected, 'workers': workers, 'scale': dpi / 96.0,
                                  'compat': compat, 'fontEnvironment': str(environment) if environment else None,
                                  'pythonExecutable': sys.executable})
    vs.run(['node', str(root / 'scripts/export-wasm-png-for-sweep.mjs'), '--pkg', str(package),
            '--input', str(hwp), '--out', str(base), '--fonts', str(catalogue), '--options', str(options)],
           cwd=root, log_path=base / 'wasm-canvas.log')
    metadata = vs.load_json_object(base / 'wasm-export-complete.json', 'WASM PNG export')
    count = metadata.get('pageCount')
    if type(count) is not int or count < 1 or metadata.get('layoutGeneration') != compat:
        raise SystemExit('WASM Canvas 전체 쪽수 또는 조판 세대가 올바르지 않습니다.')
    expected = set(selected or range(1, count + 1))
    if (metadata.get('rasterBackend') != 'wasm-canvas2d'
            or metadata.get('comparisonProfile') != 'print'
            or {item.get('page') for item in metadata.get('pages', [])} != expected
            or len(metadata.get('pages', [])) != len(expected)):
        raise SystemExit('WASM Canvas PNG 요청 쪽이 누락·중복되었습니다.')
    for folder, pattern in [('rhwp_png', '*.png'), ('render_tree', '*.json')]:
        if {vs.page_num(path) for path in (base / folder).glob(pattern)} != expected:
            raise SystemExit('WASM Canvas 실제 PNG/render tree 산출물이 누락됐습니다.')
    for item in metadata['pages']:
        path = base / 'rhwp_png' / f"rhwp_{item['page']:03d}.png"
        validate_png(path, item['pngSha256'], (item['width'], item['height']))


def validate_png(path: Path, digest=None, size=None) -> None:
    from PIL import Image
    if digest is not None and hashlib.sha256(path.read_bytes()).hexdigest() != digest:
        raise SystemExit('Canvas PNG manifest와 실제 파일 SHA가 다릅니다.')
    try:
        with Image.open(path) as image:
            if image.format != 'PNG' or (size is not None and image.size != size):
                raise ValueError('PNG 형식 또는 크기가 다릅니다.')
            image.verify()
    except (OSError, ValueError) as error:
        raise SystemExit(f'직접 PNG 산출물 손상: {path}: {error}') from error


def embedded_font_catalog(requests: Path, output: Path) -> Path:
    """WASM 문서가 제공한 바이트의 선택 TTC face·논리 family를 공급한다."""
    from types import SimpleNamespace
    entries = json.loads(requests.read_text(encoding='utf-8'))
    root = requests.parent
    adapter = SimpleNamespace(resolve_input_path=lambda root, path: Path(path),
                              sha256_file=lambda path: hashlib.sha256(path.read_bytes()).hexdigest(),
                              write_json_atomic=lambda path, value: path.write_text(json.dumps(value), encoding='utf-8'))
    catalogue = font_catalog(adapter, root, [root / 'source-fonts'], output)
    value = json.loads(catalogue.read_text(encoding='utf-8'))
    selected = []
    for face in value['faces']:
        matches = [entry for entry in entries if Path(entry['path']).resolve() == Path(face['source'])
                   and entry['faceIndex'] == face['faceIndex']]
        if not matches:
            continue
        face['families'] = sorted(set(face['families']) | {name for entry in matches for name in entry['families']})
        face['sourceKind'] = 'document-embedded'
        selected.append(face)
    if any(not any(face['source'] == str(Path(entry['path']).resolve())
                   and face['faceIndex'] == entry['faceIndex'] for face in selected) for entry in entries):
        raise SystemExit('문서 내장 글꼴의 선택 face를 공급할 수 없습니다.')
    value['faces'] = selected
    catalogue.write_text(json.dumps(value, ensure_ascii=False), encoding='utf-8')
    return catalogue


if __name__ == '__main__':
    import argparse
    parser = argparse.ArgumentParser(description='WASM 내장 글꼴의 브라우저 공급 catalogue')
    parser.add_argument('--font-requests', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    embedded_font_catalog(args.font_requests, args.out)
