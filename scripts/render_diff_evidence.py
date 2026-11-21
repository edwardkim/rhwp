#!/usr/bin/env python3
"""Index existing Render Diff and explicitly selected Print Visual Sweep results."""

import argparse
import csv
import hashlib
import json
from collections import defaultdict
from html import escape
from itertools import groupby
from pathlib import Path, PurePosixPath
from zipfile import Path as ZipPath, ZipFile


DIFF = 'rhwp-studio/e2e/screenshots/render-diff'
READINESS = 'output/renderer-baseline/canvaskit-readiness'
BUILD = 'output/render-diff-build'


def code(value):
    escaped = escape(" ".join(str(value).splitlines())).replace("|", "&#124;")
    return f"<code>{escaped}</code>"


def digest(path):
    sha = hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b''):
            sha.update(chunk)
    return sha.hexdigest()


def file_exists(path):
    # Python 3.9 zipfile.Path.is_file() accepts missing names.
    return path.exists() and path.is_file()


def read_json(path):
    return json.loads(path.read_bytes()) if file_exists(path) else {}


def page_range(pages):
    spans = []
    for _, group in groupby(enumerate(sorted(set(pages))), lambda pair: pair[1] - pair[0]):
        span = [page for _, page in group]
        spans.append(str(span[0]) if len(span) == 1 else f'{span[0]}–{span[-1]}')
    return ', '.join(spans)


def scope(rows):
    pages = defaultdict(list)
    for document, page in rows:
        pages[document].append(page)
    return '<br>'.join(f'{code(doc)} p{page_range(items)}' for doc, items in pages.items()) or 'No pages recorded'


def fixture_pages(rows, comparison):
    return scope((row['fixture'], row['pageIndex'] + 1) for row in rows if row.get(comparison))


def result_counts(rows, passed):
    if not rows:
        return 'No results recorded'
    passes = sum(row.get(passed) is True for row in rows)
    failures = sum(row.get(passed) is False for row in rows)
    return f'{passes} passed, {failures} failed, {len(rows) - passes - failures} unreported'


def print_sweep(root, directory):
    # The producer writes these siblings. Recorded absolute paths from a runner
    # are not portable, and a PNG-pair diagnostic has no Print provenance.
    path = PurePosixPath(directory)
    if path.is_absolute() or '..' in path.parts:
        raise ValueError(f'Print Sweep directory must be relative to the evidence root: {directory}')
    manifest = (path / 'silhouette_manifest.json').as_posix()
    tsv = (path / 'silhouette.tsv').as_posix()
    data = read_json(root / manifest)
    provenance = data.get('provenance', {})
    pages = data.get('completed_pages', [])
    if (data.get('run_state') != 'complete' or data.get('mode') != 'silhouette_only'
            or provenance.get('comparison_profile') != 'print'
            or not pages or not file_exists(root / tsv)):
        return None
    if any(not provenance.get(name, {}).get(field) for name in ('hwp', 'pdf')
           for field in ('path', 'sha256')):
        return None
    raw = 'silhouette_raw_match_percent'
    with (root / tsv).open('r', encoding='utf-8') as stream:
        reader = csv.DictReader(stream, delimiter='\t')
        if not {'page', raw}.issubset(reader.fieldnames or []):
            return None
        tsv_metrics = [(item['page'], item[raw]) for item in reader]
    recorded_metrics = [(str(item['page']), f'{item[raw]:.2f}') for item in data.get('metrics', [])]
    if (tsv_metrics != recorded_metrics or [page for page, _ in tsv_metrics] != list(map(str, pages))
            or len(set(pages)) != len(pages) or min(pages) < 1):
        return None
    return data, manifest, tsv


def render_index(root, *, run_url=None, pr_head=None, sweep_dirs=()):
    lines = ['### Visual comparison evidence', '']
    if run_url:
        lines.append(f'- Actions attempt {code(run_url)}')
    if pr_head:
        lines.append(f'- PR head {code(pr_head)}')
    lines.extend([
        '', '| Comparison | Role | Recorded result | Compared documents and pages | Evidence |',
        '| --- | --- | --- | --- | --- |',
    ])
    files = {}

    def evidence(*paths):
        present = [path for path in paths if file_exists(root / path)]
        for path in present:
            if path not in files:
                files[path] = digest(root / path)
        return ', '.join(code(path) for path in present) or 'Unavailable'

    def row(kind, role, status, measured, *paths):
        lines.append(f'| {kind} | {role} | {status} | {measured} | {evidence(*paths)} |')

    canvas = f'{DIFF}/results.json'
    canvas_rows = read_json(root / canvas).get('results', [])
    row('Canvas legacy vs layer', 'Render Diff gate', result_counts(canvas_rows, 'pass'),
        scope((item['fixture'], item['pageIndex'] + 1) for item in canvas_rows),
        canvas, f'{DIFF}/summary.md', 'output/e2e/canvas-render-diff-report.html')

    pdf = f'{DIFF}/pdf-results.json'
    pdf_report = read_json(root / pdf)
    pdf_rows = pdf_report.get('results', [])
    pdf_config = pdf_report.get('config', {})
    notes = 'No results recorded'
    if pdf_rows:
        warnings = sum(bool(item.get('exceedsReportThreshold')) for item in pdf_rows)
        errors = sum(bool(item.get('reportError')) for item in pdf_rows)
        notes = f'{warnings} warnings, {errors} errors'
    row('Canvas vs compatibility PDF raster', 'Reference', notes,
        fixture_pages(pdf_rows, 'reportComparison'), pdf, f'{DIFF}/pdf-summary.md',
        'output/e2e/pdf-render-diff-report.mjs-report.html')
    direct_rows = [item for item in pdf_rows if item.get('directExpected')]
    direct_count = sum(bool(item.get('directComparison')) for item in direct_rows)
    direct_failures = sum(bool(item.get('directError') or not item.get('directComparison')
                               or item.get('directGateFailed')) for item in direct_rows)
    direct_status = 'Unavailable'
    if pdf_config.get('directPdfEnabled'):
        direct_status = f'{direct_count}/{len(direct_rows)} compared, {direct_failures} failures'
        if pdf_config.get('directPdfGate') and not direct_rows:
            direct_status += '. Gate requires at least one page.'
    elif pdf_report:
        direct_status = 'Disabled'
    row('Direct vs compatibility PDF raster', 'Render Diff gate' if pdf_config.get('directPdfGate')
        else 'Reference' if pdf_report else 'Unavailable', direct_status,
        fixture_pages(direct_rows, 'directComparison'), pdf, f'{DIFF}/pdf-summary.md')

    browser = f'{READINESS}/browser/browser-baseline-report.json'
    browser_report = read_json(root / browser)
    readiness = browser_report.get('canvaskitReadinessGate') or {}
    readiness_scope = [
        (item['file'], item['page'] + 1) for item in browser_report.get('results', [])
        if item.get('readinessGateRequired') is True
    ]
    counts = readiness.get('summary', {})
    readiness_status = result_counts(readiness.get('checks', []), 'passed')
    if counts:
        readiness_status = f"{counts['passed']} passed, {counts['failed']} failed, {counts['missing']} missing"
    row('CanvasKit readiness (screen)', 'Readiness gate', readiness_status, scope(readiness_scope),
        browser, f'{READINESS}/baseline-report.md')

    incomplete_sweeps = []
    for directory in sweep_dirs:
        sweep = print_sweep(root, directory)
        if sweep is None:
            incomplete_sweeps.append(directory)
            continue
        data, manifest, tsv = sweep
        provenance = data['provenance']
        backend = code(provenance.get('raster_backend', 'unreported'))
        status = code(data.get('pr_review_gate', {}).get('status', 'unreported'))
        measured = scope((provenance['hwp']['path'], page) for page in data['completed_pages'])
        reference = code(provenance['pdf']['path'])
        row(f'Visual Sweep (Print, {backend})', 'Visual Sweep PR review evidence', status,
            f'{measured} vs {reference}', manifest, tsv)
    if not sweep_dirs:
        lines.extend(['', 'Visual Sweep evidence was not supplied for this run.'])
    elif incomplete_sweeps:
        lines.extend(['', 'Complete Print manifest and matching TSV unavailable for '
                      + ', '.join(code(directory) for directory in incomplete_sweeps) + '.'])

    lines.extend(['', '#### Build and execution records', ''])
    build_json = f'{BUILD}/build.json'
    build = read_json(root / build_json)
    if build:
        lines.append(f"- WASM build source {code(build.get('source_sha', 'unreported'))}, "
                     f"dirty {code(build.get('source_dirty', 'unreported'))}, "
                     f"profile {code(build.get('profile', 'unreported'))}, "
                     f"success {code(build.get('success', 'unreported'))}. {evidence(build_json)}")
        for name, artifact in build.get('artifacts', {}).items():
            lines.append(f"  - {code(name)} SHA-256 {code(artifact['sha256'])}")
    else:
        lines.append('- WASM build record unavailable.')
    consumption = root / f'{BUILD}/consumption'
    records = sorted(
        (item.name for item in consumption.iterdir() if item.is_file() and item.name.endswith('.json'))
    ) if consumption.is_dir() else []
    for name in records:
        path = f'{BUILD}/consumption/{name}'
        record = read_json(root / path)
        status = {True: 'matched', False: 'mismatched'}.get(record.get('matched'), 'unreported')
        lines.append(f'- Browser WASM consumption ({status}) {evidence(path)}')
    native = evidence(f'{BUILD}/native-build.txt')
    modes = sorted({item['rhwpMode'] for item in pdf_rows if item.get('rhwpMode')})
    lines.append(f"- Native PDF build record {native}. Execution mode {', '.join(map(code, modes)) or 'unreported'}.")

    if files:
        lines.extend(['', '#### Evidence paths and SHA-256', ''])
        lines.extend(f'- {code(path)} {code(checksum)}' for path, checksum in files.items())
    return '\n'.join(lines) + '\n'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('input', type=Path, help='result directory or Render Diff artifact ZIP')
    parser.add_argument('--run-url', help='Actions attempt URL')
    parser.add_argument('--pr-head', help='PR head SHA from Actions metadata')
    parser.add_argument('--print-sweep', action='append', default=[], metavar='DIR',
                        help='existing Print silhouette result directory, relative to input')
    args = parser.parse_args()
    options = {'run_url': args.run_url, 'pr_head': args.pr_head, 'sweep_dirs': args.print_sweep}
    if args.input.is_dir():
        index = render_index(args.input, **options)
    else:
        with ZipFile(args.input) as archive:
            index = render_index(ZipPath(archive), **options)
    print(index, end='')


if __name__ == '__main__':
    main()
