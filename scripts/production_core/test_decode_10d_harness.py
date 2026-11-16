"""Unit tests for independent `.10d` decode, provenance inspect, and B0 ablation."""
from __future__ import annotations

import json
import math
import struct
import tempfile
import unittest
from pathlib import Path

from decode_10d_harness import (
    Decode10dError,
    PROVENANCE_SNAPSHOT_NAMES,
    b0_identity_ablation,
    crc32c,
    decode_10d_mesh_bytes,
    decode_mesh_section,
    dequantize_u16,
    inspect_collapse_provenance,
    inspect_sealed_10d,
    inspect_sealed_10d_dir,
)
from production_core_artifacts import f32, sha
from run_qualification_campaign import SKIP_INVOCATION_FAMILIES, geometry_invocation_args
from qualification_corpus import FAMILIES

EVIDENCE = Path(__file__).resolve().parents[2] / 'docs' / 'production-core' / 'evidence'
CORPUS = Path(__file__).resolve().parents[2] / 'docs' / 'production-core' / 'corpus'
JOINED_GEOMETRY = EVIDENCE / 'joined-tracer-01' / 'geometry-1'
IDENTITY_GEOMETRY = EVIDENCE / 'qualification-campaign-run-02' / 'geometry-identity-two-subpart-seam'


def _quantize(value, min_v, max_v):
    extent = max_v - min_v
    if extent <= 0.0:
        return 0
    n = min(1.0, max(0.0, (value - min_v) / extent))
    return int(n * 65535.0 + 0.5)


def encode_test_container(positions, triangles, *, version=2, corrupt_crc=False):
    mins = [min(p[i] for p in positions) for i in range(3)]
    maxs = [max(p[i] for p in positions) for i in range(3)]
    vertex_count = len(positions)
    triangle_count = len(triangles)
    flags = 1
    payload = bytearray(40 + vertex_count * 6 + triangle_count * 6)
    struct.pack_into('<HHII', payload, 0, flags, 0, vertex_count, triangle_count)
    struct.pack_into('<3f', payload, 12, *mins)
    struct.pack_into('<3f', payload, 24, *maxs)
    struct.pack_into('<I', payload, 36, 0)
    offset = 40
    for point in positions:
        for axis in range(3):
            struct.pack_into('<H', payload, offset, _quantize(point[axis], mins[axis], maxs[axis]))
            offset += 2
    for tri in triangles:
        struct.pack_into('<3H', payload, offset, *tri)
        offset += 6
    payload = bytes(payload)
    payload_crc = crc32c(payload)
    if corrupt_crc:
        payload_crc ^= 1
    table_off = 64
    mesh_off = 64 + 24
    header = bytearray(64)
    header[0:4] = b'10d\0'
    struct.pack_into('<HH', header, 4, version, 1)
    struct.pack_into('<III', header, 52, 0, table_off, 1)
    descriptor = struct.pack('<BBHIIIII', 1, 1, 0, mesh_off, len(payload), 0, 0, payload_crc)
    return bytes(header) + descriptor + payload


class CrcAndDequantTests(unittest.TestCase):
    def test_crc32c_rfc3720_check_value(self):
        self.assertEqual(crc32c(b'123456789'), 0xE3069283)

    def test_dequantize_preserves_endpoint_bits(self):
        self.assertEqual(struct.pack('<f', dequantize_u16(0, -0.0, 1.0)), struct.pack('<f', -0.0))
        self.assertEqual(struct.pack('<f', dequantize_u16(65535, 0.0, 2.5)), struct.pack('<f', 2.5))
        self.assertEqual(dequantize_u16(12, 4.0, 4.0), 4.0)
        interior = dequantize_u16(32768, 0.0, 1.0)
        self.assertEqual(interior, f32(32768 / 65535.0))
        self.assertTrue(math.isfinite(interior))


class MeshSectionTests(unittest.TestCase):
    def test_roundtrip_triangle_and_reject_bad_flags(self):
        positions = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]
        triangles = [[0, 1, 2]]
        container = encode_test_container(positions, triangles)
        mesh = decode_10d_mesh_bytes(container)
        self.assertEqual(mesh['vertex_count'], 3)
        self.assertEqual(mesh['triangle_count'], 1)
        self.assertEqual(mesh['triangles'], [[0, 1, 2]])
        self.assertEqual(mesh['vertices'][0], [0.0, 0.0, 0.0])
        self.assertEqual(mesh['vertices'][1][0], 1.0)
        self.assertEqual(mesh['vertices'][2][1], 1.0)
        self.assertTrue(mesh['crc32c']['match'])
        with self.assertRaises(Decode10dError):
            decode_10d_mesh_bytes(encode_test_container(positions, triangles, corrupt_crc=True))
        payload = bytearray(40)
        struct.pack_into('<HHII', payload, 0, 2, 0, 0, 0)
        with self.assertRaises(Decode10dError):
            decode_mesh_section(bytes(payload))

    def test_inspect_does_not_trust_geometry_receipt_counts(self):
        positions = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
        triangles = [[0, 1, 2], [0, 2, 3]]
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / 'simplified.10d').write_bytes(encode_test_container(positions, triangles))
            (root / 'decoded-vertices.json').write_text(json.dumps(positions) + '\n', encoding='utf-8')
            (root / 'decoded-triangles.json').write_text(json.dumps([[9, 9, 9]]) + '\n', encoding='utf-8')
            (root / 'geometry-receipt.json').write_text(
                json.dumps(dict(result=dict(vertices=99, faces=99), status='passed')) + '\n',
                encoding='utf-8',
            )
            record = inspect_sealed_10d_dir(root)
            self.assertEqual(record['status'], 'executed')
            self.assertEqual(record['vertex_count'], 4)
            self.assertEqual(record['triangle_count'], 2)
            self.assertNotEqual(record['vertex_count'], 99)
            self.assertEqual(record['geometry_receipt']['status'], 'present_not_trusted')
            self.assertFalse(record['trusted_geometry_receipt'])
            self.assertTrue(record['cross_check']['mismatch'])
            self.assertEqual(record['cross_check']['status'], 'mismatch')
            self.assertFalse(record['cross_check']['triangles']['match'])
            self.assertFalse(record['qualified'])
            self.assertFalse(record['gpu'])

    def test_missing_sidecar_is_not_supplied_not_zero(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / 'simplified.10d'
            path.write_bytes(encode_test_container([[0, 0, 0], [1, 0, 0], [0, 1, 0]], [[0, 1, 2]]))
            record = inspect_sealed_10d(path)
            self.assertEqual(record['status'], 'executed')
            self.assertEqual(record['cross_check']['status'], 'decoded_without_sidecar')
            self.assertIsNone(record['cross_check']['vertices'].get('match'))
            self.assertEqual(record['geometry_receipt']['status'], 'not_supplied')
            self.assertIsNone(record['geometry_receipt']['sha256'])

    def test_missing_file_does_not_invent_a_pass(self):
        record = inspect_sealed_10d(Path('definitely-missing.10d'))
        self.assertEqual(record['status'], 'missing_file')
        self.assertIsNone(record['vertex_count'])
        self.assertFalse(record['qualified'])


class ProvenanceAndB0Tests(unittest.TestCase):
    def test_missing_provenance_is_not_supplied_not_zero(self):
        self.assertEqual(PROVENANCE_SNAPSHOT_NAMES[0], 'collapse-provenance.json')
        record = inspect_collapse_provenance(path=None, search_dir=None)
        self.assertEqual(record['status'], 'not_supplied')
        self.assertIsNone(record['sha256'])
        self.assertIsNone(record['bytes'])
        self.assertIsNone(record['error'])
        self.assertNotEqual(record['status'], 0)
        self.assertNotEqual(record['sha256'], 0.0)
        with tempfile.TemporaryDirectory() as tmp:
            absent = inspect_collapse_provenance(search_dir=tmp)
            self.assertEqual(absent['status'], 'not_supplied')
            self.assertIsNone(absent['sha256'])
            self.assertNotEqual(absent['bytes'], 0.0)

    def test_present_provenance_is_hashed_not_qualified(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / 'lineage-snapshot.json'
            path.write_text('{"schema":"collapse-provenance/v1","rows":[]}\n', encoding='utf-8')
            record = inspect_collapse_provenance(search_dir=tmp)
            self.assertEqual(record['status'], 'present')
            self.assertEqual(record['sha256'], sha(path))
            self.assertEqual(record['bytes'], path.stat().st_size)
            self.assertFalse(record['qualified'])
            self.assertIsNone(record['error'])

    def test_b0_cpu_identity_is_executed_not_qualified(self):
        seam = json.loads((CORPUS / 'identity-two-subpart-seam.json').read_text(encoding='utf-8'))
        affine = json.loads((CORPUS / 'identity-affine-k1.json').read_text(encoding='utf-8'))
        slot = b0_identity_ablation([seam, affine])
        self.assertEqual(slot['execution_status'], 'executed')
        self.assertTrue(slot['compared'])
        self.assertTrue(slot['identity_preserved'])
        self.assertFalse(slot['qualified'])
        self.assertFalse(slot['gpu'])
        self.assertIsNone(slot['error'])
        self.assertIsNone(slot['cost'])
        self.assertEqual(slot['production_no_reduction_path'], 'not_invoked')
        self.assertEqual(slot['timing']['kind'], 'cpu_wall')
        self.assertFalse(slot['timing']['gpu'])

    def test_b0_mismatch_is_labelled_and_empty_is_not_executed(self):
        case = dict(
            id='broken-identity',
            family='identity',
            oracle=dict(must_pass=True),
            data=dict(
                kind='representation_candidate',
                input=dict(vertices=[[0, 0, 0]], triangles=[[0, 0, 0]], domains=[1], uvs=[[0, 0]], tangent_w=[1.0]),
                candidate=dict(vertices=[[1, 0, 0]], triangles=[[0, 0, 0]], domains=[1], uvs=[[0, 0]], tangent_w=[1.0]),
            ),
        )
        slot = b0_identity_ablation([case])
        self.assertEqual(slot['execution_status'], 'executed')
        self.assertFalse(slot['identity_preserved'])
        self.assertFalse(slot['qualified'])
        self.assertIsNone(slot['error'])
        empty = b0_identity_ablation([])
        self.assertEqual(empty['execution_status'], 'not_executed')
        self.assertIsNone(empty['error'])
        self.assertIsNone(empty['cost'])
        self.assertFalse(empty['gpu'])


class SkipInvocationTests(unittest.TestCase):
    def test_known_bad_holdout_maximum_size_still_skip_exe(self):
        self.assertTrue({'known_bad', 'holdout', 'maximum_size_descriptor'} <= SKIP_INVOCATION_FAMILIES)
        self.assertIsNone(geometry_invocation_args('known-bad-cross-domain-weld', fixture=dict(family='known_bad')))
        self.assertIsNone(geometry_invocation_args('holdout-rotated-plane', fixture=dict(family='holdout', partition='holdout')))
        self.assertIsNone(geometry_invocation_args(
            'maximum-size-descriptor',
            fixture=dict(family='maximum_size_descriptor'),
        ))
        self.assertIn('identity', FAMILIES)


class LiveEvidenceDecodeTests(unittest.TestCase):
    def test_joined_tracer_simplified_10d_matches_sidecar(self):
        if not (JOINED_GEOMETRY / 'simplified.10d').is_file():
            self.skipTest('joined-tracer-01 geometry-1 simplified.10d not present')
        record = inspect_sealed_10d_dir(JOINED_GEOMETRY)
        self.assertEqual(record['status'], 'executed')
        self.assertGreater(record['vertex_count'], 0)
        self.assertGreater(record['triangle_count'], 0)
        self.assertEqual(record['cross_check']['status'], 'match')
        self.assertFalse(record['cross_check']['mismatch'])
        self.assertFalse(record['qualified'])
        self.assertEqual(record['geometry_receipt']['status'], 'present_not_trusted')
        receipt = json.loads((JOINED_GEOMETRY / 'geometry-receipt.json').read_text(encoding='utf-8'))
        self.assertEqual(record['vertex_count'], receipt['result']['vertices'])
        self.assertEqual(record['triangle_count'], receipt['result']['faces'])

    def test_identity_campaign_simplified_10d_matches_sidecar(self):
        if not (IDENTITY_GEOMETRY / 'simplified.10d').is_file():
            self.skipTest('campaign identity simplified.10d not present')
        record = inspect_sealed_10d_dir(IDENTITY_GEOMETRY)
        self.assertEqual(record['status'], 'executed')
        self.assertEqual(record['vertex_count'], 6)
        self.assertEqual(record['triangle_count'], 2)
        self.assertEqual(record['cross_check']['status'], 'match')
        self.assertEqual(record['triangles'], [[0, 1, 2], [3, 4, 5]])
        self.assertFalse(record['qualified'])


if __name__ == '__main__':
    unittest.main()
