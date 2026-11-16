#!/usr/bin/env python3
"""Summarize cold/warm/scroll/zoom producer traces; does not run a compositor."""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path

MAXIMUM = 9007199254740991
MAX_BYTES = 8 * 1024 * 1024
STAGES = ('cold', 'warm', 'scroll', 'zoom')
COUNTERS = ('cacheHits', 'cacheMisses', 'cacheEvictions', 'canonicalReplays',
            'viewportFallbacks', 'targetAdmissionFallbacks')
GAUGES = ('cachedTiles', 'cachedBytes', 'managedTargets', 'managedBytes')
STAMP_FIELDS = ('documentId', 'revision', 'fontGeneration', 'viewGeneration',
                'deviceGeneration', 'deviceScale')
SCOPE = 'publishedFramePostRenderSnapshot'


def integer(value):
    return type(value) is int and 0 <= value <= MAXIMUM


def stamp(value):
    if not isinstance(value, dict) or set(value) != set(STAMP_FIELDS):
        raise ValueError('producer sample needs the complete stamp')
    for key in STAMP_FIELDS[:-1]:
        if not integer(value[key]) or (key not in ('revision', 'fontGeneration') and value[key] == 0):
            raise ValueError('invalid producer stamp integer')
    scale = value['deviceScale']
    if type(scale) not in (int, float) or not math.isfinite(scale) or not 0 < scale <= 16:
        raise ValueError('invalid producer device scale')
    return value


def sample(value):
    if (not isinstance(value, dict) or type(value.get('schemaVersion')) is not int or value.get('schemaVersion') != 1 or
            value.get('scope') != SCOPE or value.get('members') not in (1, 2) or
            type(value.get('members')) is not int or value.get('backend') not in
            ('openGlGanesh', 'metalGanesh', 'd3d12Ganesh')):
        raise ValueError('unsupported producer diagnostic sample')
    stamp(value.get('stamp'))
    fields = value.get('counters')
    if not isinstance(fields, dict) or not fields:
        raise ValueError('producer diagnostics have no measured counters')
    if any(key not in COUNTERS + GAUGES or not integer(amount) for key, amount in fields.items()):
        raise ValueError('unknown or malformed producer counter')
    return value


def summarize(trace):
    if (not isinstance(trace, dict) or trace.get('kind') != 'rhwp-gpu-producer-trace' or
            type(trace.get('schemaVersion')) is not int or trace.get('schemaVersion') != 1 or trace.get('compositorDisplayVerified') is not False or
            trace.get('consumerGpuCompletionVerified') is not False):
        raise ValueError('expected a producer trace without display/consumer claims')
    events = trace.get('events')
    if not isinstance(events, list) or len(events) > 512:
        raise ValueError('invalid or unbounded producer events')
    samples, seen = [], set()
    for event in events:
        if not isinstance(event, dict):
            raise ValueError('invalid trace event')
        if event.get('stage') != 'producerReady':
            continue
        if not integer(event.get('generation')) or not integer(event.get('elapsedMicros')):
            raise ValueError('invalid producer milestone')
        if event.get('producerDiagnostics') is None:
            # Preserve an unknown publication so it cannot bridge measured
            # frames or make an older gauge look like the latest frame.
            samples.append(None)
            continue
        value = sample(event['producerDiagnostics'])
        identity = (event['generation'], value['backend'], value['members'],
                    tuple(value['stamp'][key] for key in STAMP_FIELDS))
        if identity not in seen:
            samples.append(value)
            seen.add(identity)
    result = {'scope': SCOPE, 'measuredSamples': sum(v is not None for v in samples), 'unknownCounterIncreases': [], 'unknownGauges': [],
              'counterIncreases': {}, 'lastCounters': {}, 'gauges': {}}
    last = samples[-1] if samples else None
    # A presenter replacement or another document invalidates cumulative deltas.
    epochs = {(v['backend'], v['members'], *(v['stamp'][key] for key in
              ('documentId', 'revision', 'fontGeneration', 'deviceGeneration'))) for v in samples if v is not None}
    for key in COUNTERS:
        if last is not None and key in last['counters']:
            result['lastCounters'][key] = last['counters'][key]
        values = [v['counters'].get(key) if v is not None else None for v in samples]
        if (len(values) >= 2 and len(epochs) == 1 and all(integer(v) for v in values) and
                all(a <= b for a, b in zip(values, values[1:]))):
            result['counterIncreases'][key] = values[-1] - values[0]
        else:
            result['unknownCounterIncreases'].append(key)
    for key in GAUGES:
        values = [v['counters'][key] for v in samples if v is not None and key in v['counters']]
        if values:
            result['gauges'][key] = {'measuredSamples': len(values), 'observedMin': min(values),
                                     'observedMax': max(values)}
            if last is not None and key in last['counters']:
                result['gauges'][key]['lastPublishedFrame'] = last['counters'][key]
        else:
            result['unknownGauges'].append(key)
    timing = trace.get('requestToProducerReadyMicros', {})
    if (isinstance(timing, dict) and timing.get('scope') == 'retainedCompletedSamples' and
            timing.get('method') == 'nearestRank' and integer(timing.get('retainedSamples')) and
            timing['retainedSamples'] > 0):
        percentiles = {key: timing[key] for key in ('p50', 'p95', 'p99') if integer(timing.get(key))}
        if percentiles:
            result['requestToProducerReadyMicros'] = {'scope': 'retainedCompletedSamples',
                'includes': 'controller capture/RPC/producer/publication/status; not GPU-only',
                'retainedSamples': timing['retainedSamples'], **percentiles}
    return result


def compare(traces):
    if set(traces) != set(STAGES):
        raise ValueError('provide exactly cold, warm, scroll and zoom traces')
    return {'schemaVersion': 1, 'kind': 'rhwp-gpu-cache-benchmark',
            'measurementScope': SCOPE, 'caseLabelsVerifiedByCollector': False,
            'cases': {name: summarize(traces[name]) for name in STAGES},
            'drawReplayComposeTimingsAvailable': False, 'compositorDisplayVerified': False,
            'consumerGpuCompletionVerified': False, 'physicalGpuPerformanceVerified': False}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for stage in STAGES:
        parser.add_argument('--' + stage, type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    traces, hashes = {}, {}
    for stage in STAGES:
        path = getattr(args, stage)
        with path.open('rb') as source:
            data = source.read(MAX_BYTES + 1)
        if len(data) > MAX_BYTES:
            parser.error('trace exceeds 8 MiB bound')
        traces[stage] = json.loads(data)
        hashes[stage] = hashlib.sha256(data).hexdigest()
    result = compare(traces)
    result['inputSha256'] = hashes
    # Avoid replacing an earlier benchmark or an input trace by accident.
    descriptor = os.open(args.output, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(descriptor, 'w', encoding='utf-8') as target:
        json.dump(result, target, indent=2)
        target.write('\n')
    args.output.chmod(0o600)
    print(args.output)


if __name__ == '__main__':
    main()
