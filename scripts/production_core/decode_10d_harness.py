"""Independent sealed `.10d` QuantizedMesh decode and P03/P22 inspect helpers.

Pure-Python mesh-section decoder from the in-tree QuantizedMesh layout
(`qualiaDB/docs/manuals/standards/10d-container-format.md` and
`container_10d/{header,section,mesh_section}.rs`). Does not read
`geometry-receipt.json` for vertex/triangle counts or positions. Does not
invoke GPU, cargo, or the admitted geometry executable.

Collapse-provenance absence is `not_supplied`, never a numeric 0.0.
B0 identity ablation is inspect/CPU only and is never marked qualified.
"""
from __future__ import annotations

from pathlib import Path
import argparse
import datetime
import json
import math
import struct
import sys
import time

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

from production_core_artifacts import check_affine, f32, sha

MAGIC_10D = b'10d\0'
HEADER_BYTE_SIZE = 64
SECTION_DESCRIPTOR_SIZE = 24
MESH_MINI_HEADER_SIZE = 40
FLAG_U16_INDICES = 0x0001
SECTION_QUANTIZED_MESH = 1
MAX_SECTION_COUNT = 1024
MAX_VERTEX_COUNT = 4_194_304
MAX_TRIANGLE_COUNT = 4_194_304
SUPPORTED_HEADER_VERSIONS = (1, 2)
U16_MAX = 65535
DEFAULT_MAX_BYTES = 64 * 1024 * 1024
LAYOUT_SOURCE = (
    'qualiaDB/docs/manuals/standards/10d-container-format.md and '
    'container_10d/{header,section,mesh_section}.rs QuantizedMesh payload'
)
PROVENANCE_SNAPSHOT_NAMES = (
    'collapse-provenance.json',
    'lineage-snapshot.json',
    'provenance-snapshot.json',
    'collapse-lineage.json',
    'collapse-provenance.bin',
)


class Decode10dError(ValueError):
    pass


def crc32c(data):
    """CRC-32C (Castagnoli, reflected). Check value of ASCII 123456789 is 0xE3069283."""
    crc = 0xFFFFFFFF
    for byte in data:
        crc ^= byte
        for _ in range(8):
            crc = ((crc >> 1) ^ (0x82F63B78 & (0xFFFFFFFF if crc & 1 else 0))) & 0xFFFFFFFF
    return (~crc) & 0xFFFFFFFF


def _u32(value):
    if isinstance(value, bool) or not isinstance(value, int) or value < 0:
        raise Decode10dError('expected non-negative integer')
    return value


def dequantize_u16(q, min_f32, max_f32):
    """Documented QuantizedMesh decode: endpoints bit-for-bit, interior in f64 then f32."""
    q = _u32(q)
    if q > U16_MAX:
        raise Decode10dError(f'quantized coordinate {q} exceeds u16')
    if q == 0:
        return min_f32
    if q == U16_MAX:
        return max_f32
    if min_f32 == max_f32:
        return min_f32
    fraction = q / float(U16_MAX)
    return f32(min_f32 + fraction * (max_f32 - min_f32))


def _cpu_wall(wall_seconds, source='python_perf_counter'):
    return dict(
        kind='cpu_wall',
        wall_seconds=wall_seconds,
        gpu=False,
        synthetic=False,
        source=source,
        note='CPU wall time. Not a GPU timestamp sample.',
    )


def parse_container_header(data):
    if len(data) < HEADER_BYTE_SIZE:
        raise Decode10dError(f'10d header too short: got {len(data)}, need {HEADER_BYTE_SIZE}')
    if data[:4] != MAGIC_10D:
        raise Decode10dError(f'10d bad magic: got {data[:4]!r}')
    version, flags = struct.unpack_from('<HH', data, 4)
    if version not in SUPPORTED_HEADER_VERSIONS:
        raise Decode10dError(f'10d unknown version {version}; supported {SUPPORTED_HEADER_VERSIONS}')
    if data[18:20] != b'\x00\x00':
        raise Decode10dError('10d non-zero header pad0')
    header_crc32c, table_off, section_count = struct.unpack_from('<III', data, 52)
    if (table_off == 0) != (section_count == 0):
        raise Decode10dError(f'10d bad section-table pointer offset={table_off} count={section_count}')
    if section_count > MAX_SECTION_COUNT:
        raise Decode10dError(f'10d section_count {section_count} exceeds {MAX_SECTION_COUNT}')
    if table_off != 0 and table_off < HEADER_BYTE_SIZE:
        raise Decode10dError(f'10d section table overlaps header: offset={table_off}')
    return dict(
        version=version,
        flags=flags,
        header_crc32c=header_crc32c,
        section_table_offset=table_off,
        section_count=section_count,
    )


def parse_section_table(data, header):
    table_off = header['section_table_offset']
    count = header['section_count']
    if count == 0:
        return []
    end = table_off + count * SECTION_DESCRIPTOR_SIZE
    if end > len(data):
        raise Decode10dError('10d section table out of bounds')
    rows = []
    for index in range(count):
        base = table_off + index * SECTION_DESCRIPTOR_SIZE
        typ, align, reserved, offset, length, stride, element_count, payload_crc = struct.unpack_from(
            '<BBHIIIII', data, base,
        )
        if reserved != 0:
            raise Decode10dError(f'10d section[{index}] non-zero reserved16')
        payload_end = offset + length
        if payload_end > len(data):
            raise Decode10dError(f'10d section[{index}] payload out of bounds')
        rows.append(dict(
            index=index,
            section_type=typ,
            alignment_tier=align,
            byte_offset=offset,
            byte_length=length,
            stride=stride,
            element_count=element_count,
            crc32c=payload_crc,
        ))
    return rows


def decode_mesh_section(payload):
    if len(payload) < MESH_MINI_HEADER_SIZE:
        raise Decode10dError(
            f'10d MESH payload too short: got {len(payload)}, need {MESH_MINI_HEADER_SIZE}'
        )
    flags, reserved_u16, vertex_count, triangle_count = struct.unpack_from('<HHII', payload, 0)
    mins = struct.unpack_from('<3f', payload, 12)
    maxs = struct.unpack_from('<3f', payload, 24)
    reserved_u32, = struct.unpack_from('<I', payload, 36)
    if reserved_u16 != 0:
        raise Decode10dError('10d MESH non-zero reserved_u16')
    if reserved_u32 != 0:
        raise Decode10dError('10d MESH non-zero reserved_u32')
    if flags & ~FLAG_U16_INDICES:
        raise Decode10dError(f'10d MESH unknown flags bits {flags:#06x}')
    for axis in range(3):
        if not math.isfinite(mins[axis]) or not math.isfinite(maxs[axis]) or mins[axis] > maxs[axis]:
            raise Decode10dError(f'10d MESH invalid bounds on axis {axis}')
    if vertex_count > MAX_VERTEX_COUNT:
        raise Decode10dError(f'10d MESH vertex_count {vertex_count} exceeds max {MAX_VERTEX_COUNT}')
    if triangle_count > MAX_TRIANGLE_COUNT:
        raise Decode10dError(f'10d MESH triangle_count {triangle_count} exceeds max {MAX_TRIANGLE_COUNT}')
    u16_idx = bool(flags & FLAG_U16_INDICES)
    index_width = 2 if u16_idx else 4
    expected = MESH_MINI_HEADER_SIZE + vertex_count * 6 + triangle_count * index_width * 3
    if len(payload) != expected:
        raise Decode10dError(f'10d MESH trailing or truncated payload: expected {expected}, got {len(payload)}')
    vertices = []
    for i in range(vertex_count):
        q = struct.unpack_from('<3H', payload, MESH_MINI_HEADER_SIZE + i * 6)
        vertices.append([dequantize_u16(q[a], mins[a], maxs[a]) for a in range(3)])
    start = MESH_MINI_HEADER_SIZE + vertex_count * 6
    fmt = '<3H' if u16_idx else '<3I'
    triangles = []
    for i in range(triangle_count):
        tri = list(struct.unpack_from(fmt, payload, start + i * index_width * 3))
        if any(index >= vertex_count for index in tri):
            raise Decode10dError(f'10d MESH index {tri} outside {vertex_count} vertices')
        triangles.append(tri)
    return dict(
        flags=flags,
        u16_indices=u16_idx,
        vertex_count=vertex_count,
        triangle_count=triangle_count,
        min=list(mins),
        max=list(maxs),
        vertices=vertices,
        triangles=triangles,
    )


def decode_10d_mesh_bytes(data, *, verify_section_crc=True):
    """Decode the first QuantizedMesh section. Independent of geometry-receipt.json."""
    header = parse_container_header(data)
    sections = parse_section_table(data, header)
    meshes = [row for row in sections if row['section_type'] == SECTION_QUANTIZED_MESH]
    if not meshes:
        raise Decode10dError('10d has no QuantizedMesh section')
    row = meshes[0]
    payload = data[row['byte_offset']:row['byte_offset'] + row['byte_length']]
    computed_crc = crc32c(payload)
    crc_match = computed_crc == row['crc32c']
    if verify_section_crc and not crc_match:
        raise Decode10dError(
            f'10d QuantizedMesh CRC-32C mismatch: descriptor {row["crc32c"]:#010x} computed {computed_crc:#010x}'
        )
    mesh = decode_mesh_section(payload)
    mesh.update(
        container=dict(
            version=header['version'],
            flags=header['flags'],
            section_count=header['section_count'],
            quantized_mesh_sections=len(meshes),
        ),
        section=row,
        crc32c=dict(
            descriptor=row['crc32c'],
            computed=computed_crc,
            verified=bool(verify_section_crc),
            match=crc_match,
        ),
    )
    return mesh


def decode_10d_mesh_path(path, *, max_bytes=DEFAULT_MAX_BYTES, verify_section_crc=True):
    path = Path(path)
    if not path.is_file():
        raise Decode10dError(f'missing .10d file: {path}')
    size = path.stat().st_size
    if max_bytes <= 0 or size > max_bytes:
        raise Decode10dError(f'.10d exceeds explicit byte budget ({size} > {max_bytes})')
    return decode_10d_mesh_bytes(path.read_bytes(), verify_section_crc=verify_section_crc)


def _load_json_list(path):
    path = Path(path)
    if not path.is_file():
        return None
    payload = json.loads(path.read_text(encoding='utf-8'))
    if not isinstance(payload, list):
        raise Decode10dError(f'{path.name} is not a JSON array')
    return payload


def _position_cross_check(decoded, sidecar):
    if len(decoded) != len(sidecar):
        return dict(
            match=False,
            compared=min(len(decoded), len(sidecar)),
            decoded_count=len(decoded),
            sidecar_count=len(sidecar),
            bit_mismatches=None,
            maximum_absolute_error=None,
            note='vertex count mismatch versus decoded-vertices.json',
        )
    bit_mismatches = 0
    maximum = 0.0
    compared = 0
    for left, right in zip(decoded, sidecar):
        if len(left) != 3 or not isinstance(right, (list, tuple)) or len(right) != 3:
            raise Decode10dError('expected xyz triples in decoded vertices sidecar')
        for x, y in zip(left, right):
            actual = f32(x)
            reported = f32(y)
            compared += 1
            error = abs(actual - reported)
            if not math.isfinite(error):
                raise Decode10dError('non-finite vertex cross-check error')
            maximum = max(maximum, error)
            if struct.pack('<f', actual) != struct.pack('<f', reported):
                bit_mismatches += 1
    return dict(
        match=bit_mismatches == 0,
        compared=compared,
        decoded_count=len(decoded),
        sidecar_count=len(sidecar),
        bit_mismatches=bit_mismatches,
        maximum_absolute_error=maximum,
        note='independent f32 decode compared to decoded-vertices.json after f32 domain narrowing',
    )


def _triangle_cross_check(decoded, sidecar):
    if len(decoded) != len(sidecar):
        return dict(
            match=False,
            decoded_count=len(decoded),
            sidecar_count=len(sidecar),
            mismatches=None,
            note='triangle count mismatch versus decoded-triangles.json',
        )
    mismatches = 0
    for left, right in zip(decoded, sidecar):
        if list(left) != list(right):
            mismatches += 1
    return dict(
        match=mismatches == 0,
        decoded_count=len(decoded),
        sidecar_count=len(sidecar),
        mismatches=mismatches,
        note='independent triangle indices compared to decoded-triangles.json',
    )


def inspect_collapse_provenance(path=None, search_dir=None):
    """Hash a collapse-provenance snapshot if present. Absence is not_supplied, not 0.0."""
    snapshot = Path(path) if path is not None else None
    searched = []
    if snapshot is None and search_dir is not None:
        root = Path(search_dir)
        for name in PROVENANCE_SNAPSHOT_NAMES:
            candidate = root / name
            searched.append(str(candidate))
            if candidate.is_file():
                snapshot = candidate
                break
    if snapshot is None or not snapshot.is_file():
        return dict(
            status='not_supplied',
            sha256=None,
            bytes=None,
            path=None if snapshot is None else str(snapshot),
            searched=searched,
            qualified=False,
            error=None,
            note='Collapse provenance snapshot was not supplied. Absence is not_supplied, not 0.0.',
        )
    return dict(
        status='present',
        sha256=sha(snapshot),
        bytes=snapshot.stat().st_size,
        path=str(snapshot),
        searched=searched,
        qualified=False,
        error=None,
        note='Snapshot hashed only. Hashing is not collapse-provenance qualification.',
    )


def inspect_sealed_10d(
    ten_d,
    *,
    vertices_json=None,
    triangles_json=None,
    geometry_receipt=None,
    max_bytes=DEFAULT_MAX_BYTES,
    verify_section_crc=True,
):
    """Decode simplified.10d independently. geometry-receipt.json is never a count source."""
    ten_d = Path(ten_d) if ten_d is not None else None
    receipt_path = Path(geometry_receipt) if geometry_receipt is not None else None
    vertices_path = Path(vertices_json) if vertices_json is not None else None
    triangles_path = Path(triangles_json) if triangles_json is not None else None
    record = dict(
        schema='production-core.independent-10d-decode/v1',
        status='not_executed',
        qualified=False,
        decoder='pure_python_quantized_mesh_section',
        layout_source=LAYOUT_SOURCE,
        trusted_geometry_receipt=False,
        gpu=False,
        path=str(ten_d) if ten_d is not None else None,
        sha256=None,
        bytes=None,
        vertex_count=None,
        triangle_count=None,
        vertices=None,
        triangles=None,
        cross_check=None,
        geometry_receipt=dict(status='not_supplied', sha256=None, path=None),
        crc32c=None,
        timing=None,
        note='Independent QuantizedMesh decode. geometry-receipt.json is not a geometry oracle.',
    )
    if receipt_path is not None and receipt_path.is_file():
        record['geometry_receipt'] = dict(
            status='present_not_trusted',
            sha256=sha(receipt_path),
            path=str(receipt_path),
            bytes=receipt_path.stat().st_size,
            note='Hashed as sidecar identity only; result.vertices/faces were not used.',
        )
    elif receipt_path is not None:
        record['geometry_receipt'] = dict(
            status='not_supplied',
            sha256=None,
            path=str(receipt_path),
            note='geometry-receipt.json path was named but the file is absent. Absence is not_supplied, not 0.0.',
        )
    if ten_d is None:
        record['status'] = 'not_supplied'
        record['note'] = 'No .10d path supplied. Absence is not_supplied, not 0.0.'
        return record
    if not ten_d.is_file():
        record['status'] = 'missing_file'
        record['note'] = f'Independent decode skipped; {ten_d} is not a file.'
        return record
    started = time.perf_counter()
    try:
        mesh = decode_10d_mesh_path(ten_d, max_bytes=max_bytes, verify_section_crc=verify_section_crc)
    except Decode10dError as error:
        record['status'] = 'decode_failed'
        record['reason'] = str(error)
        record['timing'] = _cpu_wall(time.perf_counter() - started)
        return record
    record['timing'] = _cpu_wall(time.perf_counter() - started)
    record['sha256'] = sha(ten_d)
    record['bytes'] = ten_d.stat().st_size
    record['vertex_count'] = mesh['vertex_count']
    record['triangle_count'] = mesh['triangle_count']
    record['vertices'] = mesh['vertices']
    record['triangles'] = mesh['triangles']
    record['bounds'] = dict(min=mesh['min'], max=mesh['max'])
    record['container'] = mesh['container']
    record['crc32c'] = mesh['crc32c']
    record['status'] = 'executed'
    sidecar_vertices = _load_json_list(vertices_path) if vertices_path is not None else None
    sidecar_triangles = _load_json_list(triangles_path) if triangles_path is not None else None
    vertex_check = (
        _position_cross_check(mesh['vertices'], sidecar_vertices)
        if sidecar_vertices is not None else
        dict(status='not_supplied', match=None, note='decoded-vertices.json not supplied')
    )
    triangle_check = (
        _triangle_cross_check(mesh['triangles'], sidecar_triangles)
        if sidecar_triangles is not None else
        dict(status='not_supplied', match=None, note='decoded-triangles.json not supplied')
    )
    if sidecar_vertices is None and sidecar_triangles is None:
        cross_status = 'decoded_without_sidecar'
        mismatch = False
    else:
        vertex_ok = vertex_check.get('match') is not False
        triangle_ok = triangle_check.get('match') is not False
        mismatch = not (vertex_ok and triangle_ok)
        cross_status = 'mismatch' if mismatch else 'match'
    record['cross_check'] = dict(
        status=cross_status,
        mismatch=mismatch,
        vertices_json=str(vertices_path) if vertices_path is not None else None,
        triangles_json=str(triangles_path) if triangles_path is not None else None,
        vertices=vertex_check,
        triangles=triangle_check,
        note=(
            'Sidecar JSON is a cross-check only. Counts and positions come from the QuantizedMesh section.'
            if not mismatch else
            'Independent decode disagrees with decoded-vertices.json / decoded-triangles.json. Mismatch labelled, not repaired.'
        ),
    )
    record['qualified'] = False
    return record


def inspect_sealed_10d_dir(directory, **kwargs):
    directory = Path(directory)
    return inspect_sealed_10d(
        directory / 'simplified.10d',
        vertices_json=directory / 'decoded-vertices.json',
        triangles_json=directory / 'decoded-triangles.json',
        geometry_receipt=directory / 'geometry-receipt.json',
        **kwargs,
    )


def _b0_representation(case):
    data = case.get('data') or {}
    src = data.get('input') or {}
    cand = data.get('candidate') or {}
    keys = ('vertices', 'triangles', 'domains', 'uvs', 'tangent_w')
    mismatches = [key for key in keys if src.get(key) != cand.get(key)]
    return dict(
        id=case.get('id'),
        kind=data.get('kind'),
        status='identity_preserved' if not mismatches else 'identity_mismatch',
        mismatches=mismatches,
        gpu=False,
        production_no_reduction_path='not_invoked',
    )


def _b0_affine(case):
    data = case.get('data') or {}
    try:
        report = check_affine((data.get('input') or {})['vertices'], (data.get('candidate') or {})['vertices'])
    except (KeyError, TypeError, ValueError) as error:
        return dict(
            id=case.get('id'),
            kind=data.get('kind'),
            status='identity_mismatch',
            reason=str(error),
            gpu=False,
            production_no_reduction_path='not_invoked',
        )
    return dict(
        id=case.get('id'),
        kind=data.get('kind'),
        status='identity_preserved',
        affine=dict(compared=report['compared'], maximum_absolute_error=report['maximum_absolute_error']),
        gpu=False,
        production_no_reduction_path='not_invoked',
    )


def _b0_oracle(case):
    from oracle_harness import OracleError, evaluate_fixture
    try:
        evaluate_fixture(case)
        status = 'identity_preserved'
        reason = None
    except OracleError as error:
        status = 'identity_mismatch'
        reason = str(error)
    return dict(
        id=case.get('id'),
        kind=(case.get('data') or {}).get('kind'),
        status=status,
        reason=reason,
        route='oracle_harness.evaluate_fixture',
        gpu=False,
        production_no_reduction_path='not_invoked',
    )


def b0_identity_ablation(cases):
    """Inspect/CPU identity ablation. Never GPU. Never qualified. Absence is not 0.0."""
    cases = [case for case in (cases or []) if (case or {}).get('family') == 'identity' or (case or {}).get('oracle', {}).get('must_pass') is True]
    slot = dict(
        id='B0',
        title='identity/no reduction',
        purpose='Reference geometry, semantics, and response.',
        relevance='required',
        execution_status='not_executed',
        compared=False,
        qualified=False,
        error=None,
        cost=None,
        timing=None,
        improvement=None,
        gpu=False,
        production_no_reduction_path='not_invoked',
        identity_preserved=None,
        fixtures=[],
        note='No identity fixtures selected. Absence is not_executed, not a zero error or cost.',
    )
    if not cases:
        return slot
    started = time.perf_counter()
    results = []
    for case in cases:
        kind = (case.get('data') or {}).get('kind')
        if kind == 'representation_candidate':
            results.append(_b0_representation(case))
        elif kind == 'affine_candidate':
            results.append(_b0_affine(case))
        else:
            results.append(_b0_oracle(case))
    preserved = all(item['status'] == 'identity_preserved' for item in results)
    slot.update(
        execution_status='executed',
        compared=True,
        qualified=False,
        error=None,
        cost=None,
        timing=_cpu_wall(time.perf_counter() - started),
        gpu=False,
        production_no_reduction_path='not_invoked',
        identity_preserved=preserved,
        fixtures=results,
        note=(
            'CPU inspect of published identity candidates. Production no-reduction path was not invoked. '
            'Executed is not qualified. Unimplemented numeric error/cost remain null, not 0.0.'
        ),
    )
    return slot


def compact_decode_record(record):
    """Drop vertex/triangle arrays for receipts that only need counts and cross-check."""
    item = dict(record)
    item.pop('vertices', None)
    item.pop('triangles', None)
    return item


def run_independent_decode_evidence(asset_dirs, *, identity_cases=None, provenance_path=None, provenance_search=None):
    assets = []
    for directory in asset_dirs:
        directory = Path(directory)
        record = inspect_sealed_10d_dir(directory)
        assets.append(dict(
            dir=str(directory),
            present=directory.is_dir(),
            decode=compact_decode_record(record),
            collapse_provenance=inspect_collapse_provenance(search_dir=directory),
        ))
    executed = sum(1 for item in assets if item['decode'].get('status') == 'executed')
    mismatches = sum(1 for item in assets if (item['decode'].get('cross_check') or {}).get('mismatch'))
    return dict(
        schema='production-core.integration-wave-p03.decode-receipt/v1',
        status='executed_not_qualified' if executed else 'recorded_not_executed',
        qualified=False,
        gpu=False,
        invented_passing_geometry_receipt=False,
        trusted_geometry_receipt=False,
        decoder='pure_python_quantized_mesh_section',
        layout_source=LAYOUT_SOURCE,
        started_at=datetime.datetime.now(datetime.timezone.utc).isoformat(),
        assets=assets,
        independent_decode=dict(
            present=True,
            callable=True,
            executed=executed > 0,
            qualified=False,
            state='executed' if executed else 'callable',
            asset_count=len(assets),
            executed_count=executed,
            sidecar_mismatches=mismatches,
            note='Independent decode of existing evidence .10d files. Execution is not qualification.',
        ),
        collapse_provenance=inspect_collapse_provenance(path=provenance_path, search_dir=provenance_search),
        b0_identity_ablation=b0_identity_ablation(identity_cases),
    )


def main(argv=None):
    parser = argparse.ArgumentParser(description='Independent sealed .10d QuantizedMesh decode harness')
    parser.add_argument('--10d', dest='ten_d', type=Path)
    parser.add_argument('--dir', action='append', default=None, type=Path, help='Evidence directory containing simplified.10d')
    parser.add_argument('--vertices-json', type=Path)
    parser.add_argument('--triangles-json', type=Path)
    parser.add_argument('--geometry-receipt', type=Path)
    parser.add_argument('--provenance', type=Path)
    parser.add_argument('--identity-json', action='append', default=None, type=Path)
    parser.add_argument('--out', type=Path, help='Create-only directory for a decode receipt')
    parser.add_argument('--skip-crc', action='store_true')
    args = parser.parse_args(argv)

    identity_cases = []
    for path in args.identity_json or []:
        identity_cases.append(json.loads(Path(path).read_text(encoding='utf-8')))

    if args.dir:
        receipt = run_independent_decode_evidence(
            args.dir,
            identity_cases=identity_cases,
            provenance_path=args.provenance,
        )
    else:
        decode = inspect_sealed_10d(
            args.ten_d,
            vertices_json=args.vertices_json,
            triangles_json=args.triangles_json,
            geometry_receipt=args.geometry_receipt,
            verify_section_crc=not args.skip_crc,
        )
        receipt = dict(
            schema='production-core.independent-10d-decode-receipt/v1',
            status=decode['status'],
            qualified=False,
            gpu=False,
            invented_passing_geometry_receipt=False,
            decode=compact_decode_record(decode),
            collapse_provenance=inspect_collapse_provenance(path=args.provenance),
            b0_identity_ablation=b0_identity_ablation(identity_cases),
        )

    if args.out is not None:
        out = Path(args.out).resolve()
        out.mkdir(parents=False, exist_ok=False)
        (out / 'decode-receipt.json').write_text(
            json.dumps(receipt, indent=2, allow_nan=False) + '\n',
            encoding='utf-8',
        )
        receipt['out'] = str(out)
    summary = dict(
        status=receipt['status'],
        qualified=False,
        gpu=False,
        invented_passing_geometry_receipt=False,
        out=receipt.get('out'),
    )
    if 'independent_decode' in receipt:
        summary['executed_count'] = receipt['independent_decode']['executed_count']
        summary['sidecar_mismatches'] = receipt['independent_decode']['sidecar_mismatches']
    elif 'decode' in receipt:
        summary['vertex_count'] = receipt['decode'].get('vertex_count')
        summary['triangle_count'] = receipt['decode'].get('triangle_count')
        summary['cross_check'] = (receipt['decode'].get('cross_check') or {}).get('status')
    print(json.dumps(summary, indent=2))
    return receipt


if __name__ == '__main__':
    main()
