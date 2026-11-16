"""Pure-Python unit tests for P22 qualification-campaign helpers."""
from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from qualification_corpus import FAMILIES, bounded_maximum_size_mesh
from run_qualification_campaign import (
    SEAM_ANALYTIC_FIXTURE_IDS,
    SKIP_INVOCATION_FAMILIES,
    ablation_slots,
    absent_timing,
    b1_geometric_ablation,
    b1_invoked_from_records,
    campaign_status,
    cpu_wall_timing,
    create_only_dir,
    denominator_summary,
    fixture_verdict,
    geometry_example_driver_support,
    geometry_exe_plan,
    geometry_invocation_args,
    known_bad_verdict,
    parse_family_filters,
    run_campaign,
    whole_surface_bound_from_existing_receipts,
)

CORPUS = Path(__file__).resolve().parents[2] / 'docs' / 'production-core' / 'corpus'


def _fixture(fid, family, **oracle):
    return dict(
        id=fid,
        family=family,
        partition='holdout' if family == 'holdout' else 'test',
        oracle=oracle,
        exclusions=[],
    )


class FamilyFilterTests(unittest.TestCase):
    def test_none_means_all(self):
        self.assertIsNone(parse_family_filters(None))
        self.assertIsNone(parse_family_filters([]))

    def test_comma_and_repeat(self):
        self.assertEqual(parse_family_filters(['identity,holdout', 'known_bad']), [
            'identity', 'holdout', 'known_bad',
        ])

    def test_unknown_family_fails_closed(self):
        with self.assertRaises(ValueError):
            parse_family_filters(['not_a_family'])


class GeometryPlanTests(unittest.TestCase):
    def test_supported_seam_analytic_ids(self):
        self.assertIsNone(geometry_invocation_args('identity-two-subpart-seam'))
        seam = geometry_invocation_args('identity-two-subpart-seam', corpus=CORPUS)
        self.assertEqual(seam[0], '--corpus-json')
        self.assertTrue(seam[1].endswith('identity-two-subpart-seam.json'))
        plane = geometry_invocation_args('analytic-plane-scale-1', corpus=CORPUS)
        self.assertEqual(plane[0], '--corpus-json')
        self.assertTrue(plane[1].endswith('analytic-plane-scale-1.json'))
        self.assertEqual(geometry_invocation_args('empty-mesh'), ['--empty'])
        self.assertEqual(geometry_invocation_args('impossible-ratio'), ['--target-faces', '0'])

    def test_unsupported_analytic_and_holdout_are_not_invoked(self):
        self.assertIsNone(geometry_invocation_args('analytic-sphere-octahedron'))
        self.assertIsNone(geometry_invocation_args('analytic-plane-scale-1e-06'))
        self.assertIsNone(geometry_invocation_args('holdout-rotated-plane'))
        self.assertIsNone(geometry_invocation_args('known-bad-cross-domain-weld'))
        self.assertIsNone(geometry_invocation_args('maximum-size-descriptor'))
        self.assertIsNone(geometry_invocation_args(
            'known-bad-cross-domain-weld',
            fixture=dict(family='known_bad'),
            driver='geometric',
            supports_driver=True,
            corpus=CORPUS,
        ))

    def test_driver_geometric_appended_only_when_example_parses_flag(self):
        without = geometry_invocation_args(
            'identity-two-subpart-seam', corpus=CORPUS, driver='geometric', supports_driver=False,
        )
        self.assertEqual(without[0], '--corpus-json')
        self.assertNotIn('--driver', without)
        with_flag = geometry_invocation_args(
            'identity-two-subpart-seam', corpus=CORPUS, driver='geometric', supports_driver=True,
        )
        self.assertEqual(with_flag[-2:], ['--driver', 'geometric'])
        empty = geometry_invocation_args('empty-mesh', driver='geometric', supports_driver=True)
        self.assertEqual(empty, ['--empty'])

    def test_inspect_only_never_invokes(self):
        plan = geometry_exe_plan(
            _fixture('identity-two-subpart-seam', 'identity', must_pass=True),
            inspect_only=True,
            geometry_exe_present=True,
        )
        self.assertEqual(plan['action'], 'skip')
        self.assertEqual(plan['execution_status'], 'not_executed')
        self.assertTrue(plan['in_denominator'])

    def test_missing_exe_records_not_executed(self):
        plan = geometry_exe_plan(
            _fixture('uv-duplicated-seam', 'uv_seam_tangents'),
            inspect_only=False,
            geometry_exe_present=False,
        )
        self.assertEqual(plan['action'], 'skip')
        self.assertEqual(plan['execution_status'], 'not_executed')

    def test_maximum_size_is_descriptor_only(self):
        plan = geometry_exe_plan(
            _fixture('maximum-size-descriptor', 'maximum_size_descriptor'),
            inspect_only=False,
            geometry_exe_present=True,
        )
        self.assertEqual(plan['action'], 'descriptor_only')
        self.assertEqual(plan['execution_status'], 'descriptor_only')
        self.assertIn('50k', plan['reason'])

    def test_holdout_labelled_and_not_used_for_thresholds(self):
        plan = geometry_exe_plan(
            _fixture('holdout-rotated-plane', 'holdout'),
            inspect_only=False,
            geometry_exe_present=True,
        )
        self.assertTrue(plan['holdout'])
        self.assertFalse(plan['used_for_threshold_selection'])
        self.assertEqual(plan['action'], 'skip')
        self.assertTrue(plan['in_denominator'])

    def test_invoke_plan_for_supported_seam(self):
        plan = geometry_exe_plan(
            _fixture('identity-two-subpart-seam', 'identity', must_pass=True),
            inspect_only=False,
            geometry_exe_present=True,
            corpus=CORPUS,
            fixture_file='identity-two-subpart-seam.json',
        )
        self.assertEqual(plan['action'], 'invoke')
        self.assertEqual(plan['extra_args'][0], '--corpus-json')
        self.assertTrue(plan['extra_args'][1].endswith('identity-two-subpart-seam.json'))
        self.assertTrue(plan['consumes_corpus_bytes'])


class KnownBadTests(unittest.TestCase):
    def setUp(self):
        self.case = _fixture(
            'known-bad-cross-domain-weld',
            'known_bad',
            must_fail=True,
            pass_is_corpus_defect=True,
            failing_tests=['forbidden_semantic_merges', 'no_cross_domain_support'],
        )

    def test_passing_exe_is_not_corpus_success(self):
        verdict = known_bad_verdict(
            self.case,
            execution_status='executed',
            exe_passed=True,
            observed_failed_gates=(),
        )
        self.assertTrue(verdict['must_fail'])
        self.assertFalse(verdict['corpus_success'])
        self.assertTrue(verdict['in_denominator'])
        self.assertIn('passing executable is a corpus defect', verdict['reason'])

    def test_named_gates_failed_meets_must_fail_contract(self):
        verdict = known_bad_verdict(
            self.case,
            execution_status='executed',
            exe_passed=True,
            observed_failed_gates=[
                'forbidden_semantic_merges',
                'no_cross_domain_support',
            ],
        )
        self.assertTrue(verdict['named_gates_failed'])
        self.assertTrue(verdict['corpus_success'])

    def test_not_executed_stays_in_denominator(self):
        verdict = fixture_verdict(
            self.case,
            execution_status='not_executed',
            exe_passed=False,
            observed_failed_gates=(),
            consumed_corpus_bytes=False,
        )
        self.assertFalse(verdict['corpus_success'])
        self.assertTrue(verdict['in_denominator'])
        self.assertFalse(verdict['qualified'])

    def test_oracle_observed_gates_meet_must_fail_without_exe(self):
        verdict = known_bad_verdict(
            self.case,
            execution_status='not_executed',
            exe_passed=False,
            observed_failed_gates=[
                'forbidden_semantic_merges',
                'no_cross_domain_support',
            ],
        )
        self.assertTrue(verdict['named_gates_failed'])
        self.assertTrue(verdict['corpus_success'])
        self.assertFalse(verdict['qualified'])
        self.assertFalse(verdict['production_exe_invoked'])
        self.assertTrue(verdict['in_denominator'])

    def test_unobserved_gate_is_not_fake_fail(self):
        verdict = known_bad_verdict(
            self.case,
            execution_status='not_executed',
            exe_passed=False,
            observed_failed_gates=(),
            unevaluated_gates=['forbidden_semantic_merges', 'no_cross_domain_support'],
        )
        self.assertFalse(verdict['named_gates_failed'])
        self.assertFalse(verdict['corpus_success'])
        self.assertEqual(verdict['observed_failed_gates'], [])
        self.assertIn('unobserved', verdict['reason'])


class AblationAndTimingTests(unittest.TestCase):
    def test_unimplemented_ablations_are_not_executed_not_zero(self):
        slots = ablation_slots(b2_executed=False)
        self.assertEqual([item['id'] for item in slots], ['B0', 'B1', 'B2', 'B3', 'B4', 'B5'])
        for slot in slots:
            self.assertEqual(slot['execution_status'], 'not_executed')
            self.assertIsNone(slot['error'])
            self.assertIsNone(slot['cost'])
            self.assertIsNone(slot['timing'])
            self.assertIsNone(slot['improvement'])
            self.assertFalse(slot.get('gpu', False))
            self.assertFalse(slot['qualified'])
            self.assertNotEqual(slot['error'], 0)
            self.assertNotEqual(slot['cost'], 0)

    def test_b0_record_is_inspect_cpu_not_qualified(self):
        b0 = dict(
            id='B0',
            execution_status='executed',
            compared=True,
            qualified=False,
            error=None,
            cost=None,
            gpu=False,
            identity_preserved=True,
            production_no_reduction_path='not_invoked',
        )
        slots = {item['id']: item for item in ablation_slots(b2_executed=False, b0_record=b0)}
        self.assertEqual(slots['B0']['execution_status'], 'executed')
        self.assertFalse(slots['B0']['qualified'])
        self.assertFalse(slots['B0']['gpu'])
        self.assertIsNone(slots['B0']['error'])
        self.assertEqual(slots['B1']['execution_status'], 'not_executed')
        self.assertEqual(slots['B1']['planned_argv'], ['--driver', 'geometric'])
        self.assertFalse(slots['B1']['driver_flag_in_example_source'])

    def test_b2_executed_is_not_qualified_and_not_zeroed(self):
        slots = {item['id']: item for item in ablation_slots(b2_executed=True)}
        self.assertEqual(slots['B2']['execution_status'], 'executed')
        self.assertFalse(slots['B2']['qualified'])
        self.assertIsNone(slots['B2']['error'])
        self.assertEqual(slots['B2']['timing']['kind'], 'cpu_wall')
        self.assertFalse(slots['B2']['timing']['gpu'])
        self.assertFalse(slots['B2']['timing']['synthetic'])
        self.assertEqual(slots['B1']['execution_status'], 'not_executed')

    def test_b1_stays_not_executed_until_driver_flag_exists(self):
        slot = b1_geometric_ablation(supports_driver=False, invoked=False)
        self.assertEqual(slot['execution_status'], 'not_executed')
        self.assertFalse(slot['qualified'])
        self.assertFalse(slot['gpu'])
        self.assertIsNone(slot['error'])
        self.assertIsNone(slot['cost'])
        self.assertEqual(slot['planned_argv'], ['--driver', 'geometric'])
        self.assertFalse(slot['driver_flag_in_example_source'])
        self.assertFalse(slot['production_exe_invoked'])
        present = geometry_example_driver_support({'sources': {'example': {'required_snippets': {'--driver': False}}}})
        self.assertFalse(present['driver_flag_in_example_source'])
        self.assertFalse(present['callable'])
        live = geometry_example_driver_support({'sources': {'example': {'required_snippets': {'--driver': True}}}})
        self.assertTrue(live['driver_flag_in_example_source'])
        still_not = b1_geometric_ablation(supports_driver=True, invoked=False)
        self.assertEqual(still_not['execution_status'], 'not_executed')
        ran = b1_geometric_ablation(supports_driver=True, invoked=True)
        self.assertEqual(ran['execution_status'], 'executed')
        self.assertFalse(ran['qualified'])
        self.assertIsNone(ran['error'])
        self.assertIsNone(ran['cost'])
        self.assertTrue(ran['production_exe_invoked'])
        self.assertEqual(ran['timing']['kind'], 'cpu_wall')
        self.assertFalse(ran['timing']['gpu'])
        self.assertFalse(b1_invoked_from_records([]))
        self.assertFalse(b1_invoked_from_records([
            {'execution_status': 'executed', 'geometry_invocation': {'argv': ['exe', '--corpus-json', 'x.json']}},
        ]))
        self.assertTrue(b1_invoked_from_records([
            {
                'execution_status': 'executed',
                'geometry_invocation': {'argv': ['exe', '--out', 'd', '--corpus-json', 'x.json', '--driver', 'geometric']},
            },
        ]))

    def test_timing_labels_cpu_never_gpu_synthetic(self):
        cpu = cpu_wall_timing(0.04)
        self.assertEqual(cpu['kind'], 'cpu_wall')
        self.assertFalse(cpu['gpu'])
        self.assertFalse(cpu['synthetic'])
        missing = absent_timing()
        self.assertEqual(missing['kind'], 'not_executed')
        self.assertIsNone(missing['wall_seconds'])
        self.assertFalse(missing['gpu'])


class DenominatorTests(unittest.TestCase):
    def test_failures_holdout_and_exclusions_remain(self):
        records = [
            dict(execution_status='executed', in_denominator=True, holdout=False, must_fail=False, exclusions=[]),
            dict(execution_status='not_executed', in_denominator=True, holdout=True, must_fail=False, exclusions=[]),
            dict(execution_status='not_executed', in_denominator=True, holdout=False, must_fail=True, exclusions=[]),
            dict(execution_status='descriptor_only', in_denominator=True, holdout=False, must_fail=False, exclusions=['descriptor']),
            dict(execution_status='execution_failed', in_denominator=True, holdout=False, must_fail=False, exclusions=[]),
        ]
        summary = denominator_summary(records)
        self.assertEqual(summary['selected'], 5)
        self.assertEqual(summary['in_denominator'], 5)
        self.assertTrue(summary['failures_and_exclusions_in_denominator'])
        self.assertFalse(summary['holdout_used_for_threshold_selection'])
        self.assertEqual(summary['holdout'], 1)
        self.assertEqual(summary['must_fail'], 1)
        self.assertEqual(summary['descriptor_only'], 1)

    def test_campaign_status_never_qualified(self):
        self.assertEqual(campaign_status(True, []), 'inspected_not_executed')
        self.assertEqual(campaign_status(False, [dict(execution_status='not_executed')]), 'recorded_not_executed')
        self.assertEqual(
            campaign_status(False, [dict(execution_status='executed')]),
            'executed_partial_not_qualified',
        )


class CampaignRunnerTests(unittest.TestCase):
    def test_create_only_out(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / 'campaign-01'
            create_only_dir(path)
            with self.assertRaises(FileExistsError):
                create_only_dir(path)

    def test_inspect_only_real_corpus_does_not_invoke_or_materialize(self):
        if not (CORPUS / 'manifest.json').is_file():
            self.skipTest('published corpus missing')
        invoked = []

        def boom(*args, **kwargs):
            invoked.append((args, kwargs))
            raise AssertionError('geometry exe must not run in inspect-only')

        materialize = mock.patch(
            'qualification_corpus.bounded_maximum_size_mesh',
            side_effect=AssertionError('must not materialize 50k mesh'),
        )
        with tempfile.TemporaryDirectory() as tmp, materialize:
            out = Path(tmp) / 'inspect-01'
            receipt = run_campaign(
                corpus=CORPUS,
                out=out,
                inspect_only=True,
                geometry_exe=Path(tmp) / 'missing-geometry.exe',
                invoke_geometry=boom,
            )
            self.assertTrue((out / 'qualification-receipt.json').is_file())
        self.assertEqual(invoked, [])
        self.assertEqual(receipt['status'], 'inspected_not_executed')
        self.assertFalse(receipt['invented_passing_geometry_receipt'])
        self.assertFalse(receipt['gpu_identical_timing_claimed'])
        self.assertFalse(receipt['maximum_size_materialized'])
        self.assertFalse(receipt['holdout_used_for_threshold_selection'])
        self.assertEqual(len(receipt['fixtures']), 37)
        self.assertEqual(receipt['denominator']['in_denominator'], 37)
        by_id = {item['id']: item for item in receipt['fixtures']}
        for item in receipt['fixtures']:
            self.assertTrue(item['in_denominator'])
            self.assertNotEqual(item['execution_status'], 'executed')
            self.assertFalse(item['used_for_threshold_selection'])
            self.assertFalse(item['timing']['gpu'])
            self.assertFalse(item['timing']['synthetic'])
            if item['family'] != 'known_bad':
                self.assertFalse(item['corpus_success'])
        self.assertEqual(by_id['maximum-size-descriptor']['execution_status'], 'descriptor_only')
        self.assertFalse(by_id['maximum-size-descriptor']['materialized'])
        self.assertIsNone(by_id['maximum-size-descriptor']['vertex_count'])
        self.assertTrue(by_id['holdout-rotated-plane']['holdout'])
        weld = by_id['known-bad-cross-domain-weld']
        self.assertTrue(weld['must_fail'])
        self.assertEqual(weld['execution_status'], 'not_executed')
        self.assertIsNone(weld.get('geometry_invocation'))
        self.assertFalse(weld.get('qualified', False))
        self.assertTrue(weld['corpus_success'])
        self.assertTrue(weld['verdict']['named_gates_failed'])
        self.assertFalse(weld['verdict']['production_exe_invoked'])
        self.assertEqual(set(weld['observed_failed_gates']), set(weld['verdict']['failing_tests']))
        affine = by_id['known-bad-poisoned-affine']
        self.assertTrue(affine['corpus_success'])
        self.assertIn('independent_affine_oracle', affine['observed_failed_gates'])
        self.assertEqual(by_id['known-bad-tetra-inversion']['observed_failed_gates'], ['accept_when_inversion_forbidden'])
        contact = by_id['known-bad-infeasible-contact']
        self.assertIn('contact_feasibility', contact['observed_failed_gates'])
        self.assertIn('authoritative_full_model_required', contact['observed_failed_gates'])
        ablations = {item['id']: item for item in receipt['ablations']}
        self.assertEqual(ablations['B0']['execution_status'], 'executed')
        self.assertFalse(ablations['B0']['qualified'])
        self.assertFalse(ablations['B0']['gpu'])
        self.assertIsNone(ablations['B0']['error'])
        self.assertEqual(ablations['B0']['production_no_reduction_path'], 'not_invoked')
        self.assertTrue(ablations['B0']['identity_preserved'])
        for slot_id in ('B1', 'B2', 'B3', 'B4', 'B5'):
            self.assertEqual(ablations[slot_id]['execution_status'], 'not_executed')
            self.assertIsNone(ablations[slot_id]['error'])
        self.assertEqual(ablations['B1']['planned_argv'], ['--driver', 'geometric'])
        self.assertEqual(
            ablations['B1']['driver_flag_in_example_source'],
            receipt['driver_support']['driver_flag_in_example_source'],
        )
        self.assertFalse(ablations['B1']['production_exe_invoked'])
        self.assertEqual(
            receipt['driver_support']['callable'],
            bool(receipt['geometry_exe']['present'] and receipt['driver_support']['driver_flag_in_example_source']),
        )
        self.assertEqual(receipt['collapse_provenance']['status'], 'not_supplied')
        self.assertIsNone(receipt['collapse_provenance']['sha256'])
        self.assertEqual(by_id['identity-two-subpart-seam']['independent_decode']['status'], 'not_executed')
        self.assertEqual(receipt['whole_surface_bound']['status'], 'not_measured')
        self.assertIsNone(receipt['whole_surface_bound']['class'])
        self.assertFalse(receipt['whole_surface_bound']['qualified'])
        self.assertNotEqual(receipt['status'], 'qualified')

    def test_family_filter_keeps_selected_failures_in_denominator(self):
        if not (CORPUS / 'manifest.json').is_file():
            self.skipTest('published corpus missing')
        with tempfile.TemporaryDirectory() as tmp:
            out = Path(tmp) / 'filter-01'
            receipt = run_campaign(
                corpus=CORPUS,
                out=out,
                inspect_only=True,
                families=['known_bad,holdout,maximum_size_descriptor'],
                geometry_exe=Path(tmp) / 'missing.exe',
            )
        families = {item['family'] for item in receipt['fixtures']}
        self.assertEqual(families, {'known_bad', 'holdout', 'maximum_size_descriptor'})
        self.assertEqual(receipt['denominator']['in_denominator'], len(receipt['fixtures']))
        self.assertTrue(receipt['denominator']['failures_and_exclusions_in_denominator'])
        self.assertGreaterEqual(receipt['denominator']['must_fail'], 4)
        self.assertGreaterEqual(receipt['denominator']['holdout'], 4)
        ablations = {item['id']: item for item in receipt['ablations']}
        self.assertEqual(ablations['B0']['execution_status'], 'not_executed')
        self.assertIsNone(ablations['B0']['error'])
        for item in receipt['fixtures']:
            self.assertIn(item['family'], SKIP_INVOCATION_FAMILIES)
            self.assertNotEqual(item['execution_status'], 'executed')

    def test_fake_invoke_uses_unique_create_only_child_and_cpu_timing(self):
        if not (CORPUS / 'manifest.json').is_file():
            self.skipTest('published corpus missing')

        def fake_invoke(exe, out_dir, extra_args=None, timeout=600):
            extra_args = list(extra_args or [])
            out_dir = Path(out_dir)
            out_dir.mkdir(parents=False, exist_ok=False)
            negative = extra_args[:1] in (['--empty'], ['--target-faces'])
            consumes = '--corpus-json' in extra_args
            fixture_name = 'normalized-tilted-two-chart-grid'
            if consumes:
                fixture_name = Path(extra_args[extra_args.index('--corpus-json') + 1]).stem
            payload = dict(
                status='invalid_input' if negative else 'passed',
                fixture=dict(
                    name=fixture_name,
                    consumes_corpus_json=consumes,
                    source='corpus_json' if consumes else 'builtin',
                ),
                timings_ms=dict(admitted_decimate=1.5, geometry_probe_total=2.5),
                limitations=['no GPU stage in this executable'],
            )
            (out_dir / 'geometry-receipt.json').write_text(
                json.dumps(payload) + '\n', encoding='utf-8',
            )
            return dict(
                label='admitted-geometry',
                argv=[str(exe), '--out', str(out_dir), *(extra_args or [])],
                status='executed',
                exit_code=2 if negative else 0,
                wall_seconds=0.04,
                stdout='{}',
                stderr='',
                qualified=False,
                note='Execution is not qualification. GPU timing is not produced by this executable.',
            )

        with tempfile.TemporaryDirectory() as tmp:
            fake_exe = Path(tmp) / 'production_core_geometry.exe'
            fake_exe.write_bytes(b'fake')
            out = Path(tmp) / 'invoke-01'
            receipt = run_campaign(
                corpus=CORPUS,
                out=out,
                inspect_only=False,
                families=['identity', 'empty_impossible_degenerate', 'known_bad'],
                geometry_exe=fake_exe,
                invoke_geometry=fake_invoke,
            )
            self.assertTrue((out / 'geometry-identity-two-subpart-seam' / 'geometry-receipt.json').is_file())
            self.assertTrue((out / 'geometry-identity-two-subpart-seam.stdout.txt').is_file())
        by_id = {item['id']: item for item in receipt['fixtures']}
        seam = by_id['identity-two-subpart-seam']
        self.assertEqual(seam['execution_status'], 'executed')
        self.assertEqual(seam['timing']['kind'], 'cpu_wall')
        self.assertFalse(seam['timing']['gpu'])
        self.assertTrue(seam['consumes_corpus_bytes'])
        self.assertTrue(seam['corpus_success'])
        self.assertFalse(seam.get('qualified', False))
        empty = by_id['empty-mesh']
        self.assertEqual(empty['geometry_invocation']['extra_args'], ['--empty'])
        self.assertEqual(empty['execution_status'], 'executed')
        bad = by_id['known-bad-cross-domain-weld']
        self.assertEqual(bad['execution_status'], 'not_executed')
        self.assertTrue(bad['must_fail'])
        self.assertTrue(bad['corpus_success'])
        self.assertFalse(bad.get('qualified', False))
        self.assertIsNone(bad.get('geometry_invocation'))
        self.assertTrue(bad['verdict']['named_gates_failed'])
        self.assertFalse(bad['verdict']['production_exe_invoked'])
        self.assertGreaterEqual(len(bad['observed_failed_gates']), 5)
        affine = by_id['identity-affine-k1']
        self.assertEqual(affine['execution_status'], 'not_executed')
        b2 = {item['id']: item for item in receipt['ablations']}['B2']
        self.assertEqual(b2['execution_status'], 'executed')
        self.assertFalse(b2['qualified'])
        self.assertIsNone(b2['error'])
        b0 = {item['id']: item for item in receipt['ablations']}['B0']
        self.assertEqual(b0['execution_status'], 'executed')
        self.assertFalse(b0['qualified'])
        self.assertFalse(b0['gpu'])
        self.assertEqual(seam['independent_decode']['status'], 'missing_file')
        self.assertEqual(seam['collapse_provenance']['status'], 'not_supplied')
        self.assertIsNone(seam['collapse_provenance']['sha256'])
        self.assertNotEqual(receipt['status'], 'qualified')
        self.assertFalse(receipt['invented_passing_geometry_receipt'])

    def test_second_out_dir_is_create_only(self):
        if not (CORPUS / 'manifest.json').is_file():
            self.skipTest('published corpus missing')
        with tempfile.TemporaryDirectory() as tmp:
            out = Path(tmp) / 'once'
            run_campaign(
                corpus=CORPUS,
                out=out,
                inspect_only=True,
                families=['holdout'],
                geometry_exe=Path(tmp) / 'missing.exe',
            )
            with self.assertRaises(FileExistsError):
                run_campaign(
                    corpus=CORPUS,
                    out=out,
                    inspect_only=True,
                    families=['holdout'],
                    geometry_exe=Path(tmp) / 'missing.exe',
                )

    def test_materializer_symbol_still_exists_but_campaign_does_not_call_it(self):
        self.assertTrue(callable(bounded_maximum_size_mesh))
        self.assertIn('identity', FAMILIES)
        self.assertIn('analytic-plane-scale-1', SEAM_ANALYTIC_FIXTURE_IDS)
        self.assertNotIn('analytic-open-cylinder', SEAM_ANALYTIC_FIXTURE_IDS)

    def test_existing_receipts_have_no_enclosing_bound_class(self):
        synth = Path(__file__).resolve().parents[2]
        report = whole_surface_bound_from_existing_receipts(synth)
        self.assertEqual(report['status'], 'not_measured')
        self.assertIsNone(report['class'])
        self.assertFalse(report['qualified'])
        self.assertEqual(report['enclosing_receipts'], [])
        self.assertGreaterEqual(report['inspected'], 1)


if __name__ == '__main__':
    unittest.main()
