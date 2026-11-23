from pathlib import Path
import hashlib, json, subprocess, time

R = Path('/home/tsjang/rhwp')
O = R / 'output/pr-review/issue7754-nine'
O.mkdir(parents=True, exist_ok=True)
M = R / 'mydocs/pr/assets/visual-protected-baseline/manifest.json'
sha = lambda data: hashlib.sha256(data).hexdigest()
manifest = json.loads(M.read_text())
reports = {}
records = []

def record_sha(row):
    return row.get('tsv_sha256') or row.get('head', {}).get('sha256')

for dataset in manifest['datasets']:
    source = dataset['source']
    path = source['condition_evidence']
    if path not in reports:
        raw = (R / path).read_bytes()
        expected = source['condition_evidence_sha256']
        origin = 'current tracked file'
        if sha(raw) != expected:
            raw = subprocess.check_output(['git', 'show', source['evidence_commit'] + ':' + path], cwd=R)
            origin = 'pinned evidence commit'
        assert sha(raw) == expected, (path, expected)
        reports[path] = (json.loads(raw), sha(raw), origin)
    report, report_sha, origin = reports[path]
    rows = report.get('conditions') or report.get('datasets')
    assert isinstance(rows, list), ('unknown condition report schema', path)
    matches = [r for r in rows if r.get('backend') == dataset['backend']
               and (r.get('document_id') == dataset['document_id']
                    or (r.get('document') or r.get('key') or r.get('logical_key')) == dataset['logical_key'])
               and record_sha(r) == source['sha256']]
    assert len(matches) == 1, (dataset['logical_key'], dataset['backend'], path, len(matches))
    row = matches[0]
    if 'font_environment_sha256' in row:
        field, digest = 'conditions.font_environment_sha256', row['font_environment_sha256']
    elif 'environment_sha256' in row.get('font_supply', {}):
        field, digest = 'conditions.font_supply.environment_sha256', row['font_supply']['environment_sha256']
    elif 'font_environment_sha256' in row.get('head', {}):
        field, digest = 'datasets.head.font_environment_sha256', row['head']['font_environment_sha256']
    elif isinstance(row.get('provenance'), dict) and 'comparison_profile' in row['provenance']:
        env = row['provenance'].get('font_environment')
        field = 'datasets.provenance.font_environment' if env else 'complete capture provenance: no explicit environment'
        digest = env.get('sha256') if env else None
    else:
        raise RuntimeError(('unresolved environment schema; never infer None', dataset['logical_key'], path))
    if digest: assert len(digest) == 64
    records.append({'key': dataset['logical_key'], 'backend': dataset['backend'],
        'document_id': dataset['document_id'], 'condition_evidence': path,
        'condition_evidence_sha256': report_sha, 'condition_source': origin,
        'accepted_tsv_sha256': source['sha256'], 'source_field': field,
        'expected_environment_sha256': digest, 'matching_files': []})

needed = {r['expected_environment_sha256'] for r in records if r['expected_environment_sha256']}
files = {}
previous = O / 'source33-protected-environment-resolution.json'
old = json.loads(previous.read_text()) if previous.is_file() else []
for row in old:
    for name in row.get('matching_files', []):
        p = R / name
        if p.is_file(): files.setdefault(sha(p.read_bytes()), []).append(name)

paths = subprocess.check_output(['git', 'ls-files', '-z', '--', ':(glob)mydocs/pr/assets/**/*.json'], cwd=R).decode().split('\0')
candidates = [name for name in paths if name and any(t in name.lower() for t in ['font', 'environment'])]
for name in candidates:
    p = R / name
    digest = sha(p.read_bytes())
    if digest in needed:
        files.setdefault(digest, []).append(name)

for folder in [R / 'output/pr-review/native-webfont-cache/environments', O / 'source33-recovered-font-environments']:
    if folder.is_dir():
        for p in folder.glob('*.json'):
            digest = sha(p.read_bytes())
            if digest in needed: files.setdefault(digest, []).append(str(p.relative_to(R)))

for row in records:
    digest = row['expected_environment_sha256']
    row['matching_files'] = list(dict.fromkeys(files.get(digest, []))) if digest else []
    row['resolved'] = digest is None or bool(row['matching_files'])

missing = [r for r in records if not r['resolved']]
changed = []
for row in records:
    prior = next((x for x in old if x['key'] == row['key'] and x['backend'] == row['backend']), None)
    if prior is not None and prior['expected_environment_sha256'] != row['expected_environment_sha256']:
        changed.append({'key': row['key'], 'backend': row['backend'],
            'old_expected_environment_sha256': prior['expected_environment_sha256'],
            'expected_environment_sha256': row['expected_environment_sha256'],
            'source_field': row['source_field']})
assert len(records) == len(manifest['datasets']) == 46
result = {'recorded': time.time(), 'manifest': str(M.relative_to(R)), 'manifest_sha256': sha(M.read_bytes()),
          'schema_coverage': 'Every accepted table SHA and pinned condition-evidence SHA checked. All known report layouts resolved; unknown layouts fail.',
          'datasets': records, 'changed_from_previous_resolution': changed,
          'missing': missing, 'all_conditions_resolved': not missing,
          'candidate_json_files_hashed': len(candidates)}
(O / 'source36-canonical-environment-resolution.json').write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n')
print('resolved schemas', len(records), 'changed', len(changed), 'missing exact environments', len(missing))
for row in changed: print('corrected', row['key'], row['backend'], row['source_field'])
for row in missing: print('missing', row['key'], row['backend'], row['expected_environment_sha256'])
