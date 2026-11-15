"""공개 웹폰트로 재현한 보호 문서의 Native/fresh WASM 공통 검사."""
from __future__ import annotations

import argparse
import csv
import json
import subprocess
import sys
from decimal import Decimal
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
from scripts import visual_sweep as sweep
from scripts import visual_sweep_handbook as handbook
from scripts.visual_sweep_resume import prepare_target_plan

CATALOG = Path('scripts/data/contributor_visual_gate.json')
DOCUMENT_IDS = {'52ee61fe7201e36b', '59f6ab1155623bda',
                'cdacd874b6e6ac66', 'd03a9dde3d5dc7ac'}
BACKENDS = {'native': 'native-skia', 'wasm': 'wasm-canvas2d'}
METRIC = handbook.METRIC


def load_catalog(root):
    """정본과 동일한 입력·PDF를 사용하며 누락된 문서로 범위를 줄이지 않는다."""
    catalog = json.loads((root / CATALOG).read_text(encoding='utf-8'))
    if (catalog['schema_version'] != 1
            or catalog['baseline'] != str(handbook.BASELINE)
            or catalog['baseline_sha256'] != handbook.BASELINE_SHA
            or handbook.sha(root / handbook.BASELINE) != handbook.BASELINE_SHA):
        raise ValueError('공통 검사와 정본 보호 TSV의 신원이 다릅니다')
    documents = catalog['documents']
    if len(documents) != len(DOCUMENT_IDS) or {d['id'] for d in documents} != DOCUMENT_IDS:
        raise ValueError('공통 검사는 등록된 4개 문서 전체가 필요합니다')
    with (root / handbook.BASELINE).open(encoding='utf-8', newline='') as stream:
        rows = list(csv.DictReader(stream, delimiter='\t'))
    for doc in documents:
        for label in ('input', 'pdf'):
            path = root / doc[label]['path']
            if handbook.sha(path) != doc[label]['sha256']:
                raise ValueError(f'공통 검사 {label} 해시가 다릅니다: {path}')
        doc['baseline'] = {}
        for backend in BACKENDS:
            selected = [r for r in rows if r['document_id'] == doc['id'] and r['backend'] == backend]
            if (len(selected) != doc['pages']
                    or {int(r['page']) for r in selected} != set(range(1, doc['pages'] + 1))
                    or any(r['input_path'] != doc['input']['path']
                           or r['input_sha256'] != doc['input']['sha256']
                           or r['reference_pdf_sha256'] != doc['pdf']['sha256']
                           or r['compat'] != doc['compat'] or r['dpi'] != '96' for r in selected)):
                raise ValueError(f'공통 검사와 정본의 문서·측정 조건이 다릅니다: {doc["id"]}')
            doc['baseline'][backend] = {int(r['page']): Decimal(r[METRIC]) for r in selected}
    return documents


def compare(doc, backend, manifest):
    """같은 원본의 전쪽 raw 표시값을 해당 backend의 정본과 비교한다."""
    p = manifest['provenance']
    if (manifest['run_state'] != 'complete' or manifest.get('missing_pages')
            or p['hwp']['sha256'] != doc['input']['sha256']
            or p['pdf']['sha256'] != doc['pdf']['sha256']
            or p['raster_backend'] != BACKENDS[backend]
            or p['layout_generation'] != doc['compat']
            or p['comparison_profile'] != 'print'
            or p['pdf_rasterizer']['policy'] != handbook.POLICY
            or manifest['radius_px'] != 2 or manifest['silhouette_method'] != handbook.METHOD
            or manifest.get('excluded_regions')
            or manifest['pr_review_gate']['document_page_counts'] !=
            {'rhwp': doc['pages'], 'pdf': doc['pages']}):
        raise ValueError('실행 미완료 또는 입력·backend·페이지·측정 조건이 다릅니다')
    values = {}
    for row in manifest['metrics']:
        page, raw = int(row['page']), Decimal(str(row[METRIC]))
        if page in values or not raw.is_finite() or not 0 <= raw <= 100:
            raise ValueError('중복 쪽 또는 유효하지 않은 점수입니다')
        values[page] = raw
    if set(values) != set(doc['baseline'][backend]):
        raise ValueError('등록된 전체 페이지의 점수가 필요합니다')
    losses = [{'page': page, 'baseline': str(doc['baseline'][backend][page]),
               'candidate': f'{float(raw):.2f}'} for page, raw in values.items()
              if raw < 90 or Decimal(f'{float(raw):.2f}') < doc['baseline'][backend][page]]
    return {'status': 're_review_required' if losses else 'passed',
            'minimum': str(min(values.values())), 'regressions': losses}


def render_command(root, out, doc, backend, binary, package, workers, fonts):
    """공식 checkpoint를 준비하고 실루엣 CLI의 --resume 경로로 실행한다."""
    target = sweep.Target('document', Path(doc['input']['path']), Path(doc['pdf']['path']))
    plan = out / 'silhouette-target-plan.json'
    if not plan.exists():
        if out.exists() and any(out.iterdir()):
            raise ValueError(f'실행계획이 없는 기존 디렉터리는 재사용하지 않습니다: {out}')
        out.mkdir(parents=True, exist_ok=True)
        prepare_target_plan(sweep, root, out, [target], None, resume=False,
                            rhwp_bin=binary, dpi=96, compat=doc['compat'], rasterizer='png',
                            wasm_pkg=package if backend == 'wasm' else None, linked_images=[])
    command = [sys.executable, str(root / 'scripts/visual_sweep.py'),
               '--silhouette-only', '--resume', '--hwp', str(target.hwp), '--pdf', str(target.pdf),
               '--key', target.key, '--rhwp-bin', binary, '--compat', doc['compat'],
               '--dpi', '96', '--page-workers', str(workers), '--out', str(out)]
    if backend == 'wasm':
        command += ['--wasm-pkg', str(package)]
    for font in fonts:
        command += ['--font-path', str(font)]
    return command


def make_reviews(root, doc, backend, base, manifest):
    """측정한 PNG를 재사용하며 대표 review와 overlay를 함께 만든다."""
    record = min(manifest['metrics'], key=lambda r: Decimal(str(r[METRIC])))
    page = int(record['page'])
    pair = next(r for r in manifest['inputs'] if r['page'] == page)
    images = []
    for label in ('rhwp', 'pdf'):
        path = Path(pair[f'{label}_png'])
        if handbook.sha(path) != pair[f'{label}_sha256']:
            raise ValueError(f'측정한 PNG가 변경되었습니다: {path}')
        images.append(path)
    for name in ('compare', 'overlay', 'review'):
        (base / name).mkdir(exist_ok=True)
    label = f'{Path(doc["input"]["path"]).name} ({backend})'
    compares = sweep.make_compares([images[0]], [images[1]], base / 'compare', label)
    overlays = sweep.make_overlay_compares([images[0]], [images[1]], base / 'overlay', label,
                                          pixel_diff_threshold=sweep.DEFAULT_PIXEL_DIFF_THRESHOLD)
    if f'{overlays["metrics"][0][METRIC]:.2f}' != f'{float(record[METRIC]):.2f}':
        raise ValueError('대표 이미지와 TSV 점수가 다릅니다')
    reviews = sweep.make_review_panels(compares, overlays['pages'], overlays['metrics'], base / 'review')
    return {'page': page, 'review': str(reviews[0]), 'overlay': str(overlays['pages'][0])}


def run(root, out, binary, package, workers, fonts):
    """실패한 실행이나 과거 결과를 통과로 집계하지 않고 8개 결과를 모두 남긴다."""
    out.mkdir(parents=True, exist_ok=True)
    report_path = out / 'contributor-gate.json'
    report = {'status': 're_review_required', 'baseline': str(handbook.BASELINE),
              'baseline_sha256': handbook.BASELINE_SHA, 'results': []}
    sweep.write_json_atomic(report_path, report)
    try:
        documents = load_catalog(root)
        report['catalog_sha256'] = handbook.sha(root / CATALOG)
        report['script_sha256'] = handbook.sha(Path(__file__))
        report['binary'] = sweep.rhwp_binary_identifier(root, binary)
        report['wasm'] = sweep.wasm_package_provenance(root, package)
        for doc in documents:
            for backend in BACKENDS:
                base = out / doc['id'] / backend
                entry = {'document_id': doc['id'], 'input': doc['input']['path'], 'backend': backend,
                         'status': 're_review_required', 'log': str(base.with_suffix('.log'))}
                Path(entry['log']).parent.mkdir(parents=True, exist_ok=True)
                try:
                    command = render_command(root, base, doc, backend, binary, package, workers, fonts)
                    entry['command'] = command
                    with Path(entry['log']).open('w', encoding='utf-8') as log:
                        result = subprocess.run(command, cwd=root, stdout=log, stderr=subprocess.STDOUT)
                    entry['exit_code'] = result.returncode
                    if result.returncode != 0:
                        raise ValueError(f'Visual Sweep 실행 실패: {result.returncode}')
                    manifest_path = base / 'document/silhouette_manifest.json'
                    manifest = json.loads(manifest_path.read_text(encoding='utf-8'))
                    checkpoint = json.loads((manifest_path.parent / 'silhouette-checkpoint.json')
                                            .read_text(encoding='utf-8'))
                    if checkpoint['dpi'] != 96 or checkpoint['selection'] is not None:
                        raise ValueError('96 DPI 전쪽 실행 조건이 아닙니다')
                    entry.update(compare(doc, backend, manifest))
                    entry['manifest'] = str(manifest_path)
                    entry['tsv'] = str(manifest_path.with_name('silhouette.tsv'))
                    entry['images'] = make_reviews(root, doc, backend, base, manifest)
                except (Exception, SystemExit) as error:
                    entry.update(status='re_review_required', error=str(error))
                    with Path(entry['log']).open('a', encoding='utf-8') as log:
                        log.write(f'공통 검사 실패: {error}\n')
                report['results'].append(entry)
                sweep.write_json_atomic(report_path, report)
                print(f'{doc["id"]} {backend}: {entry["status"]}; 로그: {entry["log"]}', flush=True)
        if len(report['results']) == 8 and all(r['status'] == 'passed' for r in report['results']):
            report['status'] = 'passed'
    except (Exception, SystemExit) as error:
        report['error'] = str(error)
    sweep.write_json_atomic(report_path, report)
    print(f'기여자 공통 검사: {report_path} ({report["status"]})', flush=True)
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--rhwp-bin', required=True, help='검증할 source의 Native Skia 실행 파일')
    parser.add_argument('--wasm-pkg', required=True, type=Path, help='같은 source의 fresh 최적화 WASM package')
    parser.add_argument('--out', required=True, type=Path, help='ignored output 아래 전용 결과 디렉터리')
    parser.add_argument('--page-workers', type=int, default=4)
    parser.add_argument('--font-path', type=Path, action='append', default=[])
    args = parser.parse_args()
    out = (ROOT / args.out).resolve()
    if not out.is_relative_to(ROOT / 'output') or args.page_workers < 1:
        parser.error('--out은 output 하위여야 하며 page-workers는 1 이상이어야 합니다')
    report = run(ROOT, out, args.rhwp_bin, (ROOT / args.wasm_pkg).resolve(),
                 args.page_workers, args.font_path)
    raise SystemExit(0 if report['status'] == 'passed' else 1)


if __name__ == '__main__':
    main()
