import copy
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('cache_benchmark', HERE.parent / 'tool/gpu_cache_benchmark.py')
collector = importlib.util.module_from_spec(spec)
spec.loader.exec_module(collector)
STAMP = dict(documentId=1, revision=0, fontGeneration=0, viewGeneration=1,
             deviceGeneration=2, deviceScale=1.25)


def trace():
    events = []
    for index, hits in enumerate((1, 4, 9)):
        events.append({'stage': 'producerReady', 'generation': index + 1, 'elapsedMicros': index * 10,
          'producerDiagnostics': {'schemaVersion': 1, 'scope': collector.SCOPE, 'backend': 'metalGanesh',
            'members': 1, 'stamp': dict(STAMP, viewGeneration=index + 1),
            'counters': {'cacheHits': hits, 'cacheMisses': 2, 'cachedBytes': index * 512}}})
    return dict(schemaVersion=1, kind='rhwp-gpu-producer-trace', events=events,
                compositorDisplayVerified=False, consumerGpuCompletionVerified=False)


class CacheBenchmarkContracts(unittest.TestCase):
    def test_measured_increases_and_gauge_scope(self):
        result = collector.summarize(trace())
        self.assertEqual(result['counterIncreases'], {'cacheHits': 8, 'cacheMisses': 0})
        self.assertEqual(result['lastCounters'], {'cacheHits': 9, 'cacheMisses': 2})
        self.assertEqual(result['gauges']['cachedBytes']['lastPublishedFrame'], 1024)
        self.assertIn('managedTargets', result['unknownGauges'])
        self.assertNotIn('managedTargets', result['gauges'])

    def test_unknown_is_not_reported_as_zero(self):
        value = trace()
        value['events'] = []
        result = collector.summarize(value)
        self.assertEqual(result['counterIncreases'], {})
        self.assertEqual(result['gauges'], {})

    def test_missing_middle_field_does_not_prove_a_complete_delta(self):
        value = trace()
        del value['events'][1]['producerDiagnostics']['counters']['cacheHits']
        self.assertNotIn('cacheHits', collector.summarize(value)['counterIncreases'])

    def test_missing_middle_diagnostics_do_not_bridge_an_unknown_frame(self):
        for explicit_null in (False, True):
            with self.subTest(explicit_null=explicit_null):
                value = trace()
                if explicit_null:
                    value['events'][1]['producerDiagnostics'] = None
                else:
                    del value['events'][1]['producerDiagnostics']
                result = collector.summarize(value)
                self.assertEqual(result['measuredSamples'], 2)
                self.assertEqual(result['counterIncreases'], {})
                self.assertEqual(result['lastCounters']['cacheHits'], 9)
                self.assertEqual(result['gauges']['cachedBytes']['lastPublishedFrame'], 1024)

    def test_missing_last_diagnostics_do_not_label_an_older_frame_as_latest(self):
        value = trace()
        del value['events'][-1]['producerDiagnostics']
        result = collector.summarize(value)
        self.assertEqual(result['counterIncreases'], {})
        self.assertEqual(result['lastCounters'], {})
        self.assertEqual(result['gauges']['cachedBytes']['observedMax'], 512)
        self.assertNotIn('lastPublishedFrame', result['gauges']['cachedBytes'])

    def test_deduplication_does_not_hide_a_changed_producer_epoch(self):
        for change in [{'backend': 'd3d12Ganesh'}, {'members': 2}]:
            with self.subTest(change=change):
                value = trace()
                replacement = copy.deepcopy(value['events'][0])
                replacement['producerDiagnostics'].update(change)
                value['events'].insert(1, replacement)
                result = collector.summarize(value)
                self.assertEqual(result['measuredSamples'], 4)
                self.assertEqual(result['counterIncreases'], {})

    def test_reset_or_another_presenter_does_not_produce_negative_deltas(self):
        value = trace()
        value['events'][-1]['producerDiagnostics']['counters']['cacheHits'] = 0
        result = collector.summarize(value)
        self.assertNotIn('cacheHits', result['counterIncreases'])
        self.assertEqual(result['lastCounters']['cacheHits'], 0)
        for field in ['deviceGeneration', 'revision', 'fontGeneration', 'documentId']:
            value = trace()
            value['events'][-1]['producerDiagnostics']['stamp'][field] += 1
            self.assertEqual(collector.summarize(value)['counterIncreases'], {})

    def test_single_cold_frame_preserves_measured_totals_without_inventing_deltas(self):
        value = trace(); value['events'] = value['events'][:1]
        result = collector.summarize(value)
        self.assertEqual(result['lastCounters'], {'cacheHits': 1, 'cacheMisses': 2})
        self.assertEqual(result['counterIncreases'], {})

    def test_only_ready_events_and_unique_requests_count(self):
        value = trace()
        value['events'].append(copy.deepcopy(value['events'][0]))
        value['events'].append(dict(value['events'][0], stage='statusPolled'))
        self.assertEqual(collector.summarize(value)['measuredSamples'], 3)

    def test_bad_stamps_and_counter_types_are_rejected(self):
        edits = [lambda v: v['stamp'].pop('fontGeneration'),
                 lambda v: v['stamp'].update(deviceScale=True),
                 lambda v: v['counters'].update(cacheHits=True),
                 lambda v: v['counters'].update(cacheHits=-1),
                 lambda v: v['counters'].update(cacheHits=2**53),
                 lambda v: v.update(members=True), lambda v: v.update(backend='cpu'),
                 lambda v: v.update(scope='driverVram')]
        for edit in edits:
            value = trace(); edit(value['events'][0]['producerDiagnostics'])
            with self.assertRaises(ValueError): collector.summarize(value)

    def test_consumer_or_display_claim_does_not_qualify(self):
        for name in ['compositorDisplayVerified', 'consumerGpuCompletionVerified']:
            value = trace(); value[name] = True
            with self.assertRaises(ValueError): collector.summarize(value)

    def test_cases_are_explicit_and_missing_phases_fail(self):
        values = {name: trace() for name in collector.STAGES}
        result = collector.compare(values)
        self.assertFalse(result['physicalGpuPerformanceVerified'])
        self.assertFalse(result['drawReplayComposeTimingsAvailable'])
        del values['zoom']
        with self.assertRaises(ValueError): collector.compare(values)

    def test_actual_cli_json_and_existing_output_protection(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary); args = [sys.executable, '-B', str(HERE.parent / 'tool/gpu_cache_benchmark.py')]
            for name in collector.STAGES:
                path = root / (name + '.json'); path.write_text(json.dumps(trace()))
                args += ['--' + name, str(path)]
            output = root / 'result.json'; args += ['--output', str(output)]
            result = subprocess.run(args, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            value = json.loads(output.read_text())
            self.assertEqual(len(value['inputSha256']), 4)
            self.assertEqual(value['cases']['warm']['counterIncreases']['cacheHits'], 8)
            original = output.read_bytes()
            self.assertNotEqual(subprocess.run(args, capture_output=True).returncode, 0)
            self.assertEqual(output.read_bytes(), original)


if __name__ == '__main__': unittest.main()
