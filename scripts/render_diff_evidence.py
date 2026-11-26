#!/usr/bin/env python3
"""Index existing Render Diff and explicitly selected Print Visual Sweep results."""

import argparse
import csv
import hashlib
import json
import math
import re
import subprocess
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


def print_sweep(root, directory, allowed_commits):
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
            or not provenance.get('raster_backend') or data.get('missing_pages')
            or not pages or not file_exists(root / tsv)):
        return None
    if (not re.fullmatch(r'[0-9a-f]{40}', provenance.get('git_head', ''))
            or provenance['git_head'] not in allowed_commits
            or data.get('tsv_sha256') != digest(root / tsv)):
        return None
    if any(not re.fullmatch(r'[0-9a-f]{64}', provenance.get(name, {}).get('sha256', ''))
           for name in ('hwp', 'pdf')):
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
    if any(not isinstance(r.get(raw), (int, float)) or not 0 <= r[raw] <= 100 for r in data.get('metrics', [])):
        return None
    recorded_metrics = [(str(item['page']), f'{item[raw]:.2f}') for item in data.get('metrics', [])]
    if (tsv_metrics != recorded_metrics or [page for page, _ in tsv_metrics] != list(map(str, pages))
            or len(set(pages)) != len(pages) or min(pages) < 1):
        return None
    return data, manifest, tsv


def worst_page(rows, key=None, flag=None):
    candidates = []
    for item in rows:
        comparison = item.get(key) if key else item
        if not comparison:
            continue
        ratio = comparison.get('diffRatio')
        if type(ratio) not in (int, float) or not math.isfinite(ratio) or not 0 <= ratio <= 1:
            continue
        candidates.append((bool(flag and flag(item)), ratio, item, comparison))
    if not candidates:
        return None
    _, ratio, item, comparison = max(candidates, key=lambda entry: entry[:2])
    return {'document': item['fixture'], 'page': item['pageIndex'] + 1, 'diff_ratio': ratio,
            'diff_pixels': comparison.get('diffPixels'),
            'dimensions': [comparison.get(k) for k in ('referenceWidth','referenceHeight','pdfWidth','pdfHeight')]}


def review_point(worst, limit=None, failed=False):
    if not worst:
        return ''
    document = " ".join(worst['document'].splitlines())
    label = '실패 쪽 최대 픽셀 차이율' if failed else '최대 픽셀 차이율'
    text = f"{document} {worst['page']}쪽\n{label} {worst['diff_ratio']:.3%}"
    if limit is not None:
        text += f' (허용 {limit:.3%})'
        if worst['diff_ratio'] > limit:
            text += f" / {100 * (worst['diff_ratio'] - limit):.3f}%p 초과"
    return text


def checkout_identity(checkout, pr_head):
    cmd = ['git', '-C', str(checkout)]
    tested_sha = subprocess.check_output(cmd + ['rev-parse', '--verify', 'HEAD'], text=True).strip()
    # The raw commit headers retain both parents in a shallow checkout.
    raw_commit = subprocess.check_output(cmd + ['cat-file', 'commit', tested_sha])
    headers = raw_commit.split(b'\n\n', 1)[0].splitlines()
    tested_tree = next(line[5:].decode('ascii') for line in headers if line.startswith(b'tree '))
    parents = [line[7:].decode('ascii') for line in headers if line.startswith(b'parent ')]
    first_parent = parents[0] if len(parents) == 2 and parents[1] == pr_head else None
    return tested_sha, tested_tree, first_parent


def build_index(root, *, run_url=None, pr_head=None, tested_sha=None, sweep_dirs=(),
                pr_event_base=None, tested_tree=None, tested_merge_first_parent=None):
    lines = ['### Visual comparison evidence', '']
    if run_url:
        lines.append(f'- Actions attempt {code(run_url)}')
    for label, value in (
        ('PR head', pr_head),
        ('PR event base', pr_event_base),
        ('Tested checkout', tested_sha),
        ('Tested tree', tested_tree),
        ('Tested PR merge first parent', tested_merge_first_parent),
    ):
        if value:
            lines.append(f'- {label} {code(value)}')
    lines.extend([
        '', '| Comparison | Role | Recorded result | Compared documents and pages | Evidence |',
        '| --- | --- | --- | --- | --- |',
    ])
    files = {}
    comparisons = {}
    def comparison(key, name, role, result, point, priority, metrics, pages, paths):
        comparisons[key] = dict(name=name, role=role, result=result, point=point,
                                priority=priority, metrics=metrics, pages=list(pages),
                                evidence=[path for path in paths if file_exists(root / path)])


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

    canvas_limit = read_json(root / canvas).get('config', {}).get('maxDiffRatio')
    passed = sum(item.get('pass') is True for item in canvas_rows)
    failed = sum(item.get('pass') is False for item in canvas_rows)
    worst = worst_page(canvas_rows, flag=lambda r: r.get('pass') is False)
    total = len(canvas_rows)
    complete = total and passed + failed == total
    point = review_point(worst, canvas_limit, failed > 0)
    if worst and worst['diff_pixels'] is not None:
        point += f" / 차이 픽셀 {worst['diff_pixels']}개"
    comparison('canvas', 'Canvas 시각 비교', 'gate',
               (f'{failed}/{total}쪽 판정 실패' if failed else f'{passed}/{total}쪽 통과') if complete else '결과 미확인',
               point or 'results.json', 0 if failed else 3 if complete else 1,
               dict(passed=passed, failed=failed, total=total, worst=worst, limit=canvas_limit),
               [(r['fixture'], r['pageIndex']+1) for r in canvas_rows], [canvas, f'{DIFF}/summary.md'])

    warnings = sum(bool(r.get('exceedsReportThreshold')) for r in pdf_rows)
    errors = sum(bool(r.get('reportError') or not r.get('reportComparison')) for r in pdf_rows)
    compared = len(pdf_rows) - errors
    worst = worst_page(pdf_rows, 'reportComparison')
    point = review_point(worst)
    if worst and all(type(n) is int for n in worst['dimensions']):
        a,b,c,d = worst['dimensions']
        point += f'\n비교 크기 {a}×{b} ↔ {c}×{d}'
    result = f'참고 경고 {warnings}/{compared}쪽' if compared else '결과 미확인'
    if errors:
        result += f', 측정 오류 {errors}쪽'
    comparison('browser', '브라우저 ↔ 호환 PDF', 'reference', result, point or 'pdf-results.json',
               1 if errors or not compared else 3, dict(compared=compared, warnings=warnings, errors=errors, worst=worst),
               [(r['fixture'],r['pageIndex']+1) for r in pdf_rows if r.get('reportComparison')], [pdf,f'{DIFF}/pdf-summary.md'])

    limit = pdf_config.get('directMaxDiffRatio')
    worst = worst_page(direct_rows, 'directComparison', lambda r: r.get('directGateFailed'))
    total = len(direct_rows)
    direct_errors = total - direct_count
    failed = sum(bool(r.get('directComparison') and r.get('directGateFailed')) for r in direct_rows)
    if not pdf_config.get('directPdfEnabled'):
        result = '비활성' if pdf_report else '결과 미확인'
    elif not direct_count:
        result = '결과 미확인'
    elif not pdf_config.get('directPdfGate'):
        result = f'참고 비교 {direct_count}쪽'
    elif failed:
        exceeded = sum(bool(r.get('directGateFailed')) and r['directComparison']['diffRatio'] > limit
                       for r in direct_rows if r.get('directComparison') and limit is not None)
        result = f'{failed}/{total}쪽 기준 초과' if exceeded == failed else f'{failed}/{total}쪽 판정 실패'
    else:
        result = f'{direct_count}/{total}쪽 통과'
    if direct_errors:
        result += f', 측정 오류 {direct_errors}쪽'
    comparison('direct', '직접 PDF ↔ 호환 PDF', 'gate' if pdf_config.get('directPdfGate') else 'reference',
               result, review_point(worst, limit, failed > 0) or 'pdf-results.json', 0 if direct_failures else 2 if direct_count else 1,
               dict(compared=direct_count, total=total, failures=direct_failures, errors=direct_errors, limit=limit, worst=worst),
               [(r['fixture'],r['pageIndex']+1) for r in direct_rows if r.get('directComparison')], [pdf,f'{DIFF}/pdf-summary.md'])

    baseline_path = f'{READINESS}/baseline-report.json'
    baseline = read_json(root / baseline_path)
    diagnostics = baseline.get('performance', {}).get('canvaskitSurfaceDiagnostics', [])
    gpu = [r['usedGpuSurfaceCount'] for r in diagnostics if r.get('backend') == 'canvaskit-default'
           and r.get('profile') == 'screen' and type(r.get('usedGpuSurfaceCount')) is int]
    checks = readiness.get('checks', [])
    ids = {r['sampleId'] for r in checks}
    targets = {s['id'] for s in baseline.get('manifest',{}).get('samples',[]) if s.get('canvaskitReadinessGate')}
    absent = sorted(targets - ids)
    point = ' / '.join(absent[:3]) or '프로필 ' + readiness.get('criteria',{}).get('profile','screen')
    if gpu:
        point += f'\nGPU 사용 확인 {sum(gpu)}개'
    result = '결과 미확인'
    if counts.get('total'):
        result = f"{counts['passed']}/{counts['total']}개 통과"
        if counts['missing']:
            result += f", 누락 {counts['missing']}개"
        if counts['failed']:
            result += f", 실패 {counts['failed']}건"
    comparison('readiness', 'CanvasKit 전환 검사', 'readiness', result, point if counts else 'baseline-report.json',
               0 if counts.get('failed') else 1 if not counts or counts.get('missing') else 3,
               dict(**counts, missing_samples=absent, gpu_confirmed=sum(gpu) if gpu else None),
               readiness_scope, [browser,baseline_path,f'{READINESS}/baseline-report.md'])
    evidence(baseline_path)
    comparisons['readiness']['metrics']['active_backends'] = sorted({
        check['activeBackend'] for check in checks if check.get('activeBackend')})


    incomplete_sweeps = []
    for directory in sweep_dirs:
        sweep = print_sweep(root, directory, {pr_head, tested_sha})
        if sweep is None:
            incomplete_sweeps.append(directory)
            continue
        data, manifest, tsv = sweep
        provenance = data['provenance']
        metrics = data['metrics']
        raw = 'silhouette_raw_match_percent'
        worst = min(metrics, key=lambda r: r[raw])
        low = sum(r[raw] < 90 for r in metrics)
        backend_name = provenance['raster_backend']
        comparison('print:' + directory, 'Visual Sweep ' + backend_name, 'print',
                   f'90% 미달 {low}/{len(metrics)}쪽',
                   f"{provenance['hwp']['path']} {worst['page']}쪽\n최저 실루엣 일치율 {worst[raw]:.2f}%",
                   0 if low else 3, dict(below_90=low, measured_pages=len(metrics),
                   measured_commit=provenance['git_head'], backend=backend_name, source=provenance['hwp'],
                   oracle=provenance['pdf'], document_page_counts=data.get('pr_review_gate',{}).get('document_page_counts')),
                   [(provenance['hwp']['path'],page) for page in data['completed_pages']], [manifest,tsv])
        backend = code(backend_name)
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
    consumption_records = []
    for name in records:
        path = f'{BUILD}/consumption/{name}'
        record = read_json(root / path)
        consumption_records.append(dict(path=path, matched=record.get('matched')))
        status = {True: 'matched', False: 'mismatched'}.get(record.get('matched'), 'unreported')
        lines.append(f'- Browser WASM consumption ({status}) {evidence(path)}')
    native_path = f'{BUILD}/native-build.txt'
    native = evidence(native_path)
    native_build = {}
    if file_exists(root / native_path):
        for line in (root / native_path).read_bytes().decode().splitlines():
            key, _, value = line.partition(' ')
            if key == 'source_sha' or key == 'sha256':
                native_build[key] = value.split()[0]
    modes = sorted({item['rhwpMode'] for item in pdf_rows if item.get('rhwpMode')})
    lines.append(f"- Native PDF build record {native}. Execution mode {', '.join(map(code, modes)) or 'unreported'}.")

    if files:
        lines.extend(['', '#### Evidence paths and SHA-256', ''])
        lines.extend(f'- {code(path)} {code(checksum)}' for path, checksum in files.items())
    run = re.search(r'/actions/runs/(\d+)/attempts/(\d+)$', run_url or '')
    index = dict(schema_version=1, run_id=int(run[1]) if run else None,
                 run_attempt=int(run[2]) if run else None, native_build=native_build or None,
                 consumption=consumption_records, run_url=run_url, pr_head=pr_head, tested_sha=tested_sha,
                 comparisons=comparisons, files=files, build=build, incomplete_print_sweeps=incomplete_sweeps)
    if pr_event_base:
        index['pr_event_base'] = pr_event_base
    if tested_tree:
        index.update(tested_tree=tested_tree, tested_merge_first_parent=tested_merge_first_parent)
    # The Actions overview and PR comment both display these projection fields.
    overview = ['### PR 검증 결과', '', '| 검증 항목 | 결과 | 리뷰 포인트 |',
                '| --- | --- | --- |']
    order = ['direct', 'browser', 'canvas', 'readiness'] + [k for k in comparisons if k.startswith('print:')]
    for item in sorted((comparisons[k] for k in order), key=lambda r: r['priority']):
        point = '<br>'.join(code(line) for line in item['point'].splitlines())
        overview.append(f"| {code(item['name'])} | {code(item['result'])} | {point} |")
    return index, '\n'.join(overview + [''] + lines) + '\n'


def render_index(root, **options):
    return build_index(root, **options)[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('input', type=Path, help='result directory or Render Diff artifact ZIP')
    parser.add_argument('--run-url', help='Actions attempt URL')
    parser.add_argument('--pr-head', help='PR head SHA from Actions metadata')
    parser.add_argument('--print-sweep', action='append', default=[], metavar='DIR',
                        help='existing Print silhouette result directory, relative to input')
    checkout = parser.add_mutually_exclusive_group()
    checkout.add_argument('--tested-sha', help='recorded checkout SHA, for replay without Git objects')
    checkout.add_argument('--checkout', type=Path, help='Git checkout actually used by the measurement job')
    parser.add_argument('--pr-event-base', help='base SHA recorded in the PR event')
    parser.add_argument('--json-output', type=Path, help='write the versioned machine-readable evidence index')
    args = parser.parse_args()
    options = dict(run_url=args.run_url, pr_head=args.pr_head, sweep_dirs=args.print_sweep,
                   tested_sha=args.tested_sha, pr_event_base=args.pr_event_base)
    if args.checkout is not None:
        tested_sha, tree, first_parent = checkout_identity(args.checkout, args.pr_head)
        options.update(tested_sha=tested_sha, tested_tree=tree, tested_merge_first_parent=first_parent)
    if args.input.is_dir():
        index, markdown = build_index(args.input, **options)
    else:
        with ZipFile(args.input) as archive:
            index, markdown = build_index(ZipPath(archive), **options)
    if args.json_output:
        args.json_output.write_text(json.dumps(index, ensure_ascii=False, allow_nan=False) + '\n', encoding='utf-8')
    print(markdown, end='')


if __name__ == '__main__':
    main()
