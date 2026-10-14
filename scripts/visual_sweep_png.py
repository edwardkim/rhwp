"""Visual Sweep 직접 PNG 생산과 공급 글꼴의 파일 기반 catalogue."""
from __future__ import annotations

import hashlib
import json
import shutil
import struct
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


def font_catalog(vs, root: Path, font_paths, base: Path) -> Path:
    """실제 name/weight/face-index를 기록한다. 글꼴을 SVG마다 복제하지 않는다."""
    from fontTools.ttLib import TTFont, TTCollection
    records = []
    seen = set()
    resource_dir = base / 'font_resources'
    for directory in font_paths:
        directory = vs.resolve_input_path(root, directory)
        for source in sorted(directory.rglob('*')):
            if not source.is_file() or source.suffix.lower() not in {'.ttf', '.otf', '.ttc', '.woff', '.woff2'}:
                continue
            digest = vs.sha256_file(source)
            if digest in seen:
                continue
            seen.add(digest)
            collection = TTCollection(source, lazy=True) if source.suffix.lower() == '.ttc' else None
            fonts = collection.fonts if collection else [TTFont(source, lazy=True)]
            try:
                for index, font in enumerate(fonts):
                    names = sorted({entry.toUnicode() for entry in font['name'].names
                                    if entry.nameID in {1, 4, 16} and entry.toUnicode()})
                    weight = font['OS/2'].usWeightClass if 'OS/2' in font else 400
                    italic = bool(font['head'].macStyle & 2) if 'head' in font else False
                    path = source
                    repaired_cmap = repair_terminal_cmap(font.getTableData('cmap'), font['maxp'].numGlyphs) if 'cmap' in font else None
                    if repaired_cmap is not None:
                        from fontTools.ttLib.tables.DefaultTable import DefaultTable
                        table = DefaultTable('cmap')
                        table.data = repaired_cmap
                        font['cmap'] = table
                    if collection or repaired_cmap is not None:
                        resource_dir.mkdir(exist_ok=True)
                        path = resource_dir / f'{digest}-{index}.ttf'
                        font.save(path)
                    records.append({'path': str(path.resolve()), 'source': str(source.resolve()),
                                    'sourceSha256': digest, 'sha256': vs.sha256_file(path),
                                    'faceIndex': index, 'families': names, 'weight': weight,
                                    'style': 'italic' if italic else 'normal',
                                    'repairs': ['cmap-format4-terminal-notdef'] if repaired_cmap is not None else []})
            finally:
                if collection:
                    collection.close()
                else:
                    fonts[0].close()
    path = base / 'font-catalog.json'
    vs.write_json_atomic(path, {'faces': records})
    return path


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
                              write_json_atomic=lambda path, value: path.write_text(json.dumps(value)))
    catalogue = font_catalog(adapter, root, [root / 'source-fonts'], output)
    value = json.loads(catalogue.read_text())
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
    catalogue.write_text(json.dumps(value, ensure_ascii=False))
    return catalogue


if __name__ == '__main__':
    import argparse
    parser = argparse.ArgumentParser(description='WASM 내장 글꼴의 브라우저 공급 catalogue')
    parser.add_argument('--font-requests', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    embedded_font_catalog(args.font_requests, args.out)
