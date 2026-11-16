"""Unit tests for published known-bad candidate evaluation.

The geometry executable must not be invoked on a welded candidate as if it
were an input mesh. Unobservable gates stay unobserved, never a fake fail.
"""
from __future__ import annotations

import copy
import json
import unittest
from pathlib import Path

from oracle_harness import OracleError, evaluate_fixture, observe_known_bad_gates
from run_qualification_campaign import SKIP_INVOCATION_FAMILIES, geometry_invocation_args

CORPUS = Path(__file__).resolve().parents[2] / 'docs' / 'production-core' / 'corpus'


def _load(name):
    return json.loads((CORPUS / name).read_text(encoding='utf-8'))


class KnownBadOracleTests(unittest.TestCase):
    def test_published_candidates_fail_named_gates_without_exe(self):
        expected = {
            'known-bad-cross-domain-weld.json': {
                'forbidden_semantic_merges',
                'no_cross_domain_support',
                'preserve_corner_chart',
                'preserve_tangent_handedness',
                'coincident_pairs_are_not_welded',
            },
            'known-bad-poisoned-affine.json': {
                'independent_affine_oracle',
                'k1_abs_rel_tolerance',
            },
            'known-bad-tetra-inversion.json': {'accept_when_inversion_forbidden'},
            'known-bad-infeasible-contact.json': {
                'contact_feasibility',
                'authoritative_full_model_required',
            },
        }
        for name, gates in expected.items():
            case = _load(name)
            with self.subTest(name=name):
                self.assertEqual(case['execution_status'], 'not_executed')
                observation = observe_known_bad_gates(case['data'], case['oracle'])
                self.assertFalse(observation['production_exe_invoked'])
                self.assertFalse(observation['gpu'])
                self.assertIsNone(observation['publication_defect'])
                self.assertEqual(observation['unevaluated'], [])
                self.assertEqual(set(observation['observed']), gates)
                result = evaluate_fixture(case)
                self.assertEqual(result['publication_check'], 'consistent')
                self.assertEqual(set(result['observed_failed_gates']), gates)
                self.assertFalse(result['production_exe_invoked'])
                self.assertIsNone(geometry_invocation_args(
                    case['id'], fixture=case, corpus=CORPUS, driver='geometric', supports_driver=True,
                ))

    def test_cross_domain_weld_evaluates_candidate_not_input(self):
        case = _load('known-bad-cross-domain-weld.json')
        observation = observe_known_bad_gates(case['data'], case['oracle'])
        self.assertIn('forbidden_semantic_merges', observation['observed'])
        input_only = copy.deepcopy(case)
        input_only['data']['candidate'] = copy.deepcopy(input_only['data']['input'])
        input_only['data']['candidate'].pop('collapsed_vertex_pairs', None)
        input_only['data']['candidate']['claimed_status'] = 'accepted'
        honest = observe_known_bad_gates(input_only['data'], input_only['oracle'])
        self.assertNotIn('forbidden_semantic_merges', honest['observed'])
        self.assertNotIn('coincident_pairs_are_not_welded', honest['observed'])
        with self.assertRaises(OracleError):
            evaluate_fixture(input_only)

    def test_missing_candidate_leaves_gates_unobserved(self):
        case = _load('known-bad-cross-domain-weld.json')
        data = dict(kind='representation_candidate', input=case['data']['input'])
        observation = observe_known_bad_gates(data, case['oracle'])
        self.assertEqual(observation['observed'], [])
        self.assertTrue(observation['unevaluated'])
        self.assertNotIn(0.0, observation['unevaluated'])
        self.assertNotEqual(observation['observed'], case['oracle']['failing_tests'])

    def test_correct_affine_is_publication_defect_not_fake_fail(self):
        verts = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]
        fused = [[2.0 * x + 0.25 for x in point] for point in verts]
        data = dict(
            kind='affine_candidate',
            kernel=dict(scale=2.0, bias=0.25),
            input=dict(vertices=verts),
            candidate=dict(vertices=fused),
        )
        oracle = dict(
            must_fail=True,
            failing_tests=['independent_affine_oracle', 'k1_abs_rel_tolerance'],
        )
        observation = observe_known_bad_gates(data, oracle)
        self.assertEqual(observation['observed'], [])
        self.assertIn('unexpectedly matches closed form', observation['publication_defect'])
        case = dict(
            schema='production-core.qualification-fixture/v1',
            id='not-actually-poisoned',
            family='known_bad',
            partition='test',
            seed=0,
            provenance='test',
            license='Apache-2.0',
            data=data,
            oracle=oracle,
            exclusions=[],
            execution_status='not_executed',
        )
        with self.assertRaises(OracleError):
            evaluate_fixture(case)

    def test_known_bad_family_never_in_invocation_set(self):
        self.assertIn('known_bad', SKIP_INVOCATION_FAMILIES)


class IdentityStillPassesTests(unittest.TestCase):
    def test_identity_seam_and_affine_still_must_pass(self):
        for name in ('identity-two-subpart-seam.json', 'identity-affine-k1.json'):
            case = _load(name)
            result = evaluate_fixture(case)
            self.assertEqual(result['publication_check'], 'consistent')
            self.assertEqual(result['family'], 'identity')


if __name__ == '__main__':
    unittest.main()
