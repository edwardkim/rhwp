"""실루엣 전용 검사의 쪽 묶음을 검증 가능한 단위로 이어받는다."""
from __future__ import annotations

import os
import shutil
from pathlib import Path

BATCH_PAGES = 64


def prepare_target_plan(vs, root, out_root, targets, selected_pages, *, resume,
                        rhwp_bin, dpi, compat, rasterizer, wasm_pkg, linked_images):
    """다중 대상의 시작 전·중단 후 목록과 입력을 고정한다."""
    plan = {
        'version': 1,
        'targets': [{
            'key': target.key,
            'source': str(vs.resolve_input_path(root, target.hwp)),
            'source_sha256': vs.sha256_file(vs.resolve_input_path(root, target.hwp)),
            'pdf': str(vs.resolve_input_path(root, target.pdf)),
            'pdf_sha256': vs.sha256_file(vs.resolve_input_path(root, target.pdf)),
        } for target in targets],
        'selection': selected_pages,
        'binary': vs.rhwp_binary_identifier(root, rhwp_bin),
        'dpi': dpi,
        'compat': compat,
        'rasterizer': rasterizer,
        'wasm': vs.wasm_package_provenance(root, wasm_pkg, rasterizer) if wasm_pkg else None,
        'linked_images': linked_images,
        'sweep_sha256': vs.sha256_file(Path(vs.__file__)),
        'implementation_sha256': vs.sha256_file(Path(__file__)),
    }
    path = out_root / 'silhouette-target-plan.json'
    if resume:
        if not path.is_file() or vs.load_json_object(path, '실루엣 대상 목록') != plan:
            raise SystemExit('--resume 대상 목록 또는 실행 조건이 기존 실행과 다릅니다.')
    else:
        vs.write_json_atomic(path, plan)


def render_silhouette_resumable(vs, root, target, out_root, rhwp_bin, dpi,
                                pixel_diff_threshold, selected_pages, *, resume,
                                **kwargs):
    base = out_root / vs.safe_target_key(target.key)
    checkpoint = base / 'silhouette-checkpoint.json'
    identity = {
        'version': 1,
        'key': target.key,
        'source': str(vs.resolve_input_path(root, target.hwp)),
        'source_sha256': vs.sha256_file(vs.resolve_input_path(root, target.hwp)),
        'pdf': str(vs.resolve_input_path(root, target.pdf)),
        'pdf_sha256': vs.sha256_file(vs.resolve_input_path(root, target.pdf)),
        'binary': vs.rhwp_binary_identifier(root, rhwp_bin),
        'sweep_sha256': vs.sha256_file(Path(vs.__file__)),
        'dpi': dpi,
        'pixel_diff_threshold': pixel_diff_threshold,
        'compat': kwargs.get('compat', '2022'),
        'rasterizer': kwargs.get('rasterizer', 'svg'),
        'selection': selected_pages,
        'batch_pages': BATCH_PAGES,
        'implementation_sha256': vs.sha256_file(Path(__file__)),
    }
    if resume:
        if not checkpoint.is_file():
            raise SystemExit(f'실루엣 체크포인트가 없습니다: {checkpoint}')
        if vs.load_json_object(checkpoint, '실루엣 체크포인트') != identity:
            raise SystemExit('--resume 실행 조건이 기존 실루엣 실행과 다릅니다.')
    else:
        vs.clean_dir(base)
        vs.write_json_atomic(checkpoint, identity)
    (base / 'silhouette.tsv').unlink(missing_ok=True)
    incomplete = {
        'key': target.key, 'mode': 'silhouette_only', 'run_state': 'incomplete',
        'pr_review_gate': {'status': 'not_evaluated', 'reason': '실루엣 쪽 묶음 처리 중'},
    }
    vs.write_json_atomic(base / 'silhouette_manifest.json', incomplete)
    vs.update_root_summary(out_root, incomplete)
    parts = base / 'silhouette-parts'
    parts.mkdir(parents=True, exist_ok=True)
    manifests = []

    def run_part(pages):
        key = f'p{pages[0]:04d}-{pages[-1]:04d}'
        part = parts / key
        done = part / 'silhouette_manifest.json'
        part_complete = False
        if resume and done.is_file():
            saved = vs.load_json_object(done, '실루엣 부분 결과')
            if saved.get('run_state') == 'complete':
                if (saved.get('completed_pages') != pages
                        or [item.get('page') for item in saved.get('inputs', [])] != pages):
                    raise SystemExit(f'실루엣 체크포인트의 완료 쪽이 다릅니다: {done}')
                for item in saved['inputs']:
                    for label in ('rhwp', 'pdf'):
                        path = Path(item[f'{label}_png'])
                        if not path.is_file() or vs.sha256_file(path) != item[f'{label}_sha256']:
                            raise SystemExit(f'실루엣 체크포인트 PNG가 바뀌었거나 없습니다: {path}')
                artifacts = saved.get('checkpoint_artifacts')
                if not isinstance(artifacts, list):
                    raise SystemExit(f'실루엣 체크포인트 산출물 정보가 없습니다: {done}')
                for item in artifacts:
                    path = Path(item['path'])
                    if not path.is_file() or vs.sha256_file(path) != item['sha256']:
                        raise SystemExit(f'실루엣 체크포인트 산출물이 바뀌었거나 없습니다: {path}')
                part_complete = True
        subtarget = vs.Target(key, target.hwp, target.pdf)
        # 기존 run manifest가 있을 때만 provenance를 검사하며 재개한다.
        part_resume = part_complete
        result = vs.render_target(root, subtarget, parts, rhwp_bin, dpi,
                                  pixel_diff_threshold, pages, resume=part_resume,
                                  silhouette_only=True, **kwargs)
        result['checkpoint_artifacts'] = [
            {'path': str(path), 'sha256': vs.sha256_file(path)}
            for folder, pattern in (('render_tree', '*.json'), ('svg', '*.svg'),
                                    ('analysis', 'note_shape.json'), ('.', 'run_manifest.json'),
                                    ('.', 'native-export.json'), ('.', 'wasm-export-complete.json'))
            for path in sorted((part / folder).glob(pattern))
        ]
        vs.write_json_atomic(done, result)
        manifests.append(result)
        return result

    if selected_pages is None:
        first = run_part([1])
        counts = first['pr_review_gate']['document_page_counts']
        total = counts['rhwp']
        remaining = list(range(2, total + 1))
    else:
        remaining = list(selected_pages)
        counts = None
    for offset in range(0, len(remaining), BATCH_PAGES):
        part = run_part(remaining[offset:offset + BATCH_PAGES])
        part_counts = part['pr_review_gate']['document_page_counts']
        if counts is None:
            counts = part_counts
        elif counts != part_counts:
            raise SystemExit('실루엣 부분 실행의 전체 쪽수가 일치하지 않습니다.')
    pairs = []
    for folder in ('rhwp_png', 'pdf_png', 'render_tree', 'svg'):
        (base / folder).mkdir(exist_ok=True)

    def publish(source, dest):
        dest.unlink(missing_ok=True)
        try:
            os.link(source, dest)
        except OSError:
            shutil.copy2(source, dest)

    for manifest in manifests:
        part = parts / manifest['key']
        for item in manifest['inputs']:
            page = item['page']
            rhwp_dest = base / 'rhwp_png' / f'rhwp_{page:03d}.png'
            pdf_dest = base / 'pdf_png' / f'pdf-{page:03d}.png'
            publish(Path(item['rhwp_png']), rhwp_dest)
            publish(Path(item['pdf_png']), pdf_dest)
            for folder, pattern in (('render_tree', f'render_tree_{page:03d}.json'),):
                source = part / folder / pattern
                if source.is_file():
                    publish(source, base / folder / pattern)
            pairs.append((page, rhwp_dest, pdf_dest))
        for source in (part / 'svg').glob('*.svg'):
            publish(source, base / 'svg' / source.name)
    pairs.sort(key=lambda item: item[0])
    result = vs.write_silhouette_tsv(
        pairs, base, target.key, document_counts=counts,
        workers=kwargs.get('page_workers', 1),
        excluded_regions=kwargs.get('excluded_regions'),
        exclusion_evidence=kwargs.get('exclusion_evidence'),
        resume=resume and (base / 'silhouette-metrics').is_dir(),
    )
    result['provenance'] = manifests[0]['provenance']
    result['checkpoint'] = str(checkpoint)
    metadata_name = ('wasm-export-complete.json' if kwargs.get('wasm_pkg') is not None
                     else 'native-export.json')
    records = []
    metadata = None
    for manifest in manifests:
        path = parts / manifest['key'] / metadata_name
        current = vs.load_json_object(path, '실루엣 내보내기 메타데이터')
        metadata = metadata or current
        for record in current.get('pages', []):
            record = dict(record)
            if metadata_name == 'native-export.json' and 'path' in record:
                page = record['page'] + 1
                record['path'] = str(base / 'rhwp_png' / f'rhwp_{page:03d}.png')
                record['renderTreePath'] = str(base / 'render_tree' / f'render_tree_{page:03d}.json')
            records.append(record)
    metadata['pages'] = sorted(records, key=lambda record: record['page'])
    if metadata_name == 'native-export.json':
        metadata['all_pages_exported'] = selected_pages is None
    vs.write_json_atomic(base / metadata_name, metadata)
    first_run = vs.load_json_object(parts / manifests[0]['key'] / 'run_manifest.json', '실루엣 실행 메타데이터')
    first_run.update({
        'key': target.key,
        'requested_pages': [page for page, _, _ in pairs],
        'run_state': 'complete',
        'document_page_counts': counts,
    })
    vs.write_json_atomic(base / 'run_manifest.json', first_run)
    vs.write_json_atomic(base / 'silhouette_manifest.json', result)
    vs.update_root_summary(out_root, result)
    return result
